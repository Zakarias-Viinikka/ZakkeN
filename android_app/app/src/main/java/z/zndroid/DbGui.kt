package z.zndroid

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import z.zndroid.components.GlobalPopupManager
import z.zndroid.ui.theme.ZndroidTheme

@Composable
fun DbGui(
    modifier: Modifier = Modifier,
    onInspectTable: (String) -> Unit = {},
    onViewSchema: () -> Unit = {}
) {
    var tableNames by remember { mutableStateOf(emptyList<String>()) }
    var isLoading by remember { mutableStateOf(true) }

    // Fetch table names from your Rust library on a background thread
    LaunchedEffect(Unit) {
        isLoading = true
        retryUntilReady {
            DbManager.listTables()
        }.onSuccess {
            tableNames = it.tableNames
            isLoading = false
        }.onFailure { error ->
            GlobalPopupManager.show("Error: ${error.message}")
            isLoading = false
        }
    }

    Scaffold(modifier = modifier.fillMaxSize()) { innerPadding ->
        Column(
            modifier = Modifier
                .padding(innerPadding)
                .fillMaxSize()
                .padding(16.dp),
            verticalArrangement = Arrangement.Top,
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = "Database GUI",
                    style = MaterialTheme.typography.headlineMedium
                )
                Button(onClick = onViewSchema) {
                    Text("View Schema")
                }
            }

            Spacer(modifier = Modifier.height(24.dp))

            Text(
                text = "Tables:",
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.align(Alignment.Start)
            )

            Spacer(modifier = Modifier.height(8.dp))

            if (isLoading) {
                Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    CircularProgressIndicator()
                }
            } else {
                // List of buttons for each table
                Column(
                    modifier = Modifier.fillMaxWidth(),
                    verticalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    tableNames.forEach { tableName ->
                        Button(
                            onClick = { onInspectTable(tableName) },
                            modifier = Modifier.fillMaxWidth()
                        ) {
                            Text(text = tableName)
                        }
                    }

                    if (tableNames.isEmpty()) {
                        Text(
                            text = "No tables found",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.outline
                        )
                    }
                }
            }
        }
    }
}

@Preview(showBackground = true)
@Composable
fun DbGuiPreview() {
    ZndroidTheme {
        DbGui()
    }
}
