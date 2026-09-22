package z.zndroid.MainPages.ViewPage

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import rustlib.my_yrs_lib.BossOfYrs
import rustlib.my_yrs_lib.docFromSnapshot
import uniffi.protocol.Col
import z.zndroid.DbManager
import z.zndroid.DocEvents.AddBlock
import z.zndroid.Storage.StorageAccess
import z.zndroid.Storage.StorageKey
import z.zndroid.components.GlobalPopupManager
import androidx.compose.ui.text.TextRange
import z.zndroid.log.ViewPageLogs
import z.zndroid.MainPages.ViewPage.helpers.ViewPageHelper

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ViewPage(
    pageId: String, 
    onBack: () -> Unit,
    onOpenAdminView: () -> Unit
) {
    var boss by remember { mutableStateOf<BossOfYrs?>(null) }
    var uiStates by remember { mutableStateOf(emptyList<BlockUiState>()) }
    var isLoading by remember { mutableStateOf(true) }
    val coroutineScope = rememberCoroutineScope()
    val listState = rememberLazyListState()
    
    // Recovery mechanism: incrementing this forces a full rebuild of the BossOfYrs from DB.
    var reloadToken by remember { mutableStateOf(0) }

    fun updateUI(focusId: String? = null, cursorPos: Int? = null) {
        boss?.let {
            try {
                uiStates = ViewPageHelper.syncUiStates(it, uiStates)
                
                // Handle focus request after state update
                if (focusId != null) {
                    coroutineScope.launch {
                        delay(50) // Wait for Compose to layout
                        uiStates.find { it.blockId == focusId }?.let { target ->
                            target.focusRequester.requestFocus()
                            if (cursorPos != null) {
                                target.textFieldValue = target.textFieldValue.copy(
                                    selection = TextRange(cursorPos)
                                )
                            }
                            
                            val targetIndex = uiStates.indexOf(target)
                            if (targetIndex != -1) {
                                launch {
                                    listState.animateScrollToItem(targetIndex)
                                }
                            }
                        }
                    }
                }
            } catch (e: Exception) {
                ViewPageLogs.logDocEventError("updateUI", e.message ?: "Unknown error")
                coroutineScope.launch {
                    GlobalPopupManager.show("Failed to load blocks: ${e.message}")
                }
            }
        }
    }

    LaunchedEffect(pageId, reloadToken) {
        ViewPageLogs.logPageInit(pageId)
        // Update current document in both FastStorage and KeyValueStorage
        StorageAccess.setValue(StorageKey.CURRENT_DOCUMENT, pageId)

        isLoading = true
        // Clean up old boss if this is a reload
        boss?.destroy()
        boss = null
        
        DbManager.awaitReady()
        
        DbManager.getPage(pageId).onSuccess { row ->
            val userIdRes = StorageAccess.rummage_in_storage(StorageKey.USER_ID)
            val userId = when (userIdRes) {
                is z.zndroid.Storage.RummageResult.StringValue -> userIdRes.value
                else -> {
                    ViewPageLogs.logPageLoadError(pageId, "User ID not found in storage")
                    GlobalPopupManager.show("Error: User ID not found in storage")
                    isLoading = false
                    return@onSuccess
                }
            }
            
            // blobbed_page is at index 2 (id:0, page_id:1, blobbed_page:2)
            val blob = (row.cols.getOrNull(2) as? Col.Blob)?.v1
            if (blob != null) {
                try {
                    val newBoss = docFromSnapshot(blob, userId, pageId)
                    boss = newBoss
                    uiStates = ViewPageHelper.syncUiStates(newBoss, emptyList())
                    ViewPageLogs.logPageLoadSuccess(pageId, uiStates.size)
                    
                    // Logic: Ensure a new page has a title block
                    ViewPageHelper.initializePageContent(newBoss, coroutineScope) { newId ->
                        if (newId != null) {
                            updateUI(focusId = newId)
                        }
                    }
                } catch (e: Exception) {
                    ViewPageLogs.logBossInstanceError(pageId, e.message ?: "Unknown error")
                    GlobalPopupManager.show("Failed to instance Yrs Doc: ${e.message}")
                }
            } else {
                ViewPageLogs.logPageLoadError(pageId, "Snapshot blob not found")
                GlobalPopupManager.show("Error: Snapshot blob not found for page $pageId")
            }
            isLoading = false
        }.onFailure { error ->
            ViewPageLogs.logPageLoadError(pageId, error.message ?: "Unknown error")
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
                title = { Text(pageId) },
                navigationIcon = {
                    IconButton(onClick = onBack) {
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
        if (isLoading) {
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
                if (uiStates.isNotEmpty()) {
                    itemsIndexed(uiStates, key = { _, state -> state.blockId }) { index, state ->
                        EditableBlock(
                            state = state,
                            index = index,
                            boss = boss!!,
                            scope = coroutineScope,
                            onRefreshWithFocus = { id, pos ->
                                updateUI(focusId = id, cursorPos = pos)
                            },
                            onHardReload = { reloadToken++ }
                        )
                    }
                }
            }
        }
    }
}
