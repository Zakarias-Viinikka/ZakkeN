package z.zndroid.executor

import rustlib.client_table_blueprints.*
import uniffi.protocol.*
import z.zndroid.DbManager
import z.zndroid.db.schema.ClientLoveLetterTableSchema
import z.zndroid.db.schema.EveryBlockInExistenceTableSchema
import z.zndroid.db.schema.PagesTableSchema

internal suspend fun setColumnWhere(
    table: String,
    keyCol: String,
    keyValue: String,
    column: String,
    newValue: Col
) {
    DbManager.editColInRowWhere(EditColInRowWhereIn(
        tableName = table,
        whereClause = SelectArguments.Single(SelectArgument.XEqualY(keyCol, keyValue)),
        column = column,
        newValue = newValue
    )).getOrThrow()
}

internal suspend fun updatePagesBlob(pageId: String, blob: ByteArray) {
    setColumnWhere(
        table = PagesTableSchema.TABLE_NAME,
        keyCol = PagesTableSchema.pageId.name,
        keyValue = pageId,
        column = PagesTableSchema.blobbedPage.name,
        newValue = Col.Blob(blob)
    )
}

internal suspend fun updatePagesStatus(pageId: String, blob: ByteArray) {
    setColumnWhere(
        table = PagesTableSchema.TABLE_NAME,
        keyCol = PagesTableSchema.pageId.name,
        keyValue = pageId,
        column = PagesTableSchema.pageStatus.name,
        newValue = Col.Blob(blob)
    )
}

internal suspend fun insertPageRow(
    pageId: String,
    isMainMenuPage: Boolean,
    blobbedPage: ByteArray,
    version: ByteArray
) {
    val row = newPageRow(pageId, isMainMenuPage, blobbedPage, version)
    DbManager.insertData(InsertDataIn(PagesTableSchema.TABLE_NAME, row)).getOrThrow()
}

internal suspend fun insertBlockRow(
    isTitle: Boolean,
    isPartOfMainMenuPage: Boolean,
    content: String,
    blockId: String,
    pageId: String,
    fracPosition: String
) {
    val row = newEveryBlockInExistenceRow(
        isTitle = isTitle,
        isPartOfMainMenuPage = isPartOfMainMenuPage,
        content = content,
        myIdAsGivenByYrs = blockId,
        idOfPageIBelongTo = pageId,
        fracPosition = fracPosition
    )
    DbManager.insertData(InsertDataIn(EveryBlockInExistenceTableSchema.TABLE_NAME, row)).getOrThrow()
}

internal suspend fun insertBlockRowAndGetId(
    isTitle: Boolean,
    isPartOfMainMenuPage: Boolean,
    content: String,
    blockId: String,
    pageId: String,
    fracPosition: String
): Long {
    val row = newEveryBlockInExistenceRow(
        isTitle = isTitle,
        isPartOfMainMenuPage = isPartOfMainMenuPage,
        content = content,
        myIdAsGivenByYrs = blockId,
        idOfPageIBelongTo = pageId,
        fracPosition = fracPosition
    )
    val out = DbManager.insertDataAndGetCol(InsertDataAndGetColIn(
        tableName = EveryBlockInExistenceTableSchema.TABLE_NAME,
        values = row,
        columnToReturn = "id"
    )).getOrThrow()
    return (out.value as Col.Integer).v1
}

internal suspend fun deleteEveryBlockRow(blockId: String) {
    DbManager.deleteRowWhere(DeleteRowWhereIn(
        tableName = EveryBlockInExistenceTableSchema.TABLE_NAME,
        arguments = SelectArguments.Single(SelectArgument.XEqualY(
            EveryBlockInExistenceTableSchema.myIdAsGivenByYrs.name, blockId
        ))
    )).getOrThrow()
}

internal suspend fun updateBlockFrac(blockId: String, newFrac: String) {
    setColumnWhere(
        table = EveryBlockInExistenceTableSchema.TABLE_NAME,
        keyCol = EveryBlockInExistenceTableSchema.myIdAsGivenByYrs.name,
        keyValue = blockId,
        column = EveryBlockInExistenceTableSchema.fracPosition.name,
        newValue = Col.Text(newFrac)
    )
}

internal suspend fun updateBlockContent(blockId: String, newContent: String) {
    setColumnWhere(
        table = EveryBlockInExistenceTableSchema.TABLE_NAME,
        keyCol = EveryBlockInExistenceTableSchema.myIdAsGivenByYrs.name,
        keyValue = blockId,
        column = EveryBlockInExistenceTableSchema.content.name,
        newValue = Col.Text(newContent)
    )
}

internal suspend fun markAllBlocksForPageDisabled(pageId: String) {
    setColumnWhere(
        table = EveryBlockInExistenceTableSchema.TABLE_NAME,
        keyCol = EveryBlockInExistenceTableSchema.idOfPageIBelongTo.name,
        keyValue = pageId,
        column = EveryBlockInExistenceTableSchema.pageIsDisabled.name,
        newValue = Col.Text("true")
    )
}

internal suspend fun insertClientLoveLetter(
    snapshotOfEdit: ByteArray,
    loveLetterSketch: ByteArray,
    sessionId: String,
    targetId: String
) {
    val row = newClientLoveLetterRow(snapshotOfEdit, loveLetterSketch, sessionId, targetId)
    DbManager.insertData(InsertDataIn(ClientLoveLetterTableSchema.TABLE_NAME, row)).getOrThrow()
}
