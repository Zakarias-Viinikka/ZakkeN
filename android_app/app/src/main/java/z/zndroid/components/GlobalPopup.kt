package z.zndroid.components

import android.util.Log
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.launch
import z.zndroid.BuildConfig
import z.zndroid.ui.theme.LocalFeelColors
import z.zndroid.ui.theme.LocalReducedMotion
import z.zndroid.ui.theme.Motion

/**
 * Global Manager for showing popups from anywhere in the app.
 *
 * API is unchanged from the previous Snackbar implementation:
 *   - show(msg)      : informational popup.
 *   - showError(msg) : error popup; rendered with the danger colour.
 *
 * Both are fire-and-forget. The backing channel is unbounded, so a popup
 * requested before [GlobalPopup] first composes (or while the activity is
 * being recreated) sits in the channel and is shown by the next collector.
 *
 * The UI is the lab's toast stack: fading in from the right with a spring,
 * a 2dp progress bar on the bottom edge, a 3dp severity tick on the top-left,
 * and a click-to-dismiss on the whole row. Toasts auto-dismiss after
 * INFO_LIFE_MS (info) or ERROR_LIFE_MS (error), coalescing on (text, isError)
 * so the same message does not stack.
 */
internal data class Popup(val text: String, val isError: Boolean)

object GlobalPopupManager {
    private val channel = Channel<Popup>(capacity = Channel.UNLIMITED)

    fun show(message: String) {
        if (BuildConfig.DEBUG) {
            Log.i("GlobalPopup", message)
        }
        channel.trySend(Popup(message, isError = false))
    }

    fun showError(message: String) {
        if (BuildConfig.DEBUG) {
            Log.e("GlobalPopup", message)
        }
        val cleaned = message
            .removePrefix("ERROR: ")
            .removePrefix("Error: ")
            .removePrefix("error: ")
        channel.trySend(Popup(cleaned, isError = true))
    }

    internal val events = channel.receiveAsFlow()
}

internal data class LivePopup(
    val id: Long,
    val text: String,
    val isError: Boolean,
    val bornMs: Long,
    val lifeMs: Long,
)

private const val INFO_LIFE_MS = 3200L
private const val ERROR_LIFE_MS = 5000L
private const val MAX_VISIBLE = 4

@Composable
fun GlobalPopup() {
    val reduced = LocalReducedMotion.current
    val live = remember { mutableStateListOf<LivePopup>() }
    var nextId by remember { mutableLongStateOf(1L) }
    var clock by remember { mutableLongStateOf(0L) }
    var lastFrameNanos by remember { mutableLongStateOf(0L) }

    // One frame loop owns the virtual clock. It only runs while at least one
    // toast is live, so an idle app requests no frames at all.
    val hasLive = live.isNotEmpty()
    LaunchedEffect(hasLive) {
        if (!hasLive) return@LaunchedEffect
        lastFrameNanos = 0L
        while (true) {
            withFrameNanos { now ->
                if (lastFrameNanos == 0L) {
                    lastFrameNanos = now
                    return@withFrameNanos
                }
                val dt = ((now - lastFrameNanos) / 1_000_000L).coerceAtMost(64L)
                lastFrameNanos = now
                clock += dt
                live.removeAll { clock - it.bornMs >= it.lifeMs }
            }
        }
    }

    LaunchedEffect(Unit) {
        GlobalPopupManager.events.collect { msg ->
            val existing = live.firstOrNull { it.text == msg.text && it.isError == msg.isError }
            if (existing != null) {
                val idx = live.indexOf(existing)
                live[idx] = existing.copy(bornMs = clock)
            } else {
                if (live.size >= MAX_VISIBLE) live.removeAt(0)
                val life = if (msg.isError) ERROR_LIFE_MS else INFO_LIFE_MS
                live.add(LivePopup(nextId++, msg.text, msg.isError, clock, life))
            }
        }
    }

    if (live.isEmpty()) return

    Box(
        modifier = Modifier
            .fillMaxSize()
            .imePadding()
            .navigationBarsPadding()
            .padding(16.dp),
        contentAlignment = Alignment.BottomEnd,
    ) {
        Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
            live.forEach { p ->
                key(p.id) {
                    PopupToast(
                        popup = p,
                        clock = clock,
                        reduced = reduced,
                        onDismiss = { live.remove(p) },
                    )
                }
            }
        }
    }
}

@Composable
private fun PopupToast(
    popup: LivePopup,
    clock: Long,
    reduced: Boolean,
    onDismiss: () -> Unit,
) {
    val colors = LocalFeelColors.current
    val alpha = remember { Animatable(if (reduced) 1f else 0f) }
    val dx = remember { Animatable(if (reduced) 0f else 28f) }
    LaunchedEffect(popup.id) {
        if (!reduced) {
            coroutineScope {
                launch { alpha.animateTo(1f, tween(180)) }
                launch { dx.animateTo(0f, tween(380, easing = Motion.easeSpring)) }
            }
        }
    }
    val pct = ((clock - popup.bornMs).toFloat() / popup.lifeMs.toFloat()).coerceIn(0f, 1f)
    val barColor = if (popup.isError) colors.danger else colors.accent

    Row(
        Modifier
            .widthIn(max = 340.dp)
            .graphicsLayer {
                this.alpha = alpha.value.coerceIn(0f, 1f)
                translationX = dx.value * density
            }
            .clip(RoundedCornerShape(6.dp))
            .background(colors.surface1)
            .border(1.dp, colors.border, RoundedCornerShape(6.dp))
            .drawBehind {
                val h = size.height
                drawLine(
                    color = barColor,
                    start = Offset(0f, h),
                    end = Offset(size.width * (1f - pct), h),
                    strokeWidth = 2f * density,
                )
                drawLine(
                    color = barColor,
                    start = Offset(0f, 0f),
                    end = Offset(3f * density, 0f),
                    strokeWidth = 3f * density,
                )
            }
            .clickable { onDismiss() }
            .clearAndSetSemantics {
                // e2e reads this: "popup|<error|info>|<message>"
                contentDescription = "popup|${if (popup.isError) "error" else "info"}|${popup.text}"
                onClick { onDismiss(); true }
            }
            .padding(horizontal = 12.dp, vertical = 10.dp),
        verticalAlignment = Alignment.Top,
    ) {
        Box(
            Modifier
                .padding(top = 6.dp)
                .size(8.dp)
                .background(barColor, CircleShape),
        )
        Spacer(Modifier.width(10.dp))
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            BasicText(
                if (popup.isError) "Error" else "Info",
                style = TextStyle(
                    color = colors.fg,
                    fontSize = 13.sp,
                    fontWeight = FontWeight.SemiBold,
                ),
            )
            BasicText(
                popup.text,
                style = TextStyle(color = colors.fgDim, fontSize = 12.sp),
            )
        }
    }
}