package z.zndroid.MainPages.special_menu

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.changedToDown
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import z.zndroid.ui.theme.LocalFeelColors

// Double-tap detector, ported from android_labs/special_menu/MainActivity.kt::interceptDoubleTap.
// Listens at PointerEventPass.Initial so it sees the down event before the WebView.
// A single tap is not consumed, so CM6 still gets it (which is what places the caret
// before the overlay opens). The second tap IS consumed at the Compose layer, but a
// WebView inside AndroidView doesn't observe Compose consumption, so the WebView may
// still see it — that's fine, the second tap position is where we want to insert.
fun Modifier.interceptDoubleTap(onDoubleTap: () -> Unit): Modifier = pointerInput(Unit) {
    var lastTapTime = 0L
    val doubleTapTimeout = viewConfiguration.doubleTapTimeoutMillis
    awaitPointerEventScope {
        while (true) {
            val event = awaitPointerEvent(PointerEventPass.Initial)
            val down = event.changes.firstOrNull { it.changedToDown() }
            if (down != null) {
                val currentTime = System.currentTimeMillis()
                if (currentTime - lastTapTime < doubleTapTimeout) {
                    down.consume()
                    onDoubleTap()
                    lastTapTime = 0L
                } else {
                    lastTapTime = currentTime
                }
            }
        }
    }
}

private const val KEYWORD_LINK_TO_EXISTING = "link to existing page"

/**
 * Special menu overlay. Mirrors web's leptos_components/special_menu/core.rs.
 *
 * First screen: keyword search field. Only "link to existing page" is wired today.
 * Second screen: title picker (LinkToExistingPage.kt). Picking a title invokes
 * [onFinishLinkingToExistingPage]; the caller inserts the wikilink.
 *
 * Marker spike was NOT ported (see INVENTED_ON_ANDROID.md).
 */
@Composable
fun SpecialMenu(
    onDismiss: () -> Unit,
    onFinishLinkingToExistingPage: (TitleMatch) -> Unit,
) {
    val colors = LocalFeelColors.current
    var menuState by remember { mutableStateOf<MenuOption?>(null) }
    BackHandler(enabled = true) { onDismiss() }
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black.copy(alpha = 0.5f))
            .clickable(
                interactionSource = remember { MutableInteractionSource() },
                indication = null,
                onClick = onDismiss
            )
    ) {
        Box(
            modifier = Modifier
                .align(Alignment.Center)
                .widthIn(max = 340.dp)
                .fillMaxWidth(0.9f)
                .clip(RoundedCornerShape(16.dp))
                .background(colors.surface1)
                .border(1.dp, colors.border, RoundedCornerShape(16.dp))
                .clickable(
                    interactionSource = remember { MutableInteractionSource() },
                    indication = null,
                    onClick = {}
                )
                .padding(16.dp)
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    if (menuState != null) {
                        Box(
                            modifier = Modifier
                                .size(32.dp)
                                .clip(CircleShape)
                                .background(colors.surface2)
                                .clickable { menuState = null },
                            contentAlignment = Alignment.Center
                        ) {
                            BasicText("‹", style = TextStyle(fontSize = 20.sp, fontWeight = FontWeight.Bold, color = colors.fg))
                        }
                        Spacer(Modifier.width(8.dp))
                    }
                    BasicText(
                        text = when (menuState) {
                            null -> "Special menu"
                            is MenuOption.LinkToExistingPage -> "Link to existing page"
                            is MenuOption.MakeNewPageAndLinkToIt -> "New page (todo)"
                        },
                        style = TextStyle(fontSize = 16.sp, fontWeight = FontWeight.Bold, color = colors.fg),
                        modifier = Modifier.weight(1f)
                    )
                    Box(
                        modifier = Modifier
                            .size(32.dp)
                            .clip(CircleShape)
                            .background(colors.surface2)
                            .clickable(onClick = onDismiss),
                        contentAlignment = Alignment.Center
                    ) {
                        BasicText("✕", style = TextStyle(fontSize = 16.sp, fontWeight = FontWeight.Bold, color = colors.fg))
                    }
                }
                when (menuState) {
                    null -> WriteField(onSelect = { menuState = it })
                    is MenuOption.LinkToExistingPage -> LinkToExistingPage(
                        onFinish = { onFinishLinkingToExistingPage(it); menuState = null }
                    )
                    is MenuOption.MakeNewPageAndLinkToIt -> BasicText(
                        "TODO (matches web's NewPage stub)",
                        style = TextStyle(fontSize = 14.sp, color = colors.fgDim)
                    )
                }
            }
        }
    }
}

@Composable
private fun WriteField(onSelect: (MenuOption) -> Unit) {
    val colors = LocalFeelColors.current
    var text by remember { mutableStateOf("") }
    val focus = remember { FocusRequester() }
    LaunchedEffect(Unit) { runCatching { focus.requestFocus() } }
    // Matching mirrors web: key.contains(query.lowercase()). Only one real
    // keyword today.
    val matches = remember(text) {
        if (KEYWORD_LINK_TO_EXISTING.contains(text.lowercase())) {
            listOf(KEYWORD_LINK_TO_EXISTING)
        } else emptyList()
    }
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        BasicTextField(
            value = text,
            onValueChange = { text = it },
            modifier = Modifier
                .fillMaxWidth()
                .focusRequester(focus)
                .clip(RoundedCornerShape(6.dp))
                .border(1.dp, colors.border, RoundedCornerShape(6.dp))
                .padding(10.dp),
            textStyle = TextStyle(fontSize = 15.sp, color = colors.fg),
            singleLine = true,
            decorationBox = { inner ->
                if (text.isEmpty()) {
                    BasicText("Search keywords...", style = TextStyle(fontSize = 15.sp, color = colors.fgFaint))
                }
                inner()
            }
        )
        matches.forEach { keyword ->
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(8.dp))
                    .background(colors.surface2)
                    .clickable { onSelect(MenuOption.LinkToExistingPage) }
                    .padding(12.dp)
            ) {
                BasicText(keyword, style = TextStyle(fontSize = 15.sp, color = colors.fg))
            }
        }
    }
}
