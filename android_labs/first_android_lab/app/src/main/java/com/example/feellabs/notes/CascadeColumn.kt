package com.example.feellabs.notes

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.Composable
import androidx.compose.runtime.key
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.layout.Layout
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalDensity

@Composable
fun CascadeColumn(
    controller: CascadeController,
    blockIds: List<Long>,
    modifier: Modifier = Modifier,
    content: @Composable (Long) -> Unit,
) {
    val density = LocalDensity.current.density
    val heightState = controller.pageHeightState()

    Layout(
        modifier = modifier.fillMaxWidth(),
        content = {
            blockIds.forEach { id ->
                key(id) {
                    val yState = controller.yState(id)
                    Box(
                        Modifier
                            .fillMaxWidth()
                            .graphicsLayer { translationY = yState.floatValue * density }
                            .onSizeChanged { size ->
                                controller.setHeight(id, size.height / density)
                            },
                    ) {
                        content(id)
                    }
                }
            }
        },
        measurePolicy = { measurables, constraints ->
            val placeables = measurables.map { it.measure(constraints) }
            val width = if (constraints.hasBoundedWidth) constraints.maxWidth else 0
            val heightPx = (heightState.floatValue * density).toInt().coerceAtLeast(1)
            layout(width, heightPx) {
                placeables.forEach { it.placeRelative(0, 0) }
            }
        },
    )
}