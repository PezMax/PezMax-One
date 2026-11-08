package io.github.pezmax.one.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.remember
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color
import io.github.pezmax.one.ServiceLocator
import io.github.takahashirinta.kanesumi.core.theme.MetroColors
import io.github.takahashirinta.kanesumi.core.theme.MetroTheme

/**
 * PezMax 的 Kanesumi 主题。
 *
 * 颜色只有一套语义键（Kanesumi 的 MetroColors），强调色按 D11 的 5 色预设，
 * 默认钴蓝；Windows 端跟随系统强调色，Android 保留预设（D11）。
 * 深浅色由设置决定，不跟 `isSystemInDarkTheme` 之外的自定义逻辑。
 */
@Composable
fun PezMaxTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    accent: String = ServiceLocator.settings.accent,
    content: @Composable () -> Unit,
) {
    val colors = remember(darkTheme, accent) { pezMaxColors(darkTheme, accent) }
    MetroTheme(colors = colors) {
        CompositionLocalProvider(LocalPezMaxAccent provides accentColor(accent)) {
            content()
        }
    }
}

/** 当前强调色；图表 / 选中态之外的零散着色点用它，不重新查设置。 */
val LocalPezMaxAccent = staticCompositionLocalOf { AccentPresets.first().second }

/** 强调色预设 id 与色值（D11）。 */
val AccentPresets: List<Pair<String, Color>> = listOf(
    "cobalt" to Color(0xFF3B82F6),
    "spruce" to Color(0xFF1DB954),
    "crimson" to Color(0xFFEF4444),
    "amber" to Color(0xFFF59E0B),
    "violet" to Color(0xFF8B5CF6),
)

fun accentColor(id: String): Color = AccentPresets.firstOrNull { it.first == id }?.second ?: AccentPresets.first().second

fun pezMaxColors(dark: Boolean, accent: String): MetroColors {
    val primary = accentColor(accent)
    return if (dark) {
        MetroColors(
            background = Color(0xFF0B0D10),
            surface = Color(0xFF14171C),
            surfaceVariant = Color(0xFF1D2127),
            divider = Color(0xFF2A2F36),
            primary = primary,
            onPrimary = Color(0xFFFFFFFF),
            onBackground = Color(0xFFF2F4F6),
            onSurface = Color(0xFFF2F4F6),
            onSurfaceVariant = Color(0xFF9AA3AD),
            pressTint = Color(0x22FFFFFF),
        )
    } else {
        MetroColors(
            background = Color(0xFFF7F8FA),
            surface = Color(0xFFFFFFFF),
            surfaceVariant = Color(0xFFEDEFF3),
            divider = Color(0xFFD8DDE4),
            primary = primary,
            onPrimary = Color(0xFFFFFFFF),
            onBackground = Color(0xFF111417),
            onSurface = Color(0xFF111417),
            onSurfaceVariant = Color(0xFF5B646E),
            pressTint = Color(0x1A000000),
        )
    }
}
