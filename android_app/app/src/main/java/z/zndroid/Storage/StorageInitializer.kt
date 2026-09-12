package z.zndroid.Storage

import rustlib.client_table_blueprints.keyValueStorageColumns
import rustlib.client_table_blueprints.newKeyValueItem
import uniffi.protocol.ColumnValue
import uniffi.protocol.CreateTableIn
import uniffi.protocol.InsertDataIn
import z.zndroid.DbManager
import z.zndroid.protocol.SafeRowMapper
import java.util.UUID

/**
 * Handles the initialization of storage-related tables and default data.
 */
object StorageInitializer {

    /**
     * Ensures all necessary storage tables exist and initial values are populated.
     */
    suspend fun create_all_these_things_if_they_dont_exist() {
        DbManager.withTransaction {
            // 1. Create the table if it doesn't exist
            DbManager.executeNative { 
                it.createTable(CreateTableIn("key_value_storage", keyValueStorageColumns()))
            }.getOrThrow()

            // 2. Ensure the default records exist
            ensureUserIdExists()
        }
    }

    private suspend fun ensureUserIdExists() {
        when (val res = StorageAccess.rummage_in_storage(StorageKey.USER_ID)) {
            is RummageResult.StringValue -> {
                // Already exists, nothing to do
            }
            is RummageResult.NotFound, is RummageResult.Error -> {
                // Generate and store new User ID
                val newId = UUID.randomUUID().toString()
                val row = newKeyValueItem("user_id", newId)
                
                val values = SafeRowMapper.mapRow(
                    row = row,
                    columnDefs = keyValueStorageColumns(),
                    expectedNames = listOf("key", "value"),
                    skipId = false // No auto-increment ID in this table
                )
                
                DbManager.insertData(InsertDataIn("key_value_storage", values))
            }
        }
    }
}
