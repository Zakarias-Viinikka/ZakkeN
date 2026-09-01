package z.zndroid

import android.content.Context
import com.z_db.android_mascot.LiveForever
import uniffi.protocol.CreateTableIn
import z.zndroid.schema.testTableDef
import z.zndroid.schema.testTableName

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
        val create_table_in = CreateTableIn(
            testTableName(),
            testTableDef()
        )
        db.createTable(create_table_in)
    }
}
