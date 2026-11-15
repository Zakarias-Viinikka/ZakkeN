package z.zndroid.Storage

import z.zndroid.components.GlobalPopupManager
import java.util.concurrent.atomic.AtomicLong

/**
 * Manages a persistent session ID. The id increments exactly once per
 * process start, on the first call to [initialize]. Navigation does not
 * bump it.
 */
object SessionManager {
    private val _currentSessionId = AtomicLong(0)

    /**
     * The current session ID as a String, for use in DB entries.
     */
    val currentSessionId: String
        get() = _currentSessionId.get().toString()

    /**
     * Reads the last stored id, sets [currentSessionId] to that id + 1,
     * and persists the new value. Call once per process start.
     *
     * A read error or an unparsable stored value does not reset the
     * counter: the stored row is left alone and the in-memory id stays
     * at 0, so ids already written to client_love_letter are not
     * re-issued by the next launch.
     */
    suspend fun initialize() {
        val stored: Long = when (val res = StorageAccess.rummage_in_storage(StorageKey.SESSION_ID)) {
            is RummageResult.StringValue -> {
                val parsed = res.value.toLongOrNull()
                if (parsed == null) {
                    GlobalPopupManager.showError(
                        "Session Error: stored session_id '${res.value}' is not a number"
                    )
                    return
                }
                parsed
            }
            is RummageResult.NotFound -> 0L
            is RummageResult.Error -> {
                GlobalPopupManager.showError(
                    "Session Error: could not read session_id: ${res.message}"
                )
                return
            }
        }

        val newVal = stored + 1
        _currentSessionId.set(newVal)
        try {
            StorageAccess.setValue(StorageKey.SESSION_ID, newVal.toString())
        } catch (e: Exception) {
            GlobalPopupManager.showError("Session Error: ${e.message ?: e.toString()}")
        }
    }
}