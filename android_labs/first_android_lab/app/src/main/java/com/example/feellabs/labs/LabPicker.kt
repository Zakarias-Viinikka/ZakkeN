package com.example.feellabs.labs

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.example.feellabs.theme.Btn
import com.example.feellabs.theme.Label
import com.example.feellabs.theme.LocalFeelColors
import com.example.feellabs.theme.MockPage
import com.example.feellabs.theme.MockTitle

@Composable
fun LabPicker(onOpen: (Lab) -> Unit, onBack: () -> Unit) {
    val colors = LocalFeelColors.current
    Column(
        Modifier.fillMaxSize().background(colors.bg)
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        Row(verticalAlignment = androidx.compose.ui.Alignment.CenterVertically) {
            Btn("Back", onClick = onBack)
            Spacer(Modifier.padding(horizontal = 4.dp))
            MockTitle("Labs")
        }
        for ((group, labs) in Lab.grouped()) {
            if (labs.isEmpty()) continue
            MockPage {
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Label(group.label)
                    for (lab in labs) {
                        Btn(lab.label, onClick = { onOpen(lab) })
                    }
                }
            }
        }
    }
}
