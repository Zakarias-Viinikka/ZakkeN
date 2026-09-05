package z.zndroid.Storage

/**
 * The possible results of rummaging through key-value storage.
 * Callers should use a 'when' expression to handle each case.
 */
sealed class RummageResult {
    /**
     * Found a string value for the requested key.
     */
    data class StringValue(val value: String) : RummageResult()

    /**
     * The key was not found in the storage table.
     */
    object NotFound : RummageResult()
    
    /**
     * An error occurred while accessing the database or processing the row.
     */
    data class Error(val message: String) : RummageResult()
}
