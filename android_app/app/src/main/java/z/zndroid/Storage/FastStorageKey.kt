package z.zndroid.Storage

/**
 * Defines which [StorageKey]s are permitted to be stored in the in-memory FastStorage.
 */
enum class FastStorageKey(val storageKey: StorageKey) {
    SESSION(StorageKey.SESSION_ID),
    CURRENT_DOC(StorageKey.CURRENT_DOCUMENT);

    companion object {
        /**
         * Returns the [FastStorageKey] associated with a [StorageKey], if one exists.
         */
        fun fromStorageKey(key: StorageKey): FastStorageKey? {
            return entries.find { it.storageKey == key }
        }
    }
}