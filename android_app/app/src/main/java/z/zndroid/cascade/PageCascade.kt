package z.zndroid.cascade

import cascade.Config
import cascade.Mode

/**
 * One cascade engine per open page. Maps the page's String list keys (block ids, ghost
 * ids) to the Long ids the engine wants, and keeps the engine's block set + order in
 * line with the list the UI shows.
 *
 * sync() is called from a SideEffect in ViewPage (after composition, before layout), so
 * blocks exist in the engine before CascadeColumn reports their sizes.
 * Newly added blocks are "snapped" (placed at their slot immediately); every other
 * block animates to its new position (the cascade).
 */
class PageCascade : AutoCloseable {
    val controller = CascadeController(pageCascadeConfig())
    private val ids = HashMap<String, Long>()
    private var nextId = 1L
    private val inEngine = HashSet<Long>()
    private var lastOrder: List<Long> = emptyList()

    fun idOf(key: String): Long = ids.getOrPut(key) { nextId++ }

    fun sync(keys: List<String>) {
        val order = keys.map { idOf(it) }
        if (order == lastOrder) return
        val isFirstSync = lastOrder.isEmpty()
        val current = order.toHashSet()
        for (id in inEngine.filter { it !in current }) {
            controller.removeBlock(id)
            inEngine.remove(id)
        }
        val added = order.filter { inEngine.add(it) }
        for (id in added) controller.addBlock(id, 0, ESTIMATED_BLOCK_HEIGHT_DP)
        val snapIds = if (isFirstSync) order else added
        controller.commitOrder(listOf(order), snapIds)
        lastOrder = order
    }

    override fun close() {
        controller.close()
    }

    companion object {
        // Until a block reports its real size. Roughly one line of body text + padding.
        private const val ESTIMATED_BLOCK_HEIGHT_DP = 32f
    }
}

// Single column, no outer padding (ViewPage supplies its own 16dp), 8dp between blocks
// (what the old LazyColumn spacedBy(8.dp) gave). FALL = blocks cascade downward.
private fun pageCascadeConfig(): Config = Config(
    blockGap = 8f,
    topPad = 0f,
    edgePad = 0f,
    pageGap = 0f,
    minPageH = 0f,
    blockW = 300f,
    mode = Mode.FALL,
)
