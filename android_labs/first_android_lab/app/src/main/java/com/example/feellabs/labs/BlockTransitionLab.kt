package com.example.feellabs.labs

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.AnimationVector1D
import androidx.compose.animation.core.keyframes
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.input.TextFieldLineLimits
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.example.feellabs.theme.Btn
import com.example.feellabs.theme.Label
import com.example.feellabs.theme.LocalFeelColors
import com.example.feellabs.theme.MockBlock
import com.example.feellabs.theme.MockPage
import com.example.feellabs.theme.Motion
import kotlinx.coroutines.delay

@Composable
fun BlockTransitionLab() {
    val colors = LocalFeelColors.current
    val input = remember { TextFieldState("") }
    val lines = remember { mutableStateListOf<String>() }
    var generation by remember { mutableIntStateOf(0) }
    val rise = remember { Animatable(0f) }
    val indent = remember { Animatable(22f) }

    LaunchedEffect(Unit) {
        delay(500)
        val samples = listOf(
            "Booked the early train.",
            "Ask about the annual plan.",
            "Two people for dinner on Friday.",
        )
        for (s in samples) {
            input.edit { replace(0, length, "") }
            for (i in 1..s.length) {
                input.edit { replace(0, length, s.substring(0, i)) }
                delay(45)
            }
            delay(500)
            lines.add(s)
            input.edit { replace(0, length, "") }
            generation++
            delay(1600)
        }
    }

    LaunchedEffect(generation) {
        if (generation > 0) {
            rise.snapTo(0f)
            rise.animateTo(1f, tween(550, easing = Motion.easeOut))
        }
    }
    LaunchedEffect(generation) {
        if (generation > 0) {
            indent.snapTo(22f)
            indent.animateTo(
                targetValue = 0f,
                animationSpec = keyframes {
                    durationMillis = 500
                    -5.5f at 200
                },
            )
        }
    }

    Column(
        Modifier.fillMaxWidth().padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            BasicTextField(
                state = input,
                textStyle = TextStyle(color = colors.fg, fontSize = 15.sp),
                cursorBrush = SolidColor(colors.accent),
                lineLimits = TextFieldLineLimits.SingleLine,
                modifier = Modifier
                    .weight(1f)
                    .clip(RoundedCornerShape(6.dp))
                    .background(colors.surface1)
                    .padding(horizontal = 10.dp, vertical = 8.dp),
            )
            Spacer(Modifier.padding(horizontal = 4.dp))
            Btn("Add block", onClick = {
                val text = input.text.toString().trim()
                if (text.isEmpty()) return@Btn
                lines.add(text)
                input.edit { replace(0, length, "") }
                generation++
            })
        }

        Pane(
            title = "Soft rise",
            lines = lines,
            generation = generation,
            rise = rise,
            indent = indent,
            isRise = true,
        )
        Pane(
            title = "Indent settle",
            lines = lines,
            generation = generation,
            rise = rise,
            indent = indent,
            isRise = false,
        )
    }
}

@Composable
private fun Pane(
    title: String,
    lines: List<String>,
    generation: Int,
    rise: Animatable<Float, AnimationVector1D>,
    indent: Animatable<Float, AnimationVector1D>,
    isRise: Boolean,
) {
    val colors = LocalFeelColors.current
    MockPage {
        Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Label(title)
            if (lines.isEmpty()) {
                MockBlock(Modifier.fillMaxWidth()) {
                    BasicText(
                        "(type something, then Add block)",
                        style = TextStyle(color = colors.fgFaint, fontSize = 15.sp),
                    )
                }
            } else {
                lines.forEachIndexed { idx, line ->
                    key(idx) {
                        val isNewest = idx == lines.lastIndex && generation > 0
                        MockBlock(
                            Modifier
                                .fillMaxWidth()
                                .graphicsLayer {
                                    if (isNewest) {
                                        if (isRise) {
                                            alpha = rise.value.coerceIn(0f, 1f)
                                            translationY =
                                                ((1f - rise.value).coerceIn(0f, 1f)) * 28f * density
                                        } else {
                                            translationX = indent.value * density
                                        }
                                    }
                                },
                        ) {
                            BasicText(
                                line,
                                style = TextStyle(color = colors.fgDim, fontSize = 15.sp),
                            )
                        }
                    }
                }
            }
        }
    }
}
