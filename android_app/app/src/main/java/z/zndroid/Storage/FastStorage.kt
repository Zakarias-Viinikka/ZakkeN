package z.zndroid.Storage

import java.util.concurrent.ConcurrentHashMap

/**
 * An in-memory equivalent of KeyValueStorage (SQLite).
 * Used for high-frequency access to specific keys to avoid SQLite overhead.
 */
object FastStorage {
    private val memoryStore = ConcurrentHashMap<StorageKey, String>()

    /**
     * Retrieves a value directly from memory.
     */
    fun get(key: StorageKey): String? {
        return memoryStore[key]
    }

    /**
     * Stores a value in memory.
     */
    fun set(key: StorageKey, value: String) {
        memoryStore[key] = value
    }

    /**
     * Removes a value from memory.
     */
    fun drop(key: StorageKey) {
        memoryStore.remove(key)
    }

    /**
     * Checks if a value exists in memory for the given key.
     */
    fun exists(key: StorageKey): Boolean {
        return memoryStore.containsKey(key)
    }
}
