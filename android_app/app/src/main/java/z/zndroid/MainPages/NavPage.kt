package z.zndroid.MainPages

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import z.zndroid.DbManager
import z.zndroid.actions.createNewPage
import z.zndroid.actions.disablePage
import z.zndroid.components.GlobalPopupManager
import z.zndroid.db.reads.MenuPageRow
import z.zndroid.db.reads.getMenuPages
import z.zndroid.sync.owner_of_interacting_with_server.ConnState
import z.zndroid.sync.owner_of_interacting_with_server.connState
import z.zndroid.sync.owner_of_interacting_with_server.menuTick

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun NavPage(
    onOpenPage: (String) -> Unit,
    onOpenSettings: () -> Unit,
    onOpenRecovery: () -> Unit
) {
    var pages by remember { mutableStateOf(emptyList<MenuPageRow>()) }
    var hasLoadedOnce by remember { mutableStateOf(false) }
    var showNewPagePopup by remember { mutableStateOf(false) }
    var newPageError by remember { mutableStateOf<String?>(null) }
    var scrollToNewest by remember { mutableStateOf(false) }
    var actionInFlight by remember { mutableStateOf(false) }
    val listState = rememberLazyListState()
    val coroutineScope = rememberCoroutineScope()
    val conn by connState.collectAsState()
    val syncTick by menuTick.collectAsState()

    fun refreshPages(onRefreshed: (() -> Unit)? = null) {
        coroutineScope.launch {
            try {
                DbManager.awaitReady()
                getMenuPages()
                    .onSuccess { newPages ->
                        pages = newPages
                        hasLoadedOnce = true
                        onRefreshed?.invoke()
                    }
                    .onFailure {
                        GlobalPopupManager.showError("Failed to load pages: ${it.message}")
                        hasLoadedOnce = true
                    }
            } catch (e: Exception) {
                GlobalPopupManager.showError("Failed to load pages: ${e.message}")
                hasLoadedOnce = true
            }
        }
    }

    LaunchedEffect(syncTick) {
        refreshPages()
    }

    LaunchedEffect(pages.size, scrollToNewest) {
        if (scrollToNewest && pages.isNotEmpty()) {
            listState.animateScrollToItem(pages.size - 1)
            scrollToNewest = false
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Navigation page") },
                actions = { CogButton(onClick = onOpenSettings) }
            )
        }
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .padding(innerPadding)
                .fillMaxSize()
                .padding(16.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            if (conn == ConnState.Lonewolf || conn == ConnState.LonewolfAndOffline) {
                Card(modifier = Modifier.fillMaxWidth().padding(bottom = 12.dp)) {
                    Row(
                        modifier = Modifier.padding(start = 12.dp, top = 4.dp, bottom = 4.dp, end = 4.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Text(
                            "Out of sync with the server. Sync is paused.",
                            modifier = Modifier.weight(1f)
                        )
                        TextButton(onClick = onOpenRecovery) { Text("Fix") }
                    }
                }
            }
            if (!hasLoadedOnce) {
                Box(modifier = Modifier.weight(1f), contentAlignment = Alignment.Center) {
                    CircularProgressIndicator()
                }
            } else {
                LazyColumn(
                    state = listState,
                    modifier = Modifier.weight(1f).fillMaxWidth()
                ) {
                    itemsIndexed(pages, key = { _, info -> info.pageId }) { index, info ->
                        NavRow(
                            info = info,
                            showDivider = index < pages.lastIndex,
                            onOpen = { onOpenPage(info.pageId) },
                            // Eye = hide (disablePage; data stays). Runs after the drop
                            // animation. true = row can go, false = row springs back.
                            onHide = {
                                try {
                                    disablePage(info.pageId).fold(
                                        onSuccess = {
                                            refreshPages()
                                            true
                                        },
                                        onFailure = { error ->
                                            GlobalPopupManager.showError("Hide failed: ${error.message}")
                                            false
                                        }
                                    )
                                } catch (e: CancellationException) {
                                    throw e
                                } catch (e: Exception) {
                                    GlobalPopupManager.showError("Hide failed: ${e.message}")
                                    false
                                }
                            }
                        )
                    }

                    if (pages.isEmpty()) {
                        item {
                            Text(
                                "No pages found",
                                style = MaterialTheme.typography.bodyMedium,
                                modifier = Modifier.padding(top = 16.dp)
                            )
                        }
                    }
                }
            }

            Spacer(modifier = Modifier.height(16.dp))

            Button(
                onClick = {
                    newPageError = null
                    showNewPagePopup = true
                },
                modifier = Modifier.fillMaxWidth().padding(horizontal = 32.dp)
            ) {
                Text("+ New Page")
            }

            // Recovery / Database Inspector / Dev Tools live in SettingsPage (cog in the top bar).
        }
    }

    if (showNewPagePopup) {
        NewPagePopup(
            onDismiss = {
                if (!actionInFlight) {
                    newPageError = null
                    showNewPagePopup = false
                }
            },
            errorMessage = newPageError,
            isSubmitting = actionInFlight,
            onClearError = { newPageError = null },
            onConfirm = { name ->
                if (!actionInFlight) {
                    newPageError = null
                    actionInFlight = true
                    coroutineScope.launch {
                        try {
                            DbManager.awaitReady()
                            createNewPage(name)
                                .onSuccess {
                                    newPageError = null
                                    showNewPagePopup = false
                                    refreshPages { scrollToNewest = true }
                                }
                                .onFailure { error ->
                                    val msg = "Error creating page: ${error.message}"
                                    newPageError = msg
                                    GlobalPopupManager.showError(msg)
                                }
                        } catch (e: Exception) {
                            val msg = "Error creating page: ${e.message}"
                            newPageError = msg
                            GlobalPopupManager.showError(msg)
                        } finally {
                            actionInFlight = false
                        }
                    }
                }
            }
        )
    }
}