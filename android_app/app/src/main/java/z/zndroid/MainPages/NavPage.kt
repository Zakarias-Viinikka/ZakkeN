package z.zndroid.MainPages

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import uniffi.protocol.*
import z.zndroid.DbManager
import z.zndroid.components.GlobalPopupManager

data class NavPageInfo(
    val pageId: String,
    val titleText: String,
    val isUntitled: Boolean
)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun NavPage(
    onOpenPage: (String) -> Unit,
    onOpenAdminView: (String) -> Unit,
    onOpenDbInspector: () -> Unit,
    onOpenTests: () -> Unit
) {
    var pageInfos by remember { mutableStateOf(emptyList<NavPageInfo>()) }
    var isLoading by remember { mutableStateOf(true) }
    var showNewPagePopup by remember { mutableStateOf(false) }
    var pageToRename by remember { mutableStateOf<NavPageInfo?>(null) }
    var pageToDelete by remember { mutableStateOf<NavPageInfo?>(null) }
    var renameInputText by remember { mutableStateOf("") }
    val coroutineScope = rememberCoroutineScope()

    fun refreshPages() {
        coroutineScope.launch {
            isLoading = true
            DbManager.awaitReady()

            val pagesRes = DbManager.getPages()
            if (pagesRes.isFailure) {
                GlobalPopupManager.showError("Failed to load pages: ${pagesRes.exceptionOrNull()?.message}")
                isLoading = false
                return@launch
            }

            val pageRows = pagesRes.getOrNull() ?: emptyList()

            // Fetch title blocks from every_block_in_existence
            val titleBlocksData = DbManager.getData(GetDataIn(
                tableName = "every_block_in_existence",
                arguments = SelectArguments.Single(SelectArgument.XEqualY("is_title", "true")),
                columnsToRead = emptyList()
            )).getOrNull()

            val titlesByPageId = mutableMapOf<String, String>()
            if (titleBlocksData != null) {
                val checkOut = DbManager.checkTable(CheckTableIn("every_block_in_existence")).getOrNull()
                val pageIdColIdx = checkOut?.columns?.indexOfFirst { it.name == "id_of_page_i_belong_to" } ?: 5
                val contentColIdx = checkOut?.columns?.indexOfFirst { it.name == "content" } ?: 3

                titleBlocksData.rows.forEach { row ->
                    val pid = (row.cols.getOrNull(pageIdColIdx) as? Col.Text)?.v1
                    val content = (row.cols.getOrNull(contentColIdx) as? Col.Text)?.v1
                    if (pid != null && content != null) {
                        titlesByPageId[pid] = content
                    }
                }
            }

            // Resolve page_id column index in pages table
            val pagesCheck = DbManager.checkTable(CheckTableIn("pages")).getOrNull()
            val pageIdIdx = pagesCheck?.columns?.indexOfFirst { it.name == "page_id" } ?: 1

            pageInfos = pageRows.map { row ->
                val pid = (row.cols.getOrNull(pageIdIdx) as? Col.Text)?.v1 ?: ""
                val rawTitle = titlesByPageId[pid] ?: ""
                val isUntitled = rawTitle.isBlank()
                val displayTitle = if (isUntitled) "Untitled" else rawTitle
                NavPageInfo(pageId = pid, titleText = displayTitle, isUntitled = isUntitled)
            }

            isLoading = false
        }
    }

    LaunchedEffect(Unit) {
        refreshPages()
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Navigation page") }
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
            if (isLoading) {
                Box(modifier = Modifier.weight(1f), contentAlignment = Alignment.Center) {
                    CircularProgressIndicator()
                }
            } else {
                LazyColumn(
                    modifier = Modifier.weight(1f).fillMaxWidth(),
                    verticalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    items(pageInfos) { info ->
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
                                    text = info.titleText,
                                    color = if (info.isUntitled) {
                                        LocalContentColor.current.copy(alpha = 0.6f)
                                    } else {
                                        LocalContentColor.current
                                    }
                                )
                            }

                            TextButton(
                                onClick = { onOpenAdminView(info.pageId) }
                            ) {
                                Text("Admin")
                            }

                            TextButton(
                                onClick = {
                                    pageToRename = info
                                    renameInputText = if (info.isUntitled) "" else info.titleText
                                }
                            ) {
                                Text("Rename")
                            }

                            TextButton(
                                onClick = {
                                    pageToDelete = info
                                }
                            ) {
                                Text("Delete")
                            }
                        }
                    }
                    
                    if (pageInfos.isEmpty()) {
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

            TextButton(onClick = onOpenTests) {
                Text("Internal Test Suite")
            }
        }
    }

    if (showNewPagePopup) {
        NewPagePopup(
            onDismiss = { showNewPagePopup = false },
            onConfirm = { name ->
                coroutineScope.launch {
                    DbManager.awaitReady()
                    DbManager.addPage(name).onSuccess {
                        showNewPagePopup = false
                        refreshPages()
                    }.onFailure { error ->
                        GlobalPopupManager.showError("Error: ${error.message}")
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
                                DbManager.renamePage(currentInfo.pageId, newTitle).onSuccess {
                                    pageToRename = null
                                    refreshPages()
                                }.onFailure { error ->
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
            text = { Text("Delete \"${currentInfo.titleText}\" and all its blocks? This cannot be undone.") },
            confirmButton = {
                TextButton(onClick = {
                    coroutineScope.launch {
                        DbManager.deletePage(currentInfo.pageId).onSuccess {
                            pageToDelete = null
                            refreshPages()
                        }.onFailure { error ->
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
