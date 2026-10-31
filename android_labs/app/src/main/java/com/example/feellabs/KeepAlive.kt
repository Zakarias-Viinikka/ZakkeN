package com.example.feellabs

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.Layout
import androidx.compose.ui.layout.Measurable
import androidx.compose.ui.layout.MeasurePolicy
import androidx.compose.ui.layout.MeasureResult
import androidx.compose.ui.layout.MeasureScope
import androidx.compose.ui.unit.Constraints

@Composable
fun KeepAlive(
    active: () -> Boolean,
    modifier: Modifier = Modifier,
    content: @Composable () -> Unit,
) {
    val policy = MeasurePolicy { measurables: List<Measurable>, constraints: Constraints ->
        val placeables = measurables.map { it.measure(constraints) }
        val isActive = active()
        if (isActive) {
            val w = placeables.maxOfOrNull { it.width } ?: 0
            val h = placeables.maxOfOrNull { it.height } ?: 0
            layout(w, h) {
                placeables.forEach { it.placeRelative(0, 0) }
            }
        } else {
            layout(0, 0) {
                placeables.forEach { /* place nothing */ }
            }
        }
    }
    Layout(content = content, modifier = modifier, measurePolicy = policy)
}
