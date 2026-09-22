package z.zndroid.DocEvents

import rustlib.my_yrs_lib.BossOfYrs
import rustlib.my_yrs_lib.PositionToInsert
import rustlib.my_yrs_lib.createBookmarkOfSyncedState
import rustlib.my_yrs_lib.generateDiffSnapshot
import rustlib.love_letter.*
import rustlib.client_table_blueprints.newUncommittedDiffRow
import rustlib.client_table_blueprints.uncommittedDiffsColumns
import rustlib.client_table_blueprints.newEveryBlockInExistenceRow
import rustlib.client_table_blueprints.everyBlockInExistenceColumns
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
    val parentBlockId: String = "root" // Hierarchy parent
)

/**
 * Orchestrates adding a block: updates the Yrs doc, generates a diff,
 * and persists both the data and the sync metadata to SQLite.
 */
object AddBlock {
    private suspend fun computeDbPosition(ctx: AddBlockCtx): Double {
        val pageId = ctx.boss.pageId()
        val colDefs = everyBlockInExistenceColumns()
        val posIdx = colDefs.indexOfFirst { it.name == "position" }
        if (posIdx < 0) return 1.0

        val existing = DbManager.getData(GetDataIn(
            "every_block_in_existence",
            SelectArguments.Single(SelectArgument.XEqualY("id_of_page_i_belong_to", pageId)),
            emptyList()
        )).getOrNull()

        val positions = existing?.rows?.mapNotNull { row ->
            (row.cols.getOrNull(posIdx) as? Col.Real)?.v1
        }?.sorted() ?: emptyList()

        val insertIndex = when (val p = ctx.position) {
            PositionToInsert.AtEnd -> positions.size
            is PositionToInsert.SpecificPosition -> p.v1.toInt()
        }

        return when {
            positions.isEmpty() -> 1.0
            insertIndex <= 0 -> positions.first() - 1.0
            insertIndex >= positions.size -> positions.last() + 1.0
            else -> (positions[insertIndex - 1] + positions[insertIndex]) / 2.0
        }
    }

    suspend fun execute(ctx: AddBlockCtx): Result<String> {
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
            
            // 5. Build the data row for 'every_block_in_existence' using Rust helpers
            val dbPosition = computeDbPosition(ctx)
            val blockRow = newEveryBlockInExistenceRow(
                isTitle = ctx.isTitle,
                content = ctx.content,
                myIdAsGivenByYrs = blockId,
                idOfPageIBelongTo = pageId,
                position = dbPosition
            )
            val blockValues = SafeRowMapper.mapRow(
                row = blockRow,
                columnDefs = everyBlockInExistenceColumns(),
                expectedNames = listOf("is_title", "content", "my_id_as_given_by_yrs", "id_of_page_i_belong_to", "position")
            )

            // 6. Build the sync row for 'uncommitted_diffs'
            val diffRow = newUncommittedDiffRow(
                snapshotOfEdit = diff,
                loveLetterSketch = sketchBytes,
                sessionId = sessionId,
                targetId = blockId 
            )
            val diffValues = SafeRowMapper.mapRow(
                row = diffRow,
                columnDefs = uncommittedDiffsColumns(),
                expectedNames = listOf("snapshot_of_edit", "love_letter_sketch", "session_id", "target_id")
            )

            // 7. Persistence to SQLite
            DbManager.withTransaction {
                DbManager.insertData(InsertDataIn("every_block_in_existence", blockValues)).getOrThrow()
                DbManager.insertData(InsertDataIn("uncommitted_diffs", diffValues)).getOrThrow()
                
                // 8. Update the full page snapshot in the 'pages' table
                val newSnapshot = ctx.boss.snapshot()
                DbManager.updatePageSnapshot(pageId, newSnapshot).getOrThrow()
            }.getOrThrow()

            Result.success(blockId)
        } catch (e: Exception) {
            val errorMsg = "AddBlock failed: ${e.message ?: e.toString()}"
            GlobalPopupManager.show(errorMsg)
            Result.failure(e)
        }
    }
}
