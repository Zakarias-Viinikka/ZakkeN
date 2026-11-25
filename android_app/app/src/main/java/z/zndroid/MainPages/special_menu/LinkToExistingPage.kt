package z.zndroid.MainPages.special_menu

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import z.zndroid.db.reads.searchPageTitles
import z.zndroid.ui.theme.LocalFeelColors

/**
 * Title picker for MenuOption.LinkToExistingPage. Mirrors web's
 * leptos_components/special_menu/cmp/link_to_other.rs.
 *
 * The token counter is the Android equivalent of web's `latest` signal: each
 * new query invalidates earlier ones, so out-of-order async results can't
 * overwrite a newer list.
 */
@Composable
fun LinkToExistingPage(onFinish: (TitleMatch) -> Unit) {
    val colors = LocalFeelColors.current
    var text by remember { mutableStateOf("") }
    var matches by remember { mutableStateOf<List<TitleMatch>>(emptyList()) }
    val token = remember { mutableIntStateOf(0) }
    val focus = remember { FocusRequester() }
    LaunchedEffect(Unit) { runCatching { focus.requestFocus() } }
    LaunchedEffect(text) {
        val mine = ++token.intValue
        val r = searchPageTitles(text)
        if (token.intValue == mine) {
            matches = r.getOrElse { emptyList() }
        }
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
                    BasicText("Search page titles...", style = TextStyle(fontSize = 15.sp, color = colors.fgFaint))
                }
                inner()
            }
        )
        LazyColumn(
            modifier = Modifier.heightIn(max = 320.dp),
            verticalArrangement = Arrangement.spacedBy(4.dp)
        ) {
            items(matches, key = { it.pageId }) { item ->
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(8.dp))
                        .background(colors.surface2)
                        .clickable { onFinish(item) }
                        .padding(10.dp)
                ) {
                    BasicText(
                        item.title.ifBlank { "(untitled)" },
                        style = TextStyle(fontSize = 15.sp, color = colors.fg)
                    )
                }
            }
        }
    }
}
