package z.zndroid.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import z.zndroid.sync.owner_of_interacting_with_server.ConnState
import z.zndroid.sync.owner_of_interacting_with_server.connState

@Composable
fun MainContainer(
    modifier: Modifier = Modifier,
    content: @Composable () -> Unit
) {
    Surface(
        modifier = modifier.fillMaxSize(),
        color = MaterialTheme.colorScheme.background
    ) {
        Column(modifier = Modifier.fillMaxSize()) {
            ConnectionStatusBar()
            Box(modifier = Modifier.weight(1f).fillMaxWidth()) {
                content()
                GlobalPopup()
            }
        }
    }
}

/**
 * Thin coloured line at the very top of the app showing the sync state.
 * Deliberately different from web's AppBorder, on request:
 *   Offline, FailedToConnect  gray
 *   Connecting, LoggingIn     yellow
 *   Connected                 green
 *   Lonewolf                  purple
 *   LonewolfAndOffline        blue
 *
 * Web reference: cqrs_state_synchronization/web_interface/src/leptos_components/small_components/app_border.rs
 *
 * No TwoTabs state on Android (no two-tab guard).
 */
@Composable
private fun ConnectionStatusBar() {
    val conn by connState.collectAsState()
    val color = when (conn) {
        ConnState.Offline -> Color(0xFF777777)
        ConnState.FailedToConnect -> Color(0xFF777777)
        ConnState.Connecting -> Color(0xFFCCAA00)
        ConnState.LoggingIn -> Color(0xFFCCAA00)
        ConnState.Connected -> Color(0xFF227722)
        ConnState.Lonewolf -> Color(0xFF6A1B9A)
        ConnState.LonewolfAndOffline -> Color(0xFF2255CC)
    }
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(3.dp)
            .background(color)
    )
}