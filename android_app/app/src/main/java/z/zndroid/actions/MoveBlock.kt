package z.zndroid.actions

import rustlib.data_builder_for_operations_that_need_to_be_correct.addBlock
import rustlib.data_builder_for_operations_that_need_to_be_correct.deleteBlock
import rustlib.my_yrs_lib.BossOfYrs
import rustlib.my_yrs_lib.PositionToInsert
import z.zndroid.DbManager
import z.zndroid.Storage.SessionManager
import z.zndroid.WriteLock
import z.zndroid.db.reads.isMainMenuPage
import z.zndroid.executor.addBlockRequiresThreeDbActions
import z.zndroid.executor.deleteBlockRequiresThreeDbActions

/**
 * Move a block from one page's yrs doc to another page's yrs doc.
 * The caller must guarantee neither boss has queued writes (both must be
 * flushed and not in the middle of a batch).
 *
 * Mirrors web's move_block_to_another_page: deleteBlock on the old boss,
 * addBlock with the same block id on the new boss, both executors in one
 * transaction under WriteLock.
 */
suspend fun moveBlockToAnotherPage(
    oldBoss: BossOfYrs,
    newBoss: BossOfYrs,
    blockId: String,
    above: String?,
    below: String?
): Result<Unit> {
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
