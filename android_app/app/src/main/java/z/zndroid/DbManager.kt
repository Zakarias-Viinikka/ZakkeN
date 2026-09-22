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

            // Execute migrations or bootstrap via MigrationManager
            z.zndroid.db.migrations.MigrationManager.migrate(newDb)

            // Ensure storage initial default records exist (e.g. user_id)
            // StorageInitializer will now just populate records since tables are created by MigrationManager
            StorageInitializer.create_all_these_things_if_they_dont_exist()
            
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

    suspend fun editColInRowWhere(input: EditColInRowWhereIn): Result<Unit> =
        executeNative { it.editColInRowWhere(input) }

    /**
     * Specific helper to fetch all pages.
     */
    suspend fun getPages(): Result<List<Row>> = executeNative {
        it.getData(GetDataIn("pages", SelectArguments.Single(SelectArgument.All), emptyList())).rows
    }

    /**
     * Specific helper to fetch a single page.
     */
    suspend fun getPage(pageId: String): Result<Row> = executeNative { db ->
        val result = db.getData(GetDataIn(
            "pages",
            SelectArguments.Single(SelectArgument.XEqualY("page_id", pageId)),
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
            SelectArguments.Single(SelectArgument.XEqualY("page_id", pageId)),
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
                expectedNames = listOf("page_id", "blobbed_page", "page_status", "version", "is_main_menu_page")
            )

            db.insertData(InsertDataIn("pages", values))
        }
    }

    /**
     * Renames a page: updates pages.page_id and cascades to every_block_in_existence and backlinks.
     * Runs as a single transaction.
     */
    suspend fun renamePage(oldPageId: String, newPageId: String): Result<Unit> = withTransaction {
        val pagesQuery = DbManager.getData(GetDataIn(
            "pages",
            SelectArguments.Single(SelectArgument.XEqualY("page_id", oldPageId)),
            listOf("id")
        )).getOrThrow()
        val pageRowId = (pagesQuery.rows.firstOrNull()?.cols?.firstOrNull() as? Col.Integer)?.v1?.toString()
            ?: throw Exception("Page not found: $oldPageId")

        DbManager.editColInRow(EditColInRowIn(
            "pages", pageRowId, "page_id", Col.Text(newPageId)
        )).getOrThrow()

        DbManager.editColInRowWhere(EditColInRowWhereIn(
            "every_block_in_existence",
            SelectArguments.Single(SelectArgument.XEqualY("id_of_page_i_belong_to", oldPageId)),
            "id_of_page_i_belong_to",
            Col.Text(newPageId)
        )).getOrThrow()

        DbManager.editColInRowWhere(EditColInRowWhereIn(
            "backlinks",
            SelectArguments.Single(SelectArgument.XEqualY("page_that_holds_link_id", oldPageId)),
            "page_that_holds_link_id",
            Col.Text(newPageId)
        )).getOrThrow()

        DbManager.editColInRowWhere(EditColInRowWhereIn(
            "backlinks",
            SelectArguments.Single(SelectArgument.XEqualY("page_being_linked_to_id", oldPageId)),
            "page_being_linked_to_id",
            Col.Text(newPageId)
        )).getOrThrow()
    }

    /**
     * Deletes a page and all associated rows (blocks, backlinks).
     * Runs as a single transaction.
     */
    suspend fun deletePage(pageId: String): Result<Unit> = withTransaction {
        deleteRowsByColumn("every_block_in_existence", "id_of_page_i_belong_to", pageId)
        deleteRowsByColumn("backlinks", "page_that_holds_link_id", pageId)
        deleteRowsByColumn("backlinks", "page_being_linked_to_id", pageId)
        deleteRowsByColumn("pages", "page_id", pageId)
    }

    private suspend fun deleteRowsByColumn(table: String, column: String, value: String) {
        val rows = DbManager.getData(GetDataIn(
            table,
            SelectArguments.Single(SelectArgument.XEqualY(column, value)),
            listOf("id")
        )).getOrThrow().rows
        for (row in rows) {
            val id = (row.cols.firstOrNull() as? Col.Integer)?.v1?.toString() ?: continue
            DbManager.deleteRow(DeleteRowIn(table, id)).getOrThrow()
        }
    }
}
