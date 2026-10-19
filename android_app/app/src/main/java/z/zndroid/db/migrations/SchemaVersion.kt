package z.zndroid.db.migrations

import com.z_db.android_mascot.LiveForever
import uniffi.protocol.*
import z.zndroid.db.migrations.schemas.Version0
import z.zndroid.db.migrations.schemas.Version1
import z.zndroid.protocol.SafeRowMapper

enum class SchemaVersion(val version: Int) {
    V0(0) {
        override fun bootstrap(db: LiveForever) {
            db.createTable(CreateTableIn("key_value_storage", Version0.keyValueStorageColumns()))
            db.createTable(CreateTableIn("pages", Version0.pagesColumns()))
            db.createTable(CreateTableIn("uncommitted_diffs", Version0.uncommittedDiffsColumns()))
            db.createForeignTable(CreateForeignTableIn("backlinks", Version0.backlinksColumns(), Version0.getForeignDefBacklinks()))
            db.createForeignTable(CreateForeignTableIn("every_block_in_existence", Version0.everyBlockInExistenceColumns(), Version0.getForeignDefEveryBlockInExistence()))
        }

        override fun updateOnce(db: LiveForever): SchemaVersion {
            // Migrate uncommitted_diffs to client_love_letter
            try {
                db.copyTable(CopyTableIn("uncommitted_diffs", "client_love_letter"))
                db.dropTable(DropTableIn("uncommitted_diffs"))
            } catch (_: Exception) {}

            // Migrate every_block_in_existence
            db.addColumn(AddColumnIn("every_block_in_existence", ColumnDef("is_part_of_main_menu_page", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "false", autoincrement = false)))
            db.addColumn(AddColumnIn("every_block_in_existence", ColumnDef("page_is_disabled", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "false", autoincrement = false)))

            // Migrate key_value_storage
            val oldKvData = try {
                db.getData(GetDataIn("key_value_storage", SelectArguments.Single(SelectArgument.XEqualY("1", "1")), emptyList()))
            } catch (_: Exception) { null }

            db.dropTable(DropTableIn("key_value_storage"))
            db.createTable(CreateTableIn("key_value_storage", Version1.keyValueStorageColumns()))

            oldKvData?.rows?.forEach { oldRow ->
                val k = (oldRow.cols.getOrNull(0) as? Col.Text)?.v1
                val v = (oldRow.cols.getOrNull(1) as? Col.Text)?.v1
                if (k != null && v != null) {
                    val newRow = rustlib.client_table_blueprints.newKeyValueItem(k, v)
                    val values = SafeRowMapper.mapRow(newRow, Version1.keyValueStorageColumns(), listOf("key", "value"), skipId = true)
                    db.insertData(InsertDataIn("key_value_storage", values))
                }
            }

            // Create logs table
            db.createTable(CreateTableIn("logs", Version1.logsColumns()))

            // Create incoming_love_letters table
            db.createTable(CreateTableIn("incoming_love_letters", Version1.incomingLoveLettersColumns()))

            return V1
        }
    },
    V1(1) {
        override fun bootstrap(db: LiveForever) {
            db.createTable(CreateTableIn("key_value_storage", Version1.keyValueStorageColumns()))
            db.createTable(CreateTableIn("pages", Version1.pagesColumns()))
            db.createTable(CreateTableIn("client_love_letter", Version1.clientLoveLetterColumns()))
            db.createTable(CreateTableIn("logs", Version1.logsColumns()))
            db.createTable(CreateTableIn("incoming_love_letters", Version1.incomingLoveLettersColumns()))
            db.createForeignTable(CreateForeignTableIn("backlinks", Version1.backlinksColumns(), Version1.getForeignDefBacklinks()))
            db.createForeignTable(CreateForeignTableIn("every_block_in_existence", Version1.everyBlockInExistenceColumns(), Version1.getForeignDefEveryBlockInExistence()))
        }

        override fun updateOnce(db: LiveForever): SchemaVersion {
            // V1 → V2: repair DBs that were bootstrapped when Version1 still
            // meant "uncommitted_diffs" and later renamed to "client_love_letter"
            // in place without bumping the version. Those DBs have the old
            // table but not the new one.
            val tables = db.listTables().tableNames
            if ("client_love_letter" !in tables) {
                if ("uncommitted_diffs" in tables) {
                    try {
                        db.copyTable(CopyTableIn("uncommitted_diffs", "client_love_letter"))
                        db.dropTable(DropTableIn("uncommitted_diffs"))
                    } catch (_: Exception) {}
                } else {
                    try {
                        db.createTable(CreateTableIn("client_love_letter", Version1.clientLoveLetterColumns()))
                    } catch (_: Exception) {}
                }
            }
            return V2
        }
    },
    V2(2) {
        override fun bootstrap(db: LiveForever) {
            V1.bootstrap(db)
        }

        override fun updateOnce(db: LiveForever): SchemaVersion = this
    };

    abstract fun bootstrap(db: LiveForever)
    abstract fun updateOnce(db: LiveForever): SchemaVersion

    companion object {
        val LATEST = V2

        fun fromInt(version: Int): SchemaVersion {
            return values().find { it.version == version }
                ?: throw IllegalArgumentException("Unknown schema version: $version")
        }

        /**
         * Verifies that the latest frozen version matches the live Rust blueprints.
         * This should be called in build-time/internal tests to ensure a version bump
         * is performed when blueprints change.
         */
        fun assertLatestMatchesBlueprints() {
            val livePages = try { rustlib.client_table_blueprints.newTablePages() } catch (e: Throwable) { return }
            val frozenPages = Version1.pagesColumns()
            if (livePages != frozenPages) throw SchemaMismatchException("pages", livePages, frozenPages)

            val liveClientLove = rustlib.client_table_blueprints.newTableClientLoveLetter()
            val frozenClientLove = Version1.clientLoveLetterColumns()
            if (liveClientLove != frozenClientLove) throw SchemaMismatchException("client_love_letter", liveClientLove, frozenClientLove)

            val liveBacklinks = rustlib.client_table_blueprints.newTableBacklinks()
            val frozenBacklinks = Version1.backlinksColumns()
            if (liveBacklinks != frozenBacklinks) throw SchemaMismatchException("backlinks", liveBacklinks, frozenBacklinks)

            val liveEveryBlock = rustlib.client_table_blueprints.newTableEveryBlockInExistence()
            val frozenEveryBlock = Version1.everyBlockInExistenceColumns()
            if (liveEveryBlock != frozenEveryBlock) throw SchemaMismatchException("every_block_in_existence", liveEveryBlock, frozenEveryBlock)

            val liveKV = rustlib.client_table_blueprints.newTableKeyValueStorage()
            val frozenKV = Version1.keyValueStorageColumns()
            if (liveKV != frozenKV) throw SchemaMismatchException("key_value_storage", liveKV, frozenKV)

            val liveIncoming = rustlib.client_table_blueprints.newTableIncomingLoveLetters()
            val frozenIncoming = Version1.incomingLoveLettersColumns()
            if (liveIncoming != frozenIncoming) throw SchemaMismatchException("incoming_love_letters", liveIncoming, frozenIncoming)

            val liveLogs = rustlib.client_table_blueprints.newTableLogs()
            val frozenLogs = Version1.logsColumns()
            if (liveLogs != frozenLogs) throw SchemaMismatchException("logs", liveLogs, frozenLogs)

            val liveFKBacklinks = rustlib.client_table_blueprints.getForeignDefBacklinks()
            val frozenFKBacklinks = Version1.getForeignDefBacklinks()
            if (liveFKBacklinks != frozenFKBacklinks) throw SchemaMismatchException("FK backlinks", liveFKBacklinks, frozenFKBacklinks)

            val liveFKEveryBlock = rustlib.client_table_blueprints.getForeignDefEveryBlockInExistence()
            val frozenFKEveryBlock = Version1.getForeignDefEveryBlockInExistence()
            if (liveFKEveryBlock != frozenFKEveryBlock) throw SchemaMismatchException("FK every_block", liveFKEveryBlock, frozenFKEveryBlock)
        }
    }

    class SchemaMismatchException(tableName: String, live: Any, frozen: Any) :
        Exception("Schema mismatch for table '$tableName'. Live blueprints have changed. Please freeze a new version.\nLive: $live\nFrozen: $frozen")
}
