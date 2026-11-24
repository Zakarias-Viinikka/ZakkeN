package com.example.feellabs.labs

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.keyframes
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.input.TextFieldLineLimits
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.TextFieldBuffer
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.example.feellabs.theme.Btn
import com.example.feellabs.theme.BtnAccent
import com.example.feellabs.theme.Label
import com.example.feellabs.theme.LocalFeelColors
import com.example.feellabs.theme.LocalReducedMotion
import com.example.feellabs.theme.MockBlock
import com.example.feellabs.theme.MockPage
import com.example.feellabs.theme.MockTitle
import com.example.feellabs.theme.Motion

private data class Letter(val id: Long, val ch: Char)

@Composable
fun NewPagePopupLab() {
    val colors = LocalFeelColors.current
    var open by remember { mutableStateOf(false) }
    var lastResult by remember { mutableStateOf<String?>(null) }

    Column(
        Modifier.fillMaxWidth().padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        MockPage {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                MockTitle("workspace")
                Label("pretend page list behind the modal")
                BtnAccent("New page", onClick = { open = true })
                if (lastResult != null) {
                    MockBlock(Modifier.fillMaxWidth()) {
                        BasicText(
                            "result: $lastResult",
                            style = TextStyle(color = colors.fgDim, fontSize = 15.sp),
                        )
                    }
                }
            }
        }
    }

    if (open) {
        androidx.compose.ui.window.Dialog(
            onDismissRequest = { open = false },
            properties = androidx.compose.ui.window.DialogProperties(usePlatformDefaultWidth = false),
        ) {
            Box(
                Modifier
                    .fillMaxSize()
                    .background(colors.bgDeep.copy(alpha = 0.55f)),
                contentAlignment = Alignment.TopCenter,
            ) {
                Box(Modifier.padding(top = 80.dp)) {
                    NewPagePanel(
                        onConfirm = { text ->
                            lastResult = text
                            open = false
                        },
                        onCancel = { open = false },
                    )
                }
            }
        }
    }
}

@Composable
private fun NewPagePanel(onConfirm: (String) -> Unit, onCancel: () -> Unit) {
    val colors = LocalFeelColors.current
    val reduced = LocalReducedMotion.current
    val state = remember { TextFieldState("") }
    val letters = remember { mutableListOf<Letter>() }
    var nextId by remember { mutableStateOf(1L) }
    val shake = remember { mutableStateMapOf<Long, Animatable<Float, *>>() }
    val enter = remember { Animatable(if (reduced) 1f else 0f) }
    val focus = remember { FocusRequester() }

    LaunchedEffect(Unit) {
        if (!reduced) enter.animateTo(1f, tween(220, easing = Motion.easeOut))
        focus.requestFocus()
    }

    // Reconcile per-letter ids from text diffs.
    LaunchedEffect(Unit) {
        var prev = ""
        snapshotFlow { state.text.toString() }.collect { cur ->
            val prefix = commonPrefix(prev, cur)
            val suffix = commonSuffix(prev, cur, prefix)
            val prevMid = prev.substring(prefix, prev.length - suffix)
            val curMid = cur.substring(prefix, cur.length - suffix)
            if (prevMid != curMid) {
                val keep = letters.take(prefix) + letters.drop(prefix + prevMid.length).takeLast(suffix)
                val inserted = curMid.map { ch ->
                    val l = Letter(nextId++, ch)
                    shake[l.id] = Animatable(0f)
                    l
                }
                letters.clear()
                letters.addAll(keep.take(prefix) + inserted + keep.drop(prefix))
            }
            prev = cur
        }
    }

    Column(
        Modifier
            .graphicsLayer {
                val t = enter.value
                alpha = t.coerceIn(0f, 1f)
                translationY = ((1f - t).coerceIn(0f, 1f)) * 10f * density
                val s = 0.97f + 0.03f * t
                scaleX = s
                scaleY = s
            }
            .clip(RoundedCornerShape(14.dp))
            .background(colors.surface2)
            .border(1.dp, colors.borderStrong, RoundedCornerShape(14.dp))
            .padding(20.dp)
            .fillMaxWidth(0.85f),
        verticalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        Label("new page")
        Box(Modifier.height(50.dp)) {
            FlowRow(
                horizontalArrangement = Arrangement.spacedBy(0.dp),
                verticalArrangement = Arrangement.spacedBy(0.dp),
            ) {
                if (letters.isEmpty()) {
                    BasicText(
                        "Type a title",
                        style = TextStyle(
                            color = colors.fgFaint,
                            fontSize = 26.sp,
                            fontWeight = FontWeight.Bold,
                        ),
                    )
                }
                letters.forEach { l ->
                    key(l.id) {
                        val anim = shake.getOrPut(l.id) { Animatable(0f) }
                        LaunchedEffect(l.id) {
                            if (!reduced && anim.value == 0f) {
                                anim.animateTo(
                                    targetValue = 0f,
                                    animationSpec = keyframes {
                                        durationMillis = 320
                                        0f at 0
                                        -6f at 50
                                        5f at 110
                                        -3.5f at 160
                                        2f at 210
                                        -1f at 260
                                        0f at 320
                                    },
                                )
                            }
                        }
                        BasicText(
                            l.ch.toString(),
                            modifier = Modifier.graphicsLayer { translationX = anim.value * density },
                            style = TextStyle(
                                color = colors.accent,
                                fontSize = 26.sp,
                                fontWeight = FontWeight.Bold,
                            ),
                        )
                    }
                }
            }
        }
        BasicTextField(
            state = state,
            textStyle = TextStyle(color = colors.fg, fontSize = 15.sp),
            cursorBrush = SolidColor(colors.accent),
            lineLimits = TextFieldLineLimits.SingleLine,
            modifier = Modifier
                .focusRequester(focus)
                .fillMaxWidth()
                .clip(RoundedCornerShape(6.dp))
                .background(colors.surface1)
                .border(1.dp, colors.borderStrong, RoundedCornerShape(6.dp))
                .padding(horizontal = 10.dp, vertical = 8.dp),
        )
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Btn("Cancel", onClick = onCancel)
            Spacer(Modifier.weight(1f))
            BtnAccent("Create", onClick = {
                val t = state.text.toString().trim()
                if (t.isNotEmpty()) onConfirm(t)
            })
        }
    }
}

private fun commonPrefix(a: String, b: String): Int {
    val n = minOf(a.length, b.length)
    var i = 0
    while (i < n && a[i] == b[i]) i++
    return i
}

private fun commonSuffix(a: String, b: String, prefix: Int): Int {
    val max = minOf(a.length, b.length) - prefix
    var i = 0
    while (i < max && a[a.length - 1 - i] == b[b.length - 1 - i]) i++
    return i
}
