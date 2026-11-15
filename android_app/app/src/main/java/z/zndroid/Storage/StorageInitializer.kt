package z.zndroid.Storage

import com.z_db.android_mascot.LiveForever
import rustlib.client_table_blueprints.newTableKeyValueStorage
import rustlib.client_table_blueprints.newKeyValueItem
import uniffi.protocol.GetDataIn
import uniffi.protocol.InsertDataIn
import uniffi.protocol.SelectArgument
import uniffi.protocol.SelectArguments
import z.zndroid.protocol.SafeRowMapper
import java.util.UUID

/**
 * Handles the initialization of storage-related tables and default data.
 *
 * Takes the [LiveForever] directly rather than going through [z.zndroid.DbManager]
 * because it runs during DbManager.init, before the manager reports ready.
 */
object StorageInitializer {

    suspend fun create_all_these_things_if_they_dont_exist(db: LiveForever) {
        db.beginAllOrNothing()
        try {
            ensureUserIdExists(db)
            db.everythingWentPerfectly()
        } catch (e: Exception) {
            try {
                db.regretEverything()
            } catch (_: Exception) {}
            throw e
        }
    }

    private fun ensureUserIdExists(db: LiveForever) {
        val existing = db.getData(GetDataIn(
            "key_value_storage",
            SelectArguments.Single(SelectArgument.XEqualY("key", StorageKey.USER_ID.keyName)),
            listOf("value")
        ))
        if (existing.rows.firstOrNull() != null) return

        val newId = UUID.randomUUID().toString()
        val row = newKeyValueItem(StorageKey.USER_ID.keyName, newId)
        val values = SafeRowMapper.mapRow(
            row = row,
            columnDefs = newTableKeyValueStorage(),
            expectedNames = listOf("key", "value"),
            skipId = true
        )
        db.insertData(InsertDataIn("key_value_storage", values))
    }
}