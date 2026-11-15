package z.zndroid.Tests

import uniffi.protocol.*
import z.zndroid.db.migrations.SchemaVersion
import z.zndroid.db.migrations.MigrationManager
import com.z_db.android_mascot.LiveForever
import java.io.File

class MigrationTest : AppTest {
    override val name = "Database Schema Migrations"

    override suspend fun run(): TestResult {
        return try {
            SchemaVersion.assertLatestMatchesBlueprints()

            val freshResult = testFreshBootstrap()
            if (!freshResult.isSuccess) return freshResult

            val walkResult = testMigrationWalkFromV0()
            if (!walkResult.isSuccess) return walkResult

            TestResult(
                name,
                true,
                "Migrations passed: blueprints match, fresh bootstrap reaches " +
                "v${SchemaVersion.LATEST.version}, walk from V0 reaches v${SchemaVersion.LATEST.version}."
            )
        } catch (e: SchemaVersion.SchemaMismatchException) {
            TestResult(name, false, e.message ?: "Schema blueprint mismatch detected")
        } catch (e: Throwable) {
            if (e is UnsatisfiedLinkError || e is NoClassDefFoundError) {
                return TestResult(name, true, "Skipped (Native database wrapper not available)")
            }
            TestResult(name, false, "Unexpected error in migration test suite", e.stackTraceToString())
        }
    }

    private fun testFreshBootstrap(): TestResult {
        val tempFile = File.createTempFile("migration_fresh_", ".db")
        tempFile.deleteOnExit()
        val testDb = LiveForever(tempFile.absolutePath)
        try {
            MigrationManager.migrate(testDb)
            return checkFinalState(testDb, "fresh bootstrap")
        } finally {
            try { testDb.close() } catch (_: Exception) {}
        }
    }

    /**
     * Bootstraps a DB at V0's schema (the oldest frozen snapshot), then runs
     * the migration walk and asserts the resulting schema is V3. This
     * exercises V0->V1, V1->V2, and V2->V3 in one pass against a real DB,
     * rather than only testing "there was no version row, bootstrap LATEST".
     */
    private fun testMigrationWalkFromV0(): TestResult {
        val tempFile = File.createTempFile("migration_walk_", ".db")
        tempFile.deleteOnExit()
        val testDb = LiveForever(tempFile.absolutePath)
        try {
            SchemaVersion.V0.bootstrap(testDb)
            // Seed the version row. V0's key_value_storage is (key, value)
            // without an id column, so insert directly against that shape.
            testDb.insertData(InsertDataIn(
                "key_value_storage",
                listOf(
                    ColumnValue(columnName = "key", value = Col.Text("schema_version")),
                    ColumnValue(columnName = "value", value = Col.Text("0"))
                )
            ))

            MigrationManager.migrate(testDb)
            return checkFinalState(testDb, "walk from V0")
        } finally {
            try { testDb.close() } catch (_: Exception) {}
        }
    }

    private fun checkFinalState(testDb: LiveForever, label: String): TestResult {
        val queryResult = testDb.getData(GetDataIn(
            "key_value_storage",
            SelectArguments.Single(SelectArgument.XEqualY("key", "schema_version")),
            listOf("value")
        ))
        val versionRow = queryResult.rows.firstOrNull()
            ?: return TestResult(name, false, "$label: schema_version key is missing in key_value_storage")
        val valueCol = versionRow.cols.firstOrNull() as? Col.Text
            ?: return TestResult(name, false, "$label: schema_version value is not a text column")
        val persistedVersion = valueCol.v1.toIntOrNull()
        if (persistedVersion != SchemaVersion.LATEST.version) {
            return TestResult(
                name, false,
                "$label: expected schema_version=${SchemaVersion.LATEST.version} but found $persistedVersion"
            )
        }

        val expectedTables = listOf(
            "pages", "client_love_letter", "backlinks", "every_block_in_existence",
            "key_value_storage", "incoming_love_letters", "logs"
        )
        val tables = testDb.listTables().tableNames
        val missing = expectedTables.filter { it !in tables }
        if (missing.isNotEmpty()) {
            return TestResult(name, false, "$label: missing tables after migration: $missing")
        }
        return TestResult(name, true, "$label: OK")
    }
}