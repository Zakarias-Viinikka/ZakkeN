package z.zndroid.log

import android.util.Log
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import z.zndroid.DbManager
import z.zndroid.Storage.SessionManager

/**
 * Core logging engine for Zndroid.
 * Writes to Logcat and the logs SQLite table.
 *
 * - Outside a transaction: fire-and-forget on [logScope]. Failures are
 *   reported to Logcat, not rethrown, so a broken logs table cannot take
 *   down the caller.
 * - Inside a transaction: the entry is appended to the transaction's
 *   [DbManager.DbTransactionElement.logBuffer]. Commit drops it; rollback
 *   flushes it to the DB outside the failed transaction.
 *
 * The transaction is found through the coroutine context, not a thread-
 * local, so a switch of threads inside a transaction still routes logs to
 * the right buffer. All entry points are `suspend` for that reason.
 */
object Logger {
    private val logScope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    private const val TAG = "ZndroidLog"

    suspend fun i(message: String, category: String = "GENERAL", source: String = "Logger") {
        log(level = "INFO", category = category, source = source, message = message)
    }

    suspend fun w(message: String, category: String = "GENERAL", source: String = "Logger") {
        log(level = "WARN", category = category, source = source, message = message)
    }

    suspend fun e(
        message: String,
        category: String = "GENERAL",
        source: String = "Logger",
        throwable: Throwable? = null
    ) {
        val fullMessage = if (throwable != null) {
            "$message | Error: ${throwable.message}\n${Log.getStackTraceString(throwable)}"
        } else {
            message
        }
        log(level = "ERROR", category = category, source = source, message = fullMessage)
    }

    suspend fun log(level: String, category: String, source: String, message: String) {
        val formatted = "[$level][$category][$source] $message"
        when (level) {
            "INFO" -> Log.i(TAG, formatted)
            "WARN" -> Log.w(TAG, formatted)
            "ERROR" -> Log.e(TAG, formatted)
        }

        val entry = LogEntry(
            timestamp = System.currentTimeMillis(),
            level = level,
            category = category,
            source = source,
            sessionId = SessionManager.currentSessionId,
            message = message
        )

        val txElement = kotlin.coroutines.coroutineContext[DbManager.DbTransactionElement]
        if (txElement != null) {
            txElement.logBuffer.add(entry)
            return
        }

        logScope.launch {
            val result = DbManager.writeLogEntry(entry)
            result.onFailure { err ->
                // Logcat only — recursing into Logger here would risk
                // an infinite loop if the DB is the thing that's broken.
                Log.e(TAG, "Failed to persist log entry: ${err.message}", err)
            }
        }
    }
}