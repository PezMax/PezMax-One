// PezMax egui Desktop — 主题系统
// Metro Design 风格：扁平、大字体、内容优先、色块分区

use egui::{Color32, FontFamily, FontId, Stroke, Style, TextStyle, Vec2};

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
    pub const OVERLAY: Color32 = Color32::from_black_alpha(128);
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

/// 应用 Metro Design 主题到 egui 上下文
pub fn apply_metro_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style.text_styles = metro_text_styles();
    style.spacing.item_spacing = Vec2::new(12.0, 8.0);
    style.spacing.button_padding = Vec2::new(16.0, 8.0);
    style.spacing.indent = 24.0;

    // 圆角
    style.visuals.window_rounding = 8.0.into();
    style.visuals.widgets.noninteractive.rounding = 6.0.into();
    style.visuals.widgets.inactive.rounding = 6.0.into();
    style.visuals.widgets.hovered.rounding = 6.0.into();
    style.visuals.widgets.active.rounding = 6.0.into();

    // 颜色
    style.visuals.override_text_color = Some(colors::TEXT_PRIMARY);
    style.visuals.window_fill = colors::BG_WHITE;
    style.visuals.panel_fill = colors::BG_WHITE;
    style.visuals.faint_bg_color = colors::BG_CARD;
    style.visuals.extreme_bg_color = colors::BG_SIDEBAR;

    // 按钮
    style.visuals.widgets.inactive.bg_fill = colors::PRIMARY;
    style.visuals.widgets.inactive.weak_bg_fill = colors::BG_CARD;
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, colors::BORDER);
    style.visuals.widgets.hovered.bg_fill = colors::PRIMARY_LIGHT;
    style.visuals.widgets.active.bg_fill = colors::PRIMARY_DARK;

    // 选择
    style.visuals.selection.stroke = Stroke::new(1.0, colors::PRIMARY);
    style.visuals.selection.bg_fill = colors::BG_SELECTED;

    // 超链接
    style.visuals.hyperlink_color = colors::PRIMARY;

    ctx.set_style(style);
}