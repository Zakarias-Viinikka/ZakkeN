package z.zndroid

import android.content.Context
import com.z_db.android_mascot.LiveForever
import uniffi.protocol.*
import rustlib.client_table_blueprints.*
import z.zndroid.Storage.StorageInitializer
import java.util.UUID
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import z.zndroid.protocol.SafeRowMapper
import kotlin.coroutines.CoroutineContext
import kotlin.coroutines.coroutineContext

object DbManager {
    // Hidden database instance
    private var db: LiveForever? = null
    private val readyDeferred = CompletableDeferred<Unit>()
    
    // Serializes all database access to prevent race conditions.
    // This mirrors the 'askQueue' logic from the WASM worker implementation.
    private val dbMutex = Mutex()

    /**
     * Element to store the active database instance in the coroutine context during a transaction.
     */
    internal class DbTransactionElement(val db: LiveForever) : CoroutineContext.Element {
        companion object Key : CoroutineContext.Key<DbTransactionElement>
        override val key: CoroutineContext.Key<*> get() = Key
    }

    /**
     * Suspends until the database is initialized and tables are created.
     */
    suspend fun awaitReady() = readyDeferred.await()

    /**
     * Internal helper to execute database calls with safety checks and serialization.
     * Acquisition of dbMutex ensures that only one DB operation runs at a time.
     * If a transaction is active in the current coroutine context, it bypasses the mutex.
     */
    internal suspend inline fun <T> executeNative(crossinline block: (LiveForever) -> T): Result<T> {
        val transactionElement = coroutineContext[DbTransactionElement]
        if (transactionElement != null) {
            return try {
                Result.success(block(transactionElement.db))
            } catch (e: Exception) {
                val errorMessage = e.message ?: e.toString()
                Result.failure(LocalDbError.QueryFailed(errorMessage))
            }
        }

        val currentDb = db ?: return Result.failure(LocalDbError.NotReady)
        return dbMutex.withLock {
            try {
                Result.success(block(currentDb))
            } catch (e: Exception) {
                // Use e.toString() if message is null to ensure we don't return "null" to the UI
                val errorMessage = e.message ?: e.toString()
                Result.failure(LocalDbError.QueryFailed(errorMessage))
            }
        }
    }

    /**
     * Executes a block of database operations within a single transaction.
     * If a transaction is already active in the current coroutine context, the block
     * is executed directly within that transaction (re-entrant).
     */
    suspend fun <T> withTransaction(block: suspend () -> T): Result<T> {
        val transactionElement = coroutineContext[DbTransactionElement]
        if (transactionElement != null) {
            return try {
                Result.success(block())
            } catch (e: Exception) {
                Result.failure(e)
            }
        }

        val currentDb = db ?: return Result.failure(LocalDbError.NotReady)
        return dbMutex.withLock {
            try {
                currentDb.beginAllOrNothing()
                val result = kotlinx.coroutines.withContext(DbTransactionElement(currentDb)) {
                    block()
                }
                currentDb.everythingWentPerfectly()
                Result.success(result)
            } catch (e: Exception) {
                try {
                    currentDb.regretEverything()
                } catch (_: Exception) {
                }
                Result.failure(e)
            }
        }
    }

    // Call this once from MainApplication (runs on background thread)
    suspend fun init(context: Context) {
        if (db == null) {
            val dbPath = context.getDatabasePath("my_database.db").absolutePath
            val newDb = LiveForever(dbPath)
            
            db = newDb

            withTransaction<Unit> {
                // Initialize storage-related data first (handles its own table creation)
                StorageInitializer.create_all_these_things_if_they_dont_exist()

                // Initialize Independent Tables
                executeNative { it.createTable(CreateTableIn("pages", pagesColumns())) }.getOrThrow()
                executeNative { it.createTable(CreateTableIn("uncommitted_diffs", uncommittedDiffsColumns())) }.getOrThrow()

                // Initialize Foreign Key Tables
                // TODO: createForeignTable is missing from the updated Kotlin bindings. 
                // Commented out to allow compilation. Verify if this method was renamed or removed.
                /*
                executeNative { it.createForeignTable(CreateForeignTableIn("backlinks", backlinksColumns(), getForeignDefBacklinks())) }.getOrThrow()
                executeNative { it.createForeignTable(CreateForeignTableIn("every_block_in_existence", everyBlockInExistenceColumns(), getForeignDefEveryBlockInExistence())) }.getOrThrow()
                */
            }
            
            readyDeferred.complete(Unit)
        }
    }

    // --- API Methods ---

    suspend fun listTables(): Result<ListTablesOut> = executeNative { it.listTables() }

    suspend fun checkTable(input: CheckTableIn): Result<CheckTableOut> = executeNative { it.checkTable(input) }

    suspend fun getData(input: GetDataIn): Result<GetDataOut> = executeNative { it.getData(input) }

    suspend fun insertData(input: InsertDataIn): Result<Unit> = executeNative { it.insertData(input) }

    suspend fun deleteRow(input: DeleteRowIn): Result<Unit> = executeNative { it.deleteRow(input) }

    suspend fun editColInRow(input: EditColInRowIn): Result<Unit> = executeNative { it.editColInRow(input) }

    /**
     * Specific helper to fetch all pages.
     */
    suspend fun getPages(): Result<List<Row>> = executeNative {
        it.getData(GetDataIn("pages", listOf(SelectArgument.All), emptyList())).rows
    }

    /**
     * Specific helper to fetch a single page.
     */
    suspend fun getPage(pageId: String): Result<Row> = executeNative { db ->
        val result = db.getData(GetDataIn(
            "pages",
            listOf(SelectArgument.XEqualY("page_id", pageId, null)),
            emptyList()
        ))
        result.rows.firstOrNull() ?: throw Exception("Page not found: $pageId")
    }

    /**
     * Update the binary snapshot for a page.
     */
    suspend fun updatePageSnapshot(pageId: String, snapshot: ByteArray): Result<Unit> = executeNative { db ->
        // We first need the internal 'id' to use editColInRow
        val pageRow = getPageSync(db, pageId)
        val internalId = when (val idCol = pageRow.cols.firstOrNull()) {
            is Col.Integer -> idCol.v1.toString()
            is Col.Text -> idCol.v1
            else -> throw Exception("Could not determine internal ID for page $pageId")
        }
        
        db.editColInRow(EditColInRowIn(
            tableName = "pages",
            rowId = internalId,
            column = "blobbed_page",
            newValue = Col.Blob(snapshot)
        ))
    }

    /**
     * Internal sync helper to avoid nested executeNative calls which would deadlock the Mutex.
     */
    private fun getPageSync(db: LiveForever, pageId: String): Row {
        val result = db.getData(GetDataIn(
            "pages",
            listOf(SelectArgument.XEqualY("page_id", pageId, null)),
            emptyList()
        ))
        return result.rows.firstOrNull() ?: throw Exception("Page not found: $pageId")
    }

    /**
     * Proper way to add a new page using Yrs initial snapshots from blueprints.
     */
    suspend fun addPage(pageId: String, isMainMenu: Boolean = false): Result<Unit> {
        val userId = z.zndroid.Storage.StorageAccess.getUserId()
        return executeNative { db ->
            val row = newPageRow(pageId, isMainMenu, userId)
            
            val values = SafeRowMapper.mapRow(
                row = row,
                columnDefs = pagesColumns(),
                expectedNames = listOf("page_id", "is_main_menu_page", "user_id")
            )

            db.insertData(InsertDataIn("pages", values))
        }
    }
}
