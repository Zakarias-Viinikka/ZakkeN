package z.zndroid.Tests

/**
 * Defines the columns that the Kotlin code explicitly relies on.
 * Reflects the current live schema blueprints.
 */
object ExpectedSchema {
    val schemaVersion = 1

    val tables = mapOf(
        "pages" to PagesTableSchema.allColumns.map { it.name },
        "every_block_in_existence" to EveryBlockInExistenceTableSchema.allColumns.map { it.name },
        "uncommitted_diffs" to UncommittedDiffsTableSchema.allColumns.map { it.name },
        "key_value_storage" to KeyValueStorageTableSchema.allColumns.map { it.name },
        "backlinks" to BacklinksTableSchema.allColumns.map { it.name },
        "incoming_love_letters" to IncomingLoveLettersTableSchema.allColumns.map { it.name },
        "logs" to LogsTableSchema.allColumns.map { it.name }
    )
}
