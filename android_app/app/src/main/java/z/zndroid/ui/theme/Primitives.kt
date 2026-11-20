package z.zndroid.ui.theme

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.sizeIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

// Ported from android_labs theme/Primitives.kt (Btn, BtnAccent, Label only).
// Differences from the lab: disabled buttons are dimmed, and buttons carry Role.Button.
// Label UPPERCASES its text; do not use it where e2e needs the exact text.

@Composable
fun Btn(
    text: String,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    onClick: () -> Unit,
) {
    val c = LocalFeelColors.current
    Box(
        modifier
            .alpha(if (enabled) 1f else 0.4f)
            .clip(RoundedCornerShape(6.dp))
            .border(1.dp, c.border, RoundedCornerShape(6.dp))
            .clickable(enabled = enabled, role = Role.Button, onClick = onClick)
            .sizeIn(minWidth = 48.dp, minHeight = 48.dp)
            .padding(horizontal = 12.dp, vertical = 6.dp),
        contentAlignment = Alignment.Center,
    ) {
        BasicText(text, style = TextStyle(color = c.fgDim, fontSize = 13.sp))
    }
}

@Composable
fun BtnAccent(
    text: String,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    onClick: () -> Unit,
) {
    val c = LocalFeelColors.current
    Box(
        modifier
            .alpha(if (enabled) 1f else 0.4f)
            .clip(RoundedCornerShape(6.dp))
            .background(c.accentSoft)
            .border(1.dp, c.accentLine, RoundedCornerShape(6.dp))
            .clickable(enabled = enabled, role = Role.Button, onClick = onClick)
            .sizeIn(minWidth = 48.dp, minHeight = 48.dp)
            .padding(horizontal = 12.dp, vertical = 6.dp),
        contentAlignment = Alignment.Center,
    ) {
        BasicText(text, style = TextStyle(color = c.accent, fontSize = 13.sp))
    }
}

@Composable
fun Label(text: String, modifier: Modifier = Modifier) {
    val c = LocalFeelColors.current
    BasicText(
        text.uppercase(),
        modifier,
        style = TextStyle(
            color = c.fgFaint,
            fontSize = 11.sp,
            fontWeight = FontWeight.SemiBold,
            letterSpacing = 1.32.sp,
        ),
    )
}
