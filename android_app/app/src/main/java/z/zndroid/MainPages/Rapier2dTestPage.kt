package z.zndroid.MainPages

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.IconButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
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
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import cascade.Config
import cascade.Mode
import z.zndroid.cascade.CascadeColumn
import z.zndroid.cascade.CascadeController
import z.zndroid.cascade.InitialBlock
import z.zndroid.ui.theme.Btn
import z.zndroid.ui.theme.Label
import z.zndroid.ui.theme.LocalFeelColors
import z.zndroid.ui.theme.LocalReducedMotion

private class TestBlockModel(
    val id: Long,
    initialText: String,
    val height: Float = 44f
) {
    var text by mutableStateOf(initialText)
}

private val SEED_TEST_BLOCKS = listOf(
    "Block 1: Initial line of text" to 44f,
    "Block 2: Second paragraph with some details" to 44f,
    "Block 3: Third paragraph in rapier test" to 44f,
    "Block 4: Fourth item to test spacing" to 44f,
)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun Rapier2dTestPage(onBack: () -> Unit) {
    val colors = LocalFeelColors.current
    val reduced = LocalReducedMotion.current
    var nextId by remember { mutableStateOf(100L) }
    val blocks = remember {
        mutableStateListOf<TestBlockModel>().apply {
            SEED_TEST_BLOCKS.forEach { (t, h) -> add(TestBlockModel(nextId++, t, h)) }
        }
    }
    var mode by remember { mutableStateOf(Mode.FALL) }
    var controllerGen by remember { mutableIntStateOf(0) }

    val controller = remember(controllerGen) {
        CascadeController(
            config = Config(
                blockGap = 8f,
                topPad = 0f,
                edgePad = 0f,
                pageGap = 0f,
                minPageH = 0f,
                blockW = 300f,
                mode = mode,
            ),
            initialBlocks = blocks.map { InitialBlock(it.id, 0, it.height) },
            initialOrder = listOf(blocks.map { it.id }),
        )
    }

    LaunchedEffect(controller, mode) { controller.setMode(mode) }
    LaunchedEffect(controller, reduced) { controller.setReduced(reduced) }
    DisposableEffect(controller) { onDispose { controller.close() } }
    LaunchedEffect(controller) { controller.runFrames() }

    fun recommit(snapIds: List<Long> = emptyList()) {
        controller.commitOrder(
            listOf(blocks.map { it.id }),
            snapIds,
        )
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Rapier2D Test") },
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
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Btn("Add", onClick = {
                    val id = nextId++
                    val newBlock = TestBlockModel(id, "New block $id", 44f)
                    blocks.add(newBlock)
                    controller.addBlock(id, 0, 44f)
                    recommit(listOf(id))
                })
                Spacer(Modifier.width(8.dp))
                Btn("Reset", onClick = {
                    blocks.clear()
                    SEED_TEST_BLOCKS.forEach { (t, h) -> blocks.add(TestBlockModel(nextId++, t, h)) }
                    controllerGen++
                })
            }

            Label("blocks below are rendered with CascadeColumn")

            Box(
                Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(8.dp))
                    .background(colors.surface2)
                    .border(1.dp, colors.border, RoundedCornerShape(8.dp))
                    .padding(8.dp)
            ) {
                CascadeColumn(
                    controller = controller,
                    blockIds = blocks.map { it.id },
                ) { id ->
                    val block = blocks.firstOrNull { it.id == id } ?: return@CascadeColumn
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clip(RoundedCornerShape(6.dp))
                            .background(colors.surface1)
                            .border(1.dp, colors.border, RoundedCornerShape(6.dp))
                            .padding(8.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.SpaceBetween
                    ) {
                        BasicTextField(
                            value = block.text,
                            onValueChange = { block.text = it },
                            textStyle = TextStyle(color = colors.fg, fontSize = 15.sp),
                            cursorBrush = SolidColor(colors.accent),
                            modifier = Modifier.weight(1f)
                        )
                        Spacer(Modifier.width(8.dp))
                        Btn("×", onClick = {
                            if (blocks.size > 1) {
                                blocks.removeAll { it.id == id }
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
