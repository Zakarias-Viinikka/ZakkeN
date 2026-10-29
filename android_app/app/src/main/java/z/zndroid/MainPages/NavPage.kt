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
import z.zndroid.DbManager
import z.zndroid.actions.createNewPage
import z.zndroid.actions.disablePage
import z.zndroid.actions.renamePage
import z.zndroid.components.GlobalPopupManager
import z.zndroid.db.reads.MenuPageRow
import z.zndroid.db.reads.getMenuPages

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun NavPage(
    onOpenPage: (String) -> Unit,
    onOpenAdminView: (String) -> Unit,
    onOpenDbInspector: () -> Unit,
    onOpenTests: () -> Unit
) {
    var pages by remember { mutableStateOf(emptyList<MenuPageRow>()) }
    var isLoading by remember { mutableStateOf(true) }
    var showNewPagePopup by remember { mutableStateOf(false) }
    var pageToRename by remember { mutableStateOf<MenuPageRow?>(null) }
    var pageToDelete by remember { mutableStateOf<MenuPageRow?>(null) }
    var renameInputText by remember { mutableStateOf("") }
    var scrollToNewest by remember { mutableStateOf(false) }
    val listState = rememberLazyListState()
    val coroutineScope = rememberCoroutineScope()

    fun refreshPages() {
        coroutineScope.launch {
            isLoading = true
            DbManager.awaitReady()
            getMenuPages()
                .onSuccess {
                    pages = it
                    isLoading = false
                }
                .onFailure {
                    GlobalPopupManager.showError("Failed to load pages: ${it.message}")
                    isLoading = false
                }
        }
    }

    LaunchedEffect(Unit) {
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
            if (isLoading) {
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

                            TextButton(onClick = { onOpenAdminView(info.pageId) }) {
                                Text("Admin")
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

            TextButton(onClick = onOpenDbInspector) {
                Text("Database Inspector")
            }

            if (z.zndroid.BuildConfig.DEBUG) {
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
                coroutineScope.launch {
                    DbManager.awaitReady()
                    createNewPage(name)
                        .onSuccess {
                            showNewPagePopup = false
                            scrollToNewest = true
                            refreshPages()
                        }
                        .onFailure { error ->
                            GlobalPopupManager.showError("Error creating page: ${error.message}")
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
                        if (newTitle.isNotEmpty()) {
                            coroutineScope.launch {
                                renamePage(currentInfo.pageId, newTitle)
                                    .onSuccess {
                                        pageToRename = null
                                        scrollToNewest = true
                                        refreshPages()
                                    }
                                    .onFailure { error ->
                                        GlobalPopupManager.showError("Rename failed: ${error.message}")
                                    }
                            }
                        }
                    },
                    enabled = renameInputText.isNotBlank()
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
            text = { Text("Hide \"${currentInfo.titleText()}\" from the menu? The page stays in the database; blocks are not removed.") },
            confirmButton = {
                TextButton(onClick = {
                    coroutineScope.launch {
                        disablePage(currentInfo.pageId)
                            .onSuccess {
                                pageToDelete = null
                                refreshPages()
                            }
                            .onFailure { error ->
                                GlobalPopupManager.showError("Delete failed: ${error.message}")
                            }
                    }
                }) {
                    Text("Delete")
                }
            },
            dismissButton = {
                TextButton(onClick = { pageToDelete = null }) { Text("Cancel") }
            }
        )
    }
}

private fun MenuPageRow.titleText(): String = title