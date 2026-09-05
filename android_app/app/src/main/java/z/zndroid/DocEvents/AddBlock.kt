package z.zndroid.DocEvents

import rustlib.my_yrs_lib.BossOfYrs
import rustlib.my_yrs_lib.createBookmarkOfSyncedState
import rustlib.my_yrs_lib.generateDiffSnapshot
import rustlib.client_table_blueprints.newEveryBlockInExistenceRow
import rustlib.client_table_blueprints.newUncommittedDiffRow
import rustlib.client_table_blueprints.everyBlockInExistenceColumns
import rustlib.client_table_blueprints.uncommittedDiffsColumns
import z.zndroid.DbManager
import uniffi.protocol.InsertDataIn
import uniffi.protocol.ColumnValue

/**
 * Context required to add a new block to a page.
 */
data class AddBlockCtx(
    val boss: BossOfYrs,          // The active Yjs document for the page
    val pageId: String,           // The ID of the page (owner)
    val content: String,          // The text content of the block
    val metadata: String = "",    // Extra metadata (JSON, styling, etc)
    val parentBlockId: String = "root" // Hierarchy parent
)

/**
 * Orchestrates adding a block: updates the Yjs doc, generates a diff,
 * and persists both the data and the sync metadata to SQLite.
 */
object AddBlock {
    suspend fun execute(ctx: AddBlockCtx): Result<Unit> {
        return try {
            // 1. Capture the "Before" state of the Yjs document
            val bookmark = createBookmarkOfSyncedState(ctx.boss)

            // 2. Perform the edit on the Yjs document (in-memory)
            // Note: This creates the block in the Yjs tree. 
            // We should ideally retrieve the generated ID from the Rust side here.
            ctx.boss.insertNewBlock(ctx.content, ctx.metadata)

            // 3. Generate the binary diff update for synchronization
            val diff = generateDiffSnapshot(ctx.boss, bookmark)
            val sessionId = ctx.boss.getUserId().toLong()
            
            // 4. Build the data row for 'every_block_in_existence'
            val blockRow = newEveryBlockInExistenceRow(
                pageThatOwnsMe = ctx.pageId,
                content = ctx.content,
                idOfBlockThatOwns = ctx.parentBlockId
            )
            val blockCols = everyBlockInExistenceColumns()
            val blockValues = blockRow.cols.mapIndexed { index, col ->
                // index + 1 to skip the auto-increment 'id' column
                ColumnValue(blockCols[index + 1].name, col)
            }

            // 5. Build the sync row for 'uncommitted_diffs'
            // We use a dummy targetId for now, but this should be the Yjs Block ID
            val diffRow = newUncommittedDiffRow(
                snapshotOfEdit = diff,
                editEnum = byteArrayOf(1), // 1 = INSERT operation
                sessionId = sessionId,
                targetId = "temp_yrs_id" 
            )
            val diffCols = uncommittedDiffsColumns()
            val diffValues = diffRow.cols.mapIndexed { index, col ->
                // index + 1 to skip the auto-increment 'id' column
                ColumnValue(diffCols[index + 1].name, col)
            }

            // 6. Persistence to SQLite
            // We insert both the actual data and the sync record
            DbManager.insertData(InsertDataIn("every_block_in_existence", blockValues)).getOrThrow()
            DbManager.insertData(InsertDataIn("uncommitted_diffs", diffValues)).getOrThrow()

            Result.success(Unit)
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
}
