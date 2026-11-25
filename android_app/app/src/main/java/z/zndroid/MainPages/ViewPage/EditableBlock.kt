package z.zndroid.MainPages.ViewPage

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.keyframes
import androidx.compose.foundation.clickable
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.viewinterop.AndroidView
import z.zndroid.ui.theme.LocalReducedMotion
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import java.util.concurrent.atomic.AtomicReference
import z.zndroid.editor.BlockRow
import z.zndroid.editor.PageSession
import z.zndroid.markdown_conversion.Cm
import z.zndroid.markdown_conversion.CmEvent
import z.zndroid.markdown_conversion.Decision
import z.zndroid.markdown_conversion.Intent
import z.zndroid.markdown_conversion.applyChanges
import z.zndroid.markdown_conversion.decide
import z.zndroid.markdown_conversion.main_for_on_link_click.mainOnLinkClick

/**
 * One block, backed by a CodeMirror 6 editor in a WebView.
 *
 * `row.textFieldValue` is the Kotlin shadow of the CM6 text. Every change
 * from CM6 runs [applyChanges] against that shadow, then goes into
 * `session.onInput`. Outside changes to `row.textFieldValue.text` (server
 * sync, session split/merge/move) are pushed into CM6 with `Cm.setText`,
 * which is a minimal diff (caret preserved, no echo).
 *
 * Android-only differences vs web's `MarkdownConversion` component are
 * documented in INVENTED_ON_ANDROID.md (measured WebView height, JS thread
 * handoff for key decisions, per-block WebView instance).
 */
@Composable
fun EditableBlock(
    row: BlockRow,
    index: Int,
    session: PageSession,
    listState: LazyListState? = null,
    lazyIndex: Int = index,
    pageScroll: PageScroll? = null,
    onOpenPage: (String) -> Unit = {},
    onFocused: () -> Unit = {},
) {
    val isTitle = row.isTitle
    val context = LocalContext.current
    val density = LocalDensity.current.density
    val scope = rememberCoroutineScope()
    val currentOnOpenPage by rememberUpdatedState(onOpenPage)

    var showMenu by remember { mutableStateOf(false) }

    // Indent transition (lab BlockTransitionLab "Indent settle"): 22dp -> 0 over 500 ms
    // with a -5.5dp overshoot at 200 ms. Plays when the first character is typed into an
    // empty body block (PageSession.onInput bumps indentGen), not on Enter.
    val reduced = LocalReducedMotion.current
    val indentX = remember { Animatable(0f) }
    LaunchedEffect(row.indentGen) {
        if (row.indentGen == row.indentPlayedGen) return@LaunchedEffect
        row.indentPlayedGen = row.indentGen
        if (reduced) return@LaunchedEffect
        indentX.snapTo(22f)
        indentX.animateTo(
            targetValue = 0f,
            animationSpec = keyframes {
                durationMillis = 500
                -5.5f at 200
            },
        )
    }

    // Shadow text the JS thread reads synchronously when deciding a key press.
    // CM events come in on the WebView's JS thread; Compose state reads are safe
    // from any thread, but the per-event body needs the doc text at the instant
    // the event fired. Updating this AtomicReference in the changes handler
    // (JS thread) keeps it in lockstep with CM.
    val lastKnownText = remember(row.blockId) { AtomicReference(row.textFieldValue.text) }

    // WebView: one per block, reused across recompositions.
    val webView = remember(row.blockId) {
        android.webkit.WebView(context).apply {
            settings.javaScriptEnabled = true
            settings.domStorageEnabled = true
            setBackgroundColor(android.graphics.Color.TRANSPARENT)
            isVerticalScrollBarEnabled = false
            isHorizontalScrollBarEnabled = false
            overScrollMode = android.webkit.WebView.OVER_SCROLL_NEVER
            isFocusable = true
            isFocusableInTouchMode = true
        }
    }

    // Rendered height (CSS px == Compose dp on Android given the viewport meta).
    var webViewHeightDp by remember { mutableFloatStateOf(0f) }

    val measureHeight: () -> Unit = {
        Cm.getContentHeight(row.blockId) { px ->
            if (px > 1f) {
                webViewHeightDp = px + 4f
            }
        }
    }

    // Views (incl. WebViews) get Android focus on tap; use that as the "last
    // focused block" signal for the special menu. rememberUpdatedState so the
    // callback stays current if the caller passes a stable lambda over time.
    val latestOnFocused by rememberUpdatedState(onFocused)

    // Create once per block id. Events are forwarded through the JS bridge.
    DisposableEffect(row.blockId) {
        webView.setOnFocusChangeListener { _, hasFocus -> if (hasFocus) latestOnFocused() }
        Cm.create(
            webView = webView,
            id = row.blockId,
            initial = row.textFieldValue.text,
            onEvent = { event ->
                when (event) {
                    is CmEvent.Changes -> {
                        val base = lastKnownText.get()
                        val updated = applyChanges(base, event.changes)
                        lastKnownText.set(updated)
                        scope.launch {
                            handleChanges(row, session, event, base)
                            delay(30)
                            measureHeight()
                        }
                        false
                    }
                    is CmEvent.Key -> {
                        val textNow = lastKnownText.get()
                        val decision: Decision? = decide(
                            key = event.key,
                            anchor = event.anchor,
                            head = event.head,
                            docLen = event.docLen,
                            ranges = event.ranges,
                            isTitle = isTitle,
                            text = { textNow },
                        )
                        if (decision?.consume == true) {
                            scope.launch {
                                handleKeyIntent(session, row, decision, event.anchor, event.head)
                                delay(30)
                                measureHeight()
                            }
                            true
                        } else {
                            false
                        }
                    }
                    is CmEvent.LinkClick -> {
                        scope.launch {
                            mainOnLinkClick(event.command, event.rest) { target ->
                                currentOnOpenPage(target)
                            }
                        }
                        false
                    }
                    is CmEvent.Selection -> false
                }
            },
            onReady = { measureHeight() },
        )
        onDispose {
            webView.setOnFocusChangeListener(null)
            Cm.destroy(row.blockId)
            webView.post { runCatching { webView.destroy() } }
        }
    }

    // Push outside text changes into CM6. CM's setText is a minimal diff, so
    // when CM already has the target text this is a cheap no-op.
    LaunchedEffect(row.textFieldValue.text) {
        val wanted = row.textFieldValue.text
        Cm.setText(row.blockId, wanted)
        lastKnownText.set(wanted)
        delay(60)
        measureHeight()
    }

    // Pending focus: set the selection at the offset CM-side, then focus.
    LaunchedEffect(session.pendingFocus.value) {
        val req = session.pendingFocus.value ?: return@LaunchedEffect
        if (req.blockId != row.blockId) return@LaunchedEffect
        // Android's View focus is separate from CM's DOM focus. Without
        // requestFocus the IME and visual caret stay with whichever block
        // last held them, even though CM's selection has moved.
        webView.requestFocus()
        webView.requestFocusFromTouch()
        Cm.setSelection(row.blockId, req.offset, req.offset)
        Cm.focusKeepCaret(row.blockId)
        session.clearPendingFocus()
        withFrameNanos { }
        withFrameNanos { }
        runCatching {
            if (pageScroll != null) pageScroll.scrollToBlock(row.blockId, density)
            else listState?.animateScrollToItem(lazyIndex)
        }
    }

    val measuredDp = if (webViewHeightDp > 0f) webViewHeightDp.dp else 44.dp
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .graphicsLayer { translationX = indentX.value * density }
            .padding(vertical = 4.dp),
        verticalAlignment = Alignment.Top
    ) {
        Box(
            modifier = Modifier
                .width(24.dp)
                .padding(top = 4.dp)
                .clickable { showMenu = true },
            contentAlignment = Alignment.Center
        ) {
            Text(
                text = "::",
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.35f)
            )
        }
        Box(modifier = Modifier.weight(1f)) {
            AndroidView(
                factory = { webView },
                modifier = Modifier.fillMaxWidth().height(measuredDp),
            )
        }
    }
    if (showMenu) {
        BlockActionMenu(
            blockText = row.textFieldValue.text,
            canMoveUp = index >= 2,
            canMoveDown = !isTitle && index < session.rows.size - 1,
            canDelete = !isTitle && session.rows.count { !it.isTitle } > 1,
            onDelete = {
                session.onDeleteBlock(index)
                showMenu = false
            },
            onMoveUp = {
                session.onMoveUp(index)
                showMenu = false
            },
            onMoveDown = {
                session.onMoveDown(index)
                showMenu = false
            },
            onDismiss = { showMenu = false }
        )
    }
}

/**
 * Apply CM6's `changes` to the shadow. Runs on Main. `base` is the text CM
 * had when it emitted the change; if the row has already moved on (remote
 * edit won the race), we discard the local change and let the setText effect
 * bring CM to the row's current text.
 */
private fun handleChanges(
    row: BlockRow,
    session: PageSession,
    event: CmEvent.Changes,
    base: String,
) {
    val idx = session.rows.indexOfFirst { it === row }
    if (idx < 0) return
    val current = row.textFieldValue.text
    if (current != base) return
    val newText = applyChanges(current, event.changes)
    if (newText == current) return
    val caret = event.head.coerceIn(0, newText.length)
    session.onInput(idx, TextFieldValue(newText, selection = TextRange(caret)))
}

/** Runs on Main. `anchor`/`head` come from the CM key event. */
private fun handleKeyIntent(
    session: PageSession,
    row: BlockRow,
    decision: Decision,
    anchor: Int,
    head: Int,
) {
    val idx = session.rows.indexOfFirst { it === row }
    if (idx < 0) return
    when (decision.intent) {
        is Intent.NavUp -> session.onArrow(idx, goDown = false)
        is Intent.NavDown -> session.onArrow(idx, goDown = true)
        is Intent.Split -> session.onEnter(idx, minOf(anchor, head))
        is Intent.MergeUp -> session.onBackspaceAtStart(idx)
        is Intent.EnterWithoutSplit -> Unit
    }
}

@Composable
fun BlockActionMenu(
    blockText: String,
    canMoveUp: Boolean,
    canMoveDown: Boolean,
    canDelete: Boolean,
    onDelete: () -> Unit,
    onMoveUp: () -> Unit,
    onMoveDown: () -> Unit,
    onDismiss: () -> Unit
) {
    AlertDialog(
        onDismissRequest = onDismiss,
        confirmButton = {},
        title = {
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceEvenly
                ) {
                    Button(onClick = { }) { Text("H1") }
                    Button(onClick = { }) { Text("Todo") }
                    Button(onClick = onDelete, enabled = canDelete) { Text("Del") }
                }
                Spacer(modifier = Modifier.height(8.dp))
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceEvenly
                ) {
                    Button(onClick = onMoveUp, enabled = canMoveUp) { Text("Up") }
                    Button(onClick = onMoveDown, enabled = canMoveDown) { Text("Down") }
                }
                Spacer(modifier = Modifier.height(16.dp))
                Card(
                    modifier = Modifier.fillMaxWidth(),
                    colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant)
                ) {
                    Text(
                        text = blockText,
                        modifier = Modifier.padding(16.dp),
                        style = MaterialTheme.typography.bodyLarge
                    )
                }
            }
        }
    )
}