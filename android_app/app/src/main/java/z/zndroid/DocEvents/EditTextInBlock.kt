package z.zndroid.DocEvents

import rustlib.my_yrs_lib.*
import rustlib.love_letter.*
import rustlib.text_diff.getDiff
import rustlib.text_diff.DiffResult
import rustlib.text_diff.combineGetdiffResults
import z.zndroid.DbManager
import uniffi.protocol.*
import rustlib.client_table_blueprints.*
import z.zndroid.Storage.SessionManager
import z.zndroid.components.GlobalPopupManager
import z.zndroid.MainPages.ViewPage.BlockUiState

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
                targetPageId = pageId,
                blockId = ctx.blockId
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

                // Update content
                DbManager.editColInRow(EditColInRowIn(
                    tableName = "every_block_in_existence",
                    rowId = internalId,
                    column = "content",
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

    /**
     * Flushes the buffered edits for a block, squashing them first.
     */
    suspend fun flushBuffer(boss: BossOfYrs, state: BlockUiState): Result<Unit> {
        if (state.diffBuffer.isEmpty()) return Result.success(Unit)

        return try {
            val bufferCopy = state.diffBuffer.toList()
            state.diffBuffer.clear()

            // 1. Squash the edits using the Rust library
            val squashed = combineGetdiffResults(bufferCopy)
            if (squashed.isEmpty()) return Result.success(Unit)

            // 2. Capture baseline for CRDT diff
            val bookmark = createBookmarkOfSyncedState(boss)
            val pageId = boss.pageId()
            val sessionId = SessionManager.currentSessionId

            // 3. Apply each squashed edit
            squashed.forEach { diff ->
                val textEdit = when (diff) {
                    is DiffResult.Insert -> TextEdit.Insert(diff.v1, diff.v2)
                    is DiffResult.Delete -> TextEdit.Delete(diff.v1, diff.v2)
                    is DiffResult.Replace -> TextEdit.Replace(diff.oldText, diff.newText, diff.position)
                    else -> return@forEach
                }

                // Apply to CRDT
                boss.editTextBlock(state.blockId, textEdit, EditTarget.TEXT)

                // 4. Create sync intent (LoveLetterSketch) for this edit
                val sketch = LoveLetterSketch.EditBlock(
                    textEdit = textEdit,
                    editTarget = EditTarget.TEXT,
                    targetPageId = pageId,
                    blockId = state.blockId
                )
                val sketchBytes = sketchToBytes(sketch)

                // 5. Build sync row
                // We generate a snapshot of the specific change since the bookmark
                // Note: If we have multiple diffs, we might want to generate a snapshot after EACH one 
                // to be strictly correct, but usually squashing leads to one.
                val diffSnapshot = generateDiffSnapshot(boss, bookmark)
                val diffRow = newUncommittedDiffRow(
                    snapshotOfEdit = diffSnapshot,
                    loveLetterSketch = sketchBytes,
                    sessionId = sessionId,
                    targetId = state.blockId
                )
                val diffCols = uncommittedDiffsColumns()
                val diffValues = diffRow.cols.mapIndexed { index, col ->
                    ColumnValue(diffCols[index + 1].name, col)
                }
                DbManager.insertData(InsertDataIn("uncommitted_diffs", diffValues)).getOrThrow()
            }

            // 6. Update the main queryable tables ONCE for the whole batch
            val queryRes = DbManager.getData(GetDataIn(
                "every_block_in_existence",
                listOf(SelectArgument.XEqualY("my_id_as_given_by_yrs", state.blockId)),
                emptyList()
            )).getOrThrow()

            if (queryRes.rows.isNotEmpty()) {
                val internalId = when (val idCol = queryRes.rows.first().cols.first()) {
                    is Col.Integer -> idCol.v1.toString()
                    else -> throw Exception("Failed to get internal ID for block ${state.blockId}")
                }
                DbManager.editColInRow(EditColInRowIn(
                    tableName = "every_block_in_existence",
                    rowId = internalId,
                    column = "content",
                    newValue = Col.Text(state.text)
                )).getOrThrow()
            }

            // 7. Update the full page snapshot
            val newSnapshot = boss.snapshot()
            DbManager.updatePageSnapshot(pageId, newSnapshot).getOrThrow()

            // 8. Update tracking state
            state.lastPersistedText = state.text

            Result.success(Unit)
        } catch (e: Exception) {
            val errorMsg = "FlushBuffer failed: ${e.message ?: e.toString()}"
            GlobalPopupManager.show(errorMsg)
            Result.failure(e)
        }
    }
}
