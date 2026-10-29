package z.zndroid.db.reads

import uniffi.protocol.*
import z.zndroid.DbManager
import z.zndroid.db.schema.*

data class BlockMirrorRow(
    val blockId: String,
    val pageId: String,
    val content: String,
    val isTitle: Boolean,
    val fracPosition: String,
    val pageIsDisabled: Boolean
)

data class MenuPageRow(
    val pageId: String,
    val title: String,
    val isUntitled: Boolean
)

private fun <T> colOf(
    row: Row,
    columnsRead: List<String>,
    desc: ColumnDescriptor<T>
): T {
    val idx = columnsRead.indexOf(desc.name)
    if (idx < 0 || idx >= row.cols.size) {
        throw IllegalStateException("Column '${desc.name}' missing from row")
    }
    return desc.unwrap(row.cols[idx])
}

suspend fun getPageBlob(pageId: String): Result<ByteArray> {
    val columns = listOf(PagesTableSchema.blobbedPage.name)
    return DbManager.getData(GetDataIn(
        PagesTableSchema.TABLE_NAME,
        SelectArguments.Single(SelectArgument.XEqualY(PagesTableSchema.pageId.name, pageId)),
        columns
    )).mapCatching { data ->
        val row = data.rows.firstOrNull() ?: throw Exception("Page not found: $pageId")
        colOf(row, columns, PagesTableSchema.blobbedPage)
    }
}

suspend fun isMainMenuPage(pageId: String): Boolean {
    val columns = listOf(PagesTableSchema.isMainMenuPage.name)
    return DbManager.getData(GetDataIn(
        PagesTableSchema.TABLE_NAME,
        SelectArguments.Single(SelectArgument.XEqualY(PagesTableSchema.pageId.name, pageId)),
        columns
    )).mapCatching { data ->
        val row = data.rows.firstOrNull() ?: throw Exception("Page not found: $pageId")
        colOf(row, columns, PagesTableSchema.isMainMenuPage) == "true"
    }.getOrThrow()
}

suspend fun getActivePagesBlob(pageId: String): Result<ByteArray> {
    val columns = listOf(PagesTableSchema.pageStatus.name)
    return DbManager.getData(GetDataIn(
        PagesTableSchema.TABLE_NAME,
        SelectArguments.Single(SelectArgument.XEqualY(PagesTableSchema.pageId.name, pageId)),
        columns
    )).mapCatching { data ->
        val row = data.rows.firstOrNull() ?: throw Exception("Page not found: $pageId")
        colOf(row, columns, PagesTableSchema.pageStatus)
    }
}

suspend fun isPageDisabled(pageId: String): Boolean {
    val columns = listOf(EveryBlockInExistenceTableSchema.id.name)
    val out = DbManager.getData(GetDataIn(
        EveryBlockInExistenceTableSchema.TABLE_NAME,
        SelectArguments.Two(
            first = SelectArgument.XEqualY(EveryBlockInExistenceTableSchema.idOfPageIBelongTo.name, pageId),
            join = JoinType.AND,
            second = SelectArgument.XEqualY(EveryBlockInExistenceTableSchema.pageIsDisabled.name, "true")
        ),
        columns
    )).getOrThrow()
    return out.rows.isNotEmpty()
}

suspend fun getBlocksForPageOrdered(pageId: String): Result<List<BlockMirrorRow>> {
    val columns = listOf(
        EveryBlockInExistenceTableSchema.myIdAsGivenByYrs.name,
        EveryBlockInExistenceTableSchema.idOfPageIBelongTo.name,
        EveryBlockInExistenceTableSchema.content.name,
        EveryBlockInExistenceTableSchema.isTitle.name,
        EveryBlockInExistenceTableSchema.fracPosition.name,
        EveryBlockInExistenceTableSchema.pageIsDisabled.name
    )
    val orderBy = "${EveryBlockInExistenceTableSchema.fracPosition.name} ASC, ${EveryBlockInExistenceTableSchema.myIdAsGivenByYrs.name} ASC"
    return DbManager.getDataOrdered(GetDataOrderedIn(
        EveryBlockInExistenceTableSchema.TABLE_NAME,
        SelectArguments.Single(SelectArgument.XEqualY(EveryBlockInExistenceTableSchema.idOfPageIBelongTo.name, pageId)),
        columns,
        orderBy
    )).mapCatching { data ->
        data.rows.map { row ->
            BlockMirrorRow(
                blockId = colOf(row, columns, EveryBlockInExistenceTableSchema.myIdAsGivenByYrs),
                pageId = colOf(row, columns, EveryBlockInExistenceTableSchema.idOfPageIBelongTo),
                content = colOf(row, columns, EveryBlockInExistenceTableSchema.content),
                isTitle = colOf(row, columns, EveryBlockInExistenceTableSchema.isTitle) == "true",
                fracPosition = colOf(row, columns, EveryBlockInExistenceTableSchema.fracPosition),
                pageIsDisabled = colOf(row, columns, EveryBlockInExistenceTableSchema.pageIsDisabled) == "true"
            )
        }
    }
}

suspend fun getMenuPages(): Result<List<MenuPageRow>> {
    val columns = listOf(
        EveryBlockInExistenceTableSchema.idOfPageIBelongTo.name,
        EveryBlockInExistenceTableSchema.content.name
    )
    return DbManager.getData(GetDataIn(
        EveryBlockInExistenceTableSchema.TABLE_NAME,
        SelectArguments.Three(
            first = SelectArgument.XEqualY(EveryBlockInExistenceTableSchema.isTitle.name, "true"),
            join = JoinType.AND,
            second = SelectArgument.XEqualY(EveryBlockInExistenceTableSchema.isPartOfMainMenuPage.name, "true"),
            join2 = JoinType.AND,
            third = SelectArgument.XEqualY(EveryBlockInExistenceTableSchema.pageIsDisabled.name, "false")
        ),
        columns
    )).mapCatching { data ->
        data.rows.map { row ->
            val rawTitle = colOf(row, columns, EveryBlockInExistenceTableSchema.content)
            val isUntitled = rawTitle.isBlank()
            MenuPageRow(
                pageId = colOf(row, columns, EveryBlockInExistenceTableSchema.idOfPageIBelongTo),
                title = if (isUntitled) "Untitled" else rawTitle,
                isUntitled = isUntitled
            )
        }
    }
}
