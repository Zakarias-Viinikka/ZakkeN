package z.zndroid

import android.content.Context
import com.z_db.android_mascot.LiveForever
import uniffi.protocol.*
import rustlib.client_table_blueprints.*
import z.zndroid.Storage.StorageInitializer
import java.util.UUID

object DbManager {
    // Hidden database instance
    private var db: LiveForever? = null

    /**
     * Internal helper to execute database calls with safety checks.
     * Returns a Result containing the output or a LocalDbError.
     */
    internal inline fun <T> executeNative(block: (LiveForever) -> T): Result<T> {
        val currentDb = db ?: return Result.failure(LocalDbError.NotReady)
        return try {
            Result.success(block(currentDb))
        } catch (e: Exception) {
            // Use e.toString() if message is null to ensure we don't return "null" to the UI
            val errorMessage = e.message ?: e.toString()
            Result.failure(LocalDbError.QueryFailed(errorMessage))
        }
    }

    // Call this once from MainApplication (runs on background thread)
    fun init(context: Context) {
        if (db == null) {
            val dbPath = context.getDatabasePath("my_database.db").absolutePath
            val newDb = LiveForever(dbPath)
            
            db = newDb

            // Initialize storage-related data first (handles its own table creation)
            StorageInitializer.create_all_these_things_if_they_dont_exist()

            // Initialize Independent Tables
            newDb.createTable(CreateTableIn("pages", pagesColumns()))
            newDb.createTable(CreateTableIn("uncommitted_diffs", uncommittedDiffsColumns()))

            // Initialize Foreign Key Tables
            newDb.createForeignTable(CreateForeignTableIn("backlinks", backlinksColumns(), getForeignDefBacklinks()))
            newDb.createForeignTable(CreateForeignTableIn("every_block_in_existence", everyBlockInExistenceColumns(), getForeignDefEveryBlockInExistence()))
        }
    }

    // --- API Methods ---

    fun listTables(): Result<ListTablesOut> = executeNative { it.listTables() }

    fun checkTable(input: CheckTableIn): Result<CheckTableOut> = executeNative { it.checkTable(input) }

    fun getData(input: GetDataIn): Result<GetDataOut> = executeNative { it.getData(input) }

    fun insertData(input: InsertDataIn): Result<Unit> = executeNative { it.insertData(input) }

    fun deleteRow(input: DeleteRowIn): Result<Unit> = executeNative { it.deleteRow(input) }

    fun editColInRow(input: EditColInRowIn): Result<Unit> = executeNative { it.editColInRow(input) }

    /**
     * Specific helper to fetch all pages.
     */
    fun getPages(): Result<List<Row>> = executeNative {
        it.getData(GetDataIn("pages", listOf(SelectArgument.All), emptyList())).rows
    }

    /**
     * Specific helper to fetch a single page.
     */
    fun getPage(pageId: String): Result<Row> = executeNative { db ->
        val result = db.getData(GetDataIn(
            "pages",
            listOf(SelectArgument.XEqualY("page_id", pageId)),
            emptyList()
        ))
        result.rows.firstOrNull() ?: throw Exception("Page not found: $pageId")
    }

    /**
     * Update the binary snapshot for a page.
     */
    fun updatePageSnapshot(pageId: String, snapshot: ByteArray): Result<Unit> = executeNative { db ->
        // We first need the internal 'id' to use editColInRow
        val pageRow = getPage(pageId).getOrThrow()
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
     * Proper way to add a new page using Yrs initial snapshots from blueprints.
     */
    fun addPage(pageId: String, isMainMenu: Boolean = false): Result<Unit> = executeNative { db ->
        val userId = z.zndroid.Storage.StorageAccess.getUserId()
        val row = newPageRow(pageId, isMainMenu, userId)
        val columnDefs = pagesColumns()

        // Map columns correctly (skip 'id')
        val values = row.cols.mapIndexed { index, col ->
            ColumnValue(columnDefs[index + 1].name, col)
        }

        db.insertData(InsertDataIn("pages", values))
    }
}
