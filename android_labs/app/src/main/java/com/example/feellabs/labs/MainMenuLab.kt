package com.example.feellabs.labs

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.keyframes
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.PathParser
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.example.feellabs.theme.Label
import com.example.feellabs.theme.LocalFeelColors
import com.example.feellabs.theme.LocalReducedMotion
import com.example.feellabs.theme.Motion
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

enum class Hover { Underline, Nudge, Wave }

private class Row(val id: Long, val title: String) {
    var disabled by mutableStateOf(false)
}

private fun seedRows() = listOf(
    Row(1, "Meeting notes"),
    Row(2, "Reading list"),
    Row(3, "Q4 planning"),
    Row(4, "Archive"),
)

@Composable
fun MainMenuLab() {
    val colors = LocalFeelColors.current
    var variant by remember { mutableStateOf(Hover.Underline) }
    val columns = listOf(Hover.Underline, Hover.Nudge, Hover.Wave)

    Column(
        Modifier
            .fillMaxWidth()
            .verticalScroll(androidx.compose.foundation.rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Label("tap a row to latch the effect; tap again to unlatch")
        Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
            for (v in columns) {
                MenuColumn(variant = v)
            }
        }
    }
}

@Composable
private fun MenuColumn(variant: Hover) {
    val colors = LocalFeelColors.current
    val rows = remember(variant) {
        androidx.compose.runtime.mutableStateListOf<Row>().apply { addAll(seedRows()) }
    }
    var latchedId by remember(variant) { mutableStateOf<Long?>(null) }

    Column(
        Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(10.dp))
            .background(colors.surface2)
            .border(1.dp, colors.border, RoundedCornerShape(10.dp))
            .padding(12.dp),
        verticalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Label(
                when (variant) {
                    Hover.Underline -> "underline draw"
                    Hover.Nudge -> "nudge + pill"
                    Hover.Wave -> "letter wave"
                },
            )
            Spacer(Modifier.weight(1f))
            Label(if (latchedId != null) "latched" else "tap to latch")
        }
        for (row in rows) {
            key(row.id) {
                MenuRow(
                    row = row,
                    latched = latchedId == row.id,
                    variant = variant,
                    onTap = {
                        latchedId = if (latchedId == row.id) null else row.id
                    },
                    onPenTap = {},
                    onEyeTap = { row.disabled = !row.disabled },
                )
            }
        }
    }
}

@Composable
private fun MenuRow(
    row: Row,
    latched: Boolean,
    variant: Hover,
    onTap: () -> Unit,
    onPenTap: () -> Unit,
    onEyeTap: () -> Unit,
) {
    val colors = LocalFeelColors.current
    val reduced = LocalReducedMotion.current
    val letterOffsets = remember { mutableListOf<Animatable<Float, *>>() }
    while (letterOffsets.size < row.title.length) {
        letterOffsets.add(Animatable(0f))
    }

    LaunchedEffect(latched) {
        if (variant != Hover.Wave) return@LaunchedEffect
        row.title.forEachIndexed { i, _ ->
            val anim = letterOffsets[i]
            if (latched && !reduced) {
                kotlinx.coroutines.coroutineScope {
                    launch {
                        delay((i * 18).toLong())
                        anim.animateTo(-2f, tween(180, easing = Motion.easeOut))
                    }
                }
            } else {
                anim.animateTo(0f, tween(180))
            }
        }
    }

    val nudge = remember { Animatable(0f) }
    LaunchedEffect(latched) {
        if (variant == Hover.Nudge && !reduced) {
            nudge.animateTo(if (latched) 4f else 0f, tween(220, easing = Motion.easeSpring))
        } else {
            nudge.animateTo(0f, tween(120))
        }
    }

    val underline = remember { Animatable(0f) }
    LaunchedEffect(latched) {
        if (variant == Hover.Underline && !reduced) {
            underline.animateTo(if (latched) 1f else 0f, tween(220, easing = Motion.easeOut))
        } else {
            underline.snapTo(if (latched) 1f else 0f)
        }
    }

    Row(
        Modifier
            .fillMaxWidth()
            .graphicsLayer { alpha = if (row.disabled) 0.45f else 1f }
            .clip(RoundedCornerShape(6.dp))
            .background(if (latched && variant == Hover.Nudge) colors.accentSoft else Color.Transparent)
            .clickable { onTap() }
            .padding(horizontal = 10.dp, vertical = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            Modifier
                .weight(1f)
                .graphicsLayer { translationX = nudge.value * density },
        ) {
            if (variant == Hover.Wave) {
                Row {
                    row.title.forEachIndexed { i, ch ->
                        BasicText(
                            ch.toString(),
                            modifier = Modifier.graphicsLayer {
                                translationY = letterOffsets[i].value * density
                            },
                            style = TextStyle(
                                color = if (latched) colors.accent else colors.fg,
                                fontSize = 14.sp,
                            ),
                        )
                    }
                }
            } else {
                BasicText(
                    row.title,
                    style = TextStyle(
                        color = if (latched && variant == Hover.Underline) colors.accent
                                else colors.fg,
                        fontSize = 14.sp,
                    ),
                )
                if (variant == Hover.Underline && latched) {
                    Box(
                        Modifier
                            .padding(top = 18.dp)
                            .height(1.dp)
                            .width(with(androidx.compose.ui.platform.LocalDensity.current) {
                                (row.title.length * 7.dp.value * underline.value).dp
                            })
                            .background(colors.accent),
                    )
                }
            }
        }
        Spacer(Modifier.width(4.dp))
        PenIcon(onTap = onPenTap, reduced = reduced)
        Spacer(Modifier.width(2.dp))
        EyeIcon(disabled = row.disabled, onTap = onEyeTap, reduced = reduced)
    }
}

@Composable
private fun PenIcon(onTap: () -> Unit, reduced: Boolean) {
    val colors = LocalFeelColors.current
    val rot = remember { Animatable(0f) }
    var gen by remember { mutableIntStateOf(0) }
    LaunchedEffect(gen) {
        if (gen == 0 || reduced) return@LaunchedEffect
        rot.snapTo(0f)
        rot.animateTo(
            0f,
            keyframes {
                durationMillis = 520
                -14f at 80
                11f at 180
                -7f at 290
                4f at 390
                0f at 520
            },
        )
    }
    Box(
        Modifier
            .size(48.dp)
            .clickable { gen++; onTap() }
            .graphicsLayer {
                rotationZ = rot.value
                transformOrigin = androidx.compose.ui.graphics.TransformOrigin(0f, 1f)
            },
        contentAlignment = Alignment.Center,
    ) {
        PenGlyph(color = colors.fgDim)
    }
}

@Composable
private fun PenGlyph(color: Color) {
    // Hand-rolled: a slanted line with a nib
    Box(Modifier.size(20.dp), contentAlignment = Alignment.Center) {
        Box(
            Modifier
                .size(width = 3.dp, height = 16.dp)
                .graphicsLayer { rotationZ = 20f }
                .background(color),
        )
    }
}

@Composable
private fun EyeIcon(disabled: Boolean, onTap: () -> Unit, reduced: Boolean) {
    val colors = LocalFeelColors.current
    val t = remember { Animatable(if (disabled) 1f else 0f) }
    LaunchedEffect(disabled) {
        if (reduced) t.snapTo(if (disabled) 1f else 0f)
        else t.animateTo(if (disabled) 1f else 0f, tween(480))
    }
    val k = t.value
    Box(
        Modifier
            .size(48.dp)
            .clickable { onTap() },
        contentAlignment = Alignment.Center,
    ) {
        androidx.compose.foundation.Canvas(Modifier.size(24.dp)) {
            val cx = size.width / 2f
            val cy = size.height / 2f
            val w = size.width * 0.8f
            val h = size.height * 0.5f
            val strokePx = 1.6f * density
            // Upper lid folds down about the mid-line.
            val upperScale = 1f - 2f * k
            val path = androidx.compose.ui.graphics.Path()
            path.moveTo(cx - w / 2f, cy)
            path.quadraticTo(cx, cy - h * upperScale, cx + w / 2f, cy)
            path.quadraticTo(cx, cy + h, cx - w / 2f, cy)
            drawPath(path, color = colors.fgDim, style = androidx.compose.ui.graphics.drawscope.Stroke(width = strokePx))
            // Pupil
            val pupilAlpha = (1f - k * 2f).coerceIn(0f, 1f)
            drawCircle(
                color = colors.fgDim.copy(alpha = pupilAlpha),
                radius = 2.4.dp.toPx(),
                center = androidx.compose.ui.geometry.Offset(cx, cy),
            )
        }
    }
}
