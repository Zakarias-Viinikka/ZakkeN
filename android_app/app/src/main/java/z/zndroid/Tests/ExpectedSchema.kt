package z.zndroid.Tests

/**
 * Defines the columns that the Kotlin code explicitly relies on.
 * If the Rust library or the Database changes these, the SchemaSyncTest will fail.
 */
object ExpectedSchema {
    val schemaVersion = 0 // CURRENT STABLE SCHEMA VERSION

    val tables = mapOf(
        "every_block_in_existence" to listOf(
            "id",
            "is_title",
            "content",
            "my_id_as_given_by_yrs",
            "id_of_page_i_belong_to",
            "position"
        ),
        "pages" to listOf(
            "id",
            "page_id",
            "blobbed_page",
            "page_status",
            "version",
            "is_main_menu_page"
        ),
        "uncommitted_diffs" to listOf(
            "id",
            "snapshot_of_edit",
            "love_letter_sketch",
            "session_id",
            "target_id"
        ),
        "key_value_storage" to listOf(
            "key",
            "value"
        ),
        "backlinks" to listOf(
            "id",
            "page_that_holds_link_id",
            "page_being_linked_to_id",
            "disabled",
            "version"
        )
    )
}
