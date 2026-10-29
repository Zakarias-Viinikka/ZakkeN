package z.zndroid.MainPages.ViewPage

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.IconButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalLifecycleOwner
import androidx.compose.ui.unit.dp
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import kotlinx.coroutines.launch
import rustlib.my_yrs_lib.docFromSnapshot
import z.zndroid.DbManager
import z.zndroid.Storage.StorageAccess
import z.zndroid.Storage.StorageKey
import z.zndroid.components.GlobalPopupManager
import z.zndroid.db.reads.getBlocksForPageOrdered
import z.zndroid.db.reads.getPageBlob
import z.zndroid.db.reads.isMainMenuPage
import z.zndroid.editor.PageSession
import z.zndroid.editor.PendingFocus
import z.zndroid.log.ViewPageLogs

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ViewPage(
    pageId: String,
    onBack: () -> Unit,
    onOpenAdminView: () -> Unit
) {
    var session by remember { mutableStateOf<PageSession?>(null) }
    var isLoading by remember { mutableStateOf(true) }
    var reloadToken by remember { mutableStateOf(0) }
    val coroutineScope = rememberCoroutineScope()
    val listState = rememberLazyListState()

    LaunchedEffect(pageId, reloadToken) {
        ViewPageLogs.logPageInit(pageId)
        StorageAccess.setValue(StorageKey.CURRENT_DOCUMENT, pageId)
        isLoading = true
        session?.let { runCatching { it.boss.destroy() } }
        session = null

        DbManager.awaitReady()
        try {
            val mirrorRows = getBlocksForPageOrdered(pageId).getOrThrow()
            val blob = getPageBlob(pageId).getOrThrow()
            val userId = StorageAccess.getUserId()
            val isMainMenu = isMainMenuPage(pageId)
            val boss = docFromSnapshot(blob, userId, pageId, System.currentTimeMillis().toString())
            val newSession = PageSession(boss, pageId, isMainMenu, mirrorRows) {
                reloadToken++
            }

            if (newSession.rows.isEmpty()) {
                GlobalPopupManager.showError("Page $pageId has no blocks. Reload to try recovery.")
            } else if (newSession.rows.none { !it.isTitle }) {
                newSession.queueEmptyBody()
            }

            session = newSession

            if (newSession.rows.size == 2 && newSession.rows[1].textFieldValue.text.isEmpty()) {
                newSession.pendingFocus.value = PendingFocus(newSession.rows[1].blockId, 0)
            }
            ViewPageLogs.logPageLoadSuccess(pageId, newSession.rows.size)
        } catch (e: Exception) {
            ViewPageLogs.logPageLoadError(pageId, e.message ?: "Unknown error")
            GlobalPopupManager.showError("Failed to load page: ${e.message}")
        }
        isLoading = false
    }

    BackHandler {
        coroutineScope.launch {
            session?.closeAndFlush()
            onBack()
        }
    }

    val lifecycleOwner = LocalLifecycleOwner.current
    DisposableEffect(lifecycleOwner) {
        val observer = LifecycleEventObserver { _, event ->
            if (event == Lifecycle.Event.ON_STOP) {
                coroutineScope.launch {
                    session?.closeAndFlush()
                }
            }
        }
        lifecycleOwner.lifecycle.addObserver(observer)
        onDispose {
            lifecycleOwner.lifecycle.removeObserver(observer)
            session?.closeAndFlushAsync()
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = {},
                navigationIcon = {
                    IconButton(onClick = {
                        coroutineScope.launch {
                            session?.closeAndFlush()
                            onBack()
                        }
                    }) {
                        Text("←")
                    }
                },
                actions = {
                    TextButton(onClick = onOpenAdminView) {
                        Text("Admin")
                    }
                }
            )
        }
    ) { innerPadding ->
        val currentSession = session
        if (isLoading || currentSession == null) {
            Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                CircularProgressIndicator()
            }
        } else {
            LazyColumn(
                state = listState,
                modifier = Modifier
                    .padding(innerPadding)
                    .fillMaxSize()
                    .padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                itemsIndexed(
                    items = currentSession.rows,
                    key = { _, row -> row.blockId }
                ) { index, row ->
                    EditableBlock(
                        row = row,
                        index = index,
                        session = currentSession,
                        listState = listState
                    )
                }
            }
        }
    }
}