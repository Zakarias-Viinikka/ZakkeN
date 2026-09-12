package z.zndroid.Storage

import uniffi.protocol.*
import rustlib.client_table_blueprints.keyValueStorageColumns
import rustlib.client_table_blueprints.newKeyValueItem
import z.zndroid.DbManager

/**
 * Provides methods for accessing values from the key-value storage.
 */
object StorageAccess {

    /**
     * Retrieves a value from the storage based on the provided [StorageKey].
     * Callers must match against the returned [RummageResult] to handle the different outcomes.
     */
    suspend fun rummage_in_storage(key: StorageKey): RummageResult {
        // 1. Check if this is a FastStorage-enabled key
        val fastKey = FastStorageKey.fromStorageKey(key)
        if (fastKey != null) {
            val inMemoryValue = FastStorage.get(key)
            if (inMemoryValue != null) {
                return RummageResult.StringValue(inMemoryValue)
            }
        }

        // 2. Fallback to SQLite
        val result = DbManager.getData(GetDataIn(
            "key_value_storage",
            listOf(SelectArgument.XEqualY("key", key.keyName, null)),
            emptyList()
        ))
        
        return result.fold(
            onSuccess = { data ->
                val row = data.rows.firstOrNull()
                if (row == null) {
                    RummageResult.NotFound
                } else {
                    // In key_value_storage: key (0), value (1)
                    val valueCol = row.cols.getOrNull(1) 
                    if (valueCol is Col.Text) {
                        RummageResult.StringValue(valueCol.v1)
                    } else {
                        RummageResult.Error("Value for key '${key.keyName}' is not a string")
                    }
                }
            },
            onFailure = { error ->
                RummageResult.Error(error.message ?: error.toString())
            }
        )
    }

    /**
     * Helper to specifically retrieve the User ID as a String.
     * Use [rummage_in_storage] if you need more granular error handling.
     */
    suspend fun getUserId(): String {
        return when (val res = rummage_in_storage(StorageKey.USER_ID)) {
            is RummageResult.StringValue -> res.value
            else -> ""
        }
    }

    /**
     * Persists or updates a value in the key-value storage for a given [StorageKey].
     */
    suspend fun setValue(key: StorageKey, value: String) {
        // Update memory cache if applicable
        if (FastStorageKey.fromStorageKey(key) != null) {
            FastStorage.set(key, value)
        }

        DbManager.withTransaction {
            val current = rummage_in_storage(key)
            if (current is RummageResult.StringValue) {
                DbManager.editColInRow(EditColInRowIn(
                    tableName = "key_value_storage",
                    rowId = key.keyName,
                    column = "value",
                    newValue = Col.Text(value)
                )).getOrThrow()
            } else {
                val row = newKeyValueItem(key.keyName, value)
                val columnDefs = keyValueStorageColumns()
                val values = row.cols.mapIndexed { index, col ->
                    ColumnValue(columnDefs[index].name, col)
                }
                DbManager.insertData(InsertDataIn("key_value_storage", values)).getOrThrow()
            }
        }
    }
}
