package z.zndroid.MainPages

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.layout.layout
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.coroutineScope
import z.zndroid.db.reads.MenuPageRow
import z.zndroid.ui.theme.LocalFeelColors
import z.zndroid.ui.theme.LocalReducedMotion
import z.zndroid.ui.theme.Motion
import kotlin.math.roundToInt

/** Time between the eye press and the start of the drop. Zakke's ask: about 1.5 s. */
private const val HIDE_DELAY_MS = 1500L

/**
 * One main-menu row: clickable title + eye. Eye press = hide the page (disablePage;
 * data stays). Sequence: eye closes (480 ms) and row dims; after HIDE_DELAY_MS the row
 * plays the lab's drop (DeletingBlockLab Drop variant: lift 200 ms, then 300 ms fall
 * with 3 deg tilt from bottom-left, fade from 135 ms, 120 ms pause, 120 ms height
 * collapse); then [onHide] runs the real disable. Pressing the eye again before the drop
 * starts cancels. If [onHide] returns false the row springs back.
 *
 * e2e contract: eye content-desc is "Hide <title>" (flows.py hides pages with it).
 */
@Composable
fun NavRow(
    info: MenuPageRow,
    showDivider: Boolean,
    onOpen: () -> Unit,
    onHide: suspend () -> Boolean,
) {
    val reduced = LocalReducedMotion.current
    var pressed by remember(info.pageId) { mutableStateOf(false) }
    var committed by remember(info.pageId) { mutableStateOf(false) }
    val lift = remember { Animatable(0f) }
    val drop = remember { Animatable(0f) }
    val fade = remember { Animatable(0f) }
    val collapse = remember { Animatable(1f) }

    LaunchedEffect(pressed) {
        if (!pressed) return@LaunchedEffect
        delay(HIDE_DELAY_MS)
        committed = true
        if (!reduced) {
            lift.animateTo(1f, tween(200, easing = Motion.easeOut))
            coroutineScope {
                launch { drop.animateTo(1f, tween(300, easing = Motion.easeFall)) }
                launch {
                    delay(135)
                    fade.animateTo(1f, tween(165))
                }
            }
            delay(120)
            collapse.animateTo(0f, tween(120))
        }
        val ok = onHide()
        if (!ok) {
            lift.snapTo(0f)
            drop.snapTo(0f)
            fade.snapTo(0f)
            collapse.snapTo(1f)
            committed = false
            pressed = false
        }
    }

    Column(
        Modifier
            .fillMaxWidth()
            .layout { measurable, constraints ->
                val p = measurable.measure(constraints)
                val h = (p.height * collapse.value).roundToInt()
                layout(p.width, h) { p.placeRelative(0, 0) }
            }
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .graphicsLayer {
                    val base = if (pressed) 0.45f else 1f
                    alpha = base * (1f - fade.value)
                    translationY = (-4f * lift.value + 40f * drop.value) * density
                    rotationZ = 3f * drop.value
                    transformOrigin = TransformOrigin(0f, 1f)
                }
                .padding(vertical = 4.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Box(
                modifier = Modifier
                    .weight(1f)
                    .clickable(role = Role.Button) { onOpen() }
                    .padding(vertical = 10.dp, horizontal = 4.dp)
            ) {
                Text(
                    text = info.title,
                    style = MaterialTheme.typography.bodyLarge,
                    color = if (info.isUntitled) {
                        MaterialTheme.colorScheme.onSurface.copy(alpha = 0.5f)
                    } else {
                        MaterialTheme.colorScheme.onSurface
                    }
                )
            }
            EyeButton(
                closed = pressed,
                description = "Hide ${info.title}",
                reduced = reduced,
                onClick = { if (!committed) pressed = !pressed },
            )
        }
        if (showDivider) {
            HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
        }
    }
}

/** Canvas eye from the lab's MainMenuLab: the upper lid folds down when [closed]. */
@Composable
private fun EyeButton(
    closed: Boolean,
    description: String,
    reduced: Boolean,
    onClick: () -> Unit,
) {
    val colors = LocalFeelColors.current
    val t = remember { Animatable(if (closed) 1f else 0f) }
    LaunchedEffect(closed) {
        if (reduced) t.snapTo(if (closed) 1f else 0f)
        else t.animateTo(if (closed) 1f else 0f, tween(480))
    }
    Box(
        Modifier
            .size(48.dp)
            .clickable(role = Role.Button) { onClick() }
            .semantics { contentDescription = description },
        contentAlignment = Alignment.Center,
    ) {
        Canvas(Modifier.size(24.dp)) {
            val k = t.value
            val cx = size.width / 2f
            val cy = size.height / 2f
            val w = size.width * 0.8f
            val h = size.height * 0.5f
            val upperScale = 1f - 2f * k
            val path = Path()
            path.moveTo(cx - w / 2f, cy)
            path.quadraticTo(cx, cy - h * upperScale, cx + w / 2f, cy)
            path.quadraticTo(cx, cy + h, cx - w / 2f, cy)
            drawPath(path, color = colors.fgDim, style = Stroke(width = 1.6f * density))
            val pupilAlpha = (1f - k * 2f).coerceIn(0f, 1f)
            drawCircle(
                color = colors.fgDim.copy(alpha = pupilAlpha),
                radius = 2.4.dp.toPx(),
                center = Offset(cx, cy),
            )
        }
    }
}
