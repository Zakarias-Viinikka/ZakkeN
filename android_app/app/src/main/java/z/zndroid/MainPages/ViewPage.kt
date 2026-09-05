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
import rustlib.my_yrs_lib.Block
import rustlib.my_yrs_lib.BossOfYrs
import rustlib.my_yrs_lib.docFromSnapshot
import uniffi.protocol.Col
import z.zndroid.DbManager
import z.zndroid.DocEvents.AddBlock
import z.zndroid.DocEvents.AddBlockCtx
import z.zndroid.Storage.StorageAccess
import z.zndroid.components.GlobalPopupManager
import z.zndroid.retryUntilReady

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ViewPage(
    pageTitle: String, // This is the page_id
    onBack: () -> Unit
) {
    var boss by remember { mutableStateOf<BossOfYrs?>(null) }
    var blocks by remember { mutableStateOf(emptyList<Block>()) }
    var isLoading by remember { mutableStateOf(true) }
    val coroutineScope = rememberCoroutineScope()

    fun refreshBlocks() {
        boss?.let {
            try {
                blocks = it.getEntirePage()
            } catch (e: Exception) {
                coroutineScope.launch {
                    GlobalPopupManager.show("Failed to load blocks: ${e.message}")
                }
            }
        }
    }

    LaunchedEffect(pageTitle) {
        isLoading = true
        retryUntilReady {
            DbManager.getPage(pageTitle)
        }.onSuccess { row ->
            val userIdRes = StorageAccess.rummage_in_storage(z.zndroid.Storage.StorageKey.USER_ID)
            val userId = when (userIdRes) {
                is z.zndroid.Storage.RummageResult.StringValue -> userIdRes.value
                else -> {
                    GlobalPopupManager.show("Error: User ID not found in storage")
                    isLoading = false
                    return@onSuccess
                }
            }
            
            // blobbed_page is at index 2 (id:0, page_id:1, blobbed_page:2)
            val blob = (row.cols.getOrNull(2) as? Col.Blob)?.v1
            if (blob != null) {
                try {
                    val newBoss = docFromSnapshot(blob, userId, pageTitle)
                    boss = newBoss
                    blocks = newBoss.getEntirePage()
                } catch (e: Exception) {
                    GlobalPopupManager.show("Failed to instance Yrs Doc: ${e.message}")
                }
            } else {
                GlobalPopupManager.show("Error: Snapshot blob not found for page $pageTitle")
            }
            isLoading = false
        }.onFailure { error ->
            GlobalPopupManager.show("Error loading page: ${error.message}")
            isLoading = false
        }
    }

    // Clean up boss when leaving
    DisposableEffect(Unit) {
        onDispose {
            boss?.destroy()
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(pageTitle) },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Text("←")
                    }
                }
            )
        },
        floatingActionButton = {
            FloatingActionButton(onClick = {
                val currentBoss = boss ?: return@FloatingActionButton
                coroutineScope.launch {
                    AddBlock.execute(AddBlockCtx(
                        boss = currentBoss,
                        content = "New Block at ${System.currentTimeMillis()}"
                    )).onSuccess {
                        refreshBlocks()
                    }
                }
            }) {
                Text("+")
            }
        }
    ) { innerPadding ->
        if (isLoading) {
            Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                CircularProgressIndicator()
            }
        } else {
            LazyColumn(
                modifier = Modifier
                    .padding(innerPadding)
                    .fillMaxSize()
                    .padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                item {
                    Text(
                        text = "Welcome to $pageTitle",
                        style = MaterialTheme.typography.headlineMedium
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        text = "This page is properly initialized with Yrs.",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.secondary
                    )
                    HorizontalDivider(modifier = Modifier.padding(vertical = 16.dp))
                }

                if (blocks.isEmpty()) {
                    item {
                        Card(
                            modifier = Modifier.fillMaxWidth(),
                            colors = CardDefaults.cardColors(
                                containerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f)
                            )
                        ) {
                            Box(
                                modifier = Modifier.padding(32.dp).fillMaxWidth(),
                                contentAlignment = Alignment.Center
                            ) {
                                Text("No blocks yet. Tap + to add one.")
                            }
                        }
                    }
                } else {
                    items(blocks) { block ->
                        BlockCard(block)
                    }
                }
            }
        }
    }
}

@Composable
fun BlockCard(block: Block) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(text = block.text, style = MaterialTheme.typography.bodyLarge)
            if (block.metadata.isNotEmpty()) {
                Text(
                    text = block.metadata,
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.outline
                )
            }
            Text(
                text = "ID: ${block.idInYrs}",
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.outline.copy(alpha = 0.7f)
            )
        }
    }
}
