package z.zndroid.DocEvents

import rustlib.my_yrs_lib.BossOfYrs
import rustlib.my_yrs_lib.PositionToInsert
import rustlib.my_yrs_lib.createBookmarkOfSyncedState
import rustlib.my_yrs_lib.generateDiffSnapshot
import rustlib.client_table_blueprints.newEveryBlockInExistenceRow
import rustlib.client_table_blueprints.newUncommittedDiffRow
import rustlib.client_table_blueprints.everyBlockInExistenceColumns
import rustlib.client_table_blueprints.uncommittedDiffsColumns
import z.zndroid.DbManager
import uniffi.protocol.InsertDataIn
import uniffi.protocol.ColumnValue
import z.zndroid.components.GlobalPopupManager

/**
 * Context required to add a new block to a page.
 */
data class AddBlockCtx(
    val boss: BossOfYrs,          // The active Yrs document for the page
    val content: String,          // The text content of the block
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
                position = PositionToInsert.AtEnd
            )

            // 3. Generate the binary diff update for synchronization
            val diff = generateDiffSnapshot(ctx.boss, bookmark)
            val sessionId = System.currentTimeMillis().toString()
            val pageId = ctx.boss.pageId()
            
            // 4. Build the data row for 'every_block_in_existence'
            val blockRow = newEveryBlockInExistenceRow(
                pageThatOwnsMe = pageId,
                content = ctx.content,
                idOfBlockThatOwns = ctx.parentBlockId
            )
            val blockCols = everyBlockInExistenceColumns()
            val blockValues = blockRow.cols.mapIndexed { index, col ->
                // index + 1 to skip the auto-increment 'id' column
                ColumnValue(blockCols[index + 1].name, col)
            }

            // 5. Build the sync row for 'uncommitted_diffs'
            val diffRow = newUncommittedDiffRow(
                snapshotOfEdit = diff,
                editEnum = "add_block".toByteArray(),
                sessionId = sessionId,
                targetId = blockId 
            )
            val diffCols = uncommittedDiffsColumns()
            val diffValues = diffRow.cols.mapIndexed { index, col ->
                // index + 1 to skip the auto-increment 'id' column
                ColumnValue(diffCols[index + 1].name, col)
            }

            // 6. Persistence to SQLite
            DbManager.insertData(InsertDataIn("every_block_in_existence", blockValues)).getOrThrow()
            DbManager.insertData(InsertDataIn("uncommitted_diffs", diffValues)).getOrThrow()
            
            // 7. Update the full page snapshot in the 'pages' table
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
