package z.zndroid.MainPages.ViewPage

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import rustlib.my_yrs_lib.BossOfYrs
import rustlib.my_yrs_lib.PositionToInsert
import z.zndroid.DocEvents.AddBlock
import z.zndroid.DocEvents.AddBlockCtx
import z.zndroid.DocEvents.EditTextInBlock
import z.zndroid.DocEvents.EditTextInBlockCtx

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
                content = "" // Empty title block
            )).onSuccess {
                onUpdate()
            }
        }
    }
}

/**
 * The heavy "block splitting" operation. 
 * Sequentially cleans the current block, persists it, and spawns the next one.
 */
suspend fun ManyEntersLeadsToManyBlocks(
    boss: BossOfYrs,
    blockId: String,
    oldText: String,
    newText: String,
    nextPosition: Int,
    onUpdate: () -> Unit
) {
    // 1. Persist the current block without the trailing newlines
    val cleanedText = newText.removeSuffix("\n\n\n")
    EditTextInBlock.execute(
        EditTextInBlockCtx(
            boss = boss,
            blockId = blockId,
            oldText = oldText,
            newText = cleanedText
        )
    ).getOrThrow()

    // 2. Create the new block at the specific position
    AddBlock.execute(
        AddBlockCtx(
            boss = boss,
            content = "",
            position = PositionToInsert.SpecificPosition(nextPosition.toUInt())
        )
    ).getOrThrow()

    // 3. Trigger UI update only after both are done
    onUpdate()
}
