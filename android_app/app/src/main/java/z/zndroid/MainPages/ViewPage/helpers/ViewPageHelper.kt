package z.zndroid.MainPages.ViewPage.helpers

import rustlib.my_yrs_lib.BossOfYrs
import z.zndroid.MainPages.ViewPage.BlockUiState
import z.zndroid.MainPages.ViewPage.maybeCreateTitleBlock
import kotlinx.coroutines.CoroutineScope
import z.zndroid.log.ViewPageLogs

/**
 * Isolated logic for ViewPage state management and initialization.
 */
object ViewPageHelper {

    /**
     * Maps Yrs blocks to UI states, reusing existing ones to preserve cursor/selection.
     */
    fun syncUiStates(
        boss: BossOfYrs,
        currentStates: List<BlockUiState>
    ): List<BlockUiState> {
        val blocks = boss.getEntirePage()
        return blocks.map { b ->
            val existing = currentStates.find { s -> s.blockId == b.idInYrs }
            if (existing != null) {
                if (existing.text != b.text) {
                    existing.textFieldValue = existing.textFieldValue.copy(text = b.text)
                }
                existing
            } else {
                BlockUiState(b.idInYrs, b.text, b.metadata)
            }
        }
    }

    /**
     * Ensures the page has at least one block and logs the result.
     */
    fun initializePageContent(
        boss: BossOfYrs,
        scope: CoroutineScope,
        onUpdate: () -> Unit
    ) {
        maybeCreateTitleBlock(boss, scope) {
            ViewPageLogs.logPageLoadSuccess(boss.pageId(), 1)
            onUpdate()
        }
    }
}
