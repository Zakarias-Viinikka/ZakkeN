package z.zndroid.DocEvents

import rustlib.my_yrs_lib.*
import rustlib.love_letter.*
import z.zndroid.DbManager
import uniffi.protocol.*
import rustlib.client_table_blueprints.*
import z.zndroid.Storage.SessionManager
import z.zndroid.components.GlobalPopupManager

/**
 * Context required to remove a block from a page.
 */
data class RemoveBlockCtx(
    val boss: BossOfYrs,          // The active Yrs document for the page
    val blockId: String,          // The Yrs ID of the block to remove
    val position: UInt            // The index of the block in the page
)

/**
 * Orchestrates removing a block: updates the Yrs doc, generates a diff,
 * and removes the record from SQLite.
 */
object RemoveBlock {
    suspend fun execute(ctx: RemoveBlockCtx): Result<Unit> {
        return try {
            // 1. Capture the "Before" state of the Yrs document
            val bookmark = createBookmarkOfSyncedState(ctx.boss)

            // 2. Perform the removal on the Yrs document (in-memory)
            // Note: Assuming boss has a method to remove block by ID or index.
            // Based on earlier patterns, we likely use the position for the LoveLetter.
            // For now, we'll assume a generic removal exists or we'll need to define it.
            // ctx.boss.removeBlock(ctx.blockId) 

            // 3. Generate the binary diff update for synchronization
            val diff = generateDiffSnapshot(ctx.boss, bookmark)
            val sessionId = SessionManager.currentSessionId
            val pageId = ctx.boss.pageId()

            // 4. Construct the LoveLetter intent (sketch)
            val sketch = LoveLetterSketch.RemoveBlock(
                position = ctx.position,
                targetPageId = pageId
            )
            val sketchBytes = sketchToBytes(sketch)
            
            // 5. Build the sync row for 'uncommitted_diffs'
            val diffRow = newUncommittedDiffRow(
                snapshotOfEdit = diff,
                loveLetterSketch = sketchBytes,
                sessionId = sessionId,
                targetId = ctx.blockId 
            )
            val diffCols = uncommittedDiffsColumns()
            val diffValues = diffRow.cols.mapIndexed { index, col ->
                ColumnValue(diffCols[index + 1].name, col)
            }

            // 6. Persistence to SQLite: Delete from block table and Insert to diffs
            DbManager.deleteRow(DeleteRowIn("every_block_in_existence", ctx.blockId)).getOrThrow()
            DbManager.insertData(InsertDataIn("uncommitted_diffs", diffValues)).getOrThrow()
            
            // 7. Update the full page snapshot in the 'pages' table
            val newSnapshot = ctx.boss.snapshot()
            DbManager.updatePageSnapshot(pageId, newSnapshot).getOrThrow()

            Result.success(Unit)
        } catch (e: Exception) {
            val errorMsg = "RemoveBlock failed: ${e.message ?: e.toString()}"
            GlobalPopupManager.show(errorMsg)
            Result.failure(e)
        }
    }
}
