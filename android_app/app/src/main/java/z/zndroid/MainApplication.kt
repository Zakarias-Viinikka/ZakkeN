package z.zndroid

import android.app.Application
import com.z_db.android_mascot.uniffiEnsureInitialized as ensureDbInitialized
import rustlib.client_table_blueprints.uniffiEnsureInitialized as ensureBlueprintsInitialized
import uniffi.protocol.uniffiEnsureInitialized as ensureProtocolInitialized

class MainApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        
        // Ensure native bindings are verified and loaded
        ensureProtocolInitialized()
        ensureDbInitialized()
        ensureBlueprintsInitialized()
        
        // Initialize the database once for the entire app lifecycle
        DbManager.init(this)
    }
}
