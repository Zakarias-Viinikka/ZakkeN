package com.example.feellabs.labs

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.key
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
fun LabHost(lab: Lab, onBack: () -> Unit) {
    val colors = LocalFeelColors.current
    Column(
        Modifier.fillMaxSize().background(colors.bg).padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Row(verticalAlignment = androidx.compose.ui.Alignment.CenterVertically) {
            Btn("Back", onClick = onBack)
        }
        MockTitle(lab.label)
        Label(lab.blurb)
        Box(Modifier.weight(1f).fillMaxWidth()) {
        key(lab) {
            when (lab) {
                Lab.BlockTransition -> BlockTransitionLab()
                Lab.RunningIntoWall -> RunningIntoTheWallLab()
                Lab.SearchBar -> SearchBarLab()
                Lab.DeletingBlock -> DeletingBlockLab()
                Lab.NewPagePopup -> NewPagePopupLab()
                Lab.MainMenu -> MainMenuLab()
                Lab.Popups -> PopupsLab()
                Lab.SettingsModal -> SettingsModalLab()
                Lab.BlockCascade -> BlockCascadeLab()
                Lab.DragReorder -> DragReorderLab()
                else -> StubLab("not ported yet: ${lab.label}")
            }
        }
        }
    }
}

@Composable
private fun StubLab(text: String) {
    val colors = LocalFeelColors.current
    MockPage {
        BasicText(
            text,
            style = TextStyle(color = colors.fgDim, fontSize = 15.sp),
        )
    }
}
