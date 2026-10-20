package z.zndroid.db.migrations.schemas

import uniffi.protocol.ColumnDef
import uniffi.protocol.ForeignKeyDef

object Version2 {
    fun everyBlockInExistenceColumns(): List<ColumnDef> = listOf(
        ColumnDef("id", "INTEGER", primaryKey = true, notNull = true, unique = false, defaultValue = "", autoincrement = true),
        ColumnDef("is_title", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false),
        ColumnDef("is_part_of_main_menu_page", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false),
        ColumnDef("content", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false),
        ColumnDef("my_id_as_given_by_yrs", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false),
        ColumnDef("id_of_page_i_belong_to", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false),
        ColumnDef("frac_position", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false),
        ColumnDef("page_is_disabled", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false)
    )

    fun getForeignDefEveryBlockInExistence(): List<ForeignKeyDef> = listOf(
        ForeignKeyDef("id_of_page_i_belong_to", "pages", "page_id")
    )
}
