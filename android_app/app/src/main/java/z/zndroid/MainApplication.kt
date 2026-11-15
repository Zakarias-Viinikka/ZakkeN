package z.zndroid

import android.app.Application
import android.util.Log
import com.z_db.android_mascot.uniffiEnsureInitialized as ensureDbInitialized
import rustlib.client_table_blueprints.uniffiEnsureInitialized as ensureBlueprintsInitialized
import uniffi.protocol.uniffiEnsureInitialized as ensureProtocolInitialized
import rustlib.server_link.uniffiEnsureInitialized as ensureServerLinkInitialized
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import z.zndroid.Storage.SessionManager
import z.zndroid.sync.owner_of_interacting_with_server.connectToSocket

class MainApplication : Application() {
    override fun onCreate() {
        super.onCreate()

        // Ensure native bindings are verified and loaded
        ensureProtocolInitialized()
        ensureDbInitialized()
        ensureBlueprintsInitialized()
        ensureServerLinkInitialized()

        // Initialize the database once for the entire app lifecycle on a
        // background thread. If init fails, readyDeferred completes
        // exceptionally, so awaitReady() below throws instead of hanging.
        // The exception is logged rather than allowed to kill the process:
        // every DB call will return NotReady and the UI can show a popup.
        val appScope = CoroutineScope(Dispatchers.IO + SupervisorJob())
        appScope.launch {
            try {
                DbManager.init(this@MainApplication)
                DbManager.awaitReady()
                SessionManager.initialize()
                connectToSocket()
            } catch (e: Throwable) {
                Log.e("MainApplication", "DB init failed; app is running without a database", e)
            }
        }
    }
}