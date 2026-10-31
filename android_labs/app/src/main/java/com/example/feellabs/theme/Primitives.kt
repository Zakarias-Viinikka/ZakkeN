package com.example.feellabs.theme

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.sizeIn
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

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
            .clip(RoundedCornerShape(6.dp))
            .border(1.dp, c.border, RoundedCornerShape(6.dp))
            .clickable(enabled = enabled, onClick = onClick)
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
            .clip(RoundedCornerShape(6.dp))
            .background(c.accentSoft)
            .border(1.dp, c.accentLine, RoundedCornerShape(6.dp))
            .clickable(enabled = enabled, onClick = onClick)
            .sizeIn(minWidth = 48.dp, minHeight = 48.dp)
            .padding(horizontal = 12.dp, vertical = 6.dp),
        contentAlignment = Alignment.Center,
    ) {
        BasicText(text, style = TextStyle(color = c.accent, fontSize = 13.sp))
    }
}

@Composable
fun Seg(
    items: List<String>,
    selected: Int,
    modifier: Modifier = Modifier,
    onSelect: (Int) -> Unit,
) {
    val c = LocalFeelColors.current
    Row(
        modifier
            .clip(RoundedCornerShape(6.dp))
            .border(1.dp, c.border, RoundedCornerShape(6.dp)),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        items.forEachIndexed { i, item ->
            val active = i == selected
            Box(
                Modifier
                    .background(if (active) c.accentSoft else Color.Transparent)
                    .clickable { onSelect(i) }
                    .sizeIn(minWidth = 48.dp, minHeight = 40.dp)
                    .padding(horizontal = 10.dp, vertical = 5.dp),
                contentAlignment = Alignment.Center,
            ) {
                BasicText(
                    item,
                    style = TextStyle(
                        color = if (active) c.accent else c.fgDim,
                        fontSize = 12.sp,
                    ),
                )
            }
            if (i < items.lastIndex) {
                Box(Modifier.width(1.dp).height(24.dp).background(c.border))
            }
        }
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

@Composable
fun MockPage(
    modifier: Modifier = Modifier,
    content: @Composable ColumnScope.() -> Unit,
) {
    val c = LocalFeelColors.current
    Column(
        modifier
            .clip(RoundedCornerShape(14.dp))
            .background(c.surface2)
            .drawWithContent {
                drawContent()
                drawLine(
                    color = c.accentLine,
                    start = Offset(0f, 0.5.dp.toPx()),
                    end = Offset(size.width, 0.5.dp.toPx()),
                    strokeWidth = 1.dp.toPx(),
                )
            }
            .border(1.dp, c.border, RoundedCornerShape(14.dp))
            .padding(24.dp),
        content = content,
    )
}

@Composable
fun MockTitle(text: String, modifier: Modifier = Modifier) {
    val c = LocalFeelColors.current
    BasicText(
        text,
        modifier,
        style = TextStyle(
            color = c.fg,
            fontSize = 28.sp,
            fontWeight = FontWeight.Bold,
            letterSpacing = (-0.84).sp,
        ),
    )
}

@Composable
fun MockBlock(
    modifier: Modifier = Modifier,
    content: @Composable () -> Unit,
) {
    val c = LocalFeelColors.current
    Box(
        modifier
            .clip(RoundedCornerShape(6.dp))
            .border(1.dp, c.border.copy(alpha = 0.04f), RoundedCornerShape(6.dp))
            .padding(horizontal = 10.dp, vertical = 6.dp),
    ) { content() }
}
