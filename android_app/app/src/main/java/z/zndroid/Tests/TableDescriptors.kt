package z.zndroid.Tests

import uniffi.protocol.Col

/**
 * Column Descriptor representation for parallel column structure.
 * Android substitute for Rust's generic SchemaColumn<T>.
 */
data class ColumnDescriptor<T>(
    val name: String,
    val typeName: String,
    val unwrap: (Col) -> T
)

object PagesTableSchema {
    val id = ColumnDescriptor("id", "INTEGER") { (it as Col.Integer).v1 }
    val pageId = ColumnDescriptor("page_id", "TEXT") { (it as Col.Text).v1 }
    val blobbedPage = ColumnDescriptor("blobbed_page", "BLOB") { (it as Col.Blob).v1 }
    val pageStatus = ColumnDescriptor("page_status", "BLOB") { (it as Col.Blob).v1 }
    val version = ColumnDescriptor("version", "BLOB") { (it as Col.Blob).v1 }
    val isMainMenuPage = ColumnDescriptor("is_main_menu_page", "TEXT") { (it as Col.Text).v1 }

    val allColumns = listOf(id, pageId, blobbedPage, pageStatus, version, isMainMenuPage)
}

object EveryBlockInExistenceTableSchema {
    val id = ColumnDescriptor("id", "INTEGER") { (it as Col.Integer).v1 }
    val isTitle = ColumnDescriptor("is_title", "TEXT") { (it as Col.Text).v1 }
    val isPartOfMainMenuPage = ColumnDescriptor("is_part_of_main_menu_page", "TEXT") { (it as Col.Text).v1 }
    val content = ColumnDescriptor("content", "TEXT") { (it as Col.Text).v1 }
    val myIdAsGivenByYrs = ColumnDescriptor("my_id_as_given_by_yrs", "TEXT") { (it as Col.Text).v1 }
    val idOfPageIBelongTo = ColumnDescriptor("id_of_page_i_belong_to", "TEXT") { (it as Col.Text).v1 }
    val position = ColumnDescriptor("position", "REAL") { (it as Col.Real).v1 }
    val pageIsDisabled = ColumnDescriptor("page_is_disabled", "TEXT") { (it as Col.Text).v1 }

    val allColumns = listOf(id, isTitle, isPartOfMainMenuPage, content, myIdAsGivenByYrs, idOfPageIBelongTo, position, pageIsDisabled)
}

object BacklinksTableSchema {
    val id = ColumnDescriptor("id", "INTEGER") { (it as Col.Integer).v1 }
    val pageThatHoldsLinkId = ColumnDescriptor("page_that_holds_link_id", "TEXT") { (it as Col.Text).v1 }
    val pageBeingLinkedToId = ColumnDescriptor("page_being_linked_to_id", "TEXT") { (it as Col.Text).v1 }
    val disabled = ColumnDescriptor("disabled", "BLOB") { (it as Col.Blob).v1 }
    val version = ColumnDescriptor("version", "BLOB") { (it as Col.Blob).v1 }

    val allColumns = listOf(id, pageThatHoldsLinkId, pageBeingLinkedToId, disabled, version)
}

object ClientLoveLetterTableSchema {
    val id = ColumnDescriptor("id", "INTEGER") { (it as Col.Integer).v1 }
    val snapshotOfEdit = ColumnDescriptor("snapshot_of_edit", "BLOB") { (it as Col.Blob).v1 }
    val loveLetterSketch = ColumnDescriptor("love_letter_sketch", "BLOB") { (it as Col.Blob).v1 }
    val sessionId = ColumnDescriptor("session_id", "TEXT") { (it as Col.Text).v1 }
    val targetId = ColumnDescriptor("target_id", "TEXT") { (it as Col.Text).v1 }

    val allColumns = listOf(id, snapshotOfEdit, loveLetterSketch, sessionId, targetId)
}

object KeyValueStorageTableSchema {
    val id = ColumnDescriptor("id", "INTEGER") { (it as Col.Integer).v1 }
    val key = ColumnDescriptor("key", "TEXT") { (it as Col.Text).v1 }
    val value = ColumnDescriptor("value", "TEXT") { (it as Col.Text).v1 }

    val allColumns = listOf(id, key, value)
}

object IncomingLoveLettersTableSchema {
    val id = ColumnDescriptor("id", "INTEGER") { (it as Col.Integer).v1 }
    val loveLetter = ColumnDescriptor("love_letter", "BLOB") { (it as Col.Blob).v1 }
    val targetPageId = ColumnDescriptor("target_page_id", "TEXT") { (it as Col.Text).v1 }

    val allColumns = listOf(id, loveLetter, targetPageId)
}

object LogsTableSchema {
    val id = ColumnDescriptor("id", "INTEGER") { (it as Col.Integer).v1 }
    val timestamp = ColumnDescriptor("timestamp", "INTEGER") { (it as Col.Integer).v1 }
    val level = ColumnDescriptor("level", "TEXT") { (it as Col.Text).v1 }
    val category = ColumnDescriptor("category", "TEXT") { (it as Col.Text).v1 }
    val source = ColumnDescriptor("source", "TEXT") { (it as Col.Text).v1 }
    val sessionId = ColumnDescriptor("session_id", "TEXT") { (it as Col.Text).v1 }
    val message = ColumnDescriptor("message", "TEXT") { (it as Col.Text).v1 }

    val allColumns = listOf(id, timestamp, level, category, source, sessionId, message)
}
