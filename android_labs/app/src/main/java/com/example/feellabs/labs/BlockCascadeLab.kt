package com.example.feellabs.labs

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import cascade.Mode
import com.example.feellabs.notes.BlockEditor
import com.example.feellabs.notes.BlockModel
import com.example.feellabs.notes.CascadeColumn
import com.example.feellabs.notes.CascadeController
import com.example.feellabs.notes.InitialBlock
import com.example.feellabs.notes.defaultCascadeConfig
import com.example.feellabs.theme.Btn
import com.example.feellabs.theme.Label
import com.example.feellabs.theme.LocalFeelColors
import com.example.feellabs.theme.MockBlock
import com.example.feellabs.theme.MockPage
import com.example.feellabs.theme.MockTitle
import com.example.feellabs.theme.Seg

private val SEED = listOf(
    "Booked the early train. The afternoon is open for the museum." to 60f,
    "Ask about the annual plan — it saves a bit over monthly." to 44f,
    "Two people for dinner on Friday, not three." to 44f,
    "The schema migration is the risky part; test on staging first." to 60f,
    "Short walk after lunch if the weather holds." to 44f,
    "Renew the library card before the end of the month." to 60f,
)

private class LabState {
    var generation by mutableIntStateOf(0)
    val blocks = mutableStateListOf<BlockModel>()
    var nextId = 100L
    fun newId(): Long = nextId++
}

private fun freshBlocks(): LabState {
    val s = LabState()
    SEED.forEach { (text, h) ->
        s.blocks += BlockModel(s.newId(), TextFieldState(text), h)
    }
    return s
}

@Composable
fun BlockCascadeLab() {
    val colors = LocalFeelColors.current
    var state by remember { mutableStateOf(freshBlocks()) }
    var mode by remember { mutableStateOf(Mode.FALL) }
    var controllerGen by remember { mutableIntStateOf(0) }

    val controller = remember(controllerGen) {
        CascadeController(
            config = defaultCascadeConfig(),
            initialBlocks = state.blocks.map { InitialBlock(it.id, 0, it.height) },
            initialOrder = listOf(state.blocks.map { it.id }),
        )
    }
    val reduced = com.example.feellabs.theme.LocalReducedMotion.current
    LaunchedEffect(controller, mode) { controller.setMode(mode) }
    LaunchedEffect(controller, reduced) { controller.setReduced(reduced) }
    DisposableEffect(controller) { onDispose { controller.close() } }
    LaunchedEffect(controller) { controller.runFrames() }

    fun recommit(snapIds: List<Long> = emptyList()) {
        controller.commitOrder(
            listOf(state.blocks.map { it.id }),
            snapIds,
        )
    }

    Column(
        Modifier.fillMaxWidth().padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Seg(
                items = listOf("Fall", "Spring", "Ripple"),
                selected = when (mode) {
                    Mode.FALL -> 0
                    Mode.SPRING -> 1
                    Mode.RIPPLE -> 2
                },
                onSelect = {
                    mode = when (it) {
                        0 -> Mode.FALL
                        1 -> Mode.SPRING
                        else -> Mode.RIPPLE
                    }
                    controller.setMode(mode)
                },
            )
            Spacer(Modifier.width(8.dp))
            Btn("Reset", onClick = {
                state = freshBlocks()
                controllerGen++
            })
        }
        Label("tap + or × on a block; the cascade runs on the crate")

        MockPage {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                MockTitle("Field notes")
                val blockStyle = TextStyle(color = colors.fgDim, fontSize = 15.sp)
                CascadeColumn(
                    controller = controller,
                    blockIds = state.blocks.map { it.id },
                ) { id ->
                    val idx = state.blocks.indexOfFirst { it.id == id }
                    val block = state.blocks.getOrNull(idx) ?: return@CascadeColumn
                    MockBlock(Modifier.fillMaxWidth()) {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            BlockEditor(
                                state = block.text,
                                textStyle = blockStyle,
                                cursorColor = colors.accent,
                                canMergeUp = idx > 0,
                                onSplit = { _, after ->
                                    val newId = state.newId()
                                    val nb = BlockModel(newId, TextFieldState(after), 44f)
                                    state.blocks.add(idx + 1, nb)
                                    controller.addBlock(newId, 0, 44f)
                                    recommit(listOf(newId))
                                },
                                onMergeUp = {
                                    if (idx > 0) {
                                        val above = state.blocks[idx - 1]
                                        val merged = above.text.text.toString() +
                                            block.text.text.toString()
                                        above.text.edit {
                                            replace(0, length, merged)
                                        }
                                        state.blocks.removeAt(idx)
                                        controller.removeBlock(id)
                                        recommit()
                                    }
                                },
                                modifier = Modifier.weight(1f),
                            )
                            Btn("+", onClick = {
                                val at = state.blocks.indexOfFirst { it.id == id }
                                if (at >= 0) {
                                    val newId = state.newId()
                                    val nb = BlockModel(newId, TextFieldState(""), 44f)
                                    state.blocks.add(at + 1, nb)
                                    controller.addBlock(newId, 0, 44f)
                                    recommit(listOf(newId))
                                }
                            })
                            Btn("×", enabled = state.blocks.size > 1, onClick = {
                                val at = state.blocks.indexOfFirst { it.id == id }
                                if (at >= 0) {
                                    state.blocks.removeAt(at)
                                    controller.removeBlock(id)
                                    recommit()
                                }
                            })
                        }
                    }
                }
            }
        }
    }
}
