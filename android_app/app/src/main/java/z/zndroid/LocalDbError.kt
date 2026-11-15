package z.zndroid

/**
 * Custom error types for database operations, similar to Rust's Result enum.
 */
sealed class LocalDbError(message: String? = null) : Exception(message) {
    object NotReady : LocalDbError("Database is not initialized yet")
    data class QueryFailed(val reason: String) : LocalDbError(reason)
    data class DatabaseDowngraded(val current: Int, val maxKnown: Int) :
        LocalDbError("Database version ($current) is newer than what this app build supports ($maxKnown)")
}
