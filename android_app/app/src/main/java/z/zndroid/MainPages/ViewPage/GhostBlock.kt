package z.zndroid.MainPages.ViewPage

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.tween
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.layout.layout
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import z.zndroid.editor.BlockRow
import z.zndroid.editor.GhostRow
import z.zndroid.editor.PageSession
import z.zndroid.ui.theme.LocalReducedMotion
import z.zndroid.ui.theme.Motion
import kotlin.math.roundToInt

/**
 * One entry of the ViewPage list: either a live block or a ghost (a block that was just
 * deleted and is playing its drop animation). rowIndex is the index into
 * PageSession.rows (what session operations need); lazyIndex is the index in the
 * merged list (what LazyListState scrolling needs).
 */
sealed interface PageListEntry {
    val key: String

    class Real(val row: BlockRow, val rowIndex: Int, val lazyIndex: Int) : PageListEntry {
        override val key: String get() = row.blockId
    }

    class Ghost(val ghost: GhostRow) : PageListEntry {
        override val key: String get() = ghost.blockId
    }
}

/**
 * Merges rows and ghosts. A ghost is placed right after the row it was below when it
 * was deleted (anchor). A ghost whose anchor is gone goes at the end. A ghost whose id
 * collides with a live row (remote re-add) or another ghost is skipped, because
 * LazyColumn keys must be unique.
 */
fun buildPageEntries(session: PageSession): List<PageListEntry> {
    val rows = session.rows
    val liveIds = HashSet<String>(rows.size * 2)
    rows.forEach { liveIds.add(it.blockId) }
    val ghostsByAnchor = HashMap<String, MutableList<GhostRow>>()
    val orphans = ArrayList<GhostRow>()
    val seen = HashSet<String>()
    for (g in session.ghosts) {
        if (g.blockId in liveIds || !seen.add(g.blockId)) continue
        if (g.anchorBlockId in liveIds) {
            ghostsByAnchor.getOrPut(g.anchorBlockId) { ArrayList() }.add(g)
        } else {
            orphans.add(g)
        }
    }
    val out = ArrayList<PageListEntry>(rows.size + session.ghosts.size)
    rows.forEachIndexed { i, row ->
        out.add(PageListEntry.Real(row, i, out.size))
        ghostsByAnchor[row.blockId]?.forEach { out.add(PageListEntry.Ghost(it)) }
    }
    orphans.forEach { out.add(PageListEntry.Ghost(it)) }
    return out
}

/**
 * Display-only copy of a deleted block playing the lab's Drop variant (DeletingBlockLab):
 * lift 200 ms, then 300 ms fall with 3 deg tilt from the bottom-left and a fade starting
 * at 135 ms, 120 ms pause, 120 ms height collapse, then [onDone] removes the ghost.
 * Semantics are cleared so e2e / TalkBack never see it.
 */
@Composable
fun GhostBlock(ghost: GhostRow, onDone: () -> Unit) {
    val reduced = LocalReducedMotion.current
    val lift = remember { Animatable(0f) }
    val drop = remember { Animatable(0f) }
    val fade = remember { Animatable(0f) }
    val collapse = remember { Animatable(1f) }

    LaunchedEffect(ghost.blockId) {
        if (reduced) {
            onDone()
            return@LaunchedEffect
        }
        lift.animateTo(1f, tween(200, easing = Motion.easeOut))
        coroutineScope {
            launch { drop.animateTo(1f, tween(300, easing = Motion.easeFall)) }
            launch {
                delay(135)
                fade.animateTo(1f, tween(165))
            }
        }
        delay(120)
        collapse.animateTo(0f, tween(120, easing = Motion.easeOut))
        onDone()
    }

    val empty = ghost.text.isEmpty()
    Box(
        Modifier
            .fillMaxWidth()
            .clearAndSetSemantics { }
            .layout { measurable, constraints ->
                val p = measurable.measure(constraints)
                val h = (p.height * collapse.value).roundToInt()
                layout(p.width, h) { p.placeRelative(0, 0) }
            }
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .graphicsLayer {
                    alpha = 1f - fade.value
                    translationY = (-3f * lift.value + 40f * drop.value) * density
                    rotationZ = 3f * drop.value
                    transformOrigin = TransformOrigin(0f, 1f)
                }
                .padding(vertical = 4.dp),
            verticalAlignment = Alignment.Top,
        ) {
            Box(
                modifier = Modifier.width(24.dp).padding(top = 4.dp),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = "::",
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.35f),
                )
            }
            Text(
                text = if (empty) "Type '/' for commands..." else ghost.text,
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurface.copy(alpha = if (empty) 0.3f else 1f),
                modifier = Modifier.weight(1f),
            )
        }
    }
}
