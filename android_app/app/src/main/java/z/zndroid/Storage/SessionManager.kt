package z.zndroid.Storage

import z.zndroid.DbManager

/**
 * Manages a persistent session ID that increments when the app is opened
 * or when the user navigates between major screens.
 */
object SessionManager {
    private var _currentSessionId: Long = 0
    
    /**
     * The current session ID as a String, for use in DB entries.
     */
    val currentSessionId: String 
        get() = _currentSessionId.toString()

    /**
     * Initializes the session by reading the last stored ID and incrementing it.
     */
    fun initialize() {
        val stored = when (val res = StorageAccess.rummage_in_storage(StorageKey.SESSION_ID)) {
            is RummageResult.StringValue -> res.value.toLongOrNull() ?: 0L
            else -> 0L
        }
        _currentSessionId = stored
        incrementAndStore()
    }

    /**
     * Increments the session counter and persists it to the key-value storage.
     */
    fun incrementAndStore() {
        _currentSessionId++
        StorageAccess.setValue(StorageKey.SESSION_ID, _currentSessionId.toString())
    }
}
