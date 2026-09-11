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
 * Determines if the current input sequence should trigger the creation of a new block.
 * Logic: Requires 3 consecutive linebreaks at the end (2 empty visual rows) to trigger.
 */
fun isItTimeToMakeANewBlock(oldText: String, newText: String): Boolean {
    return oldText.endsWith("\n\n") && newText.endsWith("\n\n\n")
}

/**
 * Ensures a page has at least one block (the title) when opened.
 */
fun maybeCreateTitleBlock(
    boss: BossOfYrs,
    scope: CoroutineScope,
    onUpdate: () -> Unit
) {
    if (boss.getEntirePage().isEmpty()) {
        scope.launch {
            AddBlock.execute(AddBlockCtx(
                boss = boss,
                content = "", // Empty title block
                isTitle = true
            )).onSuccess {
                onUpdate()
            }
        }
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
    nextPosition: Int,
    onUpdate: (String?) -> Unit
) {
    // 1. Flush any pending buffered edits first
    EditTextInBlock.flushBuffer(boss, state).getOrThrow()

    val fullText = state.text
    val head = fullText.substring(0, cursorPosition)
    val tail = fullText.substring(cursorPosition)

    // 2. Truncate current block to the 'head'
    EditTextInBlock.execute(
        EditTextInBlockCtx(
            boss = boss,
            blockId = state.blockId,
            oldText = fullText,
            newText = head
        )
    ).getOrThrow()

    // 3. Create the new block with the 'tail' text
    val newBlockId = AddBlock.execute(
        AddBlockCtx(
            boss = boss,
            content = tail,
            position = PositionToInsert.SpecificPosition(nextPosition.toUInt())
        )
    ).getOrThrow()

    onUpdate(newBlockId)
}

/**
 * Merges a block with the one immediately preceding it.
 * Returns the ID of the block to focus after merge.
 */
suspend fun mergeWithPreviousBlock(
    boss: BossOfYrs,
    currentIndex: Int,
    onUpdate: (String?, Int?) -> Unit
) {
    if (currentIndex <= 0) {
        onUpdate(null, null)
        return
    }

    val blocks = boss.getEntirePage()
    val prevBlock = blocks[currentIndex - 1]
    val currBlock = blocks[currentIndex]

    val combinedText = prevBlock.text + currBlock.text
    val mergePoint = prevBlock.text.length

    // 1. Update previous block with combined content
    EditTextInBlock.execute(
        EditTextInBlockCtx(
            boss = boss,
            blockId = prevBlock.idInYrs,
            oldText = prevBlock.text,
            newText = combinedText
        )
    ).getOrThrow()

    // 2. Remove the current block
    RemoveBlock.execute(
        RemoveBlockCtx(
            boss = boss,
            blockId = currBlock.idInYrs,
            position = currentIndex.toUInt()
        )
    ).getOrThrow()

    onUpdate(prevBlock.idInYrs, mergePoint)
}

/**
 * The heavy "block splitting" operation. 
 * Sequentially cleans the current block, persists it, and spawns the next one.
 */
suspend fun ManyEntersLeadsToManyBlocks(
    boss: BossOfYrs,
    state: BlockUiState,
    newText: String,
    nextPosition: Int,
    onUpdate: (String?) -> Unit
) {
    // 1. Flush any pending buffered edits first to ensure CRDT consistency
    EditTextInBlock.flushBuffer(boss, state).getOrThrow()

    // 2. Persist the current block without the trailing newlines
    val cleanedText = newText.removeSuffix("\n\n\n")
    val oldText = state.text
    
    EditTextInBlock.execute(
        EditTextInBlockCtx(
            boss = boss,
            blockId = state.blockId,
            oldText = oldText,
            newText = cleanedText
        )
    ).getOrThrow()

    // 3. Create the new block at the specific position
    val newBlockId = AddBlock.execute(
        AddBlockCtx(
            boss = boss,
            content = "",
            position = PositionToInsert.SpecificPosition(nextPosition.toUInt())
        )
    ).getOrThrow()

    // 3. Trigger UI update only after both are done
    onUpdate(newBlockId)
}
