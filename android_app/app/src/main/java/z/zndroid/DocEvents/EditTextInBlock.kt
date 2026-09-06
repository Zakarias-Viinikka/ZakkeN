package z.zndroid.DocEvents

import rustlib.my_yrs_lib.*
import rustlib.love_letter.*
import z.zndroid.DbManager
import uniffi.protocol.*
import rustlib.client_table_blueprints.*
import z.zndroid.Storage.SessionManager
import z.zndroid.components.GlobalPopupManager

/**
 * Context required to edit text within a block.
 */
data class EditTextInBlockCtx(
    val boss: BossOfYrs,          // The active Yrs document for the page
    val blockId: String,          // The Yrs ID of the block to edit
    val textEdit: TextEdit,       // The edit operation (Insert, Delete, or Replace)
    val editTarget: EditTarget = EditTarget.TEXT // Whether editing text or metadata
)

/**
 * Orchestrates editing text in a block: updates the Yrs doc, generates a diff,
 * and persists the intent to SQLite.
 */
object EditTextInBlock {
    suspend fun execute(ctx: EditTextInBlockCtx): Result<Unit> {
        return try {
            // 1. Capture the "Before" state of the Yrs document
            val bookmark = createBookmarkOfSyncedState(ctx.boss)

            // 2. Apply the edit to the Yrs document
            // This method handles Insert, Delete, and Replace variants of TextEdit
            ctx.boss.editTextBlockInsert(
                blockId = ctx.blockId,
                textEdit = ctx.textEdit,
                editTarget = ctx.editTarget
            )

            // 3. Generate the binary diff update for synchronization
            val diff = generateDiffSnapshot(ctx.boss, bookmark)
            val sessionId = SessionManager.currentSessionId
            val pageId = ctx.boss.pageId()

            // 4. Construct the LoveLetter intent (sketch)
            val sketch = LoveLetterSketch.EditBlock(
                textEdit = ctx.textEdit,
                editTarget = ctx.editTarget,
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
                // index + 1 to skip the auto-increment 'id' column
                ColumnValue(diffCols[index + 1].name, col)
            }

            // 6. Persistence to SQLite
            DbManager.insertData(InsertDataIn("uncommitted_diffs", diffValues)).getOrThrow()
            
            // 7. Update the full page snapshot in the 'pages' table
            val newSnapshot = ctx.boss.snapshot()
            DbManager.updatePageSnapshot(pageId, newSnapshot).getOrThrow()

            Result.success(Unit)
        } catch (e: Exception) {
            val errorMsg = "EditTextInBlock failed: ${e.message ?: e.toString()}"
            GlobalPopupManager.show(errorMsg)
            Result.failure(e)
        }
    }
}
