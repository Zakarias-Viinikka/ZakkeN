package z.zndroid.MainPages

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.platform.LocalContext
import kotlinx.coroutines.launch
import uniffi.protocol.Col
import uniffi.protocol.Row
import z.zndroid.DbManager
import z.zndroid.retryUntilReady
import z.zndroid.components.GlobalPopupManager

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun NavPage(
    onOpenPage: (String) -> Unit,
    onOpenDbInspector: () -> Unit
) {
    var pages by remember { mutableStateOf(emptyList<Row>()) }
    var isLoading by remember { mutableStateOf(true) }
    var showNewPagePopup by remember { mutableStateOf(false) }
    val coroutineScope = rememberCoroutineScope()

    // Fetch table info to know which column is the title
    var titleColumnIndex by remember { mutableIntStateOf(-1) }

    fun refreshPages() {
        coroutineScope.launch {
            isLoading = true
            retryUntilReady {
                DbManager.checkTable(uniffi.protocol.CheckTableIn("pages"))
            }.onSuccess { checkOut ->
                titleColumnIndex = checkOut.columns.indexOfFirst { !it.primaryKey }
                if (titleColumnIndex == -1) titleColumnIndex = 0
                
                retryUntilReady {
                    DbManager.getPages()
                }.onSuccess {
                    pages = it
                    isLoading = false
                }
            }.onFailure { error ->
                GlobalPopupManager.show("Failed to load schema: ${error.message}")
                isLoading = false
            }
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
                    items(pages) { row ->
                        // Try to get title from the identified title column
                        val title = when (val titleCol = row.cols.getOrNull(titleColumnIndex)) {
                            is Col.Text -> titleCol.v1
                            is Col.Integer -> titleCol.v1.toString()
                            else -> "Untitled Page"
                        }
                        
                        Button(
                            onClick = { onOpenPage(title) },
                            modifier = Modifier.fillMaxWidth()
                        ) {
                            Text(title)
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
        }
    }

    if (showNewPagePopup) {
        NewPagePopup(
            onDismiss = { showNewPagePopup = false },
            onConfirm = { name ->
                coroutineScope.launch {
                    retryUntilReady {
                        DbManager.addPage(name)
                    }.onSuccess {
                        showNewPagePopup = false
                        refreshPages()
                    }.onFailure { error ->
                        GlobalPopupManager.show("Error: ${error.message}")
                    }
                }
            }
        )
    }
}
