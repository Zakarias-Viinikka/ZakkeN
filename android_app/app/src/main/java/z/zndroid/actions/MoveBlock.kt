package z.zndroid.actions

import rustlib.data_builder_for_operations_that_need_to_be_correct.addBlock
import rustlib.data_builder_for_operations_that_need_to_be_correct.deleteBlock
import rustlib.my_yrs_lib.BossOfYrs
import rustlib.my_yrs_lib.PositionToInsert
import z.zndroid.DbManager
import z.zndroid.Storage.SessionManager
import z.zndroid.WriteLock
import z.zndroid.db.reads.isMainMenuPage
import z.zndroid.editor.OpenPageRegistry
import z.zndroid.executor.addBlockRequiresThreeDbActions
import z.zndroid.executor.deleteBlockRequiresThreeDbActions

/**
 * Move a block from one page's yrs doc to another page's yrs doc.
 *
 * Mirrors web's `move_block_to_another_page` wrapped in `run_exclusive`:
 * any open PageSession for either page is flushed first, so pending
 * edits land in the DB before the cross-page move's transaction. The
 * flush happens before WriteLock because flushing itself takes the
 * lock. If a flush fails the move is aborted; the batcher has already
 * popped an error and fired onFailed in that case.
 *
 * The old and new bosses must still be alive — this function reads and
 * writes them directly, and holds no lock on either session.
 */
suspend fun moveBlockToAnotherPage(
    oldBoss: BossOfYrs,
    newBoss: BossOfYrs,
    blockId: String,
    above: String?,
    below: String?
): Result<Unit> {
    for (pageId in listOf(oldBoss.pageId(), newBoss.pageId())) {
        val open = OpenPageRegistry.get(pageId) ?: continue
        val flushed = open.flush()
        if (flushed.isFailure) {
            return Result.failure(
                flushed.exceptionOrNull() ?: Exception("flush failed for page $pageId")
            )
        }
    }

    val result = runCatching {
        WriteLock.withLock {
            DbManager.withTransaction {
                val sessionId = SessionManager.currentSessionId
                val content = oldBoss.readBlock(blockId)
                    ?: throw Exception("Block $blockId not found on old page")
                val meta = oldBoss.readBlockMeta(blockId) ?: ""
                val newPageId = newBoss.pageId()
                val isMainMenu = isMainMenuPage(newPageId)

                val deleteCtx = deleteBlock(oldBoss, blockId, sessionId)
                deleteBlockRequiresThreeDbActions(deleteCtx).getOrThrow()

                val addCtx = addBlock(
                    yrs = newBoss,
                    blockContent = content,
                    blockMetaData = meta,
                    position = PositionToInsert.Between(above, below),
                    isPartOfMainMenuPage = isMainMenu,
                    sessionId = sessionId,
                    insertWithExistingBlockId = blockId
                )
                addBlockRequiresThreeDbActions(addCtx).getOrThrow()
            }.getOrThrow()
        }
    }
    result.onSuccess { kickSend() }
    return result
}