package com.example.feellabs.notes

import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.MutableFloatState
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.withFrameNanos
import cascade.BlockPos
import cascade.CascadePhysics
import cascade.Config
import cascade.Mode
import kotlinx.coroutines.channels.Channel

data class InitialBlock(val id: Long, val page: Int, val height: Float)

fun defaultCascadeConfig(): Config = Config(
    blockGap = 2f,
    topPad = 12f,
    edgePad = 24f,
    pageGap = 24f,
    minPageH = 200f,
    blockW = 300f,
    mode = Mode.FALL,
)

class CascadeController(
    config: Config,
    initialBlocks: List<InitialBlock> = emptyList(),
    initialOrder: List<List<Long>> = emptyList(),
) : AutoCloseable {
    private val physics = CascadePhysics(config)
    private val ys = mutableStateMapOf<Long, MutableFloatState>()
    private val channel = Channel<Unit>(Channel.CONFLATED)
    private val pageHeight = mutableFloatStateOf(0f)
    @Volatile private var closed = false

    init {
        if (initialBlocks.isNotEmpty()) {
            for (b in initialBlocks) {
                physics.addBlock(b.id.toULong(), b.page.toUInt(), b.height)
            }
            val allIds = initialBlocks.map { it.id.toULong() }
            val snapChanged = physics.commitOrder(
                initialOrder.map { row -> row.map { it.toULong() } },
                allIds,
            )
            apply(snapChanged)
            pageHeight.floatValue = physics.pageRects().firstOrNull()?.h ?: 0f
        }
    }

    fun yState(id: Long): MutableFloatState =
        ys.getOrPut(id) { mutableFloatStateOf(0f) }

    fun addBlock(id: Long, page: Int, estHeight: Float) {
        if (closed) return
        physics.addBlock(id.toULong(), page.toUInt(), estHeight)
        tick()
    }

    fun removeBlock(id: Long) {
        if (closed) return
        physics.removeBlock(id.toULong())
        ys.remove(id)
        tick()
    }

    fun setHeight(id: Long, h: Float) {
        if (closed) return
        val changed = physics.setHeight(id.toULong(), h)
        tick(changed)
    }

    fun commitOrder(orders: List<List<Long>>, snapIds: List<Long>) {
        if (closed) return
        val changed = physics.commitOrder(
            orders.map { row -> row.map { it.toULong() } },
            snapIds.map { it.toULong() },
        )
        tick(changed)
    }

    fun pageRects(): List<cascade.PageRect> = physics.pageRects()

    fun pageHeightState(): MutableFloatState = pageHeight

    fun locateDrop(x: Float, y: Float, exclude: List<Long>): Pair<Int, Long?>? {
        val r = physics.locateDrop(x, y, exclude.map { it.toULong() }) ?: return null
        return (r.page.toInt()) to (r.above?.toLong())
    }

    fun setMode(mode: Mode) {
        if (closed) return
        physics.setMode(mode)
    }

    fun setReduced(reduced: Boolean) {
        if (closed) return
        physics.setReduced(reduced)
    }

    private fun tick(alsoApply: List<BlockPos> = emptyList()) {
        if (closed) return
        apply(alsoApply)
        val stepResult = physics.step(0f)
        apply(stepResult.changed)
        pageHeight.floatValue = physics.pageRects().firstOrNull()?.h ?: 0f
        channel.trySend(Unit)
    }

    private fun apply(changed: List<BlockPos>) {
        for (p in changed) {
            ys.getOrPut(p.id.toLong()) { mutableFloatStateOf(0f) }.floatValue = p.y
        }
    }

    suspend fun runFrames() {
        try {
            for (kick in channel) {
                var lastNanos = 0L
                var first = true
                kick@ while (!closed) {
                    var moving = false
                    withFrameNanos { now ->
                        val dt = if (first) 0f
                                 else ((now - lastNanos) / 1_000_000_000f).coerceIn(0f, 0.032f)
                        lastNanos = now
                        first = false
                        if (closed) return@withFrameNanos
                        val r = physics.step(dt)
                        apply(r.changed)
                        moving = r.moving
                    }
                    pageHeight.floatValue = physics.pageRects().firstOrNull()?.h ?: 0f
                    if (!moving) break@kick
                }
            }
        } catch (_: Throwable) {
        }
    }

    override fun close() {
        if (closed) return
        closed = true
        channel.close()
        physics.close()
    }
}