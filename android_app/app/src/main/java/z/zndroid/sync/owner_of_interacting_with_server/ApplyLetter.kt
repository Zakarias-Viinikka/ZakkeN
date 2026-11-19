package z.zndroid.sync.owner_of_interacting_with_server

import rustlib.data_builder_for_operations_that_need_to_be_correct.ExistingBlock
import rustlib.data_builder_for_operations_that_need_to_be_correct.MergePlan
import rustlib.data_builder_for_operations_that_need_to_be_correct.ReconcileOp
import rustlib.data_builder_for_operations_that_need_to_be_correct.buildMergeBlob
import rustlib.data_builder_for_operations_that_need_to_be_correct.buildMergeDisable
import rustlib.data_builder_for_operations_that_need_to_be_correct.buildMergeNewPage
import android.util.Log
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import rustlib.love_letter.LoveLetter
import z.zndroid.components.GlobalPopupManager
import z.zndroid.editor.OpenPageRegistry
import rustlib.my_yrs_lib.docFromSnapshot
import uniffi.protocol.GetDataIn
import uniffi.protocol.SelectArgument
import uniffi.protocol.SelectArguments
import z.zndroid.DbManager
import z.zndroid.Storage.StorageAccess
import z.zndroid.WriteLock
import z.zndroid.db.reads.getActivePagesBlob
import z.zndroid.db.reads.getBlocksForPageOrdered
import z.zndroid.db.reads.getPageBlob
import z.zndroid.db.reads.isMainMenuPage
import z.zndroid.db.schema.PagesTableSchema
import z.zndroid.executor.deleteEveryBlockRow
import z.zndroid.executor.insertBlockRow
import z.zndroid.executor.insertPageRow
import z.zndroid.executor.markAllBlocksForPageDisabled
import z.zndroid.executor.updateBlockContent
import z.zndroid.executor.updateBlockFrac
import z.zndroid.executor.updatePagesBlob
import z.zndroid.executor.updatePagesStatus

internal sealed class ApplyResult {
    object Applied : ApplyResult()
    class UnknownPage(val pageId: String) : ApplyResult()
    object Gated : ApplyResult()
}

/**
 * Apply one letter from the server to yrs + mirror. Does not touch
 * client_love_letter: received letters must not echo back to the server.
 * Throws on any db / yrs failure (the whole transaction rolls back).
 * The `when` has no else, so a new LoveLetter variant stops the build here.
 */
internal suspend fun applyLetter(letter: LoveLetter): ApplyResult {
    if (!syncMayRun()) return ApplyResult.Gated
    return when (letter) {
        is LoveLetter.CreateNewPage ->
            applyCreateNewPage(letter.pageId, letter.isMainMenuPage, letter.snapshotOfEdit)
        is LoveLetter.EditBlock -> applyBlob(letter.targetPageId, letter.snapshotOfEdit)
        is LoveLetter.CreateNewBlock -> applyBlob(letter.targetPageId, letter.snapshotOfEdit)
        is LoveLetter.CreateNewBlockWithId -> applyBlob(letter.targetPageId, letter.snapshotOfEdit)
        is LoveLetter.MoveBlock -> applyBlob(letter.targetPageId, letter.snapshotOfEdit)
        is LoveLetter.RemoveBlock -> applyBlob(letter.targetPageId, letter.snapshotOfEdit)
        is LoveLetter.DisablePage -> applyDisable(letter.pageId, letter.snapshotOfEdit)
    }
}

/** Lock order: WriteLock, then the db transaction. Same as every other writer. */
private suspend fun <T> inWriteTx(block: suspend () -> T): T =
    WriteLock.withLock { DbManager.withTransaction(block).getOrThrow() }

private suspend fun applyBlob(pageId: String, snapshot: ByteArray): ApplyResult {
    flushOpenPage(pageId)
    val result = inWriteTx<ApplyResult> {
        if (!syncMayRun()) ApplyResult.Gated else mergeBlobInTx(pageId, snapshot)
    }
    if (result is ApplyResult.Applied) refreshOpenPage(pageId)
    return result
}

private suspend fun applyCreateNewPage(
    pageId: String,
    isMainMenuPage: Boolean,
    snapshot: ByteArray
): ApplyResult {
    flushOpenPage(pageId)
    val result = createNewPageLocked(pageId, isMainMenuPage, snapshot)
    if (result is ApplyResult.Applied) refreshOpenPage(pageId)
    return result
}

private suspend fun createNewPageLocked(
    pageId: String,
    isMainMenuPage: Boolean,
    snapshot: ByteArray
): ApplyResult = inWriteTx<ApplyResult> {
    if (!syncMayRun()) {
        ApplyResult.Gated
    } else if (pageExists(pageId)) {
        mergeBlobInTx(pageId, snapshot)
    } else {
        val plan = buildMergeNewPage(
            pageId,
            isMainMenuPage,
            snapshot,
            StorageAccess.getUserId(),
            System.currentTimeMillis().toString()
        )
        executePlan(plan)
        ApplyResult.Applied
    }
}

private suspend fun applyDisable(pageId: String, snapshot: ByteArray): ApplyResult =
    inWriteTx<ApplyResult> {
        if (!syncMayRun()) {
            ApplyResult.Gated
        } else if (!pageExists(pageId)) {
            ApplyResult.UnknownPage(pageId)
        } else {
            val status = getActivePagesBlob(pageId).getOrThrow()
            val plan = buildMergeDisable(pageId, snapshot, status)
            executePlan(plan)
            if (OpenPageRegistry.get(pageId) != null) {
                GlobalPopupManager.showError(
                    "This page was disabled from another client. You can keep working on it, but it won't appear in the menu anymore."
                )
            }
            ApplyResult.Applied
        }
    }

/** Must run inside inWriteTx. */
private suspend fun mergeBlobInTx(pageId: String, snapshot: ByteArray): ApplyResult {
    if (!pageExists(pageId)) return ApplyResult.UnknownPage(pageId)
    val existing = getBlocksForPageOrdered(pageId).getOrThrow().map {
        ExistingBlock(it.blockId, it.content, it.fracPosition, it.isTitle)
    }
    val isMain = isMainMenuPage(pageId)
    val open = OpenPageRegistry.get(pageId)
    val plan = if (open != null) {
        // Merge into the open page's live doc, like web's pick_boss_for_page.
        buildMergeBlob(pageId, snapshot, existing, isMain, open.boss)
    } else {
        val userId = StorageAccess.getUserId()
        val blob = getPageBlob(pageId).getOrThrow()
        val boss = docFromSnapshot(blob, userId, pageId, System.currentTimeMillis().toString())
        try {
            buildMergeBlob(pageId, snapshot, existing, isMain, boss)
        } finally {
            boss.destroy()
        }
    }
    executePlan(plan)
    return ApplyResult.Applied
}

private suspend fun pageExists(pageId: String): Boolean {
    val out = DbManager.getData(
        GetDataIn(
            PagesTableSchema.TABLE_NAME,
            SelectArguments.Single(SelectArgument.XEqualY(PagesTableSchema.pageId.name, pageId)),
            listOf(PagesTableSchema.id.name)
        )
    ).getOrThrow()
    return out.rows.isNotEmpty()
}

/**
 * Write out the open page's queued edits before merging into its doc.
 * Must run before taking WriteLock (the flush takes it itself).
 */
private suspend fun flushOpenPage(pageId: String) {
    OpenPageRegistry.get(pageId)?.flush()
}

/** After a merge: show the merged blocks in the open page. Never throws. */
private suspend fun refreshOpenPage(pageId: String) {
    val open = OpenPageRegistry.get(pageId) ?: return
    try {
        open.applyRemoteRows { getBlocksForPageOrdered(pageId).getOrThrow() }
    } catch (e: CancellationException) {
        throw e
    } catch (e: Exception) {
        Log.w("SyncApply", "refresh open page $pageId failed", e)
    }
}

/** Kotlin side of web's apply_merge_plan. Must run inside inWriteTx. */
private suspend fun executePlan(plan: MergePlan) {
    val newPage = plan.isNewPage
    val blob = plan.newBlob
    if (newPage != null) {
        val b = blob ?: throw IllegalStateException("merge plan for new page ${plan.pageId} has no blob")
        insertPageRow(plan.pageId, newPage.isMainMenuPage, b, newPage.version)
    } else if (blob != null) {
        updatePagesBlob(plan.pageId, blob)
    }

    val status = plan.newStatus
    if (status != null) updatePagesStatus(plan.pageId, status)

    if (plan.setPageDisabled) markAllBlocksForPageDisabled(plan.pageId)

    for (op in plan.reconcile) {
        when (op) {
            is ReconcileOp.Insert -> insertBlockRow(
                op.isTitle,
                plan.isMainMenuPage,
                op.content,
                op.blockId,
                plan.pageId,
                op.fracPosition
            )
            is ReconcileOp.UpdateContent -> updateBlockContent(op.blockId, op.newContent)
            is ReconcileOp.UpdateFrac -> updateBlockFrac(op.blockId, op.newFracPosition)
            is ReconcileOp.Delete -> deleteEveryBlockRow(op.blockId)
        }
    }
}
