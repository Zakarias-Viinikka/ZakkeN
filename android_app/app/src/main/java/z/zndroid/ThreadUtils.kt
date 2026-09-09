package z.zndroid

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext

/**
 * Custom error types for database operations, similar to Rust's Result enum.
 */
sealed class LocalDbError(message: String? = null) : Exception(message) {
    object NotReady : LocalDbError("Database is not initialized yet")
    data class QueryFailed(val reason: String) : LocalDbError(reason)
}

/**
 * Helper to run a block of code on a background (IO) thread.
 */
suspend fun <T> doInBackground(block: suspend () -> T): T {
    return withContext(Dispatchers.IO) {
        block()
    }
}

/**
 * Alias for doInBackground.
 */
suspend fun <T> io(block: suspend () -> T): T = doInBackground(block)

