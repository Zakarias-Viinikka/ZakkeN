package z.zndroid.DocEvents

import rustlib.my_yrs_lib.BossOfYrs
import rustlib.data_builder_for_operations_that_need_to_be_correct.deleteBlock
import rustlib.client_table_blueprints.newClientLoveLetterRow
import rustlib.client_table_blueprints.newTableClientLoveLetter
import z.zndroid.DbManager
import uniffi.protocol.*
import z.zndroid.components.GlobalPopupManager
import z.zndroid.Storage.SessionManager
import z.zndroid.protocol.SafeRowMapper

/**
 * Context required to delete a block from a page.
 */
data class DeleteBlockCtx(
    val boss: BossOfYrs,          // The active Yrs document for the page
    val blockId: String           // The Yrs ID of the block to delete
)

/**
 * Orchestrates deleting a block using the Rust delete_block executor pattern:
 * 1. Update pages.blobbed_page from pages_update.yrs_blob
 * 2. delete_row_where on every_block_in_existence where my_id_as_given_by_yrs = blockId
 * 3. Insert into client_love_letter
 */
object DeleteBlock {
    suspend fun execute(ctx: DeleteBlockCtx): Result<Unit> {
        return try {
            val sessionId = SessionManager.currentSessionId

            // 1. Call Rust builder to generate transaction context
            val everything = deleteBlock(
                yrs = ctx.boss,
                blockId = ctx.blockId,
                sessionId = sessionId
            )

            // 2. Persistence to SQLite inside a single transaction
            DbManager.withTransaction {
                // Action 1: Update pages.blobbed_page
                val pagesUpdate = everything.pagesUpdate
                val pageRow = DbManager.getPage(pagesUpdate.pageId).getOrThrow()
                val checkOut = DbManager.checkTable(CheckTableIn("pages")).getOrThrow()
                val idColIdx = checkOut.columns.indexOfFirst { it.name == "id" }
                val internalPageId = when (val idCol = pageRow.cols.getOrNull(idColIdx)) {
                    is Col.Integer -> idCol.v1.toString()
                    is Col.Text -> idCol.v1
                    else -> throw Exception("Could not determine internal ID for page ${pagesUpdate.pageId}")
                }
                DbManager.editColInRow(EditColInRowIn(
                    tableName = "pages",
                    rowId = internalPageId,
                    column = "blobbed_page",
                    newValue = Col.Blob(pagesUpdate.yrsBlob)
                )).getOrThrow()

                // Action 2: delete_row_where on every_block_in_existence matching my_id_as_given_by_yrs
                DbManager.deleteRowWhere(DeleteRowWhereIn(
                    tableName = "every_block_in_existence",
                    arguments = SelectArguments.Single(SelectArgument.XEqualY("my_id_as_given_by_yrs", ctx.blockId))
                )).getOrThrow()

                // Action 3: Insert into client_love_letter
                val llCtx = everything.clientLoveLetter
                val loveLetterRow = newClientLoveLetterRow(
                    snapshotOfEdit = llCtx.snapshotOfEdit,
                    loveLetterSketch = llCtx.loveLetterSketch,
                    sessionId = llCtx.sessionId,
                    targetId = llCtx.targetId
                )
                val loveLetterValues = SafeRowMapper.mapRow(
                    row = loveLetterRow,
                    columnDefs = newTableClientLoveLetter(),
                    expectedNames = listOf("snapshot_of_edit", "love_letter_sketch", "session_id", "target_id")
                )
                DbManager.insertData(InsertDataIn("client_love_letter", loveLetterValues)).getOrThrow()
            }.getOrThrow()

            Result.success(Unit)
        } catch (e: Exception) {
            val errorMsg = "DeleteBlock failed: ${e.message ?: e.toString()}"
            GlobalPopupManager.show(errorMsg)
            Result.failure(e)
        }
    }
}
