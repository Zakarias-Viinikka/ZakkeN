package com.example.feellabs

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.BackHandler
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.example.feellabs.labs.Lab
import com.example.feellabs.labs.LabHost
import com.example.feellabs.labs.LabPicker
import com.example.feellabs.notes.BlockEditor
import com.example.feellabs.notes.BlockModel
import com.example.feellabs.notes.CascadeColumn
import com.example.feellabs.notes.CascadeController
import com.example.feellabs.notes.InitialBlock
import com.example.feellabs.notes.NoteStore
import com.example.feellabs.notes.PageModel
import com.example.feellabs.notes.defaultCascadeConfig
import com.example.feellabs.routes.Route
import com.example.feellabs.routes.RouteHolder
import com.example.feellabs.theme.Btn
import com.example.feellabs.theme.FeelTheme
import com.example.feellabs.theme.Label
import com.example.feellabs.theme.LocalFeelColors
import com.example.feellabs.theme.MockBlock
import com.example.feellabs.theme.MockPage
import com.example.feellabs.theme.MockTitle

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            FeelTheme { App() }
        }
    }
}

@Composable
fun App() {
    val store = remember { NoteStore() }
    val routes = remember { RouteHolder(Route.PageList) }
    BackHandler { routes.back() }
    val colors = LocalFeelColors.current
    Box(Modifier.fillMaxSize().background(colors.bg).safeDrawingPadding()) {
        PageListRoute(store, routes)
        store.pages.forEach { page ->
            KeepAlive(active = { routes.current == Route.Page(page.id) }) {
                PageRoute(page)
            }
        }
        KeepAlive(active = { routes.current == Route.LabPicker }) {
            LabPicker(
                onOpen = { lab -> routes.navigate(Route.Lab(lab.id)) },
                onBack = { routes.back() },
            )
        }
        val cur = routes.current
        if (cur is Route.Lab) {
            val lab = Lab.byId(cur.id)
            if (lab != null) {
                LabHost(lab, onBack = { routes.back() })
            }
        }
    }
}

@Composable
private fun PageListRoute(store: NoteStore, routes: RouteHolder) {
    KeepAlive(active = { routes.current == Route.PageList }) {
        val colors = LocalFeelColors.current
        Column(
            Modifier.fillMaxSize().background(colors.bg)
                .verticalScroll(rememberScrollState())
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                MockTitle("feel_labs notes")
                Spacer(Modifier.weight(1f))
                Btn("Labs", onClick = { routes.navigate(Route.LabPicker) })
            }
            Label("${store.pages.size} pages · tap to open")
            store.pages.forEach { page ->
                MockPage {
                    Column {
                        MockTitle(page.title.text.toString())
                        Spacer(Modifier.height(4.dp))
                        Label("${page.blocks.size} blocks")
                        Spacer(Modifier.height(12.dp))
                        Btn("Open", onClick = { routes.navigate(Route.Page(page.id)) })
                    }
                }
            }
        }
    }
}

@Composable
private fun PageRoute(page: PageModel) {
    val colors = LocalFeelColors.current
    val controller = remember {
        CascadeController(
            config = defaultCascadeConfig(),
            initialBlocks = page.blocks.map { InitialBlock(it.id, 0, it.height) },
            initialOrder = listOf(page.blocks.map { it.id }),
        )
    }
    val reduced = com.example.feellabs.theme.LocalReducedMotion.current
    LaunchedEffect(controller, reduced) { controller.setReduced(reduced) }
    DisposableEffect(controller) { onDispose { controller.close() } }
    LaunchedEffect(controller) { controller.runFrames() }

    Column(
        Modifier.fillMaxSize().background(colors.bg)
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
    ) {
        com.example.feellabs.theme.MockPage {
        MockTitle(page.title.text.toString())
        Label("${page.blocks.size} blocks")
        val blockStyle = TextStyle(color = colors.fgDim, fontSize = 15.sp)
        CascadeColumn(
            controller = controller,
            blockIds = page.blocks.map { it.id },
        ) { id ->
            val idx = page.blocks.indexOfFirst { it.id == id }
            val block = page.blocks.getOrNull(idx) ?: return@CascadeColumn
            MockBlock(Modifier.fillMaxWidth()) {
                Row(
                    Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    BlockEditor(
                        state = block.text,
                        textStyle = blockStyle,
                        cursorColor = colors.accent,
                        canMergeUp = idx > 0,
                        onSplit = { _before, after ->
                            val newId = page.newBlockId()
                            val newBlock = BlockModel(
                                newId,
                                TextFieldState(after),
                                44f,
                            )
                            page.blocks.add(idx + 1, newBlock)
                            controller.addBlock(newId, 0, 44f)
                            controller.commitOrder(
                                listOf(page.blocks.map { it.id }),
                                listOf(newId),
                            )
                        },
                        onMergeUp = {
                            if (idx > 0) {
                                val above = page.blocks[idx - 1]
                                val merged = above.text.text.toString() +
                                    block.text.text.toString()
                                above.text.edit {
                                    androidx.compose.foundation.text.input.TextFieldBuffer::class
                                    replace(0, length, merged)
                                }
                                page.blocks.removeAt(idx)
                                controller.removeBlock(id)
                                controller.commitOrder(
                                    listOf(page.blocks.map { it.id }),
                                    emptyList(),
                                )
                            }
                        },
                        modifier = Modifier.weight(1f),
                    )
                    Btn("+", onClick = {
                        val at = page.blocks.indexOfFirst { it.id == id }
                        if (at >= 0) {
                            val newId = page.newBlockId()
                            val newBlock = BlockModel(
                                newId,
                                TextFieldState(""),
                                44f,
                            )
                            page.blocks.add(at + 1, newBlock)
                            controller.addBlock(newId, 0, 44f)
                            controller.commitOrder(
                                listOf(page.blocks.map { it.id }),
                                listOf(newId),
                            )
                        }
                    })
                    Btn("×", enabled = page.blocks.size > 1, onClick = {
                        val at = page.blocks.indexOfFirst { it.id == id }
                        if (at >= 0) {
                            page.blocks.removeAt(at)
                            controller.removeBlock(id)
                            controller.commitOrder(
                                listOf(page.blocks.map { it.id }),
                                emptyList(),
                            )
                        }
                    })
                }
            }
        }
        }
    }
}