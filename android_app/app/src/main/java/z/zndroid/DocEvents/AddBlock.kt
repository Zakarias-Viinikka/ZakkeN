package z.zndroid.DocEvents

import rustlib.my_yrs_lib.BossOfYrs
import rustlib.my_yrs_lib.PositionToInsert
import rustlib.my_yrs_lib.createBookmarkOfSyncedState
import rustlib.my_yrs_lib.generateDiffSnapshot
import rustlib.love_letter.*
import rustlib.client_table_blueprints.newUncommittedDiffRow
import rustlib.client_table_blueprints.everyBlockInExistenceColumns
import rustlib.client_table_blueprints.uncommittedDiffsColumns
import z.zndroid.DbManager
import uniffi.protocol.*
import z.zndroid.components.GlobalPopupManager
import z.zndroid.Storage.SessionManager

/**
 * Context required to add a new block to a page.
 */
data class AddBlockCtx(
    val boss: BossOfYrs,          // The active Yrs document for the page
    val content: String,          // The text content of the block
    val position: PositionToInsert = PositionToInsert.AtEnd, // Where to insert
    val metadata: String = "",    // Extra metadata (JSON, styling, etc)
    val parentBlockId: String = "root" // Hierarchy parent
)

/**
 * Orchestrates adding a block: updates the Yrs doc, generates a diff,
 * and persists both the data and the sync metadata to SQLite.
 */
object AddBlock {
    suspend fun execute(ctx: AddBlockCtx): Result<Unit> {
        return try {
            // 1. Capture the "Before" state of the Yrs document
            val bookmark = createBookmarkOfSyncedState(ctx.boss)

            // 2. Perform the edit on the Yrs document (in-memory)
            // returns the newly generated block ID
            val blockId = ctx.boss.insertNewBlock(
                blockContent = ctx.content,
                blockMetaData = ctx.metadata,
                position = ctx.position
            )

            // 3. Generate the binary diff update for synchronization
            val diff = generateDiffSnapshot(ctx.boss, bookmark)
            val sessionId = SessionManager.currentSessionId
            val pageId = ctx.boss.pageId()

            // 4. Construct the LoveLetter intent (sketch)
            val sketch = LoveLetterSketch.CreateNewBlock(
                positionToInsert = ctx.position,
                targetPageId = pageId
            )
            val sketchBytes = sketchToBytes(sketch)
            
            // 5. Build the data row for 'every_block_in_existence'
            // We build the ColumnValue list manually because the Row helper is currently out of sync with the schema
            val blockValues = listOf(
                ColumnValue("title", Col.Text(ctx.content)),
                ColumnValue("page_that_owns_me", Col.Text(pageId)),
                ColumnValue("content", Col.Text(ctx.content)),
                ColumnValue("my_id_as_given_by_yrs", Col.Text(blockId)),
                ColumnValue("id_of_page_i_belong_to", Col.Text(pageId))
            )

            // 6. Build the sync row for 'uncommitted_diffs'
            val diffRow = newUncommittedDiffRow(
                snapshotOfEdit = diff,
                loveLetterSketch = sketchBytes,
                sessionId = sessionId,
                targetId = blockId 
            )
            val diffCols = uncommittedDiffsColumns()
            val diffValues = diffRow.cols.mapIndexed { index, col ->
                // index + 1 to skip the auto-increment 'id' column
                ColumnValue(diffCols[index + 1].name, col)
            }

            // 7. Persistence to SQLite
            DbManager.insertData(InsertDataIn("every_block_in_existence", blockValues)).getOrThrow()
            DbManager.insertData(InsertDataIn("uncommitted_diffs", diffValues)).getOrThrow()
            
            // 8. Update the full page snapshot in the 'pages' table
            val newSnapshot = ctx.boss.snapshot()
            DbManager.updatePageSnapshot(pageId, newSnapshot).getOrThrow()

            Result.success(Unit)
        } catch (e: Exception) {
            val errorMsg = "AddBlock failed: ${e.message ?: e.toString()}"
            GlobalPopupManager.show(errorMsg)
            Result.failure(e)
        }
    }
}
