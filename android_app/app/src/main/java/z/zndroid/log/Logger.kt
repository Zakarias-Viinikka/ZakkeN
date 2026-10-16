package z.zndroid.log

import android.util.Log
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import z.zndroid.DbManager
import z.zndroid.Storage.SessionManager

/**
 * Core logging engine for Zndroid.
 * Satisfies the requirement of background execution for logging
 * and persisting logs to SQLite while respecting transaction boundaries.
 */
object Logger {
    private val logScope = CoroutineScope(Dispatchers.IO)
    private const val TAG = "ZndroidLog"

    fun i(message: String, category: String = "GENERAL", source: String = "Logger") {
        log(level = "INFO", category = category, source = source, message = message)
    }

    fun w(message: String, category: String = "GENERAL", source: String = "Logger") {
        log(level = "WARN", category = category, source = source, message = message)
    }

    fun e(
        message: String,
        category: String = "GENERAL",
        source: String = "Logger",
        throwable: Throwable? = null,
        details: ByteArray? = null,
        detailsType: String? = null
    ) {
        val fullMessage = if (throwable != null) {
            "$message | Error: ${throwable.message}\n${Log.getStackTraceString(throwable)}"
        } else {
            message
        }
        log(
            level = "ERROR",
            category = category,
            source = source,
            message = fullMessage,
            details = details,
            detailsType = detailsType
        )
    }

    fun log(
        level: String,
        category: String,
        source: String,
        message: String,
        details: ByteArray? = null,
        detailsType: String? = null
    ) {
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
            message = message,
            details = details,
            detailsType = detailsType
        )

        val txElement = DbManager.activeTransactionThreadLocal.get()
        if (txElement != null) {
            txElement.logBuffer.add(entry)
        } else {
            logScope.launch {
                DbManager.writeLogEntry(entry)
            }
        }
    }
}
