package z.zndroid.Storage

import java.util.concurrent.ConcurrentHashMap

/**
 * An in-memory equivalent of KeyValueStorage (SQLite).
 * Used for high-frequency access to specific keys to avoid SQLite overhead.
 */
object FastStorage {
    private val memoryStore = ConcurrentHashMap<StorageKey, String>()

    fun get(key: StorageKey): String? {
        return memoryStore[key]
    }

    fun set(key: StorageKey, value: String) {
        memoryStore[key] = value
    }
}