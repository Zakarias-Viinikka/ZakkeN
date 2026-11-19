package z.zndroid.MainPages.ViewPage

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.consumeWindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.imePadding
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
import z.zndroid.BuildConfig
import z.zndroid.DbManager
import z.zndroid.Storage.StorageAccess
import z.zndroid.components.GlobalPopupManager
import z.zndroid.db.reads.getBlocksForPageOrdered
import z.zndroid.db.reads.getPageBlob
import z.zndroid.db.reads.isMainMenuPage
import z.zndroid.editor.OpenPageRegistry
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
    var loadFailed by remember { mutableStateOf(false) }
    var reloadToken by remember { mutableStateOf(0) }
    var isLeaving by remember { mutableStateOf(false) }
    val coroutineScope = rememberCoroutineScope()
    val listState = rememberLazyListState()

    fun leavePage() {
        if (isLeaving) return
        isLeaving = true
        coroutineScope.launch {
            val result = session?.flush()
            if (result != null && result.isFailure) {
                isLeaving = false
                return@launch
            }
            onBack()
        }
    }

    LaunchedEffect(pageId, reloadToken) {
        ViewPageLogs.logPageInit(pageId)
        isLoading = true
        loadFailed = false

        // Flush the old session's queue, destroy its boss, and cancel its
        // batcher so nothing can be pushed against a dead yrs doc.
        session?.let { old ->
            OpenPageRegistry.unregister(old)
            old.destroyAfterFlush()
        }
        session = null
        // If a previous ViewPage instance for this pageId just disposed
        // (e.g. activity recreation), wait for its flush to finish before
        // reading the DB.
        OpenPageRegistry.awaitPendingClose(pageId)

        var createdBoss: rustlib.my_yrs_lib.BossOfYrs? = null
        try {
            DbManager.awaitReady()
            val mirrorRows = getBlocksForPageOrdered(pageId).getOrThrow()
            val blob = getPageBlob(pageId).getOrThrow()
            val userId = StorageAccess.getUserId()
            val isMainMenu = isMainMenuPage(pageId)
            val boss = docFromSnapshot(blob, userId, pageId, System.currentTimeMillis().toString())
            createdBoss = boss
            val newSession = PageSession(boss, pageId, isMainMenu, mirrorRows) {
                reloadToken++
            }
            createdBoss = null

            if (newSession.rows.isEmpty()) {
                GlobalPopupManager.showError("Page $pageId has no blocks. Reload to try recovery.")
            } else if (newSession.rows.none { !it.isTitle }) {
                newSession.queueEmptyBody()
            }

            session = newSession
            OpenPageRegistry.register(newSession)

            if (newSession.rows.size == 2 && newSession.rows[1].textFieldValue.text.isEmpty()) {
                newSession.pendingFocus.value = PendingFocus(newSession.rows[1].blockId, 0)
            }
            ViewPageLogs.logPageLoadSuccess(pageId, newSession.rows.size)
        } catch (e: kotlinx.coroutines.CancellationException) {
            createdBoss?.let { runCatching { it.destroy() } }
            throw e
        } catch (e: Exception) {
            createdBoss?.let { runCatching { it.destroy() } }
            ViewPageLogs.logPageLoadError(pageId, e.message ?: "Unknown error")
            GlobalPopupManager.showError("Failed to load page: ${e.message}")
            loadFailed = true
        }
        isLoading = false
    }

    BackHandler { leavePage() }

    val lifecycleOwner = LocalLifecycleOwner.current
    DisposableEffect(lifecycleOwner) {
        val observer = LifecycleEventObserver { _, event ->
            if (event == Lifecycle.Event.ON_STOP) {
                coroutineScope.launch { session?.flush() }
            }
        }
        lifecycleOwner.lifecycle.addObserver(observer)
        onDispose {
            lifecycleOwner.lifecycle.removeObserver(observer)
            session?.let {
                OpenPageRegistry.unregisterAndClose(it)
                session = null
            }
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = {},
                navigationIcon = {
                    IconButton(onClick = { leavePage() }) {
                        Text("←")
                    }
                },
                actions = {
                    if (BuildConfig.DEBUG) {
                        TextButton(onClick = onOpenAdminView) {
                            Text("Admin")
                        }
                    }
                }
            )
        }
    ) { innerPadding ->
        val currentSession = session
        when {
            isLoading -> Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                CircularProgressIndicator()
            }
            currentSession == null -> Column(
                modifier = Modifier
                    .padding(innerPadding)
                    .consumeWindowInsets(innerPadding)
                    .imePadding()
                    .fillMaxSize()
                    .padding(16.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.Center
            ) {
                Text("Could not load this page.")
                TextButton(onClick = { leavePage() }) { Text("Back") }
            }
            else -> LazyColumn(
                state = listState,
                modifier = Modifier
                    .padding(innerPadding)
                    .consumeWindowInsets(innerPadding)
                    .imePadding()
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