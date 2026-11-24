package com.example.feellabs.labs

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
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.example.feellabs.theme.Btn
import com.example.feellabs.theme.Label
import com.example.feellabs.theme.LocalFeelColors
import com.example.feellabs.theme.LocalReducedMotion
import com.example.feellabs.theme.MockPage
import com.example.feellabs.theme.MockTitle
import com.example.feellabs.theme.Motion
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.launch
import kotlin.random.Random

enum class Sev { Info, Success, Warning, Error }

data class ToastSpec(val title: String, val summary: String, val sev: Sev, val lifeMs: Long)

data class LiveToast(
    val id: Long,
    val spec: ToastSpec,
    var bornMs: Long,
    var held: Boolean = false,
)

private val SPAWN_POOL = listOf(
    ToastSpec("Saved", "The page was saved to disk.", Sev.Success, 3200),
    ToastSpec("Connection lost", "Retrying in 2s.", Sev.Warning, 4500),
    ToastSpec("Copy failed", "Clipboard is unavailable.", Sev.Error, 5000),
    ToastSpec("Tip", "Right-click a block for more options.", Sev.Info, 4000),
    ToastSpec("Synced", "Two changes merged from the server.", Sev.Success, 3000),
    ToastSpec("Update available", "Version 0.4.2 is out.", Sev.Info, 4000),
    ToastSpec("Quota warning", "Storage is 85% full.", Sev.Warning, 4500),
    ToastSpec("Upload failed", "Check the network and retry.", Sev.Error, 5000),
)

@Composable
fun PopupsLab() {
    val colors = LocalFeelColors.current
    val reduced = LocalReducedMotion.current
    val toasts = remember { mutableStateListOf<LiveToast>() }
    var nextId by remember { mutableStateOf(1L) }
    var clock by remember { mutableLongStateOf(0L) }
    var modal by remember { mutableStateOf<LiveToast?>(null) }
    var spawnAccum by remember { mutableLongStateOf(0L) }
    var spawnEvery by remember { mutableLongStateOf(2800L) }
    var lastFrameNanos by remember { mutableLongStateOf(0L) }

    // One frame loop owns the virtual clock.
    LaunchedEffect(Unit) {
        while (true) {
            withFrameNanos { now ->
                if (lastFrameNanos == 0L) {
                    lastFrameNanos = now
                    return@withFrameNanos
                }
                val dtMs = ((now - lastFrameNanos) / 1_000_000L).coerceAtMost(64L)
                lastFrameNanos = now
                val held = modal != null || toasts.any { it.held }
                if (!held) clock += dtMs
                // Cull
                toasts.removeAll { !it.held && clock - it.bornMs >= it.spec.lifeMs }
                // Spawn
                if (modal == null && toasts.size < 4) {
                    spawnAccum += dtMs
                    if (spawnAccum >= spawnEvery) {
                        spawnAccum = 0
                        val spec = SPAWN_POOL.random()
                        toasts.add(LiveToast(nextId++, spec, clock))
                        spawnEvery = 2400L + Random.nextLong(0, 800)
                    }
                }
            }
        }
    }

    Column(
        Modifier.fillMaxWidth().padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            Btn("Spawn one", onClick = {
                val spec = SPAWN_POOL.random()
                if (toasts.size < 4) toasts.add(LiveToast(nextId++, spec, clock))
            })
            Btn("Clear", onClick = { toasts.clear() })
        }
        Label("tap a toast to open it as a modal; auto-dismiss pauses while open")
        MockPage {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                MockTitle("toast stage")
                Label("(stack sits below, in the app's bottom-right)")
            }
        }
        Spacer(Modifier.weight(1f))
        Box(Modifier.fillMaxWidth().height(180.dp))
    }

    // Toast stack - Popup so it's not confined to lab layout
    if (toasts.isNotEmpty()) {
        androidx.compose.ui.window.Popup(
            alignment = Alignment.BottomEnd,
            properties = androidx.compose.ui.window.PopupProperties(focusable = false),
        ) {
            Column(
                Modifier.padding(16.dp).width(340.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                toasts.forEach { t ->
                    key(t.id) {
                        ToastRow(
                            toast = t,
                            clock = clock,
                            onTap = { if (modal == null) modal = t },
                            held = { held -> t.held = held },
                            reduced = reduced,
                        )
                    }
                }
            }
        }
    }

    // Modal
    val m = modal
    if (m != null) {
        androidx.compose.ui.window.Dialog(
            onDismissRequest = { modal = null },
            properties = androidx.compose.ui.window.DialogProperties(usePlatformDefaultWidth = false),
        ) {
            Box(
                Modifier.fillMaxSize().background(Color(0x99101C2C)),
                contentAlignment = Alignment.Center,
            ) {
                ModalPanel(toast = m, onClose = { modal = null })
            }
        }
    }
}

@Composable
private fun ToastRow(
    toast: LiveToast,
    clock: Long,
    onTap: () -> Unit,
    held: (Boolean) -> Unit,
    reduced: Boolean,
) {
    val colors = LocalFeelColors.current
    val alpha = remember { Animatable(if (reduced) 1f else 0f) }
    val dx = remember { Animatable(if (reduced) 0f else 28f) }
    LaunchedEffect(toast.id) {
        if (!reduced) {
            coroutineScope {
                launch { alpha.animateTo(1f, tween(180)) }
                launch { dx.animateTo(0f, tween(380, easing = Motion.easeSpring)) }
            }
        }
    }

    val pct = ((clock - toast.bornMs).toFloat() / toast.spec.lifeMs.toFloat()).coerceIn(0f, 1f)
    val barColor = when (toast.spec.sev) {
        Sev.Info -> colors.fgDim
        Sev.Success -> colors.accent
        Sev.Warning -> colors.warn
        Sev.Error -> colors.danger
    }

    Row(
        Modifier
            .fillMaxWidth()
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
            .clickable { onTap() }
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
                toast.spec.title,
                style = TextStyle(
                    color = colors.fg,
                    fontSize = 13.sp,
                    fontWeight = FontWeight.SemiBold,
                ),
            )
            BasicText(
                toast.spec.summary,
                style = TextStyle(color = colors.fgDim, fontSize = 12.sp),
            )
        }
    }
}

@Composable
private fun ModalPanel(toast: LiveToast, onClose: () -> Unit) {
    val colors = LocalFeelColors.current
    Column(
        Modifier
            .width(500.dp)
            .clip(RoundedCornerShape(14.dp))
            .background(colors.surface1)
            .border(1.dp, colors.borderStrong, RoundedCornerShape(14.dp)),
    ) {
        Row(
            Modifier.fillMaxWidth().padding(14.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            BasicText(
                toast.spec.title,
                modifier = Modifier.weight(1f),
                style = TextStyle(color = colors.fg, fontSize = 15.sp, fontWeight = FontWeight.SemiBold),
            )
            Btn("×", onClick = onClose)
        }
        Box(
            Modifier
                .fillMaxWidth()
                .height(1.dp)
                .background(colors.border),
        )
        Column(
            Modifier
                .fillMaxWidth()
                .height(220.dp)
                .verticalScroll(rememberScrollState())
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            BasicText(
                toast.spec.summary,
                style = TextStyle(color = colors.fgDim, fontSize = 14.sp),
            )
            Label("kind: ${toast.spec.sev.name}")
            Label("life: ${toast.spec.lifeMs} ms")
            val log = buildString {
                appendLine("id = ${toast.id}")
                appendLine("title = ${toast.spec.title}")
                appendLine("life = ${toast.spec.lifeMs}")
                appendLine("---")
                appendLine("log line 1: request started")
                appendLine("log line 2: response 200")
                appendLine("log line 3: body length 4096")
            }
            Box(
                Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(6.dp))
                    .background(colors.bgDeep)
                    .padding(10.dp),
            ) {
                BasicText(
                    log,
                    style = TextStyle(
                        color = colors.fgDim,
                        fontSize = 11.sp,
                        fontFamily = FontFamily.Monospace,
                    ),
                )
            }
        }
    }
}
