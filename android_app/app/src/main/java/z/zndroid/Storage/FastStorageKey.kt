package z.zndroid.Storage

/**
 * Placeholder for future logic describing when a value should be refreshed.
 */
enum class RefreshCondition {
    NEVER,
    ON_PAGE_LOAD
}

/**
 * Placeholder for future logic describing when a value should be dropped from memory.
 */
enum class DropCondition {
    NEVER,
    ON_SCREEN_EXIT
}

/**
 * Defines which [StorageKey]s are permitted to be stored in the in-memory FastStorage.
 * Each entry includes metadata about cache eviction and refresh policies.
 */
enum class FastStorageKey(
    val storageKey: StorageKey,
    val refreshWhen: RefreshCondition,
    val dropWhen: DropCondition
) {
    /**
     * Cache the session ID as it is accessed frequently during sync operations.
     */
    SESSION(
        StorageKey.SESSION_ID,
        RefreshCondition.NEVER,
        DropCondition.NEVER
    );

    companion object {
        /**
         * Returns the [FastStorageKey] associated with a [StorageKey], if one exists.
         */
        fun fromStorageKey(key: StorageKey): FastStorageKey? {
            return entries.find { it.storageKey == key }
        }
    }
}
