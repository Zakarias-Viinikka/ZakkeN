package z.zndroid.db.migrations

import com.z_db.android_mascot.LiveForever
import uniffi.protocol.*
import z.zndroid.LocalDbError
import z.zndroid.protocol.SafeRowMapper
import rustlib.client_table_blueprints.keyValueStorageColumns
import rustlib.client_table_blueprints.newKeyValueItem

object MigrationManager {
    private const val SCHEMA_VERSION_KEY = "schema_version"

    fun migrate(db: LiveForever) {
        // Begin an all-or-nothing transaction for bootstrapping or updating
        db.beginAllOrNothing()
        try {
            val currentVersionInt = readCurrentVersion(db)
            if (currentVersionInt == null) {
                // Fresh Database -> bootstrap V0 directly
                SchemaVersion.V0.bootstrap(db)
                writeVersion(db, SchemaVersion.LATEST.version)
            } else {
                val maxKnownVersion = SchemaVersion.LATEST.version
                if (currentVersionInt > maxKnownVersion) {
                    throw LocalDbError.DatabaseDowngraded(currentVersionInt, maxKnownVersion)
                }

                var currentVersion = SchemaVersion.fromInt(currentVersionInt)
                while (currentVersion.version < SchemaVersion.LATEST.version) {
                    val nextVersion = currentVersion.updateOnce(db)
                    if (nextVersion == currentVersion) {
                        break
                    }
                    currentVersion = nextVersion
                }
                writeVersion(db, currentVersion.version)
            }
            db.everythingWentPerfectly()
        } catch (e: Exception) {
            try {
                db.regretEverything()
            } catch (_: Exception) {}
            throw e
        }
    }

    private fun readCurrentVersion(db: LiveForever): Int? {
        return try {
            val result = db.getData(GetDataIn(
                "key_value_storage",
                SelectArguments.Single(SelectArgument.XEqualY("key", SCHEMA_VERSION_KEY)),
                emptyList()
            ))
            val row = result.rows.firstOrNull() ?: return null
            val valueCol = row.cols.getOrNull(1) as? Col.Text ?: return null
            valueCol.v1.toIntOrNull()
        } catch (e: Exception) {
            // key_value_storage table doesn't even exist yet
            null
        }
    }

    private fun writeVersion(db: LiveForever, version: Int) {
        // Since we don't have direct rowId for 'schema_version' and key_value_storage has no auto-increment id,
        // we can check if it exists first, or just try to delete if we had raw hooks. Since deleteRow needs a rowId,
        // we can use editColInRowWhere if available, or just fetch rowId first via getData.
        try {
            val query = db.getData(GetDataIn(
                "key_value_storage",
                SelectArguments.Single(SelectArgument.XEqualY("key", SCHEMA_VERSION_KEY)),
                emptyList()
            ))
            val existingRow = query.rows.firstOrNull()
            if (existingRow != null) {
                val internalId = when (val idCol = existingRow.cols.firstOrNull()) {
                    is Col.Integer -> idCol.v1.toString()
                    is Col.Text -> idCol.v1
                    else -> SCHEMA_VERSION_KEY // fallback
                }
                db.deleteRow(DeleteRowIn("key_value_storage", internalId))
            }
        } catch (_: Exception) {}

        val row = newKeyValueItem(SCHEMA_VERSION_KEY, version.toString())
        val values = SafeRowMapper.mapRow(
            row = row,
            columnDefs = keyValueStorageColumns(),
            expectedNames = listOf("key", "value"),
            skipId = false
        )
        db.insertData(InsertDataIn("key_value_storage", values))
    }
}
