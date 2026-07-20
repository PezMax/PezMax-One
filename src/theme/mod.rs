// PezMax egui Desktop — 主题系统
// Metro Design 风格：扁平、大字体、内容优先、色块分区
// 支持浅色 / 深色模式（thread_local IS_DARK 驱动）

use egui::{FontFamily, FontId, CornerRadius, TextStyle, Vec2};
use std::cell::Cell;
use std::sync::Arc;

// ── 深色模式全局开关 ─────────────────────────────────────────────────────────

thread_local! {
    static IS_DARK: Cell<bool> = const { Cell::new(false) };
}

pub fn set_dark(dark: bool) {
    IS_DARK.with(|d| d.set(dark));
}

pub fn is_dark() -> bool {
    IS_DARK.with(|d| d.get())
}

// ── 调色板（函数化，运行时读取模式）────────────────────────────────────────

pub mod colors {
    use egui::Color32;
    use super::is_dark;

    // ── 主色（固定，两种模式下均可读）──────────────────────────────────────
    pub fn primary()       -> Color32 { Color32::from_rgb(0x00, 0x78, 0xD4) }
    pub fn primary_dark()  -> Color32 { Color32::from_rgb(0x00, 0x5A, 0x9E) }
    pub fn primary_light() -> Color32 { Color32::from_rgb(0x4B, 0xA3, 0xE0) }

    // ── 强调色（固定）───────────────────────────────────────────────────────
    pub fn accent_orange() -> Color32 { Color32::from_rgb(0xF7, 0x63, 0x00) }
    pub fn accent_green()  -> Color32 { Color32::from_rgb(0x00, 0xBC, 0x70) }
    pub fn accent_purple() -> Color32 { Color32::from_rgb(0x88, 0x44, 0xAA) }
    pub fn accent_teal()   -> Color32 { Color32::from_rgb(0x00, 0xB7, 0xC3) }

    // ── 文本色 ──────────────────────────────────────────────────────────────
    pub fn text_primary() -> Color32 {
        if is_dark() { Color32::from_rgb(0xF0, 0xF0, 0xF2) }
        else         { Color32::from_rgb(0x1A, 0x1A, 0x2E) }
    }
    pub fn text_secondary() -> Color32 {
        if is_dark() { Color32::from_rgb(0x9A, 0x9A, 0xAA) }
        else         { Color32::from_rgb(0x60, 0x60, 0x70) }
    }
    pub fn text_on_primary() -> Color32 { Color32::WHITE }

    // ── 背景色 ──────────────────────────────────────────────────────────────
    pub fn bg_white() -> Color32 {
        if is_dark() { Color32::from_rgb(0x1C, 0x1C, 0x1C) }
        else         { Color32::from_rgb(0xFA, 0xFA, 0xFA) }
    }
    pub fn bg_card() -> Color32 {
        if is_dark() { Color32::from_rgb(0x26, 0x26, 0x26) }
        else         { Color32::WHITE }
    }
    pub fn bg_sidebar() -> Color32 {
        if is_dark() { Color32::from_rgb(0x14, 0x14, 0x1E) }
        else         { Color32::from_rgb(0x1E, 0x1E, 0x2E) }
    }
    pub fn bg_hover() -> Color32 {
        if is_dark() { Color32::from_rgb(0x2E, 0x2E, 0x3A) }
        else         { Color32::from_rgb(0xE8, 0xE8, 0xF0) }
    }
    pub fn bg_selected() -> Color32 {
        if is_dark() { Color32::from_rgb(0x35, 0x35, 0x5A) }
        else         { Color32::from_rgb(0xD0, 0xD0, 0xE8) }
    }
    /// 输入框 / 预览区背景
    pub fn bg_input() -> Color32 {
        if is_dark() { Color32::from_rgb(0x20, 0x20, 0x20) }
        else         { Color32::from_rgb(0xF0, 0xF0, 0xF0) }
    }

    // ── 边框 ────────────────────────────────────────────────────────────────
    pub fn border() -> Color32 {
        if is_dark() { Color32::from_rgb(0x3A, 0x3A, 0x3A) }
        else         { Color32::from_rgb(0xE0, 0xE0, 0xE0) }
    }

    // ── 骨架屏占位色 ────────────────────────────────────────────────────────
    pub fn skeleton_base() -> Color32 {
        if is_dark() { Color32::from_gray(50) } else { Color32::from_gray(220) }
    }
    pub fn skeleton_line() -> Color32 {
        if is_dark() { Color32::from_gray(43) } else { Color32::from_gray(215) }
    }
    pub fn skeleton_line_alt() -> Color32 {
        if is_dark() { Color32::from_gray(55) } else { Color32::from_gray(225) }
    }

    // ── 状态色（固定）───────────────────────────────────────────────────────
    pub fn success() -> Color32 { Color32::from_rgb(0x00, 0xBC, 0x70) }
    pub fn warning() -> Color32 { Color32::from_rgb(0xF7, 0x63, 0x00) }
    pub fn error()   -> Color32 { Color32::from_rgb(0xE8, 0x11, 0x23) }
    pub fn info()    -> Color32 { Color32::from_rgb(0x00, 0x78, 0xD4) }

    // ── 半透明叠加 ──────────────────────────────────────────────────────────
    pub fn overlay() -> Color32 { Color32::from_rgba_premultiplied(0, 0, 0, 128) }
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

/// 应用 Metro Design 主题到 egui 上下文（每次切换深浅色后都应调用）
pub fn apply_metro_theme(ctx: &egui::Context) {
    let dark = is_dark();
    let mut style = (*ctx.style()).clone();

    style.text_styles = metro_text_styles();
    style.spacing.item_spacing = Vec2::new(12.0, 8.0);
    style.spacing.button_padding = Vec2::new(16.0, 8.0);
    style.spacing.indent = 24.0;

    // Metro Design: 全局直角
    let zero = CornerRadius::same(0);
    style.visuals.window_corner_radius = zero;
    style.visuals.menu_corner_radius = zero;
    style.visuals.widgets.noninteractive.corner_radius = zero;
    style.visuals.widgets.inactive.corner_radius = zero;
    style.visuals.widgets.hovered.corner_radius = zero;
    style.visuals.widgets.active.corner_radius = zero;
    style.visuals.widgets.open.corner_radius = zero;

    // egui 深色/浅色基础 visuals
    style.visuals = if dark {
        let mut v = egui::Visuals::dark();
        v.window_corner_radius = zero;
        v.menu_corner_radius = zero;
        v.widgets.noninteractive.corner_radius = zero;
        v.widgets.inactive.corner_radius = zero;
        v.widgets.hovered.corner_radius = zero;
        v.widgets.active.corner_radius = zero;
        v.widgets.open.corner_radius = zero;
        v
    } else {
        let mut v = egui::Visuals::light();
        v.window_corner_radius = zero;
        v.menu_corner_radius = zero;
        v.widgets.noninteractive.corner_radius = zero;
        v.widgets.inactive.corner_radius = zero;
        v.widgets.hovered.corner_radius = zero;
        v.widgets.active.corner_radius = zero;
        v.widgets.open.corner_radius = zero;
        v
    };

    style.visuals.override_text_color = Some(colors::text_primary());
    style.visuals.window_fill = colors::bg_white();
    style.visuals.panel_fill = colors::bg_white();
    style.visuals.faint_bg_color = colors::bg_card();
    style.visuals.extreme_bg_color = colors::bg_card();
    style.visuals.hyperlink_color = colors::primary();
    style.visuals.selection.stroke = egui::Stroke { width: 1.0, color: colors::primary() };
    style.visuals.selection.bg_fill = colors::bg_selected();

    // TextEdit 等控件 widget 背景色跟随 bg_card
    style.visuals.widgets.noninteractive.bg_fill = colors::bg_card();
    style.visuals.widgets.inactive.bg_fill = colors::bg_card();

    ctx.set_style(style);
}
