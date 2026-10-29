package z.zndroid.actions

import rustlib.data_builder_for_operations_that_need_to_be_correct.*
import rustlib.my_yrs_lib.docFromSnapshot
import rustlib.my_yrs_lib.YrsActivePages
import rustlib.text_diff.DiffResult
import rustlib.text_diff.getDiff
import z.zndroid.DbManager
import z.zndroid.Storage.SessionManager
import z.zndroid.Storage.StorageAccess
import z.zndroid.WriteLock
import z.zndroid.db.reads.getActivePagesBlob
import z.zndroid.db.reads.getBlocksForPageOrdered
import z.zndroid.db.reads.getPageBlob
import z.zndroid.db.reads.isPageDisabled
import z.zndroid.executor.*

suspend fun createNewPage(name: String): Result<Unit> {
    val result = runCatching {
        WriteLock.withLock {
            DbManager.withTransaction {
                val sessionId = SessionManager.currentSessionId
                val userId = StorageAccess.getUserId()
                val pageCtx = createPage(true, sessionId)
                val pageId = pageCtx.pageToInsert.pageId
                val titleId = pageCtx.blocksToInsert.titleBlock.myIdAsGivenByYrs
                val initialBlob = pageCtx.pageToInsert.blobbedPage

                insertPageRequiresThreeDbInserts(pageCtx).getOrThrow()

                val boss = docFromSnapshot(initialBlob, userId, pageId, System.currentTimeMillis().toString())
                try {
                    val diff = getDiff("", name)
                    val editCtx = editBlock(boss, titleId, diff, sessionId)
                        ?: throw Exception("edit_block returned null on non-empty rename")
                    editBlockRequiresThreeDbInserts(editCtx).getOrThrow()
                } finally {
                    boss.destroy()
                }
            }.getOrThrow()
        }
    }
    result.onSuccess { kickSend() }
    return result
}

suspend fun renamePage(pageId: String, newTitle: String): Result<Unit> {
    val result = runCatching {
        WriteLock.withLock {
            DbManager.withTransaction {
                val sessionId = SessionManager.currentSessionId
                val userId = StorageAccess.getUserId()
                val mirror = getBlocksForPageOrdered(pageId).getOrThrow()
                val titleId = mirror.firstOrNull { it.isTitle }?.blockId
                    ?: throw Exception("Page $pageId has no title block, cannot rename")
                val blob = getPageBlob(pageId).getOrThrow()
                val boss = docFromSnapshot(blob, userId, pageId, System.currentTimeMillis().toString())
                try {
                    val currentTitle = boss.readBlock(titleId)
                        ?: throw Exception("Title block $titleId not present in yrs for page $pageId")
                    // Bug workaround: text_diff.getDiff returns a bare Delete when
                    // old and new share no prefix and no suffix, dropping the new
                    // text. A rename is a replace, so send Replace explicitly.
                    val diff = DiffResult.Replace(currentTitle, newTitle, 0u)
                    val editCtx = editBlock(boss, titleId, diff, sessionId)
                        ?: throw Exception("edit_block returned null on rename")
                    editBlockRequiresThreeDbInserts(editCtx).getOrThrow()
                    val readback = boss.readBlock(titleId)
                    if (readback != newTitle) {
                        throw Exception(
                            "rename readback mismatch. currentTitle='$currentTitle' " +
                            "readback='$readback' expected='$newTitle'"
                        )
                    }
                    Unit
                } finally {
                    boss.destroy()
                }
            }.getOrThrow()
        }
    }
    result.onSuccess { kickSend() }
    return result
}

suspend fun disablePage(pageId: String): Result<Unit> {
    val didWork = runCatching {
        WriteLock.withLock {
            DbManager.withTransaction {
                if (isPageDisabled(pageId)) return@withTransaction false
                val sessionId = SessionManager.currentSessionId
                val activeBlob = getActivePagesBlob(pageId).getOrThrow()
                val active = YrsActivePages(activeBlob)
                if (!active.isPageActive(pageId)) return@withTransaction false
                val ctx = disablePage(pageId, sessionId, activeBlob)
                disabledPageRequiresThreeDbActions(ctx).getOrThrow()
                true
            }.getOrThrow()
        }
    }.getOrElse { return Result.failure(it) }
    if (didWork) kickSend()
    return Result.success(Unit)
}