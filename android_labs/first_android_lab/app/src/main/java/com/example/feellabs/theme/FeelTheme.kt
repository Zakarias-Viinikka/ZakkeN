package com.example.feellabs.theme

import android.provider.Settings
import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.animation.core.Easing
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext

@Immutable
data class FeelColors(
    val bg: Color,
    val surface1: Color,
    val surface2: Color,
    val surface3: Color,
    val bgDeep: Color,
    val fg: Color,
    val fgDim: Color,
    val fgFaint: Color,
    val accent: Color,
    val accentSoft: Color,
    val accentLine: Color,
    val border: Color,
    val borderStrong: Color,
    val danger: Color,
    val warn: Color,
    val dangerSoft: Color,
    val focusRing: Color,
)

private val darkFeelColors = FeelColors(
    bg = Color(0xFF0F141D),
    surface1 = Color(0xFF151C29),
    surface2 = Color(0xFF1C2433),
    surface3 = Color(0xFF2B3648),
    bgDeep = Color(0xFF0B1020),
    fg = Color(0xFFE4E9F2),
    fgDim = Color(0xFFE4E9F2).copy(alpha = 0.72f),
    fgFaint = Color(0xFFE4E9F2).copy(alpha = 0.42f),
    accent = Color(0xFF8AB4FF),
    accentSoft = Color(0xFF8AB4FF).copy(alpha = 0.14f),
    accentLine = Color(0xFF8AB4FF).copy(alpha = 0.5f),
    border = Color(0xFFE4E9F2).copy(alpha = 0.09f),
    borderStrong = Color(0xFFE4E9F2).copy(alpha = 0.2f),
    danger = Color(0xFFFF6B6B),
    warn = Color(0xFFF2B45A),
    dangerSoft = Color(0xFFFF6B6B).copy(alpha = 0.14f),
    focusRing = Color(0xFF8AB4FF).copy(alpha = 0.45f),
)

val LocalFeelColors = staticCompositionLocalOf { darkFeelColors }

@Immutable
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

val LocalReducedMotion = compositionLocalOf { false }

@Composable
fun FeelTheme(content: @Composable () -> Unit) {
    val context = LocalContext.current
    val scale = Settings.Global.getFloat(
        context.contentResolver,
        Settings.Global.ANIMATOR_DURATION_SCALE,
        1f,
    )
    val reduced = scale == 0f
    CompositionLocalProvider(
        LocalFeelColors provides darkFeelColors,
        LocalReducedMotion provides reduced,
    ) {
        content()
    }
}
