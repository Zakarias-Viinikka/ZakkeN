package z.zndroid.DocEvents

import rustlib.my_yrs_lib.*
import rustlib.love_letter.*
import z.zndroid.DbManager
import uniffi.protocol.*
import rustlib.client_table_blueprints.*
import z.zndroid.Storage.SessionManager
import z.zndroid.components.GlobalPopupManager
import z.zndroid.protocol.SafeRowMapper

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
            ctx.boss.deleteBlock(ctx.blockId)

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
            val diffValues = SafeRowMapper.mapRow(
                row = diffRow,
                columnDefs = uncommittedDiffsColumns(),
                expectedNames = listOf("snapshot_of_edit", "love_letter_sketch", "session_id", "target_id")
            )

            // 6. Persistence to SQLite
            DbManager.withTransaction {
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
                    DbManager.deleteRow(DeleteRowIn("every_block_in_existence", internalId)).getOrThrow()
                }
                
                DbManager.insertData(InsertDataIn("uncommitted_diffs", diffValues)).getOrThrow()
                
                // 7. Update the full page snapshot in the 'pages' table
                val newSnapshot = ctx.boss.snapshot()
                DbManager.updatePageSnapshot(pageId, newSnapshot).getOrThrow()
            }.getOrThrow()

            Result.success(Unit)
        } catch (e: Exception) {
            val errorMsg = "RemoveBlock failed: ${e.message ?: e.toString()}"
            GlobalPopupManager.show(errorMsg)
            Result.failure(e)
        }
    }
}
