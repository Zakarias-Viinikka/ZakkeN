package com.example.feellabs.labs

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.positionInRoot
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.zIndex
import cascade.Mode
import com.example.feellabs.notes.BlockEditor
import com.example.feellabs.notes.CascadeController
import com.example.feellabs.notes.InitialBlock
import com.example.feellabs.notes.defaultCascadeConfig
import com.example.feellabs.theme.Label
import com.example.feellabs.theme.LocalFeelColors
import com.example.feellabs.theme.MockPage
import com.example.feellabs.theme.MockTitle
import kotlin.math.roundToInt

private class DragLabState {
    var nextId = 1L
    val titles = mutableSetOf<Long>()
    val texts = mutableStateMapOf<Long, TextFieldState>()
    val heights = mutableStateMapOf<Long, Float>()
    val order = mutableStateListOf<MutableList<Long>>(mutableListOf(), mutableListOf())
    fun newId(): Long = nextId++
    fun addTitle(page: Int, text: String): Long {
        val id = newId()
        titles.add(id)
        texts[id] = TextFieldState(text)
        heights[id] = 44f
        order[page].add(id)
        return id
    }
    fun addBlock(page: Int, text: String, h: Float): Long {
        val id = newId()
        texts[id] = TextFieldState(text)
        heights[id] = h
        order[page].add(id)
        return id
    }
}

private fun seed(): DragLabState {
    val s = DragLabState()
    s.addTitle(0, "Page one")
    s.addBlock(0, "Some text on page one.", 44f)
    s.addBlock(0, "Another block here.", 44f)
    s.addBlock(0, "A third block, draggable.", 44f)
    s.addTitle(1, "Page two")
    s.addBlock(1, "Blocks can move between pages.", 44f)
    s.addBlock(1, "Drag me over to the left.", 44f)
    return s
}

@Composable
fun DragReorderLab() {
    val colors = LocalFeelColors.current
    val density = LocalDensity.current.density
    val state = remember { seed() }
    val gen by remember { mutableStateOf(0) }
    val controller = remember(gen) {
        CascadeController(
            config = defaultCascadeConfig().copy(mode = Mode.FALL),
            initialBlocks = state.order.flatMapIndexed { page, list ->
                list.map { InitialBlock(it, page, state.heights[it] ?: 44f) }
            },
            initialOrder = state.order.map { it.toList() },
        )
    }
    val reduced = com.example.feellabs.theme.LocalReducedMotion.current
    LaunchedEffect(controller, reduced) { controller.setReduced(reduced) }
    DisposableEffect(controller) { onDispose { controller.close() } }
    LaunchedEffect(controller) { controller.runFrames() }

    var stageTopLeft by remember { mutableStateOf(Offset.Zero) }
    var draggingId by remember { mutableStateOf<Long?>(null) }
    var dragStartY by remember { mutableFloatStateOf(0f) }
    var dragDelta by remember { mutableStateOf(Offset.Zero) }
    var stageWidthPx by remember { mutableFloatStateOf(0f) }

    val rects = controller.pageRects()

    Box(
        Modifier
            .fillMaxWidth()
            .height(((rects.lastOrNull()?.let { it.y + it.h } ?: 400f) + 40f).dp)
            .verticalScroll(rememberScrollState())
            .onGloballyPositioned { coords ->
                stageTopLeft = coords.positionInRoot()
                stageWidthPx = coords.size.width.toFloat()
            },
    ) {
        // Page panels behind everything
        for (r in rects) {
            MockPage(
                Modifier
                    .offset { IntOffset(0, (r.y * density).roundToInt()) }
                    .fillMaxWidth()
                    .height(r.h.dp),
            ) {
                // Title is rendered as a block below; panel is just chrome.
            }
        }

        // Blocks, positioned by controller y
        val allIds = state.order.flatMap { it.toList() }
        for (id in allIds) {
            val yState = controller.yState(id)
            val isTitle = id in state.titles
            val isDragging = draggingId == id
            val y = yState.floatValue
            Box(
                Modifier
                    .offset {
                        IntOffset(0, ((y + (if (isDragging) dragDelta.y else 0f)) * density).roundToInt())
                    }
                    .graphicsLayer {
                        translationX = (if (isDragging) dragDelta.x else 0f) * density
                    }
                    .zIndex(if (isDragging) 10f else 0f)
                    .fillMaxWidth()
                    .padding(horizontal = 12.dp),
            ) {
                DragBlock(
                    id = id,
                    isTitle = isTitle,
                    isDragging = isDragging,
                    state = state,
                    controller = controller,
                    blockStyle = TextStyle(color = colors.fgDim, fontSize = 15.sp),
                    onStart = { startY ->
                        draggingId = id
                        dragStartY = startY
                        dragDelta = Offset.Zero
                    },
                    onDrag = { delta ->
                        dragDelta += delta / density
                    },
                    onEnd = {
                        val dropX = stageWidthPx / 2f
                        val dropY = dragStartY + dragDelta.y + 25f
                        val result = controller.locateDrop(dropX, dropY, listOf(id))
                        if (result != null) {
                            val (destPage, above) = result
                            // Remove from its current page.
                            for (p in state.order) p.remove(id)
                            // Insert after `above` (or after the title if above is null).
                            val dest = state.order[destPage]
                            val insertAt = when {
                                above == null -> minOf(1, dest.size)
                                else -> {
                                    val idx = dest.indexOf(above)
                                    if (idx >= 0) idx + 1 else dest.size
                                }
                            }
                            dest.add(insertAt.coerceAtMost(dest.size), id)
                            controller.commitOrder(
                                state.order.map { it.toList() },
                                listOf(id),
                            )
                        }
                        draggingId = null
                        dragDelta = Offset.Zero
                    },
                )
            }
        }
    }
}

@Composable
private fun DragBlock(
    id: Long,
    isTitle: Boolean,
    isDragging: Boolean,
    state: DragLabState,
    controller: CascadeController,
    blockStyle: TextStyle,
    onStart: (Float) -> Unit,
    onDrag: (Offset) -> Unit,
    onEnd: () -> Unit,
) {
    val colors = LocalFeelColors.current
    val textField = state.texts[id] ?: return
    val pageOf = state.order.indexOfFirst { it.contains(id) }
    val idxInPage = if (pageOf >= 0) state.order[pageOf].indexOf(id) else -1

    Column(
        Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(6.dp))
            .background(if (isDragging) colors.surface3 else colors.surface2)
            .border(1.dp, colors.border, RoundedCornerShape(6.dp))
            .onGloballyPositioned { }
            .padding(vertical = 6.dp, horizontal = 4.dp),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            // 48dp drag handle
            Box(
                Modifier
                    .width(40.dp)
                    .height(48.dp)
                    .background(if (isTitle) colors.surface1 else colors.surface3)
                    .pointerInput(isTitle) {
                        if (isTitle) return@pointerInput
                        detectDragGestures(
                            onDragStart = {
                                val y = controller.yState(id).floatValue
                                onStart(y)
                            },
                            onDragEnd = { onEnd() },
                            onDragCancel = { onEnd() },
                            onDrag = { change, dragAmount ->
                                change.consume()
                                onDrag(dragAmount)
                            },
                        )
                    },
                contentAlignment = Alignment.Center,
            ) {
                BasicText(
                    if (isTitle) "▴" else "::",
                    style = TextStyle(color = colors.fgFaint, fontSize = 12.sp),
                )
            }
            if (isTitle) {
                BasicText(
                    textField.text.toString(),
                    modifier = Modifier.padding(horizontal = 8.dp),
                    style = TextStyle(
                        color = colors.fg,
                        fontSize = 20.sp,
                        fontWeight = FontWeight.Bold,
                    ),
                )
            } else {
                BlockEditor(
                    state = textField,
                    textStyle = blockStyle,
                    cursorColor = colors.accent,
                    canMergeUp = idxInPage > 1,
                    onSplit = { _, after ->
                        val page = pageOf
                        if (page < 0) return@BlockEditor
                        val newId = state.newId()
                        state.texts[newId] = TextFieldState(after)
                        state.heights[newId] = 44f
                        state.order[page].add(idxInPage + 1, newId)
                        controller.addBlock(newId, page, 44f)
                        controller.commitOrder(
                            state.order.map { it.toList() },
                            listOf(newId),
                        )
                    },
                    onMergeUp = {
                        if (idxInPage > 1 && pageOf >= 0) {
                            val aboveId = state.order[pageOf][idxInPage - 1]
                            val aboveField = state.texts[aboveId]
                            if (aboveField != null) {
                                val merged = aboveField.text.toString() +
                                    textField.text.toString()
                                aboveField.edit { replace(0, length, merged) }
                                state.order[pageOf].remove(id)
                                state.texts.remove(id)
                                state.heights.remove(id)
                                controller.removeBlock(id)
                                controller.commitOrder(
                                    state.order.map { it.toList() },
                                    emptyList(),
                                )
                            }
                        }
                    },
                    modifier = Modifier.weight(1f).padding(horizontal = 8.dp),
                )
            }
        }
    }
}
