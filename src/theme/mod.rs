// PezMax egui Desktop — 主题系统
// Metro Design 风格：扁平、大字体、内容优先、色块分区

use egui::{FontFamily, FontId, CornerRadius, TextStyle, Vec2};
use std::sync::Arc;

/// Metro Design 调色板
pub mod colors {
    use egui::Color32;

    // 主色 — 深蓝/紫（Windows 11 风格）
    pub const PRIMARY: Color32 = Color32::from_rgb(0x00, 0x78, 0xD4);
    pub const PRIMARY_DARK: Color32 = Color32::from_rgb(0x00, 0x5A, 0x9E);
    pub const PRIMARY_LIGHT: Color32 = Color32::from_rgb(0x4B, 0xA3, 0xE0);

    // 强调色 — 类似 Metro 的橙色/青色 tile
    pub const ACCENT_ORANGE: Color32 = Color32::from_rgb(0xF7, 0x63, 0x00);
    pub const ACCENT_GREEN: Color32 = Color32::from_rgb(0x00, 0xBC, 0x70);
    pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(0x88, 0x44, 0xAA);
    pub const ACCENT_TEAL: Color32 = Color32::from_rgb(0x00, 0xB7, 0xC3);

    // 文本色
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0x1A, 0x1A, 0x2E);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(0x60, 0x60, 0x70);
    pub const TEXT_ON_PRIMARY: Color32 = Color32::WHITE;

    // 背景色
    pub const BG_WHITE: Color32 = Color32::from_rgb(0xFA, 0xFA, 0xFA);
    pub const BG_CARD: Color32 = Color32::WHITE;
    pub const BG_SIDEBAR: Color32 = Color32::from_rgb(0x1E, 0x1E, 0x2E);
    pub const BG_HOVER: Color32 = Color32::from_rgb(0xE8, 0xE8, 0xF0);
    pub const BG_SELECTED: Color32 = Color32::from_rgb(0xD0, 0xD0, 0xE8);

    // 边框
    pub const BORDER: Color32 = Color32::from_rgb(0xE0, 0xE0, 0xE0);

    // 状态色
    pub const SUCCESS: Color32 = Color32::from_rgb(0x00, 0xBC, 0x70);
    pub const WARNING: Color32 = Color32::from_rgb(0xF7, 0x63, 0x00);
    pub const ERROR: Color32 = Color32::from_rgb(0xE8, 0x11, 0x23);
    pub const INFO: Color32 = Color32::from_rgb(0x00, 0x78, 0xD4);

    // 半透明叠加
    pub const OVERLAY: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 128);
}

/// 获取 Metro Design 风格的字体大小体系
pub fn metro_text_styles() -> std::collections::BTreeMap<TextStyle, FontId> {
    use TextStyle::*;
    [
        (Heading, FontId::new(28.0, FontFamily::Proportional)),
        (Name("h1".into()), FontId::new(24.0, FontFamily::Proportional)),
        (Name("h2".into()), FontId::new(20.0, FontFamily::Proportional)),
        (Name("h3".into()), FontId::new(16.0, FontFamily::Proportional)),
        (Body, FontId::new(14.0, FontFamily::Proportional)),
        (Monospace, FontId::new(13.0, FontFamily::Monospace)),
        (Button, FontId::new(14.0, FontFamily::Proportional)),
        (Small, FontId::new(12.0, FontFamily::Proportional)),
    ]
    .into()
}

/// 加载系统 CJK 字体作为 fallback，修复中文方框乱码
pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 按平台优先级尝试常见 CJK 字体路径
    let candidates: &[&str] = &[
        // Linux — Noto CJK（首选，字形最全）
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        // Linux — WenQuanYi
        "/usr/share/fonts/wenquanyi/wqy-microhei/wqy-microhei.ttc",
        "/usr/share/fonts/wqy-microhei/wqy-microhei.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
        // Windows — 微软雅黑
        "C:\\Windows\\Fonts\\msyh.ttc",
        "C:\\Windows\\Fonts\\simsun.ttc",
        "C:\\Windows\\Fonts\\simhei.ttf",
        // macOS
        "/System/Library/Fonts/PingFang.ttc",
        "/Library/Fonts/Arial Unicode.ttf",
    ];

    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            fonts
                .font_data
                .insert("system_cjk".to_owned(), Arc::new(egui::FontData::from_owned(bytes)));
            // 追加到末尾作为 fallback：Latin 字符用默认字体，CJK 字符降级到此字体
            fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .push("system_cjk".to_owned());
            fonts
                .families
                .entry(FontFamily::Monospace)
                .or_default()
                .push("system_cjk".to_owned());
            log::info!("Loaded CJK font from: {}", path);
            break;
        }
    }

    ctx.set_fonts(fonts);
}

/// 应用 Metro Design 主题到 egui 上下文
pub fn apply_metro_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style.text_styles = metro_text_styles();
    style.spacing.item_spacing = Vec2::new(12.0, 8.0);
    style.spacing.button_padding = Vec2::new(16.0, 8.0);
    style.spacing.indent = 24.0;

    // Metro Design: 全局直角，无圆角
    let zero = CornerRadius::same(0);
    style.visuals.window_corner_radius = zero;
    style.visuals.menu_corner_radius = zero;
    style.visuals.widgets.noninteractive.corner_radius = zero;
    style.visuals.widgets.inactive.corner_radius = zero;
    style.visuals.widgets.hovered.corner_radius = zero;
    style.visuals.widgets.active.corner_radius = zero;
    style.visuals.widgets.open.corner_radius = zero;

    // 颜色
    style.visuals.override_text_color = Some(colors::TEXT_PRIMARY);
    style.visuals.window_fill = colors::BG_WHITE;
    style.visuals.panel_fill = colors::BG_WHITE;
    style.visuals.faint_bg_color = colors::BG_CARD;
    // TextEdit/ComboBox 背景：白色（避免用深色侧边栏色导致输入框黑底）
    style.visuals.extreme_bg_color = colors::BG_CARD;

    // 超链接
    style.visuals.hyperlink_color = colors::PRIMARY;

    // 选择色
    style.visuals.selection.stroke = egui::Stroke { width: 1.0, color: colors::PRIMARY };
    style.visuals.selection.bg_fill = colors::BG_SELECTED;

    ctx.set_style(style);
}