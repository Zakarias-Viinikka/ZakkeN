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

/**
 * Retries a database operation until it succeeds or fails with a non-recoverable error.
 * Specifically catches [LocalDbError.NotReady] and retries after a short delay.
 */
suspend fun <T> retryUntilReady(
    maxRetries: Int = 50,
    initialDelay: Long = 100,
    block: suspend () -> Result<T>
): Result<T> {
    var currentDelay = initialDelay
    repeat(maxRetries) {
        val result = block()
        if (result.isSuccess) {
            return result
        }
        
        val error = result.exceptionOrNull()
        if (error is LocalDbError.NotReady) {
            delay(currentDelay)
            // Optionally increase delay (exponential backoff)
            // currentDelay = (currentDelay * 1.5).toLong().coerceAtMost(1000)
        } else {
            // Non-recoverable error or unexpected exception
            return result
        }
    }
    return Result.failure(LocalDbError.NotReady)
}
