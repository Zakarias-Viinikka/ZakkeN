package z.zndroid.MainPages.ViewPage

import androidx.compose.foundation.ScrollState
import z.zndroid.cascade.PageCascade

/**
 * Scroll helper for the cascade-based ViewPage (replaces LazyListState.animateScrollToItem).
 * Blocks are positioned by the engine, so "scroll to block" = scroll to the block's
 * current engine y. Only animates scroll if the target block is outside or near viewport edges.
 */
class PageScroll(
    private val scrollState: ScrollState,
    private val cascade: PageCascade,
) {
    suspend fun scrollToBlock(blockId: String, density: Float) {
        val y = cascade.controller.yState(cascade.idOf(blockId)).floatValue
        val targetYPx = (y * density).toInt().coerceAtLeast(0)
        val currentScroll = scrollState.value
        val viewport = scrollState.viewportSize
        if (viewport > 0) {
            if (targetYPx < currentScroll || targetYPx > currentScroll + viewport - 120) {
                scrollState.animateScrollTo(targetYPx)
            }
        } else {
            scrollState.animateScrollTo(targetYPx)
        }
    }
}
