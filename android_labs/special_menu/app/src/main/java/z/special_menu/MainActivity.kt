package z.special_menu

import android.content.ClipData
import android.content.ClipboardManager
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.changedToDown
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.zIndex
import kotlin.math.roundToInt

class LabState {
    var nextId = 1L
    val order = mutableStateListOf<Long>()
    val texts = mutableStateMapOf<Long, TextFieldValue>()
    val heights = mutableStateMapOf<Long, Float>()
    var titleId: Long = 0L
    var marker1 by mutableStateOf<TextMarker?>(null)
    var marker2 by mutableStateOf<TextMarker?>(null)
    var lastFocusedId by mutableStateOf<Long?>(null)

    fun newId(): Long = nextId++
}

fun seed(): LabState {
    val s = LabState()
    s.titleId = s.newId()
    s.order.add(s.titleId)
    s.texts[s.titleId] = TextFieldValue("Marker Lab")
    s.heights[s.titleId] = 56f

    val id1 = s.newId()
    s.order.add(id1)
    s.texts[id1] = TextFieldValue("First text block. Tap inside to set your cursor position, then double tap anywhere to open the marker menu.")
    s.heights[id1] = 90f

    val id2 = s.newId()
    s.order.add(id2)
    s.texts[id2] = TextFieldValue("Second text block. Position caret here to place marker 1 or marker 2. Markers can be placed across different blocks.")
    s.heights[id2] = 90f

    val id3 = s.newId()
    s.order.add(id3)
    s.texts[id3] = TextFieldValue("Third text block. When both markers are placed, text between them will highlight and can be copied, cut, or deleted via the action menu.")
    s.heights[id3] = 90f

    val id4 = s.newId()
    s.order.add(id4)
    s.texts[id4] = TextFieldValue("Fourth text block. Blocks can be reordered by dragging the handle on the left, and the selection span will update automatically according to cascade order.")
    s.heights[id4] = 90f

    return s
}

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            var gen by remember { mutableStateOf(0) }
            key(gen) {
                MainScreen(onReset = { gen++ })
            }
        }
    }
}

fun Modifier.interceptDoubleTap(onDoubleTap: () -> Unit): Modifier = pointerInput(Unit) {
    var lastTapTime = 0L
    val doubleTapTimeout = viewConfiguration.doubleTapTimeoutMillis

    awaitPointerEventScope {
        while (true) {
            val event = awaitPointerEvent(PointerEventPass.Initial)
            val down = event.changes.firstOrNull { it.changedToDown() }
            if (down != null) {
                val currentTime = System.currentTimeMillis()
                if (currentTime - lastTapTime < doubleTapTimeout) {
                    down.consume()
                    onDoubleTap()
                    lastTapTime = 0L
                } else {
                    lastTapTime = currentTime
                }
            }
        }
    }
}

@Composable
fun MainScreen(onReset: () -> Unit) {
    val context = LocalContext.current
    val density = LocalDensity.current.density
    val keyboardController = LocalSoftwareKeyboardController.current

    val state = remember { seed() }
    val controller = remember {
        CascadeController(
            config = defaultCascadeConfig(),
            initialBlocks = state.order.map { id ->
                InitialBlock(id, 0, state.heights[id] ?: 90f)
            },
            initialOrder = listOf(state.order.toList())
        )
    }

    DisposableEffect(controller) {
        onDispose { controller.close() }
    }
    LaunchedEffect(controller) {
        controller.runFrames()
    }

    var isOverlayVisible by remember { mutableStateOf(false) }

    var draggingId by remember { mutableStateOf<Long?>(null) }
    var dragStartY by remember { mutableFloatStateOf(0f) }
    var dragDeltaY by remember { mutableFloatStateOf(0f) }
    var stageWidthPx by remember { mutableFloatStateOf(0f) }

    val selection = deriveSelection(state.order, state.texts, state.marker1, state.marker2)
    val pageH = controller.pageHeightState().floatValue
    val rects = controller.pageRects()

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color(0xFFF8F9FA))
    ) {
        Box(
            modifier = Modifier
                .fillMaxSize()
                .interceptDoubleTap {
                    keyboardController?.hide()
                    isOverlayVisible = true
                }
                .verticalScroll(rememberScrollState())
                .padding(vertical = 24.dp)
        ) {
            val totalH = (rects.firstOrNull()?.let { it.y + it.h } ?: pageH) + 60f
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(totalH.dp)
                    .onGloballyPositioned { coords ->
                        stageWidthPx = coords.size.width.toFloat()
                    }
            ) {
                // Background page panel
                for (r in rects) {
                    Box(
                        modifier = Modifier
                            .offset { IntOffset(0, (r.y * density).roundToInt()) }
                            .fillMaxWidth()
                            .height(r.h.dp)
                            .padding(horizontal = 8.dp)
                            .clip(RoundedCornerShape(16.dp))
                            .background(Color(0xFFF1F5F9))
                            .border(1.dp, Color(0xFFE2E8F0), RoundedCornerShape(16.dp))
                    )
                }

                // Render blocks in document order
                val currentOrder = state.order.toList()
                for (id in currentOrder) {
                    val isTitle = id == state.titleId
                    val isDragging = draggingId == id
                    val y = controller.yState(id).floatValue
                    val blockMarkers = listOfNotNull(state.marker1, state.marker2).filter { it.blockId == id }
                    val highlight = selection.highlightRangeFor(id)

                    val onStartDrag by rememberUpdatedState {
                        draggingId = id
                        dragStartY = controller.yState(id).floatValue
                        dragDeltaY = 0f
                    }
                    val onDragDelta by rememberUpdatedState { deltaPx: Float ->
                        dragDeltaY += deltaPx / density
                    }
                    val onEndDrag by rememberUpdatedState {
                        val blockH = state.heights[id] ?: 90f
                        val dropX = stageWidthPx / 2f
                        val dropY = dragStartY + dragDeltaY + (blockH / 2f)
                        val result = controller.locateDrop(dropX, dropY, listOf(id))
                        if (result != null) {
                            val (_, above) = result
                            state.order.remove(id)
                            val insertIndex = if (above == null) 1 else {
                                val idx = state.order.indexOf(above)
                                if (idx >= 0) idx + 1 else state.order.size
                            }
                            state.order.add(insertIndex.coerceIn(1, state.order.size), id)
                            controller.commitOrder(listOf(state.order.toList()), listOf(id))
                        }
                        draggingId = null
                        dragDeltaY = 0f
                    }
                    val onCancelDrag by rememberUpdatedState {
                        draggingId = null
                        dragDeltaY = 0f
                    }

                    Box(
                        modifier = Modifier
                            .offset {
                                val curDrag = if (isDragging) dragDeltaY else 0f
                                IntOffset(0, ((y + curDrag) * density).roundToInt())
                            }
                            .zIndex(if (isDragging) 10f else 0f)
                            .fillMaxWidth()
                            .padding(horizontal = 16.dp)
                            .onSizeChanged { size ->
                                if (id in state.texts && id in state.order) {
                                    val hDp = size.height / density
                                    val oldH = state.heights[id] ?: 0f
                                    if (kotlin.math.abs(hDp - oldH) > 0.5f) {
                                        state.heights[id] = hDp
                                        controller.setHeight(id, hDp)
                                    }
                                }
                            }
                    ) {
                        if (isTitle) {
                            TitleBlock(
                                title = state.texts[id]?.text ?: "Marker Lab",
                                onReset = onReset
                            )
                        } else {
                            val tfv = state.texts[id] ?: TextFieldValue()
                            EditableBlock(
                                id = id,
                                value = tfv,
                                markers = blockMarkers,
                                highlight = highlight,
                                isDragging = isDragging,
                                onValueChange = { newVal ->
                                    val oldVal = state.texts[id] ?: TextFieldValue()
                                    if (oldVal.text != newVal.text) {
                                        state.marker1?.let { m ->
                                            if (m.blockId == id) {
                                                state.marker1 = m.copy(characterIndex = shiftMarkerOnEdit(m.characterIndex, oldVal.text, newVal.text))
                                            }
                                        }
                                        state.marker2?.let { m ->
                                            if (m.blockId == id) {
                                                state.marker2 = m.copy(characterIndex = shiftMarkerOnEdit(m.characterIndex, oldVal.text, newVal.text))
                                            }
                                        }
                                    }
                                    state.texts[id] = newVal
                                    state.lastFocusedId = id
                                },
                                onFocused = { state.lastFocusedId = id },
                                onStartDrag = onStartDrag,
                                onDrag = onDragDelta,
                                onEndDrag = onEndDrag,
                                onCancelDrag = onCancelDrag
                            )
                        }
                    }
                }
            }
        }

        if (isOverlayVisible) {
            CenteredOverlay(
                hasSelection = selection.hasSelection,
                onDismiss = { isOverlayVisible = false },
                onSelectOption = { option ->
                    val targetId = state.lastFocusedId?.takeIf { it in state.order && it != state.titleId }
                        ?: state.order.firstOrNull { it != state.titleId }

                    if (targetId != null) {
                        val tfv = state.texts[targetId] ?: TextFieldValue()
                        val charIdx = tfv.selection.min.coerceIn(0, tfv.text.length)
                        val marker = TextMarker(option, targetId, charIdx)
                        if (option == "1") state.marker1 = marker
                        else if (option == "2") state.marker2 = marker
                    }
                    isOverlayVisible = false
                },
                onAction = { action ->
                    when (action) {
                        SelectionAction.COPY -> {
                            if (selection.hasSelection) {
                                val cm = context.getSystemService(ClipboardManager::class.java)
                                cm?.setPrimaryClip(ClipData.newPlainText("Selected Text", selection.copyText))
                            }
                            state.marker1 = null
                            state.marker2 = null
                            isOverlayVisible = false
                        }
                        SelectionAction.CUT -> {
                            if (selection.hasSelection) {
                                val cm = context.getSystemService(ClipboardManager::class.java)
                                cm?.setPrimaryClip(ClipData.newPlainText("Selected Text", selection.copyText))
                                deleteSelection(state, controller, selection)
                            }
                            state.marker1 = null
                            state.marker2 = null
                            isOverlayVisible = false
                        }
                        SelectionAction.DELETE -> {
                            if (selection.hasSelection) {
                                deleteSelection(state, controller, selection)
                            }
                            state.marker1 = null
                            state.marker2 = null
                            isOverlayVisible = false
                        }
                        SelectionAction.CLEAR -> {
                            state.marker1 = null
                            state.marker2 = null
                            isOverlayVisible = false
                        }
                    }
                }
            )
        }
    }
}

fun deleteSelection(state: LabState, controller: CascadeController, sel: DerivedSelection) {
    if (!sel.hasSelection) return
    if (sel.isSingleBlock) {
        val seg = sel.segments.first()
        val tfv = state.texts[seg.blockId] ?: return
        val oldText = tfv.text
        val newText = oldText.substring(0, seg.from) + oldText.substring(seg.to)
        state.texts[seg.blockId] = TextFieldValue(newText, selection = TextRange(seg.from))
    } else {
        val startSeg = sel.segments.first()
        val endSeg = sel.segments.last()
        val intermediateSegs = sel.segments.drop(1).dropLast(1)

        val startTfv = state.texts[startSeg.blockId]
        if (startTfv != null) {
            val newText = startTfv.text.substring(0, startSeg.from.coerceAtMost(startTfv.text.length))
            state.texts[startSeg.blockId] = TextFieldValue(newText, selection = TextRange(newText.length))
        }

        val endTfv = state.texts[endSeg.blockId]
        if (endTfv != null) {
            val newText = endTfv.text.substring(endSeg.to.coerceAtMost(endTfv.text.length))
            state.texts[endSeg.blockId] = TextFieldValue(newText, selection = TextRange(0))
        }

        val removedIds = intermediateSegs.map { it.blockId }
        for (id in removedIds) {
            state.order.remove(id)
            state.texts.remove(id)
            state.heights.remove(id)
            controller.removeBlock(id)
        }
        controller.commitOrder(listOf(state.order.toList()), emptyList())

        if (state.lastFocusedId in removedIds) {
            state.lastFocusedId = null
        }
    }
}

@Composable
fun TitleBlock(
    title: String,
    onReset: () -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(10.dp))
            .background(Color.White)
            .border(1.dp, Color(0xFFCBD5E1), RoundedCornerShape(10.dp))
            .padding(horizontal = 16.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        BasicText(
            text = title,
            style = TextStyle(
                fontSize = 20.sp,
                fontWeight = FontWeight.Bold,
                color = Color(0xFF1E293B)
            )
        )
        Box(
            modifier = Modifier
                .clip(RoundedCornerShape(6.dp))
                .background(Color(0xFFF1F5F9))
                .border(1.dp, Color(0xFFCBD5E1), RoundedCornerShape(6.dp))
                .clickable { onReset() }
                .padding(horizontal = 12.dp, vertical = 6.dp)
        ) {
            BasicText(
                text = "Reset Lab",
                style = TextStyle(
                    fontSize = 12.sp,
                    fontWeight = FontWeight.SemiBold,
                    color = Color(0xFF475569)
                )
            )
        }
    }
}

@Composable
fun EditableBlock(
    id: Long,
    value: TextFieldValue,
    markers: List<TextMarker>,
    highlight: Pair<Int, Int>?,
    isDragging: Boolean,
    onValueChange: (TextFieldValue) -> Unit,
    onFocused: () -> Unit,
    onStartDrag: () -> Unit,
    onDrag: (Float) -> Unit,
    onEndDrag: () -> Unit,
    onCancelDrag: () -> Unit
) {
    val visualTransformation = remember(markers, highlight) {
        MarkerVisualTransformation(markers, highlight)
    }

    val borderColor = if (highlight != null) Color(0xFF2196F3) else Color(0xFFE2E8F0)
    val bgColor = if (isDragging) Color(0xFFF8FAFC) else Color.White

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .defaultMinSize(minHeight = 56.dp)
            .clip(RoundedCornerShape(10.dp))
            .background(bgColor)
            .border(if (highlight != null) 1.5.dp else 1.dp, borderColor, RoundedCornerShape(10.dp))
            .padding(vertical = 4.dp, horizontal = 4.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        // Drag handle
        Box(
            modifier = Modifier
                .width(40.dp)
                .height(48.dp)
                .clip(RoundedCornerShape(6.dp))
                .background(Color(0xFFF1F5F9))
                .pointerInput(id) {
                    detectDragGestures(
                        onDragStart = { onStartDrag() },
                        onDragEnd = { onEndDrag() },
                        onDragCancel = { onCancelDrag() },
                        onDrag = { change, dragAmount ->
                            change.consume()
                            onDrag(dragAmount.y)
                        }
                    )
                },
            contentAlignment = Alignment.Center
        ) {
            BasicText(
                text = "::",
                style = TextStyle(
                    color = Color(0xFF94A3B8),
                    fontSize = 16.sp,
                    fontWeight = FontWeight.Bold
                )
            )
        }

        BasicTextField(
            value = value,
            onValueChange = onValueChange,
            visualTransformation = visualTransformation,
            modifier = Modifier
                .weight(1f)
                .padding(horizontal = 10.dp, vertical = 8.dp)
                .onFocusChanged { focusState ->
                    if (focusState.isFocused) {
                        onFocused()
                    }
                },
            textStyle = TextStyle(
                fontSize = 15.sp,
                color = Color(0xFF334155),
                lineHeight = 22.sp
            )
        )
    }
}