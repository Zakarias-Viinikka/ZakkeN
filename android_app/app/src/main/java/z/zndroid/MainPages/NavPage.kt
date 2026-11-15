package z.zndroid.MainPages

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import z.zndroid.BuildConfig
import z.zndroid.DbManager
import z.zndroid.actions.createNewPage
import z.zndroid.actions.disablePage
import z.zndroid.actions.renamePage
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
    onOpenAdminView: (String) -> Unit,
    onOpenDbInspector: () -> Unit,
    onOpenTests: () -> Unit,
    onOpenRecovery: () -> Unit
) {
    var pages by remember { mutableStateOf(emptyList<MenuPageRow>()) }
    var hasLoadedOnce by remember { mutableStateOf(false) }
    var showNewPagePopup by remember { mutableStateOf(false) }
    var pageToRename by remember { mutableStateOf<MenuPageRow?>(null) }
    var pageToDelete by remember { mutableStateOf<MenuPageRow?>(null) }
    var renameInputText by remember { mutableStateOf("") }
    var scrollToNewest by remember { mutableStateOf(false) }
    var scrollToPageId by remember { mutableStateOf<String?>(null) }
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

    LaunchedEffect(pages.size, scrollToNewest, scrollToPageId) {
        val targetId = scrollToPageId
        when {
            targetId != null -> {
                val idx = pages.indexOfFirst { it.pageId == targetId }
                if (idx >= 0) {
                    listState.animateScrollToItem(idx)
                    scrollToPageId = null
                }
            }
            scrollToNewest && pages.isNotEmpty() -> {
                listState.animateScrollToItem(pages.size - 1)
                scrollToNewest = false
            }
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(title = { Text("Navigation page") })
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
                    modifier = Modifier.weight(1f).fillMaxWidth(),
                    verticalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    items(pages, key = { it.pageId }) { info ->
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(8.dp)
                        ) {
                            Button(
                                onClick = { onOpenPage(info.pageId) },
                                modifier = Modifier.weight(1f)
                            ) {
                                Text(
                                    text = info.title,
                                    color = if (info.isUntitled) {
                                        LocalContentColor.current.copy(alpha = 0.6f)
                                    } else {
                                        LocalContentColor.current
                                    }
                                )
                            }

                            if (BuildConfig.DEBUG) {
                                TextButton(onClick = { onOpenAdminView(info.pageId) }) {
                                    Text("Admin")
                                }
                            }

                            TextButton(
                                onClick = {
                                    pageToRename = info
                                    renameInputText = if (info.isUntitled) "" else info.title
                                }
                            ) {
                                Text("Rename")
                            }

                            TextButton(onClick = { pageToDelete = info }) {
                                Text("Delete")
                            }
                        }
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
                onClick = { showNewPagePopup = true },
                modifier = Modifier.fillMaxWidth().padding(horizontal = 32.dp)
            ) {
                Text("+ New Page")
            }

            if (BuildConfig.DEBUG) {
                TextButton(onClick = onOpenRecovery) {
                    Text("Recovery")
                }
                TextButton(onClick = onOpenDbInspector) {
                    Text("Database Inspector")
                }
                TextButton(onClick = onOpenTests) {
                    Text("Dev Tools")
                }
            }
        }
    }

    if (showNewPagePopup) {
        NewPagePopup(
            onDismiss = { showNewPagePopup = false },
            onConfirm = { name ->
                if (!actionInFlight) {
                    actionInFlight = true
                    coroutineScope.launch {
                        try {
                            DbManager.awaitReady()
                            createNewPage(name)
                                .onSuccess {
                                    showNewPagePopup = false
                                    refreshPages { scrollToNewest = true }
                                }
                                .onFailure { error ->
                                    GlobalPopupManager.showError("Error creating page: ${error.message}")
                                }
                        } catch (e: Exception) {
                            GlobalPopupManager.showError("Error creating page: ${e.message}")
                        } finally {
                            actionInFlight = false
                        }
                    }
                }
            }
        )
    }

    if (pageToRename != null) {
        val currentInfo = pageToRename!!
        AlertDialog(
            onDismissRequest = { pageToRename = null },
            title = { Text("Rename Page") },
            text = {
                OutlinedTextField(
                    value = renameInputText,
                    onValueChange = { renameInputText = it },
                    label = { Text("New title") },
                    singleLine = true
                )
            },
            confirmButton = {
                TextButton(
                    onClick = {
                        val newTitle = renameInputText.trim()
                        if (newTitle.isNotEmpty() && !actionInFlight) {
                            actionInFlight = true
                            coroutineScope.launch {
                                try {
                                    renamePage(currentInfo.pageId, newTitle)
                                        .onSuccess {
                                            pageToRename = null
                                            refreshPages { scrollToPageId = currentInfo.pageId }
                                        }
                                        .onFailure { error ->
                                            GlobalPopupManager.showError("Rename failed: ${error.message}")
                                        }
                                } catch (e: Exception) {
                                    GlobalPopupManager.showError("Rename failed: ${e.message}")
                                } finally {
                                    actionInFlight = false
                                }
                            }
                        }
                    },
                    enabled = renameInputText.isNotBlank() && !actionInFlight
                ) {
                    Text("Rename")
                }
            },
            dismissButton = {
                TextButton(onClick = { pageToRename = null }) { Text("Cancel") }
            }
        )
    }

    if (pageToDelete != null) {
        val currentInfo = pageToDelete!!
        AlertDialog(
            onDismissRequest = { pageToDelete = null },
            title = { Text("Delete Page") },
            text = { Text("Hide \"${currentInfo.title}\" from the menu? The page stays in the database; blocks are not removed.") },
            confirmButton = {
                TextButton(
                    onClick = {
                        if (!actionInFlight) {
                            actionInFlight = true
                            coroutineScope.launch {
                                try {
                                    disablePage(currentInfo.pageId)
                                        .onSuccess {
                                            pageToDelete = null
                                            refreshPages()
                                        }
                                        .onFailure { error ->
                                            GlobalPopupManager.showError("Delete failed: ${error.message}")
                                        }
                                } catch (e: Exception) {
                                    GlobalPopupManager.showError("Delete failed: ${e.message}")
                                } finally {
                                    actionInFlight = false
                                }
                            }
                        }
                    },
                    enabled = !actionInFlight
                ) {
                    Text("Delete")
                }
            },
            dismissButton = {
                TextButton(onClick = { pageToDelete = null }) { Text("Cancel") }
            }
        )
    }
}