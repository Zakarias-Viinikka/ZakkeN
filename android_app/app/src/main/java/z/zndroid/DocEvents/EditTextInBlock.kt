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
import z.zndroid.protocol.SafeRowMapper

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
            
            // 7. Persistence to SQLite
            DbManager.withTransaction {
                // Update sync table 'uncommitted_diffs'
                val diffRow = newUncommittedDiffRow(
                    snapshotOfEdit = diff,
                    loveLetterSketch = sketchBytes,
                    sessionId = sessionId,
                    targetId = ctx.blockId 
                )
                val diffValues = SafeRowMapper.mapRow(
                    row = diffRow,
                    columnDefs = uncommittedDiffsColumns(),
                    expectedNames = listOf("snapshot_of_edit", "love_letter_sketch", "session_id", "target_id")
                )
                DbManager.insertData(InsertDataIn("uncommitted_diffs", diffValues)).getOrThrow()

                // 8. Update queryable block table 'every_block_in_existence'
                // First, find the internal auto-increment ID
                val queryRes = DbManager.getData(GetDataIn(
                    "every_block_in_existence",
                    listOf(SelectArgument.XEqualY("my_id_as_given_by_yrs", ctx.blockId, null)),
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
            }.getOrThrow()

            Result.success(Unit)
        } catch (e: Exception) {
            val errorMsg = "EditTextInBlock failed: ${e.message ?: e.toString()}"
            GlobalPopupManager.show(errorMsg)
            Result.failure(e)
        }
    }

    /**
     * Flushes the buffered edits for a block, squashing them first.
     * Strict "all-or-nothing" model: if it fails, the in-memory state is discarded
     * and a page reload is required to recover from the last successful DB snapshot.
     */
    suspend fun flushBuffer(
        boss: BossOfYrs,
        state: BlockUiState,
        onHardReload: () -> Unit
    ): Result<Unit> {
        if (state.diffBuffer.isEmpty()) return Result.success(Unit)

        // 1. Snapshot the buffer and clear it immediately. 
        // We don't retry; the authoritative state is the last successful flush in DB.
        val itemsToFlush = state.diffBuffer.toList()
        state.diffBuffer.clear()

        return try {
            DbManager.withTransaction {
                // 2. Squash the snapshot using the Rust library
                val squashed = combineGetdiffResults(itemsToFlush)
                if (squashed.isEmpty()) return@withTransaction

                // 3. Capture baseline for CRDT diff
                var currentBookmark = createBookmarkOfSyncedState(boss)
                val pageId = boss.pageId()
                val sessionId = SessionManager.currentSessionId
                val colDefs = uncommittedDiffsColumns()
                val expectedNames = listOf("snapshot_of_edit", "love_letter_sketch", "session_id", "target_id")

                // 4. Apply each squashed edit to in-memory BossOfYrs
                squashed.forEach { diff ->
                    val textEdit = when (diff) {
                        is DiffResult.Insert -> TextEdit.Insert(diff.v1, diff.v2)
                        is DiffResult.Delete -> TextEdit.Delete(diff.v1, diff.v2)
                        is DiffResult.Replace -> TextEdit.Replace(diff.oldText, diff.newText, diff.position)
                        else -> return@forEach
                    }

                    boss.editTextBlock(state.blockId, textEdit, EditTarget.TEXT)

                    // 5. Build sync row
                    val diffSnapshot = generateDiffSnapshot(boss, currentBookmark)
                    val sketch = LoveLetterSketch.EditBlock(
                        textEdit = textEdit,
                        editTarget = EditTarget.TEXT,
                        targetPageId = pageId,
                        blockId = state.blockId
                    )
                    val sketchBytes = sketchToBytes(sketch)
                    
                    val diffRow = newUncommittedDiffRow(
                        snapshotOfEdit = diffSnapshot,
                        loveLetterSketch = sketchBytes,
                        sessionId = sessionId,
                        targetId = state.blockId
                    )
                    
                    // Refresh bookmark for incremental snapshots
                    currentBookmark = createBookmarkOfSyncedState(boss)

                    // 6. DB Write Part 1: uncommitted_diffs
                    val diffValues = SafeRowMapper.mapRow(diffRow, colDefs, expectedNames)
                    DbManager.insertData(InsertDataIn("uncommitted_diffs", diffValues)).getOrThrow()
                }

                // 7. DB Write Part 2: update every_block_in_existence
                val queryRes = DbManager.getData(GetDataIn(
                    "every_block_in_existence",
                    listOf(SelectArgument.XEqualY("my_id_as_given_by_yrs", state.blockId, null)),
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

                // 8. DB Write Part 3: update pages.blobbed_page (Authoritative State)
                val newSnapshot = boss.snapshot()
                DbManager.updatePageSnapshot(pageId, newSnapshot).getOrThrow()
            }.getOrThrow()

            // 9. Success: Update tracking state
            state.lastPersistedText = state.text
            Result.success(Unit)

        } catch (e: Exception) {
            // Failure Recovery: discard in-memory state and reload from DB.
            val errorMsg = "Something went wrong and your edits couldn't be saved. " +
                           "Reload the page to recover a consistent state. " +
                           "If there's anything important on screen that you don't want to lose, copy it now before reloading."
            GlobalPopupManager.show(errorMsg)
            onHardReload() 
            Result.failure(e)
        }
    }
}
