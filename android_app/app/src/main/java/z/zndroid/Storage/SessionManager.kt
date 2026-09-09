package z.zndroid.Storage

import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import java.util.concurrent.atomic.AtomicLong

/**
 * Manages a persistent session ID that increments when the app is opened
 * or when the user navigates between major screens.
 */
object SessionManager {
    private val _currentSessionId = AtomicLong(0)
    private val readyDeferred = CompletableDeferred<Unit>()
    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    
    /**
     * The current session ID as a String, for use in DB entries.
     */
    val currentSessionId: String 
        get() = _currentSessionId.get().toString()

    /**
     * Initializes the session by reading the last stored ID and incrementing it.
     */
    suspend fun initialize() {
        val stored = when (val res = StorageAccess.rummage_in_storage(StorageKey.SESSION_ID)) {
            is RummageResult.StringValue -> res.value.toLongOrNull() ?: 0L
            else -> 0L
        }
        _currentSessionId.set(stored)
        readyDeferred.complete(Unit)
        
        // Initial increment for app startup
        incrementAndStore()
    }

    /**
     * Increments the session counter and persists it to the key-value storage.
     * Thread-safe and handles initialization order by waiting for [initialize] to complete.
     */
    fun incrementAndStore() {
        scope.launch {
            // Ensure initialize() has finished reading the DB before we increment
            readyDeferred.await()
            
            val newVal = _currentSessionId.incrementAndGet()
            StorageAccess.setValue(StorageKey.SESSION_ID, newVal.toString())
        }
    }
}
