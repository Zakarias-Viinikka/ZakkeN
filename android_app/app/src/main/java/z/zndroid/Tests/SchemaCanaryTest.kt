package z.zndroid.Tests

import z.zndroid.db.schema.*

/**
 * Canary test for schema drift.
 * Asserts each column descriptor's expected type hardcoded.
 * If a column type changes upstream, this test fails with instructions on what else to update.
 */
class SchemaCanaryTest : AppTest {
    override val name = "Schema Type Canary Test"

    override suspend fun run(): TestResult {
        val errors = mutableListOf<String>()

        fun assertCol(tableName: String, colName: String, actualType: String, expectedType: String) {
            if (actualType != expectedType) {
                errors.add("[$tableName.$colName] Expected type $expectedType but got $actualType. Update TableSchema and all row mapper / DB code for $tableName.$colName!")
            }
        }

        PagesTableSchema.allColumns.forEach { col ->
            val expected = when (col.name) {
                "id" -> "INTEGER"
                "page_id", "is_main_menu_page" -> "TEXT"
                "blobbed_page", "page_status", "version" -> "BLOB"
                else -> "UNKNOWN"
            }
            assertCol("pages", col.name, col.typeName, expected)
        }

        EveryBlockInExistenceTableSchema.allColumns.forEach { col ->
            val expected = when (col.name) {
                "id" -> "INTEGER"
                "is_title", "is_part_of_main_menu_page", "content", "my_id_as_given_by_yrs", "id_of_page_i_belong_to", "frac_position", "page_is_disabled" -> "TEXT"
                else -> "UNKNOWN"
            }
            assertCol("every_block_in_existence", col.name, col.typeName, expected)
        }

        BacklinksTableSchema.allColumns.forEach { col ->
            val expected = when (col.name) {
                "id" -> "INTEGER"
                "page_that_holds_link_id", "page_being_linked_to_id" -> "TEXT"
                "disabled", "version" -> "BLOB"
                else -> "UNKNOWN"
            }
            assertCol("backlinks", col.name, col.typeName, expected)
        }

        ClientLoveLetterTableSchema.allColumns.forEach { col ->
            val expected = when (col.name) {
                "id" -> "INTEGER"
                "session_id", "target_id" -> "TEXT"
                "snapshot_of_edit", "love_letter_sketch" -> "BLOB"
                else -> "UNKNOWN"
            }
            assertCol("client_love_letter", col.name, col.typeName, expected)
        }

        KeyValueStorageTableSchema.allColumns.forEach { col ->
            val expected = when (col.name) {
                "id" -> "INTEGER"
                "key", "value" -> "TEXT"
                else -> "UNKNOWN"
            }
            assertCol("key_value_storage", col.name, col.typeName, expected)
        }

        IncomingLoveLettersTableSchema.allColumns.forEach { col ->
            val expected = when (col.name) {
                "id" -> "INTEGER"
                "target_page_id" -> "TEXT"
                "love_letter" -> "BLOB"
                else -> "UNKNOWN"
            }
            assertCol("incoming_love_letters", col.name, col.typeName, expected)
        }

        LogsTableSchema.allColumns.forEach { col ->
            val expected = when (col.name) {
                "id", "timestamp" -> "INTEGER"
                "level", "category", "source", "session_id", "message" -> "TEXT"
                else -> "UNKNOWN"
            }
            assertCol("logs", col.name, col.typeName, expected)
        }

        return if (errors.isEmpty()) {
            TestResult(name, true, "All hardcoded column canary assertions passed")
        } else {
            TestResult(name, false, errors.joinToString("\n"))
        }
    }
}
