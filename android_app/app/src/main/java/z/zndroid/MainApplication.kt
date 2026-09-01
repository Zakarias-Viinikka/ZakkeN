package z.zndroid

import android.app.Application

class MainApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        // Initialize the database once for the entire app lifecycle
        DbManager.init(this)
    }
}
