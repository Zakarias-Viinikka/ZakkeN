package z.zndroid.MainPages

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.IconButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import z.zndroid.BuildConfig
import z.zndroid.ui.theme.Label
import z.zndroid.ui.theme.LocalFeelColors

/**
 * Settings page, opened by the cog in the main menu (route "settings").
 *
 * Real full-screen page (the lab's SettingsModalLab is a fixed 700x560dp mock and
 * does not fit a phone, so only its look is reused). Today the only section is
 * Developer (debug builds only): Recovery, Database Inspector, Dev Tools. These used
 * to be text buttons at the bottom of NavPage.
 *
 * e2e contract: the cog has content-desc "Settings" (see CogButton), the Developer
 * header renders as "DEVELOPER" (Label uppercases), rows have the exact texts
 * "Recovery", "Database Inspector", "Dev Tools", back arrow text is "←".
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsPage(
    onBack: () -> Unit,
    onOpenRecovery: () -> Unit,
    onOpenDbInspector: () -> Unit,
    onOpenTests: () -> Unit,
    onOpenRapierTest: () -> Unit = {},
) {
    val colors = LocalFeelColors.current
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Settings") },
                navigationIcon = { IconButton(onClick = onBack) { Text("←") } },
            )
        }
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .padding(innerPadding)
                .fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            if (BuildConfig.DEBUG) {
                Label("developer")
                SettingsRow(
                    title = "Recovery",
                    description = "Fix sync when this phone and the server disagree.",
                    onClick = onOpenRecovery,
                )
                SettingsRow(
                    title = "Database Inspector",
                    description = "Browse the local tables and schema.",
                    onClick = onOpenDbInspector,
                )
                SettingsRow(
                    title = "Dev Tools",
                    description = "Run the built-in checks and test actions.",
                    onClick = onOpenTests,
                )
                SettingsRow(
                    title = "Rapier2D Test",
                    description = "Isolated sandbox for CascadeColumn & Rapier physics.",
                    onClick = onOpenRapierTest,
                )
            } else {
                BasicText(
                    "Nothing to configure yet.",
                    style = TextStyle(color = colors.fgDim, fontSize = 14.sp),
                )
            }
        }
    }
}

@Composable
private fun SettingsRow(title: String, description: String, onClick: () -> Unit) {
    val c = LocalFeelColors.current
    Column(
        Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(10.dp))
            .background(c.surface2)
            .border(1.dp, c.border, RoundedCornerShape(10.dp))
            .clickable(role = Role.Button, onClick = onClick)
            .padding(horizontal = 14.dp, vertical = 12.dp),
        verticalArrangement = Arrangement.spacedBy(2.dp),
    ) {
        BasicText(
            title,
            style = TextStyle(color = c.fg, fontSize = 15.sp, fontWeight = FontWeight.SemiBold),
        )
        // Semantics cleared so the merged clickable node's text is exactly the title
        // (e2e finds the row by @text).
        BasicText(
            description,
            modifier = Modifier.clearAndSetSemantics { },
            style = TextStyle(color = c.fgFaint, fontSize = 12.sp),
        )
    }
}

/** Cog wheel for the main menu top bar. content-desc "Settings" is what e2e taps. */
@Composable
fun CogButton(onClick: () -> Unit) {
    val c = LocalFeelColors.current
    Box(
        Modifier
            .size(48.dp)
            .clickable(role = Role.Button, onClick = onClick)
            .semantics { contentDescription = "Settings" },
        contentAlignment = Alignment.Center,
    ) {
        Canvas(Modifier.size(24.dp)) {
            val cx = size.width / 2f
            val cy = size.height / 2f
            val stroke = 1.6.dp.toPx()
            drawCircle(
                color = c.fgDim,
                radius = size.width * 0.27f,
                center = Offset(cx, cy),
                style = Stroke(width = stroke),
            )
            drawCircle(color = c.fgDim, radius = size.width * 0.08f, center = Offset(cx, cy))
            for (i in 0 until 8) {
                val a = (i * Math.PI / 4.0).toFloat()
                val cosA = kotlin.math.cos(a)
                val sinA = kotlin.math.sin(a)
                val inner = size.width * 0.34f
                val outer = size.width * 0.46f
                drawLine(
                    color = c.fgDim,
                    start = Offset(cx + cosA * inner, cy + sinA * inner),
                    end = Offset(cx + cosA * outer, cy + sinA * outer),
                    strokeWidth = stroke * 1.6f,
                    cap = StrokeCap.Round,
                )
            }
        }
    }
}
