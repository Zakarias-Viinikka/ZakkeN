package z.zndroid

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.util.Log
import androidx.core.app.NotificationCompat

/**
 * Debug-only. Posted to by the e2e test harness when a run finishes.
 * Silent notification (no sound, no vibration) so the human can see on the
 * phone that the run ended without having to check the terminal.
 */
class TestNotificationReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action != ACTION_NOTIFY) return
        val result = intent.getStringExtra("result") ?: "DONE"
        val summary = intent.getStringExtra("summary").orEmpty()

        val nm = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        ensureChannel(nm)

        val title = "Tests $result"
        val body = if (summary.isBlank()) "e2e run finished" else summary
        val notif: Notification = NotificationCompat.Builder(context, CHANNEL_ID)
            .setSmallIcon(android.R.drawable.stat_notify_more)
            .setContentTitle(title)
            .setContentText(body)
            .setStyle(NotificationCompat.BigTextStyle().bigText(body))
            .setPriority(NotificationCompat.PRIORITY_HIGH)
            .setAutoCancel(true)
            .setOnlyAlertOnce(true)
            .build()

        try {
            nm.notify(NOTIF_ID, notif)
            Log.i(TAG, "notified: $result — $body")
        } catch (e: Exception) {
            Log.e(TAG, "notification failed: ${e.message}")
        }
    }

    private fun ensureChannel(nm: NotificationManager) {
        if (nm.getNotificationChannel(CHANNEL_ID) != null) return
        val channel = NotificationChannel(
            CHANNEL_ID,
            "E2E test results",
            NotificationManager.IMPORTANCE_HIGH,
        ).apply {
            setSound(null, null)
            enableVibration(false)
            enableLights(false)
            description = "Silent notifications from the e2e test harness"
        }
        nm.createNotificationChannel(channel)
    }

    companion object {
        private const val TAG = "TestNotification"
        private const val CHANNEL_ID = "z.zndroid.e2e"
        private const val NOTIF_ID = 0x7E57
        const val ACTION_NOTIFY = "z.zndroid.debug.NOTIFY_TEST_DONE"
    }
}
