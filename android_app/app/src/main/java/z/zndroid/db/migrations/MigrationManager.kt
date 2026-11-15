package z.zndroid.db.migrations

import com.z_db.android_mascot.LiveForever
import uniffi.protocol.*
import z.zndroid.LocalDbError
import z.zndroid.protocol.SafeRowMapper
import rustlib.client_table_blueprints.newTableKeyValueStorage
import rustlib.client_table_blueprints.newKeyValueItem

object MigrationManager {
    private const val SCHEMA_VERSION_KEY = "schema_version"
    private const val KV_TABLE = "key_value_storage"

    fun migrate(db: LiveForever) {
        db.beginAllOrNothing()
        try {
            val currentVersionInt = readCurrentVersion(db)
            if (currentVersionInt == null) {
                // Fresh database (key_value_storage does not exist yet).
                SchemaVersion.LATEST.bootstrap(db)
                writeVersion(db, SchemaVersion.LATEST.version)
            } else {
                val maxKnownVersion = SchemaVersion.LATEST.version
                if (currentVersionInt > maxKnownVersion) {
                    throw LocalDbError.DatabaseDowngraded(currentVersionInt, maxKnownVersion)
                }

                var currentVersion = SchemaVersion.fromInt(currentVersionInt)
                while (currentVersion.version < SchemaVersion.LATEST.version) {
                    val nextVersion = currentVersion.updateOnce(db)
                    if (nextVersion.version <= currentVersion.version) {
                        // A step that does not advance leaves the app on an
                        // old schema with no error. Fail loudly instead.
                        throw IllegalStateException(
                            "Migration stuck at v${currentVersion.version}: " +
                            "updateOnce returned v${nextVersion.version}"
                        )
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

    /**
     * Returns the persisted schema version, or null if this is a fresh
     * database (the storage table has not been created yet). Any other
     * failure is re-thrown so a broken read on a populated DB does not
     * silently get treated as a fresh one.
     */
    private fun readCurrentVersion(db: LiveForever): Int? {
        val tables = try {
            db.listTables().tableNames
        } catch (e: Exception) {
            throw IllegalStateException(
                "MigrationManager.readCurrentVersion: listTables failed: ${e.message}", e
            )
        }
        if (KV_TABLE !in tables) return null

        val result = try {
            db.getData(GetDataIn(
                KV_TABLE,
                SelectArguments.Single(SelectArgument.XEqualY("key", SCHEMA_VERSION_KEY)),
                listOf("value")
            ))
        } catch (e: Exception) {
            throw IllegalStateException(
                "MigrationManager.readCurrentVersion: getData failed: ${e.message}", e
            )
        }

        val row = result.rows.firstOrNull() ?: return null
        val valueCol = row.cols.firstOrNull() as? Col.Text ?: return null
        return valueCol.v1.toIntOrNull()
    }

    private fun writeVersion(db: LiveForever, version: Int) {
        try {
            db.deleteRowWhere(DeleteRowWhereIn(
                KV_TABLE,
                SelectArguments.Single(SelectArgument.XEqualY("key", SCHEMA_VERSION_KEY))
            ))
        } catch (_: Exception) {}

        val row = newKeyValueItem(SCHEMA_VERSION_KEY, version.toString())
        val values = SafeRowMapper.mapRow(
            row = row,
            columnDefs = newTableKeyValueStorage(),
            expectedNames = listOf("key", "value"),
            skipId = true
        )
        db.insertData(InsertDataIn(KV_TABLE, values))
    }
}