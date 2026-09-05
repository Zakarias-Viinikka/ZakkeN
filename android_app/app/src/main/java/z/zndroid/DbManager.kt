package z.zndroid

import android.content.Context
import com.z_db.android_mascot.LiveForever
import uniffi.protocol.*
import rustlib.client_table_blueprints.*

object DbManager {
    // Hidden database instance
    private var db: LiveForever? = null

    /**
     * Internal helper to execute database calls with safety checks.
     * Returns a Result containing the output or a LocalDbError.
     */
    private inline fun <T> execute(block: (LiveForever) -> T): Result<T> {
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
            
            // Initialize Independent Tables
            newDb.createTable(CreateTableIn("pages", pagesColumns()))
            newDb.createTable(CreateTableIn("uncommitted_diffs", uncommittedDiffsColumns()))

            // Initialize Foreign Key Tables
            newDb.createForeignTable(CreateForeignTableIn("backlinks", backlinksColumns(), getForeignDefBacklinks()))
            newDb.createForeignTable(CreateForeignTableIn("every_block_in_existence", everyBlockInExistenceColumns(), getForeignDefEveryBlockInExistence()))
            
            db = newDb
        }
    }

    // --- API Methods ---

    fun listTables(): Result<ListTablesOut> = execute { it.listTables() }

    fun checkTable(input: CheckTableIn): Result<CheckTableOut> = execute { it.checkTable(input) }

    fun getData(input: GetDataIn): Result<GetDataOut> = execute { it.getData(input) }

    fun insertData(input: InsertDataIn): Result<Unit> = execute { it.insertData(input) }

    fun deleteRow(input: DeleteRowIn): Result<Unit> = execute { it.deleteRow(input) }

    /**
     * Specific helper to fetch all pages.
     */
    fun getPages(): Result<List<Row>> = execute {
        it.getData(GetDataIn("pages", listOf(SelectArgument.All), emptyList())).rows
    }

    /**
     * Proper way to add a new page using Yrs initial snapshots from blueprints.
     */
    fun addPage(pageId: String, isMainMenu: Boolean = false): Result<Unit> = execute { db ->
        val row = newPageRow(pageId, isMainMenu)
        val columnDefs = pagesColumns()

        // Skip index 0 because it's the auto-increment 'id' column
        val values = row.cols.mapIndexed { index, col ->
            ColumnValue(columnDefs[index + 1].name, col)
        }

        db.insertData(InsertDataIn("pages", values))
    }
}
