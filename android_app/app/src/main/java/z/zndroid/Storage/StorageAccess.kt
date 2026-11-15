package z.zndroid.Storage

import uniffi.protocol.*
import rustlib.client_table_blueprints.newTableKeyValueStorage
import rustlib.client_table_blueprints.newKeyValueItem
import z.zndroid.DbManager
import z.zndroid.protocol.SafeRowMapper

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

        // 2. Fallback to SQLite. Ask for only `value` so a later column
        // added to the table can't silently shift which position is read.
        val result = DbManager.getData(GetDataIn(
            "key_value_storage",
            SelectArguments.Single(SelectArgument.XEqualY("key", key.keyName)),
            listOf("value")
        ))

        return result.fold(
            onSuccess = { data ->
                val row = data.rows.firstOrNull()
                if (row == null) {
                    RummageResult.NotFound
                } else {
                    val valueCol = row.cols.firstOrNull()
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
        DbManager.withTransaction {
            val existing = DbManager.getData(GetDataIn(
                "key_value_storage",
                SelectArguments.Single(SelectArgument.XEqualY("key", key.keyName)),
                emptyList()
            )).getOrThrow()

            if (existing.rows.firstOrNull() != null) {
                DbManager.editColInRowWhere(EditColInRowWhereIn(
                    tableName = "key_value_storage",
                    whereClause = SelectArguments.Single(
                        SelectArgument.XEqualY("key", key.keyName)
                    ),
                    column = "value",
                    newValue = Col.Text(value)
                )).getOrThrow()
            } else {
                val row = newKeyValueItem(key.keyName, value)
                val values = SafeRowMapper.mapRow(
                    row = row,
                    columnDefs = newTableKeyValueStorage(),
                    expectedNames = listOf("key", "value"),
                    skipId = true
                )
                DbManager.insertData(InsertDataIn("key_value_storage", values)).getOrThrow()
            }
        }
        // Only update the memory cache after the DB write succeeded.
        if (FastStorageKey.fromStorageKey(key) != null) {
            FastStorage.set(key, value)
        }
    }
}
