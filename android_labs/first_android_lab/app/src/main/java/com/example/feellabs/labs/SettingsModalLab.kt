package com.example.feellabs.labs

import androidx.compose.animation.animateColorAsState
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
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.input.TextFieldLineLimits
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
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

enum class Category(val label: String) {
    Appearance("Appearance"),
    Editor("Editor"),
    Shortcuts("Shortcuts"),
    Privacy("Privacy"),
    Advanced("Advanced"),
    Danger("Danger zone"),
}

private val DEFAULT_ACCENT = Color(0xFF8AB4FF)
private val DEFAULT_BG = Color(0xFF1C2433)

@Composable
fun SettingsModalLab() {
    val colors = LocalFeelColors.current
    var open by remember { mutableStateOf(false) }

    Column(
        Modifier.fillMaxWidth().padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        MockPage {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                MockTitle("settings preview")
                Label("open the modal to see the live preview")
                BtnAccent("Open settings", onClick = { open = true })
            }
        }
    }

    if (open) {
        androidx.compose.ui.window.Dialog(
            onDismissRequest = { open = false },
            properties = androidx.compose.ui.window.DialogProperties(usePlatformDefaultWidth = false),
        ) {
            Box(
                Modifier.fillMaxSize().background(Color(0x99101C2C)),
                contentAlignment = Alignment.Center,
            ) {
                SettingsPanel(onClose = { open = false })
            }
        }
    }
}

@Composable
private fun SettingsPanel(onClose: () -> Unit) {
    val colors = LocalFeelColors.current
    val reduced = LocalReducedMotion.current
    var active by remember { mutableStateOf(Category.Appearance) }
    var categoryIndex by remember { mutableStateOf(0) }
    val all = Category.entries

    // Appearance settings (each its own state).
    var accent by remember { mutableStateOf(DEFAULT_ACCENT) }
    var bg by remember { mutableStateOf(DEFAULT_BG) }
    var fontSize by remember { mutableStateOf(16f) }
    var radius by remember { mutableStateOf(6f) }
    var padY by remember { mutableStateOf(6f) }

    // Editor
    var wrap by remember { mutableStateOf(true) }
    var lineHeight by remember { mutableStateOf(1.6f) }
    var tabSize by remember { mutableStateOf(2f) }

    // Shortcuts
    var shortcutText = remember { TextFieldState("cmd+k / cmd+p") }

    // Privacy
    var telemetry by remember { mutableStateOf(false) }
    var crashReports by remember { mutableStateOf(true) }

    // Advanced
    var logLevel by remember { mutableStateOf("info") }
    var cacheSize by remember { mutableStateOf(512f) }

    // Danger
    var dangerText = remember { TextFieldState("") }

    val dirty = remember {
        derivedStateOf {
            (accent != DEFAULT_ACCENT).hashCode() xor
                (bg != DEFAULT_BG).hashCode() xor
                fontSize.toInt() xor lineHeight.toInt() xor tabSize.toInt() xor
                (wrap).hashCode() xor (telemetry).hashCode() xor (crashReports).hashCode() xor
                shortcutText.text.length xor
                logLevel.hashCode() xor cacheSize.toInt()
        }.value
    }

    val dirtyCount = if (dirty != 0) 1 else 0

    val enter = remember { Animatable(if (reduced) 1f else 0f) }
    LaunchedEffect(Unit) {
        if (!reduced) enter.animateTo(1f, tween(260, easing = Motion.easeOut))
    }

    Column(
        Modifier
            .graphicsLayer {
                val t = enter.value
                alpha = t.coerceIn(0f, 1f)
                translationY = ((1f - t).coerceIn(0f, 1f)) * 12f * density
            }
            .width(700.dp)
            .height(560.dp)
            .clip(RoundedCornerShape(14.dp))
            .background(colors.surface1)
            .border(1.dp, colors.borderStrong, RoundedCornerShape(14.dp)),
    ) {
        Row(
            Modifier.fillMaxWidth().padding(16.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            BasicText(
                "Settings",
                modifier = Modifier.weight(1f),
                style = TextStyle(color = colors.fg, fontSize = 18.sp, fontWeight = FontWeight.SemiBold),
            )
            if (dirtyCount > 0) {
                Box(
                    Modifier
                        .size(8.dp)
                        .background(colors.accent, CircleShape),
                )
                Spacer(Modifier.width(8.dp))
                Label("dirty")
                Spacer(Modifier.width(8.dp))
            }
            Btn("×", onClick = onClose)
        }
        Box(Modifier.fillMaxWidth().height(1.dp).background(colors.border))
        Row(Modifier.fillMaxSize()) {
            // Category column
            Column(
                Modifier
                    .width(190.dp)
                    .background(colors.surface2)
                    .padding(horizontal = 8.dp, vertical = 12.dp),
                verticalArrangement = Arrangement.spacedBy(2.dp),
            ) {
                all.forEachIndexed { i, cat ->
                    val isActive = active == cat
                    Box(
                        Modifier
                            .fillMaxWidth()
                            .clip(RoundedCornerShape(6.dp))
                            .background(if (isActive) colors.accentSoft else Color.Transparent)
                            .clickable { active = cat; categoryIndex = i }
                            .padding(horizontal = 10.dp, vertical = 8.dp),
                    ) {
                        BasicText(
                            cat.label,
                            style = TextStyle(
                                color = if (isActive) colors.accent else colors.fgDim,
                                fontSize = 13.sp,
                            ),
                        )
                    }
                }
            }
            // Pane
            Box(Modifier.weight(1f)) {
                key(active, categoryIndex) {
                    PaneSwapper(categoryIndex) {
                        Column(
                            Modifier
                                .fillMaxSize()
                                .verticalScroll(rememberScrollState())
                                .padding(20.dp),
                            verticalArrangement = Arrangement.spacedBy(12.dp),
                        ) {
                            when (active) {
                                Category.Appearance -> AppearancePane(
                                    accent = accent, onAccent = { accent = it },
                                    bg = bg, onBg = { bg = it },
                                    fontSize = fontSize, onFontSize = { fontSize = it },
                                    radius = radius, onRadius = { radius = it },
                                    padY = padY, onPadY = { padY = it },
                                )
                                Category.Editor -> EditorPane(
                                    wrap = wrap, onWrap = { wrap = it },
                                    lineHeight = lineHeight, onLineHeight = { lineHeight = it },
                                    tabSize = tabSize, onTabSize = { tabSize = it },
                                )
                                Category.Shortcuts -> ShortcutsPane(shortcutText)
                                Category.Privacy -> PrivacyPane(
                                    telemetry = telemetry, onTelemetry = { telemetry = it },
                                    crashReports = crashReports, onCrashReports = { crashReports = it },
                                )
                                Category.Advanced -> AdvancedPane(
                                    logLevel = logLevel, onLogLevel = { logLevel = it },
                                    cacheSize = cacheSize, onCacheSize = { cacheSize = it },
                                )
                                Category.Danger -> DangerPane(dangerText)
                            }
                        }
                    }
                }
            }
        }
        Box(Modifier.fillMaxWidth().height(1.dp).background(colors.border))
        Row(
            Modifier.fillMaxWidth().padding(16.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            if (dirtyCount > 0) Label("$dirtyCount change") else Label("clean")
            Spacer(Modifier.weight(1f))
            Btn("Cancel", onClick = onClose)
            Spacer(Modifier.width(8.dp))
            BtnAccent("Save", onClick = onClose)
        }
    }
}

@Composable
private fun PaneSwapper(direction: Int, content: @Composable () -> Unit) {
    val colors = LocalFeelColors.current
    val reduced = LocalReducedMotion.current
    val t = remember { Animatable(if (reduced) 1f else 0f) }
    LaunchedEffect(Unit) { if (!reduced) t.animateTo(1f, tween(220, easing = Motion.easeOut)) }
    Box(
        Modifier
            .fillMaxSize()
            .graphicsLayer {
                val v = t.value
                alpha = v.coerceIn(0f, 1f)
                translationX = ((1f - v).coerceIn(0f, 1f)) * direction * 10f * density
            },
    ) {
        content()
    }
}

@Composable
private fun AppearancePane(
    accent: Color, onAccent: (Color) -> Unit,
    bg: Color, onBg: (Color) -> Unit,
    fontSize: Float, onFontSize: (Float) -> Unit,
    radius: Float, onRadius: (Float) -> Unit,
    padY: Float, onPadY: (Float) -> Unit,
) {
    val colors = LocalFeelColors.current
    Label("preview")
    Box(
        Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(radius.dp))
            .background(bg)
            .border(1.dp, colors.border, RoundedCornerShape(radius.dp))
            .padding(horizontal = 14.dp, vertical = padY.dp),
    ) {
        Column {
            BasicText("Live preview", style = TextStyle(color = accent, fontSize = fontSize.sp, fontWeight = FontWeight.Bold))
            BasicText("Body text at your chosen size.", style = TextStyle(color = colors.fgDim, fontSize = (fontSize - 2).sp))
        }
    }
    Label("accent")
    Swatches(
        options = listOf(Color(0xFF8AB4FF), Color(0xFF7BC4A0), Color(0xFFE8B060), Color(0xFFD67088)),
        selected = accent, onSelect = onAccent,
    )
    Label("background")
    Swatches(
        options = listOf(Color(0xFF1C2433), Color(0xFF151C29), Color(0xFF2B3648), Color(0xFF0B1020)),
        selected = bg, onSelect = onBg,
    )
    Slider("font size", fontSize, 12f, 22f, onFontSize)
    Slider("corner radius", radius, 0f, 16f, onRadius)
    Slider("vertical padding", padY, 2f, 14f, onPadY)
}

@Composable
private fun EditorPane(
    wrap: Boolean, onWrap: (Boolean) -> Unit,
    lineHeight: Float, onLineHeight: (Float) -> Unit,
    tabSize: Float, onTabSize: (Float) -> Unit,
) {
    Toggle("word wrap", wrap, onWrap)
    Slider("line height", lineHeight, 1.2f, 2.2f, onLineHeight)
    Slider("tab size", tabSize, 1f, 8f, onTabSize)
}

@Composable
private fun ShortcutsPane(state: TextFieldState) {
    val colors = LocalFeelColors.current
    Label("search shortcut")
    BasicTextField(
        state = state,
        textStyle = TextStyle(color = colors.fg, fontSize = 14.sp, fontFamily = FontFamily.Monospace),
        cursorBrush = SolidColor(colors.accent),
        lineLimits = TextFieldLineLimits.SingleLine,
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(6.dp))
            .background(colors.surface2)
            .border(1.dp, colors.border, RoundedCornerShape(6.dp))
            .padding(10.dp),
    )
    Label("(no persistence)")
}

@Composable
private fun PrivacyPane(
    telemetry: Boolean, onTelemetry: (Boolean) -> Unit,
    crashReports: Boolean, onCrashReports: (Boolean) -> Unit,
) {
    Toggle("send anonymous usage statistics", telemetry, onTelemetry)
    Toggle("send crash reports", crashReports, onCrashReports)
}

@Composable
private fun AdvancedPane(
    logLevel: String, onLogLevel: (String) -> Unit,
    cacheSize: Float, onCacheSize: (Float) -> Unit,
) {
    Label("log level")
    Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        for (opt in listOf("debug", "info", "warn", "error")) {
            val active = opt == logLevel
            Btn(opt, onClick = { onLogLevel(opt) })
            if (active) { /* no highlight in this minimal Btn */ }
        }
    }
    Slider("cache size (MB)", cacheSize, 64f, 2048f, onCacheSize)
}

@Composable
private fun DangerPane(state: TextFieldState) {
    val colors = LocalFeelColors.current
    Label("delete everything")
    BasicText(
        "Type DELETE to confirm. This is a demo — nothing is deleted.",
        style = TextStyle(color = colors.fgDim, fontSize = 13.sp),
    )
    BasicTextField(
        state = state,
        textStyle = TextStyle(color = colors.danger, fontSize = 14.sp, fontFamily = FontFamily.Monospace),
        cursorBrush = SolidColor(colors.danger),
        lineLimits = TextFieldLineLimits.SingleLine,
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(6.dp))
            .background(colors.dangerSoft)
            .border(1.dp, colors.danger, RoundedCornerShape(6.dp))
            .padding(10.dp),
    )
    Btn("Delete", onClick = {})
}

@Composable
private fun Slider(label: String, value: Float, minV: Float, maxV: Float, onChange: (Float) -> Unit) {
    val colors = LocalFeelColors.current
    Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Row {
            Label(label)
            Spacer(Modifier.weight(1f))
            BasicText(
                "%.2f".format(value),
                style = TextStyle(color = colors.fgDim, fontSize = 12.sp, fontFamily = FontFamily.Monospace),
            )
        }
        Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
            Btn("-", onClick = { onChange((value - (maxV - minV) / 20f).coerceAtLeast(minV)) })
            Btn("+", onClick = { onChange((value + (maxV - minV) / 20f).coerceAtMost(maxV)) })
        }
    }
}

@Composable
private fun Toggle(label: String, value: Boolean, onChange: (Boolean) -> Unit) {
    val colors = LocalFeelColors.current
    val bg by animateColorAsState(
        targetValue = if (value) colors.accent else colors.surface3,
        animationSpec = tween(160),
        label = "toggleBg",
    )
    Row(
        Modifier.fillMaxWidth().clickable { onChange(!value) }.padding(vertical = 4.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        BasicText(label, style = TextStyle(color = colors.fg, fontSize = 14.sp))
        Spacer(Modifier.weight(1f))
        Box(
            Modifier
                .width(40.dp)
                .height(22.dp)
                .clip(RoundedCornerShape(11.dp))
                .background(bg),
        ) {
            Box(
                Modifier
                    .padding(2.dp)
                    .graphicsLayer { translationX = if (value) 18f * density else 0f }
                    .size(18.dp)
                    .background(colors.surface1, CircleShape),
            )
        }
    }
}

@Composable
private fun Swatches(options: List<Color>, selected: Color, onSelect: (Color) -> Unit) {
    val colors = LocalFeelColors.current
    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        for (c in options) {
            Box(
                Modifier
                    .size(32.dp)
                    .clip(CircleShape)
                    .background(c)
                    .border(
                        width = if (c == selected) 2.dp else 1.dp,
                        color = if (c == selected) colors.fg else colors.border,
                        shape = CircleShape,
                    )
                    .clickable { onSelect(c) },
            )
        }
    }
}
