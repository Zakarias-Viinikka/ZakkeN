package z.zndroid.DocEvents

import rustlib.my_yrs_lib.BossOfYrs
import rustlib.my_yrs_lib.PositionToInsert
import rustlib.data_builder_for_operations_that_need_to_be_correct.addBlock
import rustlib.client_table_blueprints.newEveryBlockInExistenceRow
import rustlib.client_table_blueprints.newTableEveryBlockInExistence
import rustlib.client_table_blueprints.newClientLoveLetterRow
import rustlib.client_table_blueprints.newTableClientLoveLetter
import z.zndroid.DbManager
import uniffi.protocol.*
import z.zndroid.components.GlobalPopupManager
import z.zndroid.Storage.SessionManager
import z.zndroid.protocol.SafeRowMapper

/**
 * Context required to add a new block to a page.
 */
data class AddBlockCtx(
    val boss: BossOfYrs,          // The active Yrs document for the page
    val content: String,          // The text content of the block
    val position: PositionToInsert = PositionToInsert.AtEnd, // Where to insert
    val metadata: String = "",    // Extra metadata (JSON, styling, etc)
    val isTitle: Boolean = false, // Whether this is the title block
    val isMainMenu: Boolean = false // Whether this block belongs to a main-menu page
)

/**
 * Orchestrates adding a block using the Rust add_block executor pattern:
 * 1. Update pages.blobbed_page from pages_update.yrs_blob
 * 2. insert_data_and_get_col on every_block_in_existence with position = null
 * 3. edit_col_in_row on every_block_in_existence setting position to id as REAL
 * 4. Insert into client_love_letter
 */
object AddBlock {
    suspend fun execute(ctx: AddBlockCtx): Result<String> {
        return try {
            val sessionId = SessionManager.currentSessionId
            val isMainMenu = ctx.isMainMenu

            // 1. Call Rust builder to generate transaction context
            val everything = addBlock(
                yrs = ctx.boss,
                blockContent = ctx.content,
                blockMetaData = ctx.metadata,
                position = ctx.position,
                isPartOfMainMenuPage = isMainMenu,
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

                // Action 2: Insert into every_block_in_existence with position = null, returning id
                val insertCtx = everything.everyBlockInExistenceInsert
                val blockRow = newEveryBlockInExistenceRow(
                    isTitle = ctx.isTitle,
                    isPartOfMainMenuPage = insertCtx.isPartOfMainMenuPage,
                    content = insertCtx.content,
                    myIdAsGivenByYrs = insertCtx.myIdAsGivenByYrs,
                    idOfPageIBelongTo = insertCtx.idOfPageIBelongTo,
                    position = null
                )
                val blockValues = SafeRowMapper.mapRow(
                    row = blockRow,
                    columnDefs = newTableEveryBlockInExistence(),
                    expectedNames = listOf("is_title", "is_part_of_main_menu_page", "content", "my_id_as_given_by_yrs", "id_of_page_i_belong_to", "position", "page_is_disabled")
                )
                val insertAndGetRes = DbManager.insertDataAndGetCol(InsertDataAndGetColIn(
                    tableName = "every_block_in_existence",
                    values = blockValues,
                    columnToReturn = "id"
                )).getOrThrow()

                val newIdString = when (val colVal = insertAndGetRes.value) {
                    is Col.Integer -> colVal.v1.toString()
                    is Col.Real -> colVal.v1.toLong().toString()
                    is Col.Text -> colVal.v1
                    else -> throw Exception("Expected id, got ${insertAndGetRes.value}")
                }
                val newIdDouble = when (val colVal = insertAndGetRes.value) {
                    is Col.Integer -> colVal.v1.toDouble()
                    is Col.Real -> colVal.v1
                    else -> newIdString.toDouble()
                }

                // Action 3: Set position column to the row id as float (REAL)
                DbManager.editColInRow(EditColInRowIn(
                    tableName = "every_block_in_existence",
                    rowId = newIdString,
                    column = "position",
                    newValue = Col.Real(newIdDouble)
                )).getOrThrow()

                // Action 4: Insert into client_love_letter
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

            Result.success(everything.everyBlockInExistenceInsert.myIdAsGivenByYrs)
        } catch (e: Exception) {
            val errorMsg = "AddBlock failed: ${e.message ?: e.toString()}"
            GlobalPopupManager.showError(errorMsg)
            Result.failure(e)
        }
    }
}
