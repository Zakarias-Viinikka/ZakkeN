package z.zndroid.ui.theme

import androidx.compose.runtime.Immutable
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color

// Ported from android_labs theme/FeelTheme.kt. Exposed as a CompositionLocal so
// ported lab components keep working unchanged. Theme.kt also maps these onto the
// Material3 colour scheme, so ordinary screens get the same look for free.
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

val DarkFeelColors = FeelColors(
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

val LocalFeelColors = staticCompositionLocalOf { DarkFeelColors }
val LocalReducedMotion = compositionLocalOf { false }
