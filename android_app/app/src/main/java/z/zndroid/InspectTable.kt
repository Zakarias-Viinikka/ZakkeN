package z.zndroid

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import uniffi.protocol.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun InspectTable(tableName: String, onBack: () -> Unit) {
    // 1. Load Columns and Rows
    var columns by remember { mutableStateOf(emptyList<TableColumnInfo>()) }
    var rows by remember { mutableStateOf(emptyList<Row>()) }
    
    // 2. Input state: Map of Column Name -> Text Input
    val inputs = remember { mutableStateMapOf<String, String>() }

    fun refreshData() {
        try {
            columns = DbManager.db.checkTable(CheckTableIn(tableName)).columns
            rows = DbManager.db.getData(GetDataIn(tableName, listOf(SelectArgument.All), emptyList())).rows
        } catch (e: Exception) {
            // Handle error (e.g. table doesn't exist)
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
                    try {
                        val values = columns.filter { !it.primaryKey }.map { col ->
                            val text = inputs[col.name] ?: ""
                            val colValue = when (col.typeName.uppercase()) {
                                "INTEGER" -> Col.Integer(text.toLongOrNull() ?: 0L)
                                "REAL" -> Col.Real(text.toDoubleOrNull() ?: 0.0)
                                else -> Col.Text(text)
                            }
                            ColumnValue(col.name, colValue)
                        }
                        DbManager.db.insertData(InsertDataIn(tableName, values))
                        // Clear inputs
                        inputs.clear()
                        refreshData()
                    } catch (e: Exception) {
                        // Handle error
                    }
                },
                modifier = Modifier.padding(vertical = 8.dp).align(Alignment.End)
            ) {
                Text("Insert Row")
            }

            HorizontalDivider(modifier = Modifier.padding(vertical = 16.dp))

            // --- LIST SECTION ---
            Text("Rows", style = MaterialTheme.typography.titleMedium)
            
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

                            // Delete Button (Assumes first column is the ID for deletion)
                            IconButton(onClick = {
                                try {
                                    val idCol = row.cols.firstOrNull()
                                    val idStr = when (idCol) {
                                        is Col.Integer -> idCol.v1.toString()
                                        is Col.Text -> idCol.v1
                                        else -> ""
                                    }
                                    if (idStr.isNotEmpty()) {
                                        DbManager.db.deleteRow(DeleteRowIn(tableName, idStr))
                                        refreshData()
                                    }
                                } catch (e: Exception) {
                                    // Handle error
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
