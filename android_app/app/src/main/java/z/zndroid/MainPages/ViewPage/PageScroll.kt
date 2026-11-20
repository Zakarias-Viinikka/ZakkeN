package z.zndroid.MainPages.ViewPage

import androidx.compose.foundation.ScrollState
import z.zndroid.cascade.PageCascade

/**
 * Scroll helper for the cascade-based ViewPage (replaces LazyListState.animateScrollToItem).
 * Blocks are positioned by the engine, so "scroll to block" = scroll to the block's
 * current engine y.
 */
class PageScroll(
    private val scrollState: ScrollState,
    private val cascade: PageCascade,
) {
    suspend fun scrollToBlock(blockId: String, density: Float) {
        val y = cascade.controller.yState(cascade.idOf(blockId)).floatValue
        scrollState.animateScrollTo((y * density).toInt().coerceAtLeast(0))
    }
}
