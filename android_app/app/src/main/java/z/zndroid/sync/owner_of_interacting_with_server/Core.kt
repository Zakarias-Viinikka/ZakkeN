package z.zndroid.sync.owner_of_interacting_with_server

import android.util.Log
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import rustlib.server_link.ServerLink
import java.util.concurrent.atomic.AtomicBoolean

private const val TAG = "SyncCore"
private const val SERVER_URL_KEY = "server_url"
// LAN IP of the dev PC, not 127.0.0.1: the phone runs on its own machine
// and 127.0.0.1 on the phone is the phone itself. This value is minted
// into key_value_storage on first launch, so it can be changed in
// Database Inspector without rebuilding. Update this constant when the
// dev PC's LAN IP changes.
// Read from local.properties (gitignored) via BuildConfig. Empty when
// the key `server_url_default` isn't set, in which case the app falls
// back to loopback (works with `adb reverse tcp:3000 tcp:3000`).
private val DEFAULT_SERVER_URL: String
    get() = z.zndroid.BuildConfig.SERVER_URL_DEFAULT.ifEmpty { "ws://127.0.0.1:3000/ws" }

/** Scope for every long-lived sync task. */
internal val syncScope = CoroutineScope(Dispatchers.IO + SupervisorJob())

private val linkStarted = AtomicBoolean(false)

@Volatile
private var link: ServerLink? = null

/** The live link, or null before connectToSocket has succeeded. */
internal fun currentLink(): ServerLink? = link

/**
 * Connect to the server. Uses the key_value_storage entry `server_url`
 * (full url, e.g. ws://192.168.x.y:3000/ws). If it is unset, mints
 * ws://127.0.0.1:3000/ws, stores it, and uses that. Starts once; later
 * calls after a successful start do nothing. Never throws.
 */
suspend fun connectToSocket() {
    val stored = try {
        readKeyValue(SERVER_URL_KEY)
    } catch (e: Exception) {
        Log.e(TAG, "connectToSocket: reading $SERVER_URL_KEY failed", e)
        return
    }
    val url = stored ?: mintServerUrl()
    if (!linkStarted.compareAndSet(false, true)) return

    val started = try {
        ServerLink.connect(url, SyncListener)
    } catch (e: Exception) {
        Log.e(TAG, "connectToSocket: ServerLink.connect failed", e)
        linkStarted.set(false)
        return
    }
    link = started
    spawnAutoReconnect(started)
    spawnOwnerOfDriftChanged()
    spawnSender(started)
    spawnReceiver(started)
}

/** First run: store the default url so it is visible and editable, and return it. */
private suspend fun mintServerUrl(): String {
    try {
        writeKeyValue(SERVER_URL_KEY, DEFAULT_SERVER_URL)
        Log.i(TAG, "connectToSocket: minted $SERVER_URL_KEY = $DEFAULT_SERVER_URL")
    } catch (e: Exception) {
        Log.w(TAG, "connectToSocket: storing minted $SERVER_URL_KEY failed; using it anyway", e)
    }
    return DEFAULT_SERVER_URL
}

/** True only for the states where the client should be trying to reconnect. */
private fun shouldIReconnect(): Boolean = when (connState.value) {
    ConnState.Offline,
    ConnState.FailedToConnect,
    ConnState.LonewolfAndOffline -> true
    ConnState.Connecting,
    ConnState.LoggingIn,
    ConnState.Connected,
    ConnState.Lonewolf -> false
}

private fun spawnAutoReconnect(started: ServerLink) {
    syncScope.launch {
        while (isActive) {
            if (shouldIReconnect()) started.attemptReconnect()
            delay(1000)
        }
    }
}
