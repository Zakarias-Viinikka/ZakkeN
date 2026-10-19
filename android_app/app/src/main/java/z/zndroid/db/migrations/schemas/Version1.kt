package z.zndroid.db.migrations.schemas

import uniffi.protocol.ColumnDef
import uniffi.protocol.ForeignKeyDef

object Version1 {
    fun pagesColumns(): List<ColumnDef> = listOf(
        ColumnDef("id", "INTEGER", primaryKey = true, notNull = true, unique = false, defaultValue = "", autoincrement = true),
        ColumnDef("page_id", "TEXT", primaryKey = false, notNull = true, unique = true, defaultValue = "", autoincrement = false),
        ColumnDef("blobbed_page", "BLOB", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false),
        ColumnDef("page_status", "BLOB", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false),
        ColumnDef("version", "BLOB", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false),
        ColumnDef("is_main_menu_page", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false)
    )

    fun clientLoveLetterColumns(): List<ColumnDef> = listOf(
        ColumnDef("id", "INTEGER", primaryKey = true, notNull = true, unique = false, defaultValue = "", autoincrement = true),
        ColumnDef("snapshot_of_edit", "BLOB", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false),
        ColumnDef("love_letter_sketch", "BLOB", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false),
        ColumnDef("session_id", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false),
        ColumnDef("target_id", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false)
    )

    fun backlinksColumns(): List<ColumnDef> = listOf(
        ColumnDef("id", "INTEGER", primaryKey = true, notNull = true, unique = false, defaultValue = "", autoincrement = true),
        ColumnDef("page_that_holds_link_id", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false),
        ColumnDef("page_being_linked_to_id", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false),
        ColumnDef("disabled", "BLOB", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false),
        ColumnDef("version", "BLOB", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false)
    )

    fun getForeignDefBacklinks(): List<ForeignKeyDef> = listOf(
        ForeignKeyDef("page_that_holds_link_id", "pages", "page_id"),
        ForeignKeyDef("page_being_linked_to_id", "pages", "page_id")
    )

    fun everyBlockInExistenceColumns(): List<ColumnDef> = listOf(
        ColumnDef("id", "INTEGER", primaryKey = true, notNull = true, unique = false, defaultValue = "", autoincrement = true),
        ColumnDef("is_title", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false),
        ColumnDef("is_part_of_main_menu_page", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false),
        ColumnDef("content", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false),
        ColumnDef("my_id_as_given_by_yrs", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false),
        ColumnDef("id_of_page_i_belong_to", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false),
        ColumnDef("position", "REAL", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false),
        ColumnDef("page_is_disabled", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false)
    )

    fun getForeignDefEveryBlockInExistence(): List<ForeignKeyDef> = listOf(
        ForeignKeyDef("id_of_page_i_belong_to", "pages", "page_id")
    )

    fun keyValueStorageColumns(): List<ColumnDef> = listOf(
        ColumnDef("id", "INTEGER", primaryKey = true, notNull = true, unique = false, defaultValue = "", autoincrement = true),
        ColumnDef("key", "TEXT", primaryKey = false, notNull = true, unique = true, defaultValue = "", autoincrement = false),
        ColumnDef("value", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false)
    )

    fun incomingLoveLettersColumns(): List<ColumnDef> = listOf(
        ColumnDef("id", "INTEGER", primaryKey = true, notNull = true, unique = false, defaultValue = "", autoincrement = true),
        ColumnDef("love_letter", "BLOB", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false),
        ColumnDef("target_page_id", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false)
    )

    fun logsColumns(): List<ColumnDef> = listOf(
        ColumnDef("id", "INTEGER", primaryKey = true, notNull = true, unique = false, defaultValue = "", autoincrement = true),
        ColumnDef("timestamp", "INTEGER", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false),
        ColumnDef("level", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false),
        ColumnDef("category", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false),
        ColumnDef("source", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false),
        ColumnDef("session_id", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false),
        ColumnDef("message", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false)
    )
}
