package z.zndroid.cascade

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.Composable
import androidx.compose.runtime.key
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.layout.Layout
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalDensity

// Ported verbatim from android_labs notes/CascadeColumn.kt (only the package line changed).
// Not a lazy list: every block is composed. Each block is stacked at (0,0) and moved with
// translationY from the physics engine; total height comes from the engine's page rect.
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
            val childConstraints = constraints.copy(minHeight = 0, maxHeight = constraints.maxHeight)
            val placeables = measurables.map { it.measure(childConstraints) }
            val width = if (constraints.hasBoundedWidth) constraints.maxWidth else 0
            val heightPx = (heightState.floatValue * density).toInt().coerceAtLeast(1)
            layout(width, heightPx) {
                placeables.forEach { it.placeRelative(0, 0) }
            }
        },
    )
}
