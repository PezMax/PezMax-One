// Toast 通知组件
// 右下角叠层显示
// 入场：enter Progress 0→1, 0.25s EaseOutCubic — 淡入 + 右滑入
// 离场：exit  Progress 0→1, 0.25s EaseInCubic  — 淡出 + 右滑出

use crate::app::{PezMaxApp, ToastLevel};
use crate::sokuou::map_range;
use crate::theme::colors;
use egui::{CornerRadius, Frame, Id};

pub fn render(app: &mut PezMaxApp, ctx: &egui::Context) {
    if app.toasts.is_empty() {
        return;
    }

    // 每条 Toast 独占一个 Area，便于独立控制位置
    // 从底向上堆叠：先计算各 toast 的累计高度
    let screen = ctx.screen_rect();
    let margin = 20.0f32;
    let toast_width = 300.0f32;

    let mut y_offset = margin;

    for (i, toast) in app.toasts.iter().enumerate().rev() {
        let enter_v = toast.enter.value() as f32;
        let exit_v = toast.exit.value() as f32;
        let alpha = enter_v * (1.0 - exit_v);
        // 右滑偏移：入场时从右边滑入，离场时向右滑出
        let slide_x = map_range(
            (toast.enter.value() * (1.0 - toast.exit.value())) as f64,
            40.0,
            0.0,
        ) as f32;

        let x = screen.right() - toast_width - margin + slide_x;
        let y_bottom = screen.bottom() - y_offset;

        egui::Area::new(Id::new(("toast", i)))
            .fixed_pos(egui::pos2(x, y_bottom - 50.0)) // approximate; egui will fit height
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.set_opacity(alpha.clamp(0.0, 1.0));

                let (bg_color, icon) = match toast.level {
                    ToastLevel::Success => (colors::success(), "✓"),
                    ToastLevel::Warning => (colors::warning(), "⚠"),
                    ToastLevel::Error => (colors::error(), "✕"),
                    ToastLevel::Info => (colors::info(), "ℹ"),
                };

                Frame::new()
                    .fill(bg_color)
                    .corner_radius(CornerRadius::same(0))
                    .show(ui, |ui| {
                        ui.set_min_width(toast_width);
                        ui.set_max_width(toast_width);
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.add_space(12.0);
                            ui.label(
                                egui::RichText::new(icon)
                                    .color(colors::text_on_primary())
                                    .font(egui::FontId::new(
                                        15.0,
                                        egui::FontFamily::Proportional,
                                    )),
                            );
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new(&toast.message)
                                    .color(colors::text_on_primary())
                                    .font(egui::FontId::new(
                                        13.0,
                                        egui::FontFamily::Proportional,
                                    )),
                            );
                            ui.add_space(12.0);
                        });
                        ui.add_space(10.0);
                    });
            });

        y_offset += 60.0; // approx height per toast + gap
    }
}
