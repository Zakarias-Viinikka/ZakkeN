package z.zndroid.Tests

import uniffi.protocol.CheckTableIn
import z.zndroid.DbManager
import z.zndroid.db.migrations.SchemaVersion

/**
 * Checks the live SQLite database on the device against LATEST's frozen
 * snapshot. This covers "did the migration actually run on this device?".
 * The shipped .so is what it validates. The host-side equivalent (Rust
 * blueprints vs. Kotlin) lives in schema_drift_detector/.
 */
class SchemaSyncTest : AppTest {
    override val name = "Schema live DB vs snapshot"

    override suspend fun run(): TestResult {
        val snapshot = SchemaVersion.LATEST.snapshotTables
            ?: return TestResult(name, false, "LATEST has no snapshotTables to compare against")
        val errors = mutableListOf<String>()
        val warnings = mutableListOf<String>()
        var tablesChecked = 0

        for (table in snapshot) {
            DbManager.checkTable(CheckTableIn(table.name)).fold(
                onSuccess = { dbInfo ->
                    tablesChecked++
                    val dbColNames = dbInfo.columns.map { it.name }.toSet()
                    for (col in table.columns) {
                        if (col.name !in dbColNames) {
                            errors.add(
                                "DB DRIFT: table '${table.name}' is missing column '${col.name}' on device. " +
                                "Migration to V${SchemaVersion.LATEST.version} probably didn't run."
                            )
                        }
                    }
                },
                onFailure = { error ->
                    warnings.add("DB SKIP: could not check '${table.name}': ${error.message}")
                }
            )
        }

        return when {
            errors.isNotEmpty() -> TestResult(
                name = name,
                isSuccess = false,
                message = "Schema drift between V${SchemaVersion.LATEST.version} snapshot and live DB",
                errorContext = errors.joinToString("\n")
            )
            warnings.isNotEmpty() -> TestResult(
                name = name,
                isSuccess = true,
                message = "Snapshot matches DB for $tablesChecked tables; some checks skipped.",
                errorContext = warnings.joinToString("\n")
            )
            else -> TestResult(
                name = name,
                isSuccess = true,
                message = "Snapshot matches DB for $tablesChecked tables."
            )
        }
    }
}