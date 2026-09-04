package z.zndroid

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
import z.zndroid.components.GlobalPopupManager

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun InspectTable(tableName: String, onBack: () -> Unit) {
    // 1. State
    var columns by remember { mutableStateOf(emptyList<TableColumnInfo>()) }
    var rows by remember { mutableStateOf(emptyList<Row>()) }
    var isLoading by remember { mutableStateOf(true) }
    val coroutineScope = rememberCoroutineScope()
    
    // 2. Input state: Map of Column Name -> Text Input
    val inputs = remember { mutableStateMapOf<String, String>() }

    fun refreshData() {
        coroutineScope.launch {
            isLoading = true
            retryUntilReady {
                DbManager.checkTable(CheckTableIn(tableName))
            }.onSuccess { checkOut ->
                columns = checkOut.columns
                retryUntilReady {
                    DbManager.getData(GetDataIn(tableName, listOf(SelectArgument.All), emptyList()))
                }.onSuccess { dataOut ->
                    rows = dataOut.rows
                }.onFailure { error ->
                    GlobalPopupManager.show("Error loading data: ${error.message}")
                }
            }.onFailure { error ->
                GlobalPopupManager.show("Error loading table info: ${error.message}")
            }
            isLoading = false
        }
    }

    LaunchedEffect(tableName) {
        refreshData()
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Table: $tableName") },
                navigationIcon = {
                    Button(onClick = onBack) { Text("Back") }
                }
            )
        }
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .padding(innerPadding)
                .fillMaxSize()
                .padding(16.dp)
        ) {
            if (isLoading && rows.isEmpty()) {
                Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    CircularProgressIndicator()
                }
            } else {
                // --- INSERT SECTION ---
                Text("Add New Row", style = MaterialTheme.typography.titleMedium)
                
                columns.filter { !it.primaryKey }.forEach { col ->
                    OutlinedTextField(
                        value = inputs[col.name] ?: "",
                        onValueChange = { inputs[col.name] = it },
                        label = { Text("${col.name} (${col.typeName})") },
                        modifier = Modifier.fillMaxWidth()
                    )
                }

                Button(
                    onClick = {
                        coroutineScope.launch {
                            val values = columns.filter { !it.primaryKey }.map { col ->
                                val text = inputs[col.name] ?: ""
                                val colValue = when (col.typeName.uppercase()) {
                                    "INTEGER" -> Col.Integer(text.toLongOrNull() ?: 0L)
                                    "REAL" -> Col.Real(text.toDoubleOrNull() ?: 0.0)
                                    else -> Col.Text(text)
                                }
                                ColumnValue(col.name, colValue)
                            }
                            retryUntilReady {
                                DbManager.insertData(InsertDataIn(tableName, values))
                            }.onSuccess {
                                // Clear inputs
                                inputs.clear()
                                refreshData()
                            }.onFailure { error ->
                                GlobalPopupManager.show("Insert failed: ${error.message}")
                            }
                        }
                    },
                    modifier = Modifier.padding(vertical = 8.dp).align(Alignment.End)
                ) {
                    Text("Insert Row")
                }

                HorizontalDivider(modifier = Modifier.padding(vertical = 16.dp))

                // --- LIST SECTION ---
                Text("Rows", style = MaterialTheme.typography.titleMedium)
                
                if (isLoading) {
                    LinearProgressIndicator(modifier = Modifier.fillMaxWidth().padding(vertical = 8.dp))
                }

                LazyColumn(modifier = Modifier.fillMaxSize()) {
                    items(rows) { row ->
                        Card(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(vertical = 4.dp)
                        ) {
                            Row(
                                modifier = Modifier.padding(8.dp),
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                Column(modifier = Modifier.weight(1f)) {
                                    row.cols.forEachIndexed { index, col ->
                                        val colName = columns.getOrNull(index)?.name ?: "?"
                                        val valueStr = when (col) {
                                            is Col.Integer -> col.v1.toString()
                                            is Col.Real -> col.v1.toString()
                                            is Col.Text -> col.v1
                                            is Col.Blob -> "Blob(${col.v1.size} bytes)"
                                            Col.Null -> "NULL"
                                        }
                                        Text("$colName: $valueStr", style = MaterialTheme.typography.bodySmall)
                                    }
                                }

                                // Delete Button
                                IconButton(onClick = {
                                    coroutineScope.launch {
                                        val idCol = row.cols.firstOrNull()
                                        val idStr = when (idCol) {
                                            is Col.Integer -> idCol.v1.toString()
                                            is Col.Text -> idCol.v1
                                            else -> ""
                                        }
                                        if (idStr.isNotEmpty()) {
                                            retryUntilReady {
                                                DbManager.deleteRow(DeleteRowIn(tableName, idStr))
                                            }.onSuccess {
                                                refreshData()
                                            }.onFailure { error ->
                                                GlobalPopupManager.show("Delete failed: ${error.message}")
                                            }
                                        }
                                    }
                                }) {
                                    Text("🗑️")
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
