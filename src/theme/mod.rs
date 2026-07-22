// PezMax egui Desktop — 主题系统
// Metro Design 风格：扁平、大字体、内容优先、色块分区
// 支持浅色 / 深色 / 跟随系统，以及 Ncrust 同款强调色预设

use crate::sokuou::{EasingMode, MetroAnim, UwpEasing};
use egui::{FontFamily, FontId, TextStyle, Vec2};
use std::borrow::Cow;
use std::cell::Cell;
use std::cell::RefCell;
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
    static ACCENT_TRANSITION: RefCell<AccentTransition> = const { RefCell::new(AccentTransition::idle()) };
    static DARK_TRANSITION:   RefCell<DarkTransition>   = const { RefCell::new(DarkTransition::idle()) };
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

// ── 强调色过渡动画（MetroAnim 驱动）────────────────────────────────────────

/// 强调色过渡状态：在 old ↔ new 两个预设之间插值 RGB。
/// 使用 UWP Quadratic/EaseOut 曲线，0.3s 时长。
pub struct AccentTransition {
    anim: MetroAnim,
    from_r: u8,
    from_g: u8,
    from_b: u8,
    to_r: u8,
    to_g: u8,
    to_b: u8,
    active: bool,
}

impl AccentTransition {
    const fn idle() -> Self {
        // 用 const 兼容的构造避开 MetroAnim::new（非 const fn）
        Self {
            anim: MetroAnim::new(0.3, UwpEasing::Quadratic, EasingMode::EaseOut),
            from_r: 0,
            from_g: 0,
            from_b: 0,
            to_r: 0,
            to_g: 0,
            to_b: 0,
            active: false,
        }
    }
}

/// 开始强调色过渡：从当前颜色平滑过渡到 new_idx 对应的预设。
/// 如果已有过渡进行中，从中断位置开始（interrupt-safe）。
pub fn start_accent_transition(new_idx: usize) {
    let new_idx = new_idx.min(ACCENT_PRESETS.len().saturating_sub(1));
    let new_p = &ACCENT_PRESETS[new_idx];
    ACCENT_TRANSITION.with(|t| {
        let mut t = t.borrow_mut();
        // 当前显示的颜色（过渡中则取插值，否则取预设）
        let (cr, cg, cb) = if t.active {
            let v = t.anim.value();
            (
                (t.from_r as f64 + (t.to_r as f64 - t.from_r as f64) * v).round().clamp(0.0, 255.0) as u8,
                (t.from_g as f64 + (t.to_g as f64 - t.from_g as f64) * v).round().clamp(0.0, 255.0) as u8,
                (t.from_b as f64 + (t.to_b as f64 - t.from_b as f64) * v).round().clamp(0.0, 255.0) as u8,
            )
        } else {
            let old_p = &ACCENT_PRESETS[accent_idx()];
            (old_p.r, old_p.g, old_p.b)
        };
        t.from_r = cr;
        t.from_g = cg;
        t.from_b = cb;
        t.to_r = new_p.r;
        t.to_g = new_p.g;
        t.to_b = new_p.b;
        t.anim.jump_to(0.0);
        t.anim.set_target(1.0);
        t.active = true;
    });
}

/// 每帧推进过渡动画，应在 app.rs update() 中调用。
pub fn update_accent_transition(dt: f64) {
    ACCENT_TRANSITION.with(|t| {
        let mut t = t.borrow_mut();
        if t.active {
            t.anim.update(dt);
            if t.anim.is_steady() {
                t.active = false;
            }
        }
    });
}

/// 过渡动画是否仍在进行中。
pub fn is_transitioning() -> bool {
    ACCENT_TRANSITION.with(|t| t.borrow().active)
}

/// 获取当前应显示的强调色 RGB（过渡中则返回插值，否则返回预设）。
fn current_accent_rgb() -> (u8, u8, u8) {
    ACCENT_TRANSITION.with(|t| {
        let t = t.borrow();
        if t.active {
            let v = t.anim.value();
            (
                (t.from_r as f64 + (t.to_r as f64 - t.from_r as f64) * v).round().clamp(0.0, 255.0) as u8,
                (t.from_g as f64 + (t.to_g as f64 - t.from_g as f64) * v).round().clamp(0.0, 255.0) as u8,
                (t.from_b as f64 + (t.to_b as f64 - t.from_b as f64) * v).round().clamp(0.0, 255.0) as u8,
            )
        } else {
            let p = &ACCENT_PRESETS[accent_idx()];
            (p.r, p.g, p.b)
        }
    })
}

/// ThemeMode::System 时查询 egui 的系统主题；Light/Dark 直接返回
/// 在 Linux 上，当 egui 无法检测到系统主题时，会尝试多种 Linux 原生方案
/// 作为回退：gsettings (GNOME)、kdeglobals (KDE)、GTK_THEME 环境变量、GTK settings.ini
pub fn effective_dark(ctx: &egui::Context) -> bool {
    // 优先使用 egui 的系统主题检测（Windows/macOS 可靠，部分 Linux 桌面也支持）
    if let Some(theme) = ctx.system_theme() {
        return theme == egui::Theme::Dark;
    }
    // 回退：Linux 原生检测
    #[cfg(target_os = "linux")]
    {
        detect_linux_dark_mode()
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

/// Linux 深色模式检测：依次尝试多种桌面环境方案
#[cfg(target_os = "linux")]
fn detect_linux_dark_mode() -> bool {
    // 1. GNOME 42+：gsettings org.gnome.desktop.interface color-scheme
    if let Ok(output) = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output()
    {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();
            if s.contains("prefer-dark") || s.contains("dark") {
                return true;
            }
            if s.contains("default") || s.contains("light") {
                return false;
            }
        }
    }

    // 2. 旧版 GNOME：gsettings org.gnome.desktop.interface gtk-theme 包含 "dark"
    if let Ok(output) = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "gtk-theme"])
        .output()
    {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();
            if s.contains("dark") || s.contains("adwaita") && !s.contains("light") {
                return true;
            }
        }
    }

    // 3. KDE：读取 ~/.config/kdeglobals 中的 ColorScheme
    if let Ok(home) = std::env::var("HOME") {
        let kdeglobals_path = std::path::Path::new(&home).join(".config/kdeglobals");
        if kdeglobals_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&kdeglobals_path) {
                for line in content.lines() {
                    if line.trim().starts_with("ColorScheme=") {
                        let val = line.trim().split('=').nth(1).unwrap_or("").to_lowercase();
                        if val.contains("dark") {
                            return true;
                        }
                        break;
                    }
                }
            }
        }
    }

    // 4. GTK_THEME 环境变量
    if let Ok(gtk_theme) = std::env::var("GTK_THEME") {
        if gtk_theme.to_lowercase().contains("dark") {
            return true;
        }
    }

    // 5. GTK settings.ini（GTK 4.0 优先，3.0 回退）
    if let Ok(home) = std::env::var("HOME") {
        for config_path in [
            format!("{home}/.config/gtk-4.0/settings.ini"),
            format!("{home}/.config/gtk-3.0/settings.ini"),
        ] {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed == "gtk-application-prefer-dark-theme=1" {
                        return true;
                    }
                    if trimmed == "gtk-application-prefer-dark-theme=0" {
                        return false;
                    }
                }
            }
        }
    }

    false // 默认浅色
}

// ── 深色模式过渡动画（MetroAnim 驱动）────────────────────────────────────────

/// 深色模式过渡状态：0.0 = 浅色，1.0 = 深色，中间值插值所有颜色。
/// 使用 UWP Quadratic/EaseOut 曲线，0.3s 时长。
struct DarkTransition {
    anim: MetroAnim,
    active: bool,
    /// true = 正在过渡到深色，false = 正在过渡到浅色
    target_dark: bool,
}

impl DarkTransition {
    const fn idle() -> Self {
        Self {
            anim: MetroAnim::new(0.3, UwpEasing::Quadratic, EasingMode::EaseOut),
            active: false,
            target_dark: false,
        }
    }
}

/// 开始深色模式过渡动画。
/// 从当前显示状态平滑过渡到目标模式。
pub fn start_dark_transition(target_dark: bool) {
    DARK_TRANSITION.with(|t| {
        let mut t = t.borrow_mut();
        t.anim.jump_to(0.0);
        t.anim.set_target(1.0);
        t.target_dark = target_dark;
        t.active = true;
    });
}

/// 每帧推进深色模式过渡动画，应在 app.rs update() 中调用。
pub fn update_dark_transition(dt: f64) {
    DARK_TRANSITION.with(|t| {
        let mut t = t.borrow_mut();
        if t.active {
            t.anim.update(dt);
            if t.anim.is_steady() {
                t.active = false;
            }
        }
    });
}

/// 深色模式过渡是否仍在进行中。
pub fn is_dark_transitioning() -> bool {
    DARK_TRANSITION.with(|t| t.borrow().active)
}

/// 深色模式过渡进度：0.0 = 完全浅色，1.0 = 完全深色。
/// 过渡中返回插值，否则返回静态状态。
fn dark_progress() -> f64 {
    DARK_TRANSITION.with(|t| {
        let t = t.borrow();
        if t.active {
            if t.target_dark { t.anim.value() } else { 1.0 - t.anim.value() }
        } else {
            if is_dark() { 1.0 } else { 0.0 }
        }
    })
}

// ── 调色板（函数化，运行时读取模式）────────────────────────────────────────

pub mod colors {
    use egui::Color32;

    // ── 深浅色过渡插值辅助 ──────────────────────────────────────────────
    fn lerp_dark(light: Color32, dark: Color32) -> Color32 {
        let t = super::dark_progress();
        if t <= 0.0 { return light; }
        if t >= 1.0 { return dark; }
        Color32::from_rgb(
            (light.r() as f64 + (dark.r() as f64 - light.r() as f64) * t).round().clamp(0.0, 255.0) as u8,
            (light.g() as f64 + (dark.g() as f64 - light.g() as f64) * t).round().clamp(0.0, 255.0) as u8,
            (light.b() as f64 + (dark.b() as f64 - light.b() as f64) * t).round().clamp(0.0, 255.0) as u8,
        )
    }

    // ── 强调色（从当前预设读取，过渡时取插值）────────────────────────────
    pub fn primary() -> Color32 {
        let (r, g, b) = super::current_accent_rgb();
        Color32::from_rgb(r, g, b)
    }

    pub fn primary_dark() -> Color32 {
        let (r, g, b) = super::current_accent_rgb();
        Color32::from_rgb(r * 7 / 10, g * 7 / 10, b * 7 / 10)
    }

    pub fn primary_light() -> Color32 {
        let (r, g, b) = super::current_accent_rgb();
        Color32::from_rgb(
            (r as u16 * 7 / 10 + 255 * 3 / 10) as u8,
            (g as u16 * 7 / 10 + 255 * 3 / 10) as u8,
            (b as u16 * 7 / 10 + 255 * 3 / 10) as u8,
        )
    }

    // ── 其他固定强调色 ─────────────────────────────────────────────────────
    pub fn accent_orange() -> Color32 { Color32::from_rgb(0xF7, 0x63, 0x00) }
    pub fn accent_green()  -> Color32 { Color32::from_rgb(0x00, 0xBC, 0x70) }
    pub fn accent_purple() -> Color32 { Color32::from_rgb(0x88, 0x44, 0xAA) }
    pub fn accent_teal()   -> Color32 { Color32::from_rgb(0x00, 0xB7, 0xC3) }

    // ── 文本色 ──────────────────────────────────────────────────────────────
    pub fn text_primary() -> Color32 {
        lerp_dark(
            Color32::from_rgb(0x1A, 0x1A, 0x2E),  // light
            Color32::from_rgb(0xF0, 0xF0, 0xF2),  // dark
        )
    }
    pub fn text_secondary() -> Color32 {
        lerp_dark(
            Color32::from_rgb(0x60, 0x60, 0x70),
            Color32::from_rgb(0x9A, 0x9A, 0xAA),
        )
    }
    pub fn text_on_primary() -> Color32 { Color32::WHITE }

    // ── 背景色 ──────────────────────────────────────────────────────────────
    pub fn bg_white() -> Color32 {
        lerp_dark(
            Color32::from_rgb(0xF5, 0xF0, 0xE8),
            Color32::from_rgb(0x1C, 0x1C, 0x1C),
        )
    }
    pub fn bg_card() -> Color32 {
        lerp_dark(
            Color32::from_rgb(0xFA, 0xF7, 0xF0),
            Color32::from_rgb(0x26, 0x26, 0x26),
        )
    }
    pub fn bg_sidebar() -> Color32 {
        lerp_dark(
            Color32::from_rgb(0xC0, 0xB0, 0x9C),
            Color32::from_rgb(0x14, 0x14, 0x1E),
        )
    }
    pub fn bg_hover() -> Color32 {
        lerp_dark(
            Color32::from_rgb(0xE8, 0xE0, 0xD5),
            Color32::from_rgb(0x2E, 0x2E, 0x3A),
        )
    }
    pub fn bg_selected() -> Color32 {
        lerp_dark(
            Color32::from_rgb(0xDD, 0xD3, 0xC5),
            Color32::from_rgb(0x35, 0x35, 0x5A),
        )
    }
    pub fn bg_input() -> Color32 {
        lerp_dark(
            Color32::from_rgb(0xEE, 0xE8, 0xDE),
            Color32::from_rgb(0x20, 0x20, 0x20),
        )
    }

    /// 搜索框/输入框背景：强调色去饱和后的深/浅灰调
    /// 15% 强调色 + 85% 底色，保留微弱色彩倾向
    pub fn bg_search() -> Color32 {
        let (r, g, b) = super::current_accent_rgb();
        let (r, g, b) = (r as u16, g as u16, b as u16);
        let light = {
            let base = 0xF2_u16;
            Color32::from_rgb(
                ((r * 10 + base * 90) / 100) as u8,
                ((g * 10 + base * 90) / 100) as u8,
                ((b * 10 + base * 90) / 100) as u8,
            )
        };
        let dark = {
            let base = 0x1C_u16;
            Color32::from_rgb(
                ((r * 15 + base * 85) / 100) as u8,
                ((g * 15 + base * 85) / 100) as u8,
                ((b * 15 + base * 85) / 100) as u8,
            )
        };
        lerp_dark(light, dark)
    }

    // ── 边框 ────────────────────────────────────────────────────────────────
    pub fn border() -> Color32 {
        lerp_dark(
            Color32::from_rgb(0xD4, 0xC8, 0xB8),
            Color32::from_rgb(0x3A, 0x3A, 0x3A),
        )
    }

    // ── 骨架屏占位色 ────────────────────────────────────────────────────────
    pub fn skeleton_base() -> Color32 {
        lerp_dark(Color32::from_gray(220), Color32::from_gray(50))
    }
    pub fn skeleton_line() -> Color32 {
        lerp_dark(Color32::from_gray(215), Color32::from_gray(43))
    }
    pub fn skeleton_line_alt() -> Color32 {
        lerp_dark(Color32::from_gray(225), Color32::from_gray(55))
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
