package com.example.feellabs.labs

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.input.TextFieldLineLimits
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.TextLayoutResult
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.example.feellabs.theme.LocalFeelColors
import com.example.feellabs.theme.LocalReducedMotion
import com.example.feellabs.theme.Motion
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlin.math.roundToInt

private data class Dot(val id: Long, val yPx: Int, val delayMs: Int)

@Composable
fun RunningIntoTheWallLab() {
    val colors = LocalFeelColors.current
    val reduced = LocalReducedMotion.current
    val text = remember {
        TextFieldState(
            "Type here. When a line fills and a word wraps to the next " +
                "line, three dots trail leftward from that row."
        )
    }
    var layout by remember { mutableStateOf<TextLayoutResult?>(null) }
    val dots = remember { mutableStateListOf<Dot>() }
    var nextId by remember { mutableStateOf(1L) }
    var prevLineCount by remember { mutableIntStateOf(1) }
    var widthPx by remember { mutableIntStateOf(0) }

    LaunchedEffect(Unit) {
        delay(500)
        text.edit { replace(0, length, "") }
        val demo = "Type here. When a line fills and a word wraps to the next line, three dots trail leftward from that row. Keep typing so it wraps a few times and you can see the effect clearly."
        for (i in 1..demo.length) {
            text.edit { replace(0, length, demo.substring(0, i)) }
            delay(40)
        }
    }

    LaunchedEffect(Unit) {
        snapshotFlow { layout?.lineCount ?: 0 }
            .distinctUntilChanged()
            .collect { count ->
                val l = layout
                if (count > prevLineCount && l != null && count > 0) {
                    val prevLine = (prevLineCount - 1).coerceAtLeast(0)
                    val yPx = l.getLineBottom(prevLine).roundToInt()
                    for (d in 0..2) {
                        dots.add(Dot(id = nextId++, yPx = yPx, delayMs = d * 150))
                    }
                }
                prevLineCount = count
            }
    }

    LaunchedEffect(widthPx) {
        prevLineCount = layout?.lineCount ?: 1
    }

    Box(
        Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(6.dp))
            .background(colors.surface1)
            .padding(12.dp)
            .onSizeChanged { widthPx = it.width },
    ) {
        BasicTextField(
            state = text,
            textStyle = TextStyle(
                color = colors.fgDim,
                fontSize = 15.sp,
                fontFamily = FontFamily.Monospace,
            ),
            cursorBrush = SolidColor(colors.accent),
            lineLimits = TextFieldLineLimits.MultiLine(),
            onTextLayout = { getResult -> layout = getResult() },
            modifier = Modifier.fillMaxWidth(),
        )
        dots.forEach { dot ->
            key(dot.id) {
                RunningDot(
                    dot = dot,
                    widthPx = widthPx,
                    enabled = !reduced,
                    onDone = { dots.remove(dot) },
                )
            }
        }
    }
}

@Composable
private fun RunningDot(dot: Dot, widthPx: Int, enabled: Boolean, onDone: () -> Unit) {
    val colors = LocalFeelColors.current
    val progress = remember { Animatable(0f) }
    val density = LocalDensity.current.density
    LaunchedEffect(dot.id) {
        if (enabled) {
            delay(dot.delayMs.toLong())
            progress.animateTo(1f, tween(500, easing = Motion.easeOut))
        } else {
            delay(200)
        }
        onDone()
    }
    val p = progress.value
    val alpha = when {
        p < 0.1f -> (p / 0.1f)
        p > 0.75f -> ((1f - p) / 0.25f)
        else -> 1f
    }.coerceIn(0f, 1f)
    val startX = (widthPx - 6.dp.value * density).roundToInt()
    val translate = -96f * p * density
    Box(
        Modifier
            .offset { IntOffset(startX + translate.roundToInt(), dot.yPx) }
            .graphicsLayer { this.alpha = alpha }
            .size(6.dp)
            .background(colors.accent, CircleShape),
    )
}
