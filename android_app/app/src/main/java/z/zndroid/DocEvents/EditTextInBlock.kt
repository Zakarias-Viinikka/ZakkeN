package z.zndroid.DocEvents

import rustlib.my_yrs_lib.*
import rustlib.love_letter.*
import rustlib.text_diff.getDiff
import rustlib.text_diff.DiffResult
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
    val oldText: String,          // Previous text (for diffing)
    val newText: String,          // New text
    val editTarget: EditTarget = EditTarget.TEXT 
)

/**
 * Orchestrates editing text in a block: computes diff, updates Yrs, 
 * persists to sync table, and updates queryable block table.
 */
object EditTextInBlock {
    suspend fun execute(ctx: EditTextInBlockCtx): Result<Unit> {
        return try {
            // 1. Compute the diff using text_diff library
            val diffResult = getDiff(ctx.oldText, ctx.newText)
            
            // 2. Convert DiffResult to Yrs TextEdit
            val textEdit = when (diffResult) {
                is DiffResult.Insert -> TextEdit.Insert(diffResult.v1, diffResult.v2)
                is DiffResult.Delete -> TextEdit.Delete(diffResult.v1, diffResult.v2)
                is DiffResult.Replace -> TextEdit.Replace(diffResult.oldText, diffResult.newText, diffResult.position)
                is DiffResult.NoDiff -> return Result.success(Unit) // No changes detected
            }

            // 3. Capture the "Before" state of the Yrs document
            val bookmark = createBookmarkOfSyncedState(ctx.boss)

            // 4. Apply the edit to the Yrs document
            ctx.boss.editTextBlock(
                blockId = ctx.blockId,
                textEdit = textEdit,
                editTarget = ctx.editTarget
            )

            // 5. Generate the binary diff update for synchronization
            val diff = generateDiffSnapshot(ctx.boss, bookmark)
            val sessionId = SessionManager.currentSessionId
            val pageId = ctx.boss.pageId()

            // 6. Construct the LoveLetter intent (sketch)
            val sketch = LoveLetterSketch.EditBlock(
                textEdit = textEdit,
                editTarget = ctx.editTarget,
                targetPageId = pageId
            )
            val sketchBytes = sketchToBytes(sketch)
            
            // 7. Update sync table 'uncommitted_diffs'
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
            DbManager.insertData(InsertDataIn("uncommitted_diffs", diffValues)).getOrThrow()

            // 8. Update queryable block table 'every_block_in_existence'
            // First, find the internal auto-increment ID
            val queryRes = DbManager.getData(GetDataIn(
                "every_block_in_existence",
                listOf(SelectArgument.XEqualY("my_id_as_given_by_yrs", ctx.blockId)),
                emptyList()
            )).getOrThrow()

            if (queryRes.rows.isNotEmpty()) {
                val internalId = when (val idCol = queryRes.rows.first().cols.first()) {
                    is Col.Integer -> idCol.v1.toString()
                    else -> throw Exception("Failed to get internal ID for block ${ctx.blockId}")
                }

                // Update content and title
                DbManager.editColInRow(EditColInRowIn(
                    tableName = "every_block_in_existence",
                    rowId = internalId,
                    column = "content",
                    newValue = Col.Text(ctx.newText)
                )).getOrThrow()

                DbManager.editColInRow(EditColInRowIn(
                    tableName = "every_block_in_existence",
                    rowId = internalId,
                    column = "title",
                    newValue = Col.Text(ctx.newText)
                )).getOrThrow()
            }
            
            // 9. Update the full page snapshot in the 'pages' table
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
