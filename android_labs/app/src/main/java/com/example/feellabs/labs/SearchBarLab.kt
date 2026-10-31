package com.example.feellabs.labs

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.input.TextFieldLineLimits
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.example.feellabs.theme.Btn
import com.example.feellabs.theme.Label
import com.example.feellabs.theme.LocalFeelColors
import com.example.feellabs.theme.LocalReducedMotion
import com.example.feellabs.theme.Motion
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.coroutineScope

private data class Doc(val title: String, val breadcrumb: String, val snippet: String)

private val CORPUS = listOf(
    Doc("Onboarding new engineers", "handbook / process", "The first week is about orientation. Pair them with a buddy, walk through the build, and let them ship something small by Friday."),
    Doc("Meeting hygiene", "handbook / culture", "Every meeting needs a decision or a next step. If neither happens, the meeting shouldn't have existed."),
    Doc("Code review checklist", "engineering / process", "Small diffs. A clear description. Tests that fail without the change. No mixed concerns."),
    Doc("Schema migration plan", "engineering / db", "Two versions in the codebase during the transition. Write the reader against both. Roll forward, never back."),
    Doc("On-call rotation", "ops / on-call", "Hand off with context: what's broken, what's watched, what's about to happen. Hand-off happens in writing."),
    Doc("Writing commit messages", "engineering / git", "Subject in the imperative. Body explains why. Reference the issue when there is one."),
    Doc("Design review notes", "design / process", "Bring three variants. Pick one. Say what you're testing. Say what you'd cut if forced."),
    Doc("Weekly status update", "handbook / comms", "Three lines: what moved, what's stuck, what's next. Nothing else."),
    Doc("API versioning", "engineering / api", "Additive only. Deprecate through docs, not surprises. Never break a client silently."),
    Doc("Post-mortem template", "ops / incident", "Timeline, impact, cause, fix, follow-ups. Blame is not one of the fields."),
    Doc("Async decision doc", "handbook / process", "Context, options, recommendation, open questions. A deadline in the doc. Comments close when it does."),
    Doc("Test data strategy", "engineering / testing", "Small, hand-built fixtures. One real-looking dataset per feature. Never share rows between tests."),
    Doc("Feature flag plan", "engineering / release", "One flag per feature. Oldest flag removed every sprint. Flag names start with the area."),
    Doc("Customer feedback loop", "product / process", "Collect in one place. Tag by theme. Review weekly. Answer within two days, or say when."),
)

private data class RowState(
    val doc: Doc,
    val id: Long,
    var leaving: Boolean,
    val enterAt: Long,
    val score: Int,
)

@Composable
fun SearchBarLab() {
    val colors = LocalFeelColors.current
    val reduced = LocalReducedMotion.current
    val query = remember { TextFieldState("") }
    var rows by remember { mutableStateOf<List<RowState>>(emptyList()) }
    var nextId by remember { mutableStateOf(1L) }
    var demoRunning by remember { mutableStateOf(true) }
    var searching by remember { mutableStateOf(false) }

    fun reconcile(q: String) {
        val now = System.currentTimeMillis()
        val scored = CORPUS.map { doc ->
            val score = score(doc, q)
            doc to score
        }.filter { it.second > 0 }.sortedByDescending { it.second }

        val existingByTitle = rows.associateBy { it.doc.title }
        val stillMatched = mutableSetOf<String>()
        val newRows = mutableListOf<RowState>()

        val enteringDelay = if (rows.any { it.leaving }) 120 else 0
        for ((doc, score) in scored) {
            stillMatched.add(doc.title)
            val prior = existingByTitle[doc.title]
            if (prior != null) {
                newRows.add(prior.copy(leaving = false, score = score))
            } else {
                newRows.add(
                    RowState(
                        doc = doc,
                        id = nextId++,
                        leaving = false,
                        enterAt = now + enteringDelay,
                        score = score,
                    )
                )
            }
        }
        // Anything that was there and is no longer matched enters the leaving state.
        for (r in rows) {
            if (r.doc.title !in stillMatched) {
                newRows.add(r.copy(leaving = true))
            }
        }
        rows = newRows
    }

    // Debounced reconciliation.
    LaunchedEffect(Unit) {
        var prev = ""
        while (isActive) {
            val q = query.text.toString()
            if (q == prev) {
                delay(20)
                continue
            }
            searching = q.isNotEmpty()
            delay(140)
            if (query.text.toString() != q) continue
            prev = q
            reconcile(q)
            searching = false
        }
    }

    // Leave timers: after 320ms, drop the leaving rows.
    LaunchedEffect(rows) {
        if (rows.any { it.leaving }) {
            delay(320)
            rows = rows.filterNot { it.leaving }
        }
    }

    Column(
        Modifier.fillMaxWidth().padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            BasicTextField(
                state = query,
                textStyle = TextStyle(color = colors.fg, fontSize = 15.sp),
                cursorBrush = SolidColor(colors.accent),
                lineLimits = TextFieldLineLimits.SingleLine,
                modifier = Modifier
                    .weight(1f)
                    .clip(RoundedCornerShape(6.dp))
                    .background(colors.surface1)
                    .border(1.dp, colors.border, RoundedCornerShape(6.dp))
                    .padding(horizontal = 10.dp, vertical = 8.dp),
            )
            Spacer(Modifier.padding(horizontal = 4.dp))
            Btn(if (demoRunning) "Stop" else "Replay", onClick = {
                if (demoRunning) return@Btn
                demoRunning = true
            })
        }

        // Shimmer bar
        Box(
            Modifier
                .fillMaxWidth()
                .height(2.dp)
                .drawBehind {
                    if (searching && !reduced) {
                        val phase = (System.currentTimeMillis() % 1100) / 1100f
                        val start = (phase * size.width * 1.4f) - size.width * 0.4f
                        drawRect(
                            color = colors.accent.copy(alpha = 0.7f),
                            topLeft = Offset(start, 0f),
                            size = androidx.compose.ui.geometry.Size(size.width * 0.4f, size.height),
                        )
                    }
                },
        )

        // Results body
        Box(
            Modifier
                .fillMaxWidth()
                .height(340.dp)
                .clip(RoundedCornerShape(6.dp))
                .background(colors.surface1)
                .border(1.dp, colors.border, RoundedCornerShape(6.dp)),
        ) {
            Column(
                Modifier.fillMaxWidth().verticalScroll(rememberScrollState()).padding(4.dp),
                verticalArrangement = Arrangement.spacedBy(2.dp),
            ) {
                if (rows.none { !it.leaving } && query.text.toString().isNotEmpty() && !searching) {
                    Box(Modifier.fillMaxWidth().padding(24.dp), contentAlignment = Alignment.Center) {
                        BasicText(
                            "no matches",
                            style = TextStyle(color = colors.fgFaint, fontSize = 13.sp),
                        )
                    }
                }
                rows.forEach { row ->
                    key(row.id) {
                        RowItem(row = row, query = query.text.toString())
                    }
                }
            }
        }
    }

    // Demo script
    LaunchedEffect(demoRunning) {
        if (!demoRunning) return@LaunchedEffect
        val script = "migration"
        var cur = ""
        for (ch in script) {
            if (!demoRunning) return@LaunchedEffect
            cur += ch
            query.edit { replace(0, length, cur) }
            delay(85)
        }
        delay(900)
        while (cur.isNotEmpty()) {
            if (!demoRunning) return@LaunchedEffect
            cur = cur.dropLast(1)
            query.edit { replace(0, length, cur) }
            delay(85)
        }
        demoRunning = false
    }
}

@Composable
private fun RowItem(row: RowState, query: String) {
    val colors = LocalFeelColors.current
    val reduced = LocalReducedMotion.current
    val enter = remember { Animatable(if (reduced) 1f else 0f) }
    val leave = remember { Animatable(if (row.leaving) 0f else 1f) }
    val slide = remember { Animatable(if (reduced) 0f else 12f) }

    LaunchedEffect(row.id) {
        if (!reduced && !row.leaving) {
            val wait = (row.enterAt - System.currentTimeMillis()).coerceAtLeast(0)
            delay(wait)
            kotlinx.coroutines.coroutineScope {
                launch { enter.animateTo(1f, tween(180, easing = Motion.easeOut)) }
                launch { slide.animateTo(0f, tween(320, easing = Motion.easeSpring)) }
            }
        }
    }
    LaunchedEffect(row.leaving) {
        if (row.leaving && !reduced) {
            leave.animateTo(0f, tween(200, easing = Motion.easeOut))
        }
    }

    Column(
        Modifier
            .fillMaxWidth()
            .graphicsLayer {
                alpha = (enter.value * leave.value).coerceIn(0f, 1f)
                translationY = (slide.value + (1f - leave.value) * 10f) * density
                val s = 0.96f + 0.04f * enter.value
                scaleX = s
                scaleY = s
            }
            .padding(10.dp),
        verticalArrangement = Arrangement.spacedBy(2.dp),
    ) {
        BasicText(
            row.doc.title,
            style = TextStyle(color = colors.fg, fontSize = 14.sp, fontWeight = FontWeight.SemiBold),
        )
        BasicText(
            row.doc.breadcrumb,
            style = TextStyle(
                color = colors.fgFaint,
                fontSize = 11.sp,
                fontFamily = FontFamily.Monospace,
            ),
        )
        BasicText(
            highlight(row.doc.snippet, query),
            style = TextStyle(color = colors.fgDim, fontSize = 13.sp),
        )
    }
}

private fun score(doc: Doc, q: String): Int {
    if (q.isBlank()) return 1
    val needle = q.lowercase()
    var s = 0
    if (doc.title.lowercase().contains(needle)) s += 10
    if (doc.breadcrumb.lowercase().contains(needle)) s += 5
    if (doc.snippet.lowercase().contains(needle)) s += 2
    return s
}

private fun highlight(text: String, q: String): AnnotatedString {
    if (q.isBlank()) return AnnotatedString(text)
    val needle = q.lowercase()
    val lower = text.lowercase()
    return buildAnnotatedString {
        var start = 0
        while (true) {
            val idx = lower.indexOf(needle, start)
            if (idx < 0) {
                append(text.substring(start))
                break
            }
            append(text.substring(start, idx))
            withStyle(SpanStyle(fontWeight = FontWeight.SemiBold)) {
                append(text.substring(idx, idx + needle.length))
            }
            start = idx + needle.length
        }
    }
}
