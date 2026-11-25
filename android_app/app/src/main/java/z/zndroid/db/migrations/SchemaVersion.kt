package z.zndroid.db.migrations

import com.z_db.android_mascot.LiveForever
import uniffi.protocol.*
import z.zndroid.db.migrations.schemas.FrozenTable
import z.zndroid.db.migrations.schemas.Version0
import z.zndroid.db.migrations.schemas.Version1
import z.zndroid.db.migrations.schemas.Version2
import z.zndroid.db.migrations.schemas.Version3
import z.zndroid.db.migrations.schemas.Version4
import z.zndroid.db.migrations.schemas.Version5
import z.zndroid.protocol.SafeRowMapper

/**
 * V5's frozen snapshot = V4's tables with the backlinks table removed.
 * The web removed the backlinks CRDT table (web commit 8ae8ab9b) and
 * Android follows. Deriving it here avoids a separate Version5.kt whose
 * only difference from Version4.kt would be the missing backlinks.
 */
private val V5_FROZEN_TABLES: List<FrozenTable> =
    Version4.tables.filter { it.name != "backlinks" }

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
            db.copyTable(CopyTableIn("uncommitted_diffs", "client_love_letter"))
            db.dropTable(DropTableIn("uncommitted_diffs"))

            db.addColumn(AddColumnIn("every_block_in_existence", ColumnDef("is_part_of_main_menu_page", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "false", autoincrement = false)))
            db.addColumn(AddColumnIn("every_block_in_existence", ColumnDef("page_is_disabled", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "false", autoincrement = false)))

            val oldKvData = db.getData(GetDataIn("key_value_storage", SelectArguments.Single(SelectArgument.XEqualY("1", "1")), emptyList()))

            db.dropTable(DropTableIn("key_value_storage"))
            db.createTable(CreateTableIn("key_value_storage", Version1.keyValueStorageColumns()))

            oldKvData.rows.forEach { oldRow ->
                val k = (oldRow.cols.getOrNull(0) as? Col.Text)?.v1
                val v = (oldRow.cols.getOrNull(1) as? Col.Text)?.v1
                if (k == null || v == null) {
                    throw IllegalStateException(
                        "V0->V1: key_value_storage row with unreadable key or value during migration"
                    )
                }
                val newRow = rustlib.client_table_blueprints.newKeyValueItem(k, v)
                val values = SafeRowMapper.mapRow(newRow, Version1.keyValueStorageColumns(), listOf("key", "value"), skipId = true)
                db.insertData(InsertDataIn("key_value_storage", values))
            }

            db.createTable(CreateTableIn("logs", Version1.logsColumns()))
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
            val tables = db.listTables().tableNames
            if ("client_love_letter" !in tables) {
                if ("uncommitted_diffs" in tables) {
                    db.copyTable(CopyTableIn("uncommitted_diffs", "client_love_letter"))
                    db.dropTable(DropTableIn("uncommitted_diffs"))
                } else {
                    db.createTable(CreateTableIn("client_love_letter", Version1.clientLoveLetterColumns()))
                }
            }
            return V2
        }
    },
    V2(2) {
        override fun bootstrap(db: LiveForever) {
            db.createTable(CreateTableIn("key_value_storage", Version1.keyValueStorageColumns()))
            db.createTable(CreateTableIn("pages", Version1.pagesColumns()))
            db.createTable(CreateTableIn("client_love_letter", Version1.clientLoveLetterColumns()))
            db.createTable(CreateTableIn("logs", Version1.logsColumns()))
            db.createTable(CreateTableIn("incoming_love_letters", Version1.incomingLoveLettersColumns()))
            db.createForeignTable(CreateForeignTableIn("backlinks", Version1.backlinksColumns(), Version1.getForeignDefBacklinks()))
            db.createForeignTable(CreateForeignTableIn("every_block_in_existence", Version2.everyBlockInExistenceColumns(), Version2.getForeignDefEveryBlockInExistence()))
        }

        override fun updateOnce(db: LiveForever): SchemaVersion {
            db.dropTable(DropTableIn("every_block_in_existence"))
            db.createForeignTable(CreateForeignTableIn("every_block_in_existence", Version2.everyBlockInExistenceColumns(), Version2.getForeignDefEveryBlockInExistence()))
            return V3
        }
    },
    V3(3) {
        override fun bootstrap(db: LiveForever) {
            createSnapshotTables(db, Version3.tables)
        }

        override val snapshotTables: List<FrozenTable> get() = Version3.tables

        override fun updateOnce(db: LiveForever): SchemaVersion {
            // V3 -> V4: add seen_server_letters. Simple additive table;
            // no FK, no existing rows to migrate.
            val newTable = Version4.tables.firstOrNull { it.name == "seen_server_letters" }
                ?: throw IllegalStateException(
                    "V3->V4: Version4.tables is missing seen_server_letters"
                )
            db.createTable(CreateTableIn(newTable.name, newTable.columns))
            return V4
        }
    },
    V4(4) {
        override fun bootstrap(db: LiveForever) {
            createSnapshotTables(db, Version4.tables)
        }

        override val snapshotTables: List<FrozenTable> get() = Version4.tables

        override fun updateOnce(db: LiveForever): SchemaVersion {
            // V4 -> V5: the web removed the backlinks CRDT table (web commit
            // 8ae8ab9b) and Android follows. Drop the physical table so
            // existing dev DBs clean up after themselves; Version4.tables
            // stays as a frozen historical record of what V4 created.
            try {
                db.dropTable(DropTableIn("backlinks"))
            } catch (_: Exception) {
                // Table already gone; nothing to do.
            }
            return V5
        }
    },
    V5(5) {
        override fun bootstrap(db: LiveForever) {
            createSnapshotTables(db, Version5.tables)
        }
        override val snapshotTables: List<FrozenTable> get() = Version5.tables
        override fun updateOnce(db: LiveForever): SchemaVersion = this
    };

    abstract fun bootstrap(db: LiveForever)
    abstract fun updateOnce(db: LiveForever): SchemaVersion

    /**
     * The tables this version's bootstrap creates, as a frozen snapshot.
     * Non-null only for versions whose bootstrap goes through a
     * `Version<N>.tables`. Used by the device-side drift check.
     */
    open val snapshotTables: List<FrozenTable>? get() = null

    companion object {
        val LATEST = V5

        fun fromInt(version: Int): SchemaVersion {
            return values().find { it.version == version }
                ?: throw IllegalArgumentException("Unknown schema version: $version")
        }

        /**
         * Device-side backstop for the host-side schema_drift_detector.
         * Compares the live Rust blueprints (loaded from the shipped
         * arm64 .so) against the LATEST frozen snapshot. Names the table
         * and first differing column on failure, rather than dumping two
         * lists.
         *
         * Skips only when the native libs can't load (host JVM tests).
         */
        fun assertLatestMatchesBlueprints() {
            val snapshot = LATEST.snapshotTables ?: return
            val liveColumns: Map<String, List<ColumnDef>>
            val liveFks: Map<String, List<ForeignKeyDef>>
            try {
                liveColumns = mapOf(
                    "pages" to rustlib.client_table_blueprints.newTablePages(),
                    "every_block_in_existence" to rustlib.client_table_blueprints.newTableEveryBlockInExistence(),
                    "client_love_letter" to rustlib.client_table_blueprints.newTableClientLoveLetter(),
                    "key_value_storage" to rustlib.client_table_blueprints.newTableKeyValueStorage(),
                    "incoming_love_letters" to rustlib.client_table_blueprints.newTableIncomingLoveLetters(),
                    "logs" to rustlib.client_table_blueprints.newTableLogs(),
                    "seen_server_letters" to rustlib.client_table_blueprints.newTableSeenServerLetters(),
                )
                liveFks = mapOf(
                    "every_block_in_existence" to rustlib.client_table_blueprints.getForeignDefEveryBlockInExistence(),
                )
            } catch (e: UnsatisfiedLinkError) {
                return
            } catch (e: NoClassDefFoundError) {
                return
            }

            for (table in snapshot) {
                val liveCols = liveColumns[table.name]
                    ?: throw IllegalStateException(
                        "assertLatestMatchesBlueprints: no live blueprint for '${table.name}'. " +
                        "Add it to the fetch map in SchemaVersion.assertLatestMatchesBlueprints."
                    )
                compareTableColumns(table.name, liveCols, table.columns)
                val liveFkList = liveFks[table.name] ?: emptyList()
                compareTableFks(table.name, liveFkList, table.foreignKeys)
            }
        }

        private fun createSnapshotTables(db: LiveForever, tables: List<FrozenTable>) {
            for (t in tables) {
                if (t.foreignKeys.isEmpty()) {
                    db.createTable(CreateTableIn(t.name, t.columns))
                } else {
                    db.createForeignTable(CreateForeignTableIn(t.name, t.columns, t.foreignKeys))
                }
            }
        }

        private fun compareTableColumns(name: String, live: List<ColumnDef>, frozen: List<ColumnDef>) {
            val liveByName = live.associateBy { it.name }
            val frozenByName = frozen.associateBy { it.name }

            for (fc in frozen) {
                val lc = liveByName[fc.name]
                    ?: throw SchemaMismatchException(
                        "table '$name' column '${fc.name}': removed in Rust (frozen V${LATEST.version} expects it)"
                    )
                if (lc.columnType != fc.columnType) {
                    throw SchemaMismatchException(
                        "table '$name' column '${fc.name}': type changed. Frozen='${fc.columnType}' Rust='${lc.columnType}'"
                    )
                }
                if (lc.primaryKey != fc.primaryKey) {
                    throw SchemaMismatchException(
                        "table '$name' column '${fc.name}': primaryKey changed. Frozen=${fc.primaryKey} Rust=${lc.primaryKey}"
                    )
                }
                if (lc.notNull != fc.notNull) {
                    throw SchemaMismatchException(
                        "table '$name' column '${fc.name}': notNull changed. Frozen=${fc.notNull} Rust=${lc.notNull}"
                    )
                }
                if (lc.unique != fc.unique) {
                    throw SchemaMismatchException(
                        "table '$name' column '${fc.name}': unique changed. Frozen=${fc.unique} Rust=${lc.unique}"
                    )
                }
                if (lc.defaultValue != fc.defaultValue) {
                    throw SchemaMismatchException(
                        "table '$name' column '${fc.name}': default changed. Frozen='${fc.defaultValue}' Rust='${lc.defaultValue}'"
                    )
                }
                if (lc.autoincrement != fc.autoincrement) {
                    throw SchemaMismatchException(
                        "table '$name' column '${fc.name}': autoincrement changed. Frozen=${fc.autoincrement} Rust=${lc.autoincrement}"
                    )
                }
            }

            for (lc in live) {
                if (lc.name !in frozenByName) {
                    throw SchemaMismatchException(
                        "table '$name' column '${lc.name}': added in Rust, not in frozen V${LATEST.version}"
                    )
                }
            }

            val liveOrder = live.map { it.name }
            val frozenOrder = frozen.map { it.name }
            if (liveOrder != frozenOrder) {
                throw SchemaMismatchException(
                    "table '$name': column order changed. Frozen=$frozenOrder Rust=$liveOrder"
                )
            }
        }

        private fun compareTableFks(name: String, live: List<ForeignKeyDef>, frozen: List<ForeignKeyDef>) {
            if (live.size != frozen.size) {
                throw SchemaMismatchException(
                    "table '$name': foreign key count changed. Frozen=${frozen.size} Rust=${live.size}"
                )
            }
            for (i in live.indices) {
                val l = live[i]
                val f = frozen[i]
                if (l.columnName != f.columnName ||
                    l.referencedTableName != f.referencedTableName ||
                    l.referencedColumnName != f.referencedColumnName
                ) {
                    throw SchemaMismatchException(
                        "table '$name': FK[$i] changed. " +
                        "Frozen=(${f.columnName}->${f.referencedTableName}.${f.referencedColumnName}) " +
                        "Rust=(${l.columnName}->${l.referencedTableName}.${l.referencedColumnName})"
                    )
                }
            }
        }
    }

    class SchemaMismatchException(message: String) : Exception(message)
}