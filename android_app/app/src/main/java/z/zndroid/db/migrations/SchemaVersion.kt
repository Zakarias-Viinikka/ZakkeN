package z.zndroid.db.migrations

import com.z_db.android_mascot.LiveForever
import uniffi.protocol.CreateForeignTableIn
import uniffi.protocol.CreateTableIn
import z.zndroid.db.migrations.schemas.Version0

enum class SchemaVersion(val version: Int) {
    V0(0) {
        override fun bootstrap(db: LiveForever) {
            db.createTable(CreateTableIn("key_value_storage", Version0.keyValueStorageColumns()))
            db.createTable(CreateTableIn("pages", Version0.pagesColumns()))
            db.createTable(CreateTableIn("uncommitted_diffs", Version0.uncommittedDiffsColumns()))
            db.createForeignTable(CreateForeignTableIn("backlinks", Version0.backlinksColumns(), Version0.getForeignDefBacklinks()))
            db.createForeignTable(CreateForeignTableIn("every_block_in_existence", Version0.everyBlockInExistenceColumns(), Version0.getForeignDefEveryBlockInExistence()))
        }

        override fun updateOnce(db: LiveForever): SchemaVersion = this
    };

    abstract fun bootstrap(db: LiveForever)
    abstract fun updateOnce(db: LiveForever): SchemaVersion

    companion object {
        val LATEST = V0

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
            val livePages = try { rustlib.client_table_blueprints.pagesColumns() } catch (e: Throwable) { return }
            val frozenPages = Version0.pagesColumns()
            if (livePages != frozenPages) throw SchemaMismatchException("pages", livePages, frozenPages)

            val liveUncommitted = rustlib.client_table_blueprints.uncommittedDiffsColumns()
            val frozenUncommitted = Version0.uncommittedDiffsColumns()
            if (liveUncommitted != frozenUncommitted) throw SchemaMismatchException("uncommitted_diffs", liveUncommitted, frozenUncommitted)

            val liveBacklinks = rustlib.client_table_blueprints.backlinksColumns()
            val frozenBacklinks = Version0.backlinksColumns()
            if (liveBacklinks != frozenBacklinks) throw SchemaMismatchException("backlinks", liveBacklinks, frozenBacklinks)

            val liveEveryBlock = rustlib.client_table_blueprints.everyBlockInExistenceColumns()
            val frozenEveryBlock = Version0.everyBlockInExistenceColumns()
            if (liveEveryBlock != frozenEveryBlock) throw SchemaMismatchException("every_block_in_existence", liveEveryBlock, frozenEveryBlock)

            val liveKV = rustlib.client_table_blueprints.keyValueStorageColumns()
            val frozenKV = Version0.keyValueStorageColumns()
            if (liveKV != frozenKV) throw SchemaMismatchException("key_value_storage", liveKV, frozenKV)

            val liveFKBacklinks = rustlib.client_table_blueprints.getForeignDefBacklinks()
            val frozenFKBacklinks = Version0.getForeignDefBacklinks()
            if (liveFKBacklinks != frozenFKBacklinks) throw SchemaMismatchException("FK backlinks", liveFKBacklinks, frozenFKBacklinks)

            val liveFKEveryBlock = rustlib.client_table_blueprints.getForeignDefEveryBlockInExistence()
            val frozenFKEveryBlock = Version0.getForeignDefEveryBlockInExistence()
            if (liveFKEveryBlock != frozenFKEveryBlock) throw SchemaMismatchException("FK every_block", liveFKEveryBlock, frozenFKEveryBlock)
        }
    }

    class SchemaMismatchException(tableName: String, live: Any, frozen: Any) :
        Exception("Schema mismatch for table '$tableName'. Live blueprints have changed. Please freeze a new version.\nLive: $live\nFrozen: $frozen")
}
