// PezMax egui Desktop — 主题系统
// Metro Design 风格：扁平、大字体、内容优先、色块分区
// 支持浅色 / 深色 / 跟随系统，以及 Ncrust 同款强调色预设

use egui::{FontFamily, FontId, TextStyle, Vec2};
use std::borrow::Cow;
use std::cell::Cell;
use std::sync::Arc;

// ── 外观模式 ─────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

// ── 强调色预设（延续 Ncrust 风格）──────────────────────────────────────────

pub struct AccentPreset {
    pub name: &'static str,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub const ACCENT_PRESETS: &[AccentPreset] = &[
    AccentPreset { name: "钴蓝", r: 0x3B, g: 0x82, b: 0xF6 },
    AccentPreset { name: "云杉", r: 0x1D, g: 0xB9, b: 0x54 },
    AccentPreset { name: "绯红", r: 0xEF, g: 0x44, b: 0x44 },
    AccentPreset { name: "琥珀", r: 0xF5, g: 0x9E, b: 0x0B },
    AccentPreset { name: "堇紫", r: 0x8B, g: 0x5C, b: 0xF6 },
];

// ── 线程局部全局状态 ─────────────────────────────────────────────────────────

thread_local! {
    static IS_DARK:     Cell<bool>  = const { Cell::new(false) };
    static ACCENT_IDX:  Cell<usize> = const { Cell::new(0) };
}

pub fn set_dark(dark: bool) {
    IS_DARK.with(|d| d.set(dark));
}

pub fn is_dark() -> bool {
    IS_DARK.with(|d| d.get())
}

pub fn set_accent(idx: usize) {
    ACCENT_IDX.with(|i| i.set(idx.min(ACCENT_PRESETS.len().saturating_sub(1))));
}

pub fn accent_idx() -> usize {
    ACCENT_IDX.with(|i| i.get())
}

/// ThemeMode::System 时查询 egui 的系统主题；Light/Dark 直接返回
pub fn effective_dark(ctx: &egui::Context) -> bool {
    // 注意：ThemeMode 本身不存在线程局部，由 PezMaxApp.theme_mode 持有
    // 调用者在 update() 里传入当前 mode 以避免跨层依赖
    // 这里仅暴露系统检测逻辑供 app.rs 使用
    ctx.system_theme().map(|t| t == egui::Theme::Dark).unwrap_or(false)
}

// ── 调色板（函数化，运行时读取模式）────────────────────────────────────────

pub mod colors {
    use egui::Color32;
    use super::{accent_idx, is_dark, ACCENT_PRESETS};

    // ── 强调色（从当前预设读取）────────────────────────────────────────────
    pub fn primary() -> Color32 {
        let p = &ACCENT_PRESETS[accent_idx()];
        Color32::from_rgb(p.r, p.g, p.b)
    }

    pub fn primary_dark() -> Color32 {
        let p = &ACCENT_PRESETS[accent_idx()];
        Color32::from_rgb(p.r * 7 / 10, p.g * 7 / 10, p.b * 7 / 10)
    }

    pub fn primary_light() -> Color32 {
        let p = &ACCENT_PRESETS[accent_idx()];
        Color32::from_rgb(
            (p.r as u16 * 7 / 10 + 255 * 3 / 10) as u8,
            (p.g as u16 * 7 / 10 + 255 * 3 / 10) as u8,
            (p.b as u16 * 7 / 10 + 255 * 3 / 10) as u8,
        )
    }

    // ── 其他固定强调色 ─────────────────────────────────────────────────────
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
        else         { Color32::from_rgb(0xF5, 0xF0, 0xE8) }
    }
    pub fn bg_card() -> Color32 {
        if is_dark() { Color32::from_rgb(0x26, 0x26, 0x26) }
        else         { Color32::from_rgb(0xFA, 0xF7, 0xF0) }
    }
    pub fn bg_sidebar() -> Color32 {
        if is_dark() { Color32::from_rgb(0x14, 0x14, 0x1E) }
        else         { Color32::from_rgb(0xC0, 0xB0, 0x9C) }
    }
    pub fn bg_hover() -> Color32 {
        if is_dark() { Color32::from_rgb(0x2E, 0x2E, 0x3A) }
        else         { Color32::from_rgb(0xE8, 0xE0, 0xD5) }
    }
    pub fn bg_selected() -> Color32 {
        if is_dark() { Color32::from_rgb(0x35, 0x35, 0x5A) }
        else         { Color32::from_rgb(0xDD, 0xD3, 0xC5) }
    }
    pub fn bg_input() -> Color32 {
        if is_dark() { Color32::from_rgb(0x20, 0x20, 0x20) }
        else         { Color32::from_rgb(0xEE, 0xE8, 0xDE) }
    }

    /// 搜索框/输入框背景：强调色去饱和后的深/浅灰调
    /// 15% 强调色 + 85% 底色，保留微弱色彩倾向
    pub fn bg_search() -> Color32 {
        let p = &ACCENT_PRESETS[accent_idx()];
        let (r, g, b) = (p.r as u16, p.g as u16, p.b as u16);
        if is_dark() {
            let base = 0x1C_u16;
            Color32::from_rgb(
                ((r * 15 + base * 85) / 100) as u8,
                ((g * 15 + base * 85) / 100) as u8,
                ((b * 15 + base * 85) / 100) as u8,
            )
        } else {
            let base = 0xF2_u16;
            Color32::from_rgb(
                ((r * 10 + base * 90) / 100) as u8,
                ((g * 10 + base * 90) / 100) as u8,
                ((b * 10 + base * 90) / 100) as u8,
            )
        }
    }

    // ── 边框 ────────────────────────────────────────────────────────────────
    pub fn border() -> Color32 {
        if is_dark() { Color32::from_rgb(0x3A, 0x3A, 0x3A) }
        else         { Color32::from_rgb(0xD4, 0xC8, 0xB8) }
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

/// 将搜索框/输入框专属视觉样式注入到当前 ui 作用域：
/// 去饱和强调色背景 + 完全无边框（Metro/Ncrust 无框设计）
/// 在 ui.scope() 内调用，确保样式不溢出到周围控件
pub fn apply_search_style(ui: &mut egui::Ui) {
    ui.visuals_mut().extreme_bg_color = colors::bg_search();
    ui.visuals_mut().widgets.inactive.bg_stroke = egui::Stroke::NONE;
    ui.visuals_mut().widgets.hovered.bg_stroke  = egui::Stroke::NONE;
    ui.visuals_mut().widgets.active.bg_stroke   = egui::Stroke::NONE;
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
pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    let candidates: &[(&str, u32, &str)] = &[
        ("C:\\Windows\\Fonts\\msyh.ttc", 0, "Microsoft YaHei Regular"),
        ("C:\\Windows\\Fonts\\simsun.ttc", 0, "SimSun"),
        ("C:\\Windows\\Fonts\\simhei.ttf", 0, "SimHei"),
        ("/System/Library/Fonts/PingFang.ttc", 0, "PingFang SC Regular"),
        ("/Library/Fonts/Arial Unicode.ttf", 0, "Arial Unicode"),
        ("/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc", 0, "Noto Sans CJK Regular"),
        ("/usr/share/fonts/noto/NotoSansCJK-Regular.ttc", 0, "Noto Sans CJK Regular"),
        ("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", 0, "Noto Sans CJK Regular"),
        ("/usr/share/fonts/wenquanyi/wqy-microhei/wqy-microhei.ttc", 0, "WenQuanYi Micro Hei"),
        ("/usr/share/fonts/wqy-microhei/wqy-microhei.ttc", 0, "WenQuanYi Micro Hei"),
        ("/usr/share/fonts/truetype/wqy/wqy-microhei.ttc", 0, "WenQuanYi Micro Hei"),
    ];

    for (path, font_index, _label) in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            let font_name = "system_cjk".to_owned();
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
            let prop = fonts.families.entry(FontFamily::Proportional).or_default();
            if !prop.contains(&font_name) { prop.insert(0, font_name.clone()); }
            let mono = fonts.families.entry(FontFamily::Monospace).or_default();
            if !mono.contains(&font_name) { mono.insert(0, font_name); }
            log::info!("Loaded CJK font from: {} (index: {})", path, font_index);
            break;
        }
    }

    ctx.set_fonts(fonts);
}

/// 应用 Metro Design 主题到 egui 上下文（每次切换模式/强调色后都应调用）
pub fn apply_metro_theme(ctx: &egui::Context) {
    let dark = is_dark();
    let mut style = (*ctx.style()).clone();

    style.text_styles = metro_text_styles();
    style.spacing.item_spacing = Vec2::new(12.0, 8.0);
    style.spacing.button_padding = Vec2::new(16.0, 8.0);
    style.spacing.indent = 24.0;

    style.visuals = if dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    style.visuals.dark_mode = dark;

    style.visuals.override_text_color = None;
    style.visuals.hyperlink_color = colors::primary();
    style.visuals.selection.stroke = egui::Stroke { width: 1.0, color: colors::primary() };
    style.visuals.selection.bg_fill = colors::bg_selected();
    style.visuals.warn_fg_color = colors::warning();
    style.visuals.error_fg_color = colors::error();

    style.visuals.window_fill = colors::bg_white();
    style.visuals.panel_fill = colors::bg_white();
    style.visuals.faint_bg_color = colors::bg_card();
    style.visuals.extreme_bg_color = colors::bg_card();
    style.visuals.code_bg_color = colors::bg_input();

    style.visuals.slider_trailing_fill = false;

    let zero = egui::CornerRadius::same(0);
    style.visuals.window_corner_radius = zero;
    style.visuals.menu_corner_radius = zero;

    let border_color = colors::border();
    let text_fg   = colors::text_primary();
    let text_weak = colors::text_secondary();

    style.visuals.widgets.noninteractive = egui::style::WidgetVisuals {
        bg_fill:       colors::bg_card(),
        weak_bg_fill:  colors::bg_input(),
        bg_stroke:     egui::Stroke::NONE,
        fg_stroke:     egui::Stroke::new(1.0, text_weak),
        corner_radius: zero,
        expansion:     0.0,
    };
    style.visuals.widgets.inactive = egui::style::WidgetVisuals {
        bg_fill:       colors::bg_card(),
        weak_bg_fill:  colors::bg_white(),
        bg_stroke:     egui::Stroke::new(1.0, border_color),
        fg_stroke:     egui::Stroke::new(1.5, text_fg),
        corner_radius: zero,
        expansion:     0.0,
    };
    style.visuals.widgets.hovered = egui::style::WidgetVisuals {
        bg_fill:       colors::bg_hover(),
        weak_bg_fill:  colors::bg_hover(),
        bg_stroke:     egui::Stroke::new(1.0, colors::primary_light()),
        fg_stroke:     egui::Stroke::new(1.5, text_fg),
        corner_radius: zero,
        expansion:     0.0,
    };
    style.visuals.widgets.active = egui::style::WidgetVisuals {
        bg_fill:       colors::bg_selected(),
        weak_bg_fill:  colors::bg_selected(),
        bg_stroke:     egui::Stroke::new(1.0, colors::primary()),
        fg_stroke:     egui::Stroke::new(2.0, text_fg),
        corner_radius: zero,
        expansion:     0.0,
    };
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
