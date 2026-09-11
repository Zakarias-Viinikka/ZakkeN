package z.zndroid.lab.core

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue

/**
 * Simple state-based router for Lab Experiments.
 */
object LabNavigator {
    sealed class Route {
        object Production : Route()
        object ScrollLab : Route()
    }

    var currentRoute by mutableStateOf<Route>(Route.Production)

    // Configuration states for Production view (toggles)
    var isUndoRedoEnabled by mutableStateOf(false)
    var isSelectionModeEnabled by mutableStateOf(false)
    
    fun navigateTo(route: Route) {
        currentRoute = route
    }
}
