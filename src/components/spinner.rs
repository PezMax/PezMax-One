//! Sokuou-driven 旋转加载指示器。
//!
//! 单个组件调用即用，动画状态自动挂到 egui 的临时数据里按 id 隔离。
//! 用 `Progress` 循环 0→1 驱动一个整体旋转角度，8 个点围一圈，通过 sin 相位
//! 制造"波浪"呼吸效果。
//!
//! 用法：
//! ```
//! use crate::components::spinner;
//! spinner::render(ui, "check_update_spinner", 16.0);
//! ```

use crate::sokuou::{Easing, Progress};
use crate::theme::colors;
use egui::{Color32, Sense, Vec2};

const DOT_COUNT: usize = 8;

pub fn render(ui: &mut egui::Ui, id_source: &str, size: f32) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());

    let anim_id = egui::Id::new(("spinner_progress", id_source));
    // 1.0s 一圈，线性；靠 sin(phase) 制造非线性观感
    let mut anim = ui.ctx().data_mut(|d| {
        d.get_temp::<Progress>(anim_id).unwrap_or_else(|| {
            let mut p = Progress::with_easing(1.0, Easing::Linear);
            p.set_target(1.0);
            p
        })
    });
    let dt = ui.input(|i| i.stable_dt) as f64;
    anim.update(dt);
    // 到点了就重置：0→1→0→1 …
    if anim.is_steady() {
        anim = Progress::with_easing(1.0, Easing::Linear);
        anim.set_target(1.0);
    }
    let phase = anim.value() as f32;
    // 每帧都推进，需要重绘
    ui.ctx().request_repaint();
    ui.ctx().data_mut(|d| d.insert_temp(anim_id, anim));

    let center = rect.center();
    let radius = size * 0.35;
    let dot_radius = (size * 0.09).max(1.5);
    let painter = ui.painter();
    let base = colors::primary();

    for i in 0..DOT_COUNT {
        let frac = i as f32 / DOT_COUNT as f32;
        let theta = std::f32::consts::TAU * frac;
        let x = center.x + radius * theta.cos();
        let y = center.y + radius * theta.sin();
        // 亮度按 (phase - frac) 的相位来做正弦波形，永远在 0.25 - 1.0 之间
        let mut t = phase - frac;
        t = t - t.floor(); // 归一化到 [0,1)
        let alpha_f = 0.25 + 0.75 * ((1.0 - t).powi(2));
        let a = (alpha_f * 255.0).clamp(0.0, 255.0) as u8;
        let color = Color32::from_rgba_premultiplied(
            (base.r() as u16 * a as u16 / 255) as u8,
            (base.g() as u16 * a as u16 / 255) as u8,
            (base.b() as u16 * a as u16 / 255) as u8,
            a,
        );
        painter.circle_filled(egui::pos2(x, y), dot_radius, color);
    }

    resp
}
