package z.zndroid.Tests

import rustlib.client_table_blueprints.*
import uniffi.protocol.CheckTableIn
import z.zndroid.DbManager

/**
 * Checks for "Drift" between:
 * 1. Kotlin Code (ExpectedSchema) vs. Rust Library (Blueprints)
 * 2. Rust Library (Blueprints) vs. Live SQLite Database
 */
class SchemaSyncTest : AppTest {
    override val name = "Schema Sync & Drift Check"

    override suspend fun run(): TestResult {
        val errors = mutableListOf<String>()
        val warnings = mutableListOf<String>()

        // 1. Define the library functions to check
        val libraryBlueprints = mapOf(
            "pages" to pagesColumns(),
            "every_block_in_existence" to everyBlockInExistenceColumns(),
            "uncommitted_diffs" to uncommittedDiffsColumns(),
            "key_value_storage" to keyValueStorageColumns(),
            "backlinks" to backlinksColumns()
        )

        for ((tableName, libCols) in libraryBlueprints) {
            val expectedCols = ExpectedSchema.tables[tableName] ?: emptyList()

            // -- Check A: Library vs. Kotlin Expectations (Code Drift) --
            // Strict check: Names and Count must match.
            if (libCols.size != expectedCols.size) {
                errors.add("CODE DRIFT: Table '$tableName' has ${libCols.size} columns in Rust, but Kotlin expects ${expectedCols.size}.")
            }

            expectedCols.forEach { colName ->
                if (libCols.none { it.name == colName }) {
                    errors.add("CODE DRIFT: Table '$tableName' is missing required column '$colName' in the Rust library.")
                }
            }

            // -- Check B: Library vs. Live SQLite DB (Database Drift) --
            // This ensures the table on the device actually has the columns defined in the library.
            // Note: This part is skipped during Gradle build tests as DbManager won't be ready.
            DbManager.checkTable(CheckTableIn(tableName)).fold(
                onSuccess = { dbInfo ->
                    val dbColNames = dbInfo.columns.map { it.name }
                    libCols.forEach { libCol ->
                        if (!dbColNames.contains(libCol.name)) {
                            errors.add("DB DRIFT: Table '$tableName' is missing column '${libCol.name}' on device. (Migration needed?)")
                        }
                    }
                },
                onFailure = { error ->
                    // If DB isn't ready (e.g. during build), we treat it as a skip/warning, not a fail.
                    warnings.add("DB SKIP: Could not verify '$tableName' table (DB not initialized): ${error.message}")
                }
            )
        }

        return when {
            errors.isNotEmpty() -> TestResult(
                name = name,
                isSuccess = false,
                message = "Schema mismatches detected!",
                errorContext = errors.joinToString("\n")
            )
            warnings.isNotEmpty() -> TestResult(
                name = name,
                isSuccess = true,
                message = "Library matches Kotlin code. DB check was skipped.",
                errorContext = warnings.joinToString("\n")
            )
            else -> TestResult(
                name = name,
                isSuccess = true,
                message = "All schemas are synchronized across Kotlin, Rust, and SQLite."
            )
        }
    }
}
