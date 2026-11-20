package z.zndroid.ui.theme

import android.provider.Settings
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.platform.LocalContext

// Ported from android_labs theme/FeelTheme.kt: the app always uses the lab's dark
// palette (Zakke's ask: "make the app look more like the lab"). FeelColors also drive
// the Material3 scheme, so components reading MaterialTheme.colorScheme get the lab
// look for free. LocalReducedMotion mirrors the system animator scale.
private val LabColorScheme = darkColorScheme(
    primary = DarkFeelColors.accent,
    onPrimary = DarkFeelColors.bgDeep,
    primaryContainer = DarkFeelColors.accentSoft,
    onPrimaryContainer = DarkFeelColors.accent,

    secondary = DarkFeelColors.accent,
    onSecondary = DarkFeelColors.bgDeep,
    secondaryContainer = DarkFeelColors.accentSoft,
    onSecondaryContainer = DarkFeelColors.accent,

    tertiary = DarkFeelColors.accent,
    onTertiary = DarkFeelColors.bgDeep,

    background = DarkFeelColors.bg,
    onBackground = DarkFeelColors.fg,

    surface = DarkFeelColors.surface1,
    onSurface = DarkFeelColors.fg,
    surfaceVariant = DarkFeelColors.surface2,
    onSurfaceVariant = DarkFeelColors.fgDim,

    outline = DarkFeelColors.borderStrong,
    outlineVariant = DarkFeelColors.border,

    error = DarkFeelColors.danger,
    onError = DarkFeelColors.bgDeep,
    errorContainer = DarkFeelColors.dangerSoft,
    onErrorContainer = DarkFeelColors.danger,

    scrim = DarkFeelColors.bgDeep,
)

@Composable
fun ZndroidTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    dynamicColor: Boolean = false,
    content: @Composable () -> Unit
) {
    val context = LocalContext.current
    val scale = Settings.Global.getFloat(
        context.contentResolver,
        Settings.Global.ANIMATOR_DURATION_SCALE,
        1f,
    )
    val reduced = scale == 0f
    CompositionLocalProvider(
        LocalFeelColors provides DarkFeelColors,
        LocalReducedMotion provides reduced,
    ) {
        MaterialTheme(
            colorScheme = LabColorScheme,
            typography = Typography,
            content = content
        )
    }
}