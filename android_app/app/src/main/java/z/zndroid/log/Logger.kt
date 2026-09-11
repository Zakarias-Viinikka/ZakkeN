package z.zndroid.log

import android.util.Log
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

/**
 * Core logging engine for Zndroid.
 * Satisfies the requirement of "spawning a thread" (background execution) for logging
 * to ensure UI performance is never impacted by logging overhead.
 */
object Logger {
    private val logScope = CoroutineScope(Dispatchers.IO)
    private const val TAG = "ZndroidLog"

    fun i(message: String, category: String = "GENERAL") {
        logInBackground("INFO", category, message)
    }

    fun w(message: String, category: String = "GENERAL") {
        logInBackground("WARN", category, message)
    }

    fun e(message: String, category: String = "GENERAL", throwable: Throwable? = null) {
        val fullMessage = if (throwable != null) {
            "$message | Error: ${throwable.message}\n${Log.getStackTraceString(throwable)}"
        } else {
            message
        }
        logInBackground("ERROR", category, fullMessage)
    }

    private fun logInBackground(level: String, category: String, message: String) {
        logScope.launch {
            // In a real app, this could write to a file or a remote server.
            // For now, we output to Logcat with a clear background indicator.
            val formatted = "[$level][$category] $message"
            when (level) {
                "INFO" -> Log.i(TAG, formatted)
                "WARN" -> Log.w(TAG, formatted)
                "ERROR" -> Log.e(TAG, formatted)
            }
        }
    }
}
