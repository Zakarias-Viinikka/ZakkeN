package z.zndroid.Storage

/**
 * Enumeration of all keys used in the application's key-value storage.
 * Each member represents a specific persistent setting or identity.
 */
enum class StorageKey(val keyName: String) {
    /**
     * The unique identifier for the current user.
     * Generated on the first app launch and used for all Yrs document operations
     * to ensure consistent identity across sync sessions.
     */
    USER_ID("user_id")
}
