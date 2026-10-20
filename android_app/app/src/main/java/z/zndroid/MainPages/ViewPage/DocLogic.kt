package z.zndroid.MainPages.ViewPage

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import rustlib.my_yrs_lib.BossOfYrs
import rustlib.my_yrs_lib.PositionToInsert
import z.zndroid.DocEvents.AddBlock
import z.zndroid.DocEvents.AddBlockCtx
import z.zndroid.DocEvents.EditTextInBlock
import z.zndroid.DocEvents.EditTextInBlockCtx
import z.zndroid.DocEvents.RemoveBlock
import z.zndroid.DocEvents.RemoveBlockCtx

/**
 * Ensures a page has at least two blocks: a title (index 0) and a body (index 1).
 * The body block is a non-deletable "type here" region.
 */
fun maybeCreateTitleBlock(
    boss: BossOfYrs,
    scope: CoroutineScope,
    onUpdate: (String?) -> Unit
) {
    val blocks = boss.getEntirePage()
    when {
        blocks.isEmpty() -> {
            scope.launch {
                val titleId = AddBlock.execute(AddBlockCtx(
                    boss = boss,
                    content = "",
                    isTitle = true
                )).getOrNull()
                AddBlock.execute(AddBlockCtx(
                    boss = boss,
                    content = "",
                    isTitle = false
                )).onSuccess {
                    onUpdate(titleId)
                }.onFailure {
                    onUpdate(titleId)
                }
            }
        }
        blocks.size == 1 -> {
            scope.launch {
                AddBlock.execute(AddBlockCtx(
                    boss = boss,
                    content = "",
                    isTitle = false
                )).onSuccess {
                    onUpdate(null)
                }.onFailure {
                    onUpdate(null)
                }
            }
        }
        else -> onUpdate(null)
    }
}

/**
 * Splits a block into two at the current cursor position.
 * Returns the ID of the newly created block for focus management.
 */
suspend fun splitBlock(
    boss: BossOfYrs,
    state: BlockUiState,
    cursorPosition: Int,
    currentIndex: Int,
    onHardReload: () -> Unit,
    onUpdate: (String?) -> Unit
) {
    var newBlockId: String? = null
    z.zndroid.DbManager.withTransaction {
        EditTextInBlock.flushBuffer(boss, state, onHardReload).getOrThrow()

        val fullText = state.text
        val head = fullText.substring(0, cursorPosition.coerceAtMost(fullText.length))
        val tail = fullText.substring(cursorPosition.coerceAtMost(fullText.length))

        EditTextInBlock.execute(
            EditTextInBlockCtx(
                boss = boss,
                blockId = state.blockId,
                oldText = fullText,
                newText = head
            )
        ).getOrThrow()

        // The new block goes right after this one. Above = this block.
        // Below = whatever came next, or null if this was last.
        val blocks = boss.getEntirePage()
        val blockIdOfBelow = blocks.getOrNull(currentIndex + 1)?.idInYrs

        newBlockId = AddBlock.execute(
            AddBlockCtx(
                boss = boss,
                content = tail,
                position = PositionToInsert.Between(
                    blockIdOfAbove = state.blockId,
                    blockIdOfBelow = blockIdOfBelow
                )
            )
        ).getOrThrow()
    }.getOrThrow()

    onUpdate(newBlockId)
}

/**
 * Merges a block with the one immediately preceding it.
 * Returns the ID of the block to focus after merge.
 */
suspend fun mergeWithPreviousBlock(
    boss: BossOfYrs,
    state: BlockUiState, // Current block state to ensure cleanup
    currentIndex: Int,
    onHardReload: () -> Unit,
    onUpdate: (String?, Int?) -> Unit
) {
    // Block 0 is the title, block 1 is the non-deletable body placeholder.
    // Neither can be merged upward.
    if (currentIndex <= 1) {
        onUpdate(null, null)
        return
    }

    var targetId: String? = null
    var cursorAt: Int? = null

    z.zndroid.DbManager.withTransaction {
        // 1. Flush pending buffered edits before merging
        EditTextInBlock.flushBuffer(boss, state, onHardReload).getOrThrow()

        val blocks = boss.getEntirePage()
        if (currentIndex >= blocks.size) return@withTransaction
        val prevBlock = blocks[currentIndex - 1]
        val currBlock = blocks[currentIndex]

        val combinedText = prevBlock.text + currBlock.text
        val mergePoint = prevBlock.text.length

        // 2. Update previous block with combined content
        EditTextInBlock.execute(
            EditTextInBlockCtx(
                boss = boss,
                blockId = prevBlock.idInYrs,
                oldText = prevBlock.text,
                newText = combinedText
            )
        ).getOrThrow()

        // 3. Remove the current block
        RemoveBlock.execute(
            RemoveBlockCtx(
                boss = boss,
                blockId = currBlock.idInYrs
            )
        ).getOrThrow()

        targetId = prevBlock.idInYrs
        cursorAt = mergePoint
    }.getOrThrow()

    onUpdate(targetId, cursorAt)
}
