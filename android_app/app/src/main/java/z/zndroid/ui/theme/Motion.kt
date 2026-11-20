package z.zndroid.ui.theme

import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.animation.core.Easing

// Ported from android_labs theme/FeelTheme.kt. Mirrors the lab's Motion object.
object Motion {
    val easeOut: Easing = CubicBezierEasing(0.22f, 1f, 0.36f, 1f)
    val easeSpring: Easing = CubicBezierEasing(0.34f, 1.56f, 0.64f, 1f)
    val easeFall: Easing = CubicBezierEasing(0.11f, 0f, 0.5f, 0f)
    const val DUR_FAST = 120
    const val DUR_MED = 220

    fun fallDuration(distanceDp: Float): Int {
        val seconds = kotlin.math.sqrt((2.0 * distanceDp / 1500.0).coerceAtLeast(0.0))
        return (seconds * 1000).toInt().coerceAtLeast(1)
    }
}
