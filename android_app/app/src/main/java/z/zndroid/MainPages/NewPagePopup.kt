package z.zndroid.MainPages

import android.view.WindowManager
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.keyframes
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import androidx.compose.ui.window.DialogWindowProvider
import kotlinx.coroutines.delay
import z.zndroid.ui.theme.Btn
import z.zndroid.ui.theme.BtnAccent
import z.zndroid.ui.theme.LocalFeelColors
import z.zndroid.ui.theme.LocalReducedMotion
import z.zndroid.ui.theme.Motion

/**
 * New page dialog, ported from android_labs NewPagePopupLab.
 *
 * Look: dim scrim, panel near the top (so the keyboard never covers it), big accent
 * title that is rendered one letter at a time with a per-letter shake on insert, and
 * a small real text field underneath. The big title is display only; its semantics are
 * cleared so the only text node e2e/TalkBack sees for the name is the EditText.
 *
 * Bug fixed vs the lab: the lab kept letters in a plain mutableListOf, so the
 * "Type a title" placeholder never recomposed away. Letters live in a
 * mutableStateListOf here, reconciled synchronously in onValueChange.
 *
 * Kept from the previous version: same parameters, IME handling through
 * DialogWindowProvider, inline error, isSubmitting guard.
 * e2e contract (flows.create_page): text "New Page" exactly (NOT uppercased), one
 * EditText, button text "Create", dialog disappears after a successful create.
 */
private class Letter(val id: Long, val text: String)

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun NewPagePopup(
    onDismiss: () -> Unit,
    onConfirm: (String) -> Unit,
    errorMessage: String? = null,
    isSubmitting: Boolean = false,
    onClearError: () -> Unit = {}
) {
    val colors = LocalFeelColors.current
    val reduced = LocalReducedMotion.current
    var pageName by remember { mutableStateOf("") }
    val letters = remember { mutableStateListOf<Letter>() }
    val idGen = remember { longArrayOf(1L) }
    val focusRequester = remember { FocusRequester() }
    val enter = remember { Animatable(if (reduced) 1f else 0f) }

    fun submit() {
        val t = pageName.trim()
        if (t.isNotEmpty() && !isSubmitting) onConfirm(t)
    }

    fun onTextChange(new: String) {
        reconcileLetters(
            prev = codePoints(pageName),
            cur = codePoints(new),
            letters = letters,
            nextId = { idGen[0]++ },
        )
        pageName = new
        if (errorMessage != null) onClearError()
    }

    Dialog(
        onDismissRequest = { if (!isSubmitting) onDismiss() },
        properties = DialogProperties(usePlatformDefaultWidth = false)
    ) {
        // LocalView.current must be read inside the Dialog content so its
        // parent is the dialog's window, not the activity content view.
        val view = LocalView.current
        val keyboardController = LocalSoftwareKeyboardController.current

        LaunchedEffect(Unit) {
            (view.parent as? DialogWindowProvider)?.window?.let { window ->
                window.setSoftInputMode(
                    WindowManager.LayoutParams.SOFT_INPUT_STATE_VISIBLE or
                        WindowManager.LayoutParams.SOFT_INPUT_ADJUST_RESIZE
                )
                // We draw our own scrim; avoid dimming twice.
                window.setDimAmount(0f)
            }
            delay(200)
            focusRequester.requestFocus()
            keyboardController?.show()
        }

        LaunchedEffect(Unit) {
            if (!reduced) enter.animateTo(1f, tween(220, easing = Motion.easeOut))
        }

        Box(
            Modifier
                .fillMaxSize()
                .background(colors.bgDeep.copy(alpha = 0.55f))
                .pointerInput(isSubmitting) {
                    detectTapGestures(onTap = { if (!isSubmitting) onDismiss() })
                },
            contentAlignment = Alignment.TopCenter,
        ) {
            Box(Modifier.padding(top = 80.dp)) {
                Column(
                    Modifier
                        .fillMaxWidth(0.9f)
                        .widthIn(max = 480.dp)
                        .graphicsLayer {
                            val t = enter.value
                            alpha = t.coerceIn(0f, 1f)
                            translationY = (1f - t).coerceIn(0f, 1f) * 10f * density
                            val s = 0.97f + 0.03f * t
                            scaleX = s
                            scaleY = s
                        }
                        // Swallow taps on the panel so they don't reach the scrim.
                        .pointerInput(Unit) { detectTapGestures { } }
                        .clip(RoundedCornerShape(14.dp))
                        .background(colors.surface2)
                        .border(1.dp, colors.borderStrong, RoundedCornerShape(14.dp))
                        .padding(20.dp),
                    verticalArrangement = Arrangement.spacedBy(14.dp),
                ) {
                    BasicText(
                        "New Page",
                        style = TextStyle(
                            color = colors.fgFaint,
                            fontSize = 12.sp,
                            fontWeight = FontWeight.SemiBold,
                            letterSpacing = 1.sp,
                        ),
                    )

                    // Display-only big title. Semantics cleared: the EditText below is
                    // the one real text node.
                    Box(
                        Modifier
                            .fillMaxWidth()
                            .heightIn(min = 50.dp)
                            .clearAndSetSemantics { }
                    ) {
                        FlowRow {
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
                                    val shake = remember { Animatable(0f) }
                                    LaunchedEffect(Unit) {
                                        if (!reduced) {
                                            shake.animateTo(
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
                                        l.text,
                                        modifier = Modifier.graphicsLayer {
                                            translationX = shake.value * density
                                        },
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
                        value = pageName,
                        onValueChange = { onTextChange(it) },
                        enabled = !isSubmitting,
                        singleLine = true,
                        textStyle = TextStyle(color = colors.fg, fontSize = 15.sp),
                        cursorBrush = SolidColor(colors.accent),
                        keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
                        keyboardActions = KeyboardActions(onDone = { submit() }),
                        modifier = Modifier
                            .focusRequester(focusRequester)
                            .fillMaxWidth()
                            .clip(RoundedCornerShape(6.dp))
                            .background(colors.surface1)
                            .border(
                                1.dp,
                                if (errorMessage != null) colors.danger else colors.borderStrong,
                                RoundedCornerShape(6.dp),
                            )
                            .padding(horizontal = 10.dp, vertical = 8.dp),
                    )

                    if (errorMessage != null) {
                        BasicText(
                            errorMessage,
                            modifier = Modifier.fillMaxWidth(),
                            style = TextStyle(color = colors.danger, fontSize = 12.sp),
                        )
                    }

                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        Btn("Cancel", enabled = !isSubmitting, onClick = onDismiss)
                        Spacer(Modifier.weight(1f))
                        BtnAccent(
                            "Create",
                            enabled = pageName.isNotBlank() && !isSubmitting,
                            onClick = { submit() },
                        )
                    }
                }
            }
        }
    }
}

private fun codePoints(s: String): IntArray = s.codePoints().toArray()

private fun letterOf(cp: Int, id: Long) = Letter(id, String(Character.toChars(cp)))

/**
 * Keeps `letters` aligned one-to-one with the code points of the text. Unchanged
 * head/tail letters keep their ids (so no re-shake); only the changed middle gets
 * fresh letters. If the lists ever disagree in size, rebuilds from scratch.
 */
private fun reconcileLetters(
    prev: IntArray,
    cur: IntArray,
    letters: MutableList<Letter>,
    nextId: () -> Long,
) {
    if (letters.size != prev.size) {
        letters.clear()
        cur.forEach { letters.add(letterOf(it, nextId())) }
        return
    }
    var p = 0
    val maxP = minOf(prev.size, cur.size)
    while (p < maxP && prev[p] == cur[p]) p++
    var s = 0
    val maxS = maxOf(0, minOf(prev.size, cur.size) - p)
    while (s < maxS && prev[prev.size - 1 - s] == cur[cur.size - 1 - s]) s++

    val head = letters.subList(0, p).toList()
    val tail = letters.subList(prev.size - s, prev.size).toList()
    val mid = (p until cur.size - s).map { letterOf(cur[it], nextId()) }
    letters.clear()
    letters.addAll(head + mid + tail)
}