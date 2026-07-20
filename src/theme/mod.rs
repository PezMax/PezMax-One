// PezMax egui Desktop — 主题系统
// Metro Design 风格：扁平、大字体、内容优先、色块分区
// 支持浅色 / 深色模式（thread_local IS_DARK 驱动）

use egui::{FontFamily, FontId, TextStyle, Vec2};
use std::borrow::Cow;
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

/// 加载系统 CJK 字体作为 primary 字体，确保中文文字清晰渲染
/// 插入到字体列表最前面，避免因回退到不同度量字体导致的模糊
/// 按平台优先级尝试：Windows 微软雅黑 → macOS PingFang → Linux Noto CJK → WenQuanYi
pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // (路径, font_index, 说明) — font_index 用于 TTC 多字体集合文件
    let candidates: &[(&str, u32, &str)] = &[
        // Windows — 微软雅黑（优先，最常用）
        ("C:\\Windows\\Fonts\\msyh.ttc", 0, "Microsoft YaHei Regular"),
        ("C:\\Windows\\Fonts\\simsun.ttc", 0, "SimSun"),
        ("C:\\Windows\\Fonts\\simhei.ttf", 0, "SimHei"),
        // macOS
        ("/System/Library/Fonts/PingFang.ttc", 0, "PingFang SC Regular"),
        ("/Library/Fonts/Arial Unicode.ttf", 0, "Arial Unicode"),
        // Linux — Noto CJK（字形最全，推荐）
        ("/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc", 0, "Noto Sans CJK Regular"),
        ("/usr/share/fonts/noto/NotoSansCJK-Regular.ttc", 0, "Noto Sans CJK Regular"),
        ("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", 0, "Noto Sans CJK Regular"),
        // Linux — WenQuanYi（备选）
        ("/usr/share/fonts/wenquanyi/wqy-microhei/wqy-microhei.ttc", 0, "WenQuanYi Micro Hei"),
        ("/usr/share/fonts/wqy-microhei/wqy-microhei.ttc", 0, "WenQuanYi Micro Hei"),
        ("/usr/share/fonts/truetype/wqy/wqy-microhei.ttc", 0, "WenQuanYi Micro Hei"),
    ];

    for (path, font_index, _label) in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            let font_name = "system_cjk".to_owned();

            // 直接构造 FontData 以显式设置 index 和 tweak
            fonts.font_data.insert(
                font_name.clone(),
                Arc::new(egui::FontData {
                    font: Cow::Owned(bytes),
                    index: *font_index,
                    tweak: egui::FontTweak {
                        scale: 1.0,
                        y_offset_factor: 0.0,
                        y_offset: 0.0,
                        baseline_offset_factor: 0.0,
                    },
                }),
            );

            // 关键：插入到最前面（作为 primary 字体），而非 append
            // 这样 CJK 和 Latin 字符都由同一字体渲染，避免回退模糊
            let prop = fonts.families.entry(FontFamily::Proportional).or_default();
            if !prop.contains(&font_name) {
                prop.insert(0, font_name.clone());
            }
            let mono = fonts.families.entry(FontFamily::Monospace).or_default();
            if !mono.contains(&font_name) {
                mono.insert(0, font_name);
            }

            log::info!("Loaded CJK font from: {} (index: {})", path, font_index);
            break;
        }
    }

    ctx.set_fonts(fonts);
}

/// 应用 Metro Design 主题到 egui 上下文（每次切换深浅色后都应调用）
/// 显式设置所有 widget 状态的颜色，确保深/浅模式下文字、边框、勾选标记都可见
pub fn apply_metro_theme(ctx: &egui::Context) {
    let dark = is_dark();
    let mut style = (*ctx.style()).clone();

    style.text_styles = metro_text_styles();
    style.spacing.item_spacing = Vec2::new(12.0, 8.0);
    style.spacing.button_padding = Vec2::new(16.0, 8.0);
    style.spacing.indent = 24.0;

    // 基础 visuals（先取深/浅默认值，再逐一覆盖）
    style.visuals = if dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    style.visuals.dark_mode = dark;

    // ── 覆盖所有视觉属性 ────────────────────────────────────────────────

    // 文本色（全局覆盖，所有 label / button / textedit 文本均受控）
    // 设为 None 以允许 RichText.color() 显式设置优先，
    // 否则侧边栏（永远深色背景）上的白色图标会被浅色模式的深色 text_primary 覆盖
    style.visuals.override_text_color = None;
    style.visuals.hyperlink_color = colors::primary();
    style.visuals.selection.stroke = egui::Stroke { width: 1.0, color: colors::primary() };
    style.visuals.selection.bg_fill = colors::bg_selected();
    style.visuals.warn_fg_color = colors::warning();
    style.visuals.error_fg_color = colors::error();

    // 背景
    style.visuals.window_fill = colors::bg_white();
    style.visuals.panel_fill = colors::bg_white();
    style.visuals.faint_bg_color = colors::bg_card();
    style.visuals.extreme_bg_color = colors::bg_card();
    style.visuals.code_bg_color = colors::bg_input();

    // 滑块轨道
    style.visuals.slider_trailing_fill = false;

    // 全局直角
    let zero = egui::CornerRadius::same(0);
    style.visuals.window_corner_radius = zero;
    style.visuals.menu_corner_radius = zero;

    // ── 所有 widget 状态配色（杜绝默认值泄漏）───────────────────────────
    let border_color = colors::border();
    let text_fg   = colors::text_primary();
    let text_weak = colors::text_secondary();

    // noninteractive：提示文字、禁用边框、分隔线
    style.visuals.widgets.noninteractive = egui::style::WidgetVisuals {
        bg_fill:       colors::bg_card(),
        weak_bg_fill:  colors::bg_input(),
        bg_stroke:     egui::Stroke::new(1.0, border_color),
        fg_stroke:     egui::Stroke::new(1.0, text_weak),
        corner_radius: zero,
        expansion:     0.0,
    };

    // inactive：普通控件（按钮、输入框、复选框框体）
    style.visuals.widgets.inactive = egui::style::WidgetVisuals {
        bg_fill:       colors::bg_card(),
        weak_bg_fill:  colors::bg_white(),
        bg_stroke:     egui::Stroke::new(1.0, border_color),
        fg_stroke:     egui::Stroke::new(1.5, text_fg),
        corner_radius: zero,
        expansion:     0.0,
    };

    // hovered：悬停态
    style.visuals.widgets.hovered = egui::style::WidgetVisuals {
        bg_fill:       colors::bg_hover(),
        weak_bg_fill:  colors::bg_hover(),
        bg_stroke:     egui::Stroke::new(1.0, colors::primary_light()),
        fg_stroke:     egui::Stroke::new(1.5, text_fg),
        corner_radius: zero,
        expansion:     0.0,
    };

    // active：按下态
    style.visuals.widgets.active = egui::style::WidgetVisuals {
        bg_fill:       colors::bg_selected(),
        weak_bg_fill:  colors::bg_selected(),
        bg_stroke:     egui::Stroke::new(1.0, colors::primary()),
        fg_stroke:     egui::Stroke::new(2.0, text_fg),
        corner_radius: zero,
        expansion:     0.0,
    };

    // open：展开态（如下拉菜单）
    style.visuals.widgets.open = egui::style::WidgetVisuals {
        bg_fill:       colors::bg_selected(),
        weak_bg_fill:  colors::bg_selected(),
        bg_stroke:     egui::Stroke::new(1.0, colors::primary()),
        fg_stroke:     egui::Stroke::new(1.5, text_fg),
        corner_radius: zero,
        expansion:     0.0,
    };

    ctx.set_style(style);
}
