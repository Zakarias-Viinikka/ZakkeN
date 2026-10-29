package z.zndroid.executor

import rustlib.data_builder_for_operations_that_need_to_be_correct.*

suspend fun insertPageRequiresThreeDbInserts(pageCtx: EverythingToInsertForNewPage): Result<Long> {
    return runCatching {
        insertPageRow(
            pageId = pageCtx.pageToInsert.pageId,
            isMainMenuPage = pageCtx.pageToInsert.isMainMenuPage,
            blobbedPage = pageCtx.pageToInsert.blobbedPage,
            version = pageCtx.pageToInsert.version
        )
        val titleId = insertBlockRowAndGetId(
            isTitle = true,
            isPartOfMainMenuPage = pageCtx.blocksToInsert.titleBlock.isPartOfMainMenuPage,
            content = pageCtx.blocksToInsert.titleBlock.content,
            blockId = pageCtx.blocksToInsert.titleBlock.myIdAsGivenByYrs,
            pageId = pageCtx.blocksToInsert.titleBlock.idOfPageIBelongTo,
            fracPosition = pageCtx.blocksToInsert.titleBlock.fracPosition
        )
        insertBlockRow(
            isTitle = false,
            isPartOfMainMenuPage = pageCtx.blocksToInsert.normalBlock.isPartOfMainMenuPage,
            content = pageCtx.blocksToInsert.normalBlock.content,
            blockId = pageCtx.blocksToInsert.normalBlock.myIdAsGivenByYrs,
            pageId = pageCtx.blocksToInsert.normalBlock.idOfPageIBelongTo,
            fracPosition = pageCtx.blocksToInsert.normalBlock.fracPosition
        )
        insertClientLoveLetter(
            snapshotOfEdit = pageCtx.clientLoveLetter.snapshotOfEdit,
            loveLetterSketch = pageCtx.clientLoveLetter.loveLetterSketch,
            sessionId = pageCtx.clientLoveLetter.sessionId,
            targetId = pageCtx.clientLoveLetter.targetId
        )
        titleId
    }
}

suspend fun editBlockRequiresThreeDbInserts(ctx: EverythingForEditBlock): Result<Unit> {
    return runCatching {
        updatePagesBlob(ctx.pagesUpdate.pageId, ctx.pagesUpdate.yrsBlob)
        updateBlockContent(ctx.everyBlockInExistenceUpdate.blockId, ctx.everyBlockInExistenceUpdate.newContent)
        insertClientLoveLetter(
            snapshotOfEdit = ctx.clientLoveLetter.snapshotOfEdit,
            loveLetterSketch = ctx.clientLoveLetter.loveLetterSketch,
            sessionId = ctx.clientLoveLetter.sessionId,
            targetId = ctx.clientLoveLetter.targetId
        )
    }
}

suspend fun disabledPageRequiresThreeDbActions(ctx: EverythingForDisablePage): Result<Unit> {
    return runCatching {
        updatePagesStatus(ctx.pagesUpdate.pageId, ctx.pagesUpdate.yrsBlob)
        markAllBlocksForPageDisabled(ctx.pagesUpdate.pageId)
        insertClientLoveLetter(
            snapshotOfEdit = ctx.clientLoveLetter.snapshotOfEdit,
            loveLetterSketch = ctx.clientLoveLetter.loveLetterSketch,
            sessionId = ctx.clientLoveLetter.sessionId,
            targetId = ctx.clientLoveLetter.targetId
        )
    }
}

suspend fun addBlockRequiresThreeDbActions(ctx: EverythingForAddBlock): Result<Unit> {
    return runCatching {
        updatePagesBlob(ctx.pagesUpdate.pageId, ctx.pagesUpdate.yrsBlob)
        insertBlockRow(
            isTitle = false,
            isPartOfMainMenuPage = ctx.everyBlockInExistenceInsert.isPartOfMainMenuPage,
            content = ctx.everyBlockInExistenceInsert.content,
            blockId = ctx.everyBlockInExistenceInsert.myIdAsGivenByYrs,
            pageId = ctx.everyBlockInExistenceInsert.idOfPageIBelongTo,
            fracPosition = ctx.everyBlockInExistenceInsert.fracPosition
        )
        insertClientLoveLetter(
            snapshotOfEdit = ctx.clientLoveLetter.snapshotOfEdit,
            loveLetterSketch = ctx.clientLoveLetter.loveLetterSketch,
            sessionId = ctx.clientLoveLetter.sessionId,
            targetId = ctx.clientLoveLetter.targetId
        )
    }
}

suspend fun moveBlockRequiresThreeDbActions(ctx: EverythingForMoveBlock): Result<Unit> {
    return runCatching {
        updatePagesBlob(ctx.pagesUpdate.pageId, ctx.pagesUpdate.yrsBlob)
        updateBlockFrac(ctx.blockId, ctx.newFracPosition)
        insertClientLoveLetter(
            snapshotOfEdit = ctx.clientLoveLetter.snapshotOfEdit,
            loveLetterSketch = ctx.clientLoveLetter.loveLetterSketch,
            sessionId = ctx.clientLoveLetter.sessionId,
            targetId = ctx.clientLoveLetter.targetId
        )
    }
}

suspend fun deleteBlockRequiresThreeDbActions(ctx: EverythingForDeleteBlock): Result<Unit> {
    return runCatching {
        updatePagesBlob(ctx.pagesUpdate.pageId, ctx.pagesUpdate.yrsBlob)
        deleteEveryBlockRow(ctx.blockId)
        insertClientLoveLetter(
            snapshotOfEdit = ctx.clientLoveLetter.snapshotOfEdit,
            loveLetterSketch = ctx.clientLoveLetter.loveLetterSketch,
            sessionId = ctx.clientLoveLetter.sessionId,
            targetId = ctx.clientLoveLetter.targetId
        )
    }
}
