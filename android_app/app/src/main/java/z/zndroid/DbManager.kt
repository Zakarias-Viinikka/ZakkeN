package z.zndroid

import android.content.Context
import com.z_db.android_mascot.LiveForever
import uniffi.protocol.CreateTableIn
import rustlib.client_table_blueprints.pagesColumns
import rustlib.client_table_blueprints.uncommittedDiffsColumns

object DbManager {
    // This holds the connection to your Rust library
    lateinit var db: LiveForever
        private set

    // Call this once from MainApplication
    fun init(context: Context) {
        if (!::db.isInitialized) {
            val dbPath = context.getDatabasePath("my_database.db").absolutePath
            db = LiveForever(dbPath)
        }

        // Initialize Independent Tables from Rust Blueprints
        db.createTable(CreateTableIn("pages", pagesColumns()))
        db.createTable(CreateTableIn("uncommitted_diffs", uncommittedDiffsColumns()))

        // TODO: Initialize tables with Foreign Keys when db_wrapper supports it:
        // - backlinks (depends on pages)
        // - every_block_in_existence (depends on pages)
    }
}
