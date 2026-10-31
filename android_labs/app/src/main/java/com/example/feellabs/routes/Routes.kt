package com.example.feellabs.routes

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue

sealed interface Route {
    data object PageList : Route
    data class Page(val id: Long) : Route
    data object LabPicker : Route
    data class Lab(val id: String) : Route
}

class RouteHolder(start: Route) {
    var current by mutableStateOf(start)
        private set

    private val backStack = mutableStateListOf<Route>()

    fun navigate(to: Route) {
        backStack.add(current)
        current = to
    }

    fun back(): Boolean {
        if (backStack.isEmpty()) return false
        current = backStack.removeAt(backStack.lastIndex)
        return true
    }
}