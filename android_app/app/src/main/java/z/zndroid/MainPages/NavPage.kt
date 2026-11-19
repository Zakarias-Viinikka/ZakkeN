package z.zndroid.MainPages

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import z.zndroid.BuildConfig
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
    onOpenDbInspector: () -> Unit,
    onOpenTests: () -> Unit,
    onOpenRecovery: () -> Unit
) {
    var pages by remember { mutableStateOf(emptyList<MenuPageRow>()) }
    var hasLoadedOnce by remember { mutableStateOf(false) }
    var showNewPagePopup by remember { mutableStateOf(false) }
    var newPageError by remember { mutableStateOf<String?>(null) }
    var pageToDelete by remember { mutableStateOf<MenuPageRow?>(null) }
    var deletePageError by remember { mutableStateOf<String?>(null) }
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
                    modifier = Modifier.weight(1f).fillMaxWidth()
                ) {
                    itemsIndexed(pages, key = { _, info -> info.pageId }) { index, info ->
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(vertical = 4.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Box(
                                modifier = Modifier
                                    .weight(1f)
                                    .clickable(role = Role.Button) { onOpenPage(info.pageId) }
                                    .padding(vertical = 10.dp, horizontal = 4.dp)
                            ) {
                                Text(
                                    text = info.title,
                                    style = MaterialTheme.typography.bodyLarge,
                                    color = if (info.isUntitled) {
                                        MaterialTheme.colorScheme.onSurface.copy(alpha = 0.5f)
                                    } else {
                                        MaterialTheme.colorScheme.onSurface
                                    }
                                )
                            }

                            TextButton(onClick = {
                                deletePageError = null
                                pageToDelete = info
                            }) {
                                Text("X")
                            }
                        }
                        if (index < pages.lastIndex) {
                            HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
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
                onClick = {
                    newPageError = null
                    showNewPagePopup = true
                },
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

    if (pageToDelete != null) {
        val currentInfo = pageToDelete!!
        AlertDialog(
            onDismissRequest = {
                if (!actionInFlight) {
                    deletePageError = null
                    pageToDelete = null
                }
            },
            title = { Text("Delete Page") },
            text = {
                Column {
                    Text("Hide \"${currentInfo.title}\" from the menu? The page stays in the database; blocks are not removed.")
                    if (deletePageError != null) {
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(
                            text = deletePageError!!,
                            color = MaterialTheme.colorScheme.error,
                            style = MaterialTheme.typography.bodySmall
                        )
                    }
                }
            },
            confirmButton = {
                TextButton(
                    onClick = {
                        if (!actionInFlight) {
                            deletePageError = null
                            actionInFlight = true
                            coroutineScope.launch {
                                try {
                                    disablePage(currentInfo.pageId)
                                        .onSuccess {
                                            deletePageError = null
                                            pageToDelete = null
                                            refreshPages()
                                        }
                                        .onFailure { error ->
                                            val msg = "Delete failed: ${error.message}"
                                            deletePageError = msg
                                            GlobalPopupManager.showError(msg)
                                        }
                                } catch (e: Exception) {
                                    val msg = "Delete failed: ${e.message}"
                                    deletePageError = msg
                                    GlobalPopupManager.showError(msg)
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
                TextButton(
                    onClick = {
                        deletePageError = null
                        pageToDelete = null
                    },
                    enabled = !actionInFlight
                ) { Text("Cancel") }
            }
        )
    }
}