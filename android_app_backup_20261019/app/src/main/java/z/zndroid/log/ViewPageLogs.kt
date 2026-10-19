package z.zndroid.log

/**
 * Logging helpers specifically for the ViewPage feature area.
 */
object ViewPageLogs {
    private const val CATEGORY = "VIEW_PAGE"
    private const val SOURCE = "ViewPageLogs"

    fun logPageInit(pageId: String) {
        Logger.i("Initializing ViewPage for page: $pageId", CATEGORY, SOURCE)
    }

    fun logPageLoadSuccess(pageId: String, blockCount: Int) {
        Logger.i("Successfully loaded $blockCount blocks for page: $pageId", CATEGORY, SOURCE)
    }

    fun logPageLoadError(pageId: String, error: String) {
        Logger.e("Failed to load page: $pageId | Error: $error", CATEGORY, SOURCE)
    }

    fun logBossInstanceError(pageId: String, error: String) {
        Logger.e("Failed to instance Yrs Boss for page: $pageId | Error: $error", CATEGORY, SOURCE)
    }

    fun logDocEventError(eventName: String, error: String) {
        Logger.e("Document Event [$eventName] failed | Error: $error", CATEGORY, SOURCE)
    }
}
