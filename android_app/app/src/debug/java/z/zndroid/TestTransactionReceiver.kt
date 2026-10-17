package z.zndroid

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.util.Log
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

/**
 * Debug-only. Called by the e2e test runner over `adb shell am broadcast`.
 * Opens a DB transaction and holds it open across the whole test run, so a
 * failure or rollback discards every write the app made during the test.
 */
class TestTransactionReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        val action = intent.action ?: return
        val pending = goAsync()
        CoroutineScope(Dispatchers.IO).launch {
            try {
                DbManager.awaitReady()
                if (action == ACTION_CHECK_SYNC) {
                    val pageId = intent.getStringExtra("pageId")
                    if (pageId.isNullOrBlank()) {
                        Log.e(TAG, "SyncCheck pageId missing")
                        return@launch
                    }
                    z.zndroid.DocEvents.CheckIfTablesInSync.checkPageSync(pageId).fold(
                        onSuccess = { comparisons ->
                            val bad = comparisons.filter { !it.isMatch }
                            Log.i(TAG, "SyncCheck page=$pageId synced=${bad.isEmpty()} total=${comparisons.size} mismatches=${bad.size}")
                            bad.take(5).forEach { c ->
                                Log.i(TAG, "SyncCheckMismatch page=$pageId yrs=${c.yrsId} sqlite=${c.sqliteContent} crdt=${c.crdtContent}")
                            }
                        },
                        onFailure = { e ->
                            Log.e(TAG, "SyncCheck page=$pageId error=${e.message}")
                        }
                    )
                    return@launch
                }
                val result = when (action) {
                    ACTION_BEGIN -> DbManager.beginExternalTransaction()
                    ACTION_END_ROLLBACK -> DbManager.endExternalTransaction(commit = false)
                    ACTION_END_COMMIT -> DbManager.endExternalTransaction(commit = true)
                    else -> return@launch
                }
                if (result.isSuccess) {
                    Log.i(TAG, "$action -> OK")
                } else {
                    Log.e(TAG, "$action -> FAIL: ${result.exceptionOrNull()?.message}")
                }
            } catch (e: Exception) {
                Log.e(TAG, "$action threw", e)
            } finally {
                pending.finish()
            }
        }
    }

    companion object {
        private const val TAG = "TestTransactionReceiver"
        const val ACTION_BEGIN = "z.zndroid.debug.BEGIN_TEST_TX"
        const val ACTION_END_ROLLBACK = "z.zndroid.debug.END_TEST_TX_ROLLBACK"
        const val ACTION_END_COMMIT = "z.zndroid.debug.END_TEST_TX_COMMIT"
        const val ACTION_CHECK_SYNC = "z.zndroid.debug.CHECK_SYNC"
    }
}
