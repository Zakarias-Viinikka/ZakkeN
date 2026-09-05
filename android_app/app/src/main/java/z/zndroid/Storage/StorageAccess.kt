package z.zndroid.Storage

import uniffi.protocol.Col
import uniffi.protocol.GetDataIn
import uniffi.protocol.SelectArgument
import z.zndroid.DbManager

/**
 * Provides methods for accessing values from the key-value storage.
 */
object StorageAccess {

    /**
     * Retrieves a value from the storage based on the provided [StorageKey].
     * Callers must match against the returned [RummageResult] to handle the different outcomes.
     */
    fun rummage_in_storage(key: StorageKey): RummageResult {
        val result = DbManager.getData(GetDataIn(
            "key_value_storage",
            listOf(SelectArgument.XEqualY("key", key.keyName)),
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
    fun getUserId(): String {
        return when (val res = rummage_in_storage(StorageKey.USER_ID)) {
            is RummageResult.StringValue -> res.value
            else -> ""
        }
    }
}
