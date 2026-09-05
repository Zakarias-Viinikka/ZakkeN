package z.zndroid.Storage

import rustlib.client_table_blueprints.keyValueStorageColumns
import rustlib.client_table_blueprints.newKeyValueItem
import uniffi.protocol.ColumnValue
import uniffi.protocol.CreateTableIn
import uniffi.protocol.InsertDataIn
import z.zndroid.DbManager
import java.util.UUID

/**
 * Handles the initialization of storage-related tables and default data.
 */
object StorageInitializer {

    /**
     * Ensures all necessary storage tables exist and initial values are populated.
     */
    fun create_all_these_things_if_they_dont_exist() {
        // 1. Create the table if it doesn't exist
        DbManager.executeNative { 
            it.createTable(CreateTableIn("key_value_storage", keyValueStorageColumns()))
        }

        // 2. Ensure the default records exist
        ensureUserIdExists()
    }

    private fun ensureUserIdExists() {
        when (val res = StorageAccess.rummage_in_storage(StorageKey.USER_ID)) {
            is RummageResult.StringValue -> {
                // Already exists, nothing to do
            }
            is RummageResult.NotFound, is RummageResult.Error -> {
                // Generate and store new User ID
                val newId = UUID.randomUUID().toString()
                val row = newKeyValueItem("user_id", newId)
                val columnDefs = keyValueStorageColumns()
                
                // Map columns directly (No ID column to skip)
                val values = row.cols.mapIndexed { index, col ->
                    ColumnValue(columnDefs[index].name, col)
                }
                
                DbManager.insertData(InsertDataIn("key_value_storage", values))
            }
        }
    }
}
