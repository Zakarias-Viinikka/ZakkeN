package com.example.feellabs.labs

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.layout.layout
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.example.feellabs.theme.Btn
import com.example.feellabs.theme.Label
import com.example.feellabs.theme.LocalFeelColors
import com.example.feellabs.theme.LocalReducedMotion
import com.example.feellabs.theme.Motion
import com.example.feellabs.theme.Seg
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

enum class Variant { Red, Strike, Drop }
enum class Phase { Alive, Marked, Leaving }

class DRowState(val id: Long, val text: String) {
    var phase by mutableStateOf(Phase.Alive)
    var locked by mutableStateOf<Variant?>(null)
}

private val SEED_TEXTS = listOf(
    "Booked the early train. The afternoon is open for the museum.",
    "Ask about the annual plan — it saves a bit over monthly.",
    "Two people for dinner on Friday, not three.",
    "The schema migration is the risky part; test on staging first.",
    "Short walk after lunch if the weather holds.",
    "Renew the library card before the end of the month.",
)

private fun seedRows(): List<DRowState> =
    SEED_TEXTS.mapIndexed { i, t -> DRowState((i + 1).toLong(), t) }

@Composable
fun DeletingBlockLab() {
    val colors = LocalFeelColors.current
    var selected by remember { mutableStateOf(Variant.Red) }
    val rows = remember { mutableStateListOf<DRowState>().apply { addAll(seedRows()) } }

    fun reset() {
        rows.clear()
        rows.addAll(seedRows())
    }

    Column(
        Modifier.fillMaxWidth().padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Seg(
                items = listOf("Red fade", "Strike & slide", "Drop"),
                selected = when (selected) {
                    Variant.Red -> 0
                    Variant.Strike -> 1
                    Variant.Drop -> 2
                },
                onSelect = {
                    selected = when (it) {
                        0 -> Variant.Red
                        1 -> Variant.Strike
                        else -> Variant.Drop
                    }
                },
            )
            Spacer(Modifier.padding(horizontal = 4.dp))
            Btn("Reset", onClick = { reset() })
        }
        Label("variant locks per block at click time")
        Column(Modifier.fillMaxWidth()) {
            for (r in rows) {
                key(r.id) {
                    DRow(
                        row = r,
                        onRequestDelete = {
                            if (r.phase == Phase.Alive) {
                                r.locked = selected
                                r.phase = Phase.Marked
                            }
                        },
                        onRemoved = { rows.remove(r) },
                    )
                }
            }
        }
    }
}

@Composable
private fun DRow(row: DRowState, onRequestDelete: () -> Unit, onRemoved: () -> Unit) {
    val colors = LocalFeelColors.current
    val reduced = LocalReducedMotion.current
    val fade = remember { Animatable(1f) }
    val tx = remember { Animatable(0f) }
    val ty = remember { Animatable(0f) }
    val scale = remember { Animatable(1f) }
    val rotate = remember { Animatable(0f) }
    val strike = remember { Animatable(0f) }
    val lift = remember { Animatable(0f) }
    val redMark = remember { Animatable(0f) }
    val collapseH = remember { Animatable(1f) }

    LaunchedEffect(row.phase) {
        val v = row.locked
        when (row.phase) {
            Phase.Marked -> {
                if (reduced) {
                    row.phase = Phase.Leaving
                    return@LaunchedEffect
                }
                when (v) {
                    Variant.Red -> {
                        redMark.snapTo(1f)
                        delay(400)
                    }
                    Variant.Strike -> {
                        strike.animateTo(1f, tween(220, easing = Motion.easeOut))
                        delay(180)
                    }
                    Variant.Drop -> {
                        lift.animateTo(1f, tween(200, easing = Motion.easeOut))
                    }
                    null -> delay(200)
                }
                row.phase = Phase.Leaving
            }
            Phase.Leaving -> {
                when (v) {
                    Variant.Red -> coroutineScope {
                        launch { fade.animateTo(0f, tween(320, easing = Motion.easeFall)) }
                        launch { scale.animateTo(0.85f, tween(320, easing = Motion.easeFall)) }
                        launch { ty.animateTo(8f, tween(320, easing = Motion.easeFall)) }
                    }
                    Variant.Strike -> coroutineScope {
                        launch { fade.animateTo(0f, tween(260, easing = Motion.easeFall)) }
                        launch { tx.animateTo(64f, tween(260, easing = Motion.easeFall)) }
                    }
                    Variant.Drop -> coroutineScope {
                        launch { ty.animateTo(40f, tween(300, easing = Motion.easeFall)) }
                        launch { rotate.animateTo(3f, tween(300, easing = Motion.easeFall)) }
                        launch {
                            delay(135)
                            fade.animateTo(0f, tween(165))
                        }
                    }
                    null -> {}
                }
                delay(120)
                collapseH.animateTo(0f, tween(120, easing = Motion.easeOut))
                onRemoved()
            }
            else -> {}
        }
    }

    val redMix = redMark.value.coerceIn(0f, 1f)
    val bg = lerpColor(colors.surface2, colors.dangerSoft, redMix)
    val borderColor = lerpColor(colors.border, colors.danger, redMix)
    val textColor = lerpColor(colors.fg, colors.fgFaint, strike.value.coerceIn(0f, 1f))

    Box(
        Modifier
            .fillMaxWidth()
            .collapse { collapseH.value },
    ) {
        Row(
            Modifier
                .fillMaxWidth()
                .padding(bottom = 8.dp)
                .graphicsLayer {
                    alpha = fade.value.coerceIn(0f, 1f)
                    translationX = tx.value * density
                    translationY = (ty.value - lift.value * 3f) * density
                    val s = scale.value * (1f + 0.015f * lift.value)
                    scaleX = s
                    scaleY = s
                    rotationZ = rotate.value
                    transformOrigin =
                        androidx.compose.ui.graphics.TransformOrigin(0f, 1f)
                }
                .clip(RoundedCornerShape(6.dp))
                .background(bg)
                .border(1.dp, borderColor, RoundedCornerShape(6.dp))
                .padding(horizontal = 12.dp, vertical = 10.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Box(
                Modifier
                    .weight(1f)
                    .drawStrike(strike.value),
            ) {
                BasicText(
                    row.text,
                    style = TextStyle(color = textColor, fontSize = 15.sp),
                )
            }
            Btn("×", onClick = onRequestDelete)
        }
    }
}

private fun Modifier.collapse(progress: () -> Float): Modifier = this.layout { measurable, constraints ->
    val placeable = measurable.measure(constraints)
    val p = progress().coerceIn(0f, 1f)
    val h = (placeable.height * p).toInt()
    layout(placeable.width, h) { placeable.place(0, 0) }
}

private fun Modifier.drawStrike(progress: Float): Modifier = this.then(
    Modifier.drawWithContent {
        drawContent()
        if (progress > 0.001f) {
            val danger = androidx.compose.ui.graphics.Color(0xFFFF6B6B)
            val y = size.height * 0.55f
            drawLine(
                color = danger,
                start = androidx.compose.ui.geometry.Offset(0f, y),
                end = androidx.compose.ui.geometry.Offset(size.width * progress, y),
                strokeWidth = 1.5f * density,
            )
        }
    }
)

private fun lerpColor(
    a: androidx.compose.ui.graphics.Color,
    b: androidx.compose.ui.graphics.Color,
    t: Float,
): androidx.compose.ui.graphics.Color {
    val tt = t.coerceIn(0f, 1f)
    return androidx.compose.ui.graphics.Color(
        red = a.red + (b.red - a.red) * tt,
        green = a.green + (b.green - a.green) * tt,
        blue = a.blue + (b.blue - a.blue) * tt,
        alpha = a.alpha + (b.alpha - a.alpha) * tt,
    )
}
