// Toast 通知组件
// 非侵入式右下角通知，类似 Windows 11 通知中心

use crate::app::{PezMaxApp, ToastLevel};
use crate::theme::colors;
use egui::{Frame, Id, CornerRadius};

pub fn render(app: &mut PezMaxApp, ctx: &egui::Context) {
    if app.toasts.is_empty() {
        return;
    }

    // 在右下角叠加显示
    egui::Area::new(Id::new("toast_area"))
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::Vec2::new(-20.0, -20.0))
        .show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::RIGHT), |ui| {
                for toast in &app.toasts {
                    let (bg_color, icon) = match toast.level {
                        ToastLevel::Success => (colors::SUCCESS, "✓"),
                        ToastLevel::Warning => (colors::WARNING, "⚠"),
                        ToastLevel::Error => (colors::ERROR, "✕"),
                        ToastLevel::Info => (colors::INFO, "ℹ"),
                    };

                    Frame::new()
                        .fill(bg_color)
                        .corner_radius(CornerRadius::same(0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.add_space(12.0);
                                ui.label(
                                    egui::RichText::new(icon)
                                        .color(colors::TEXT_ON_PRIMARY)
                                        .font(egui::FontId::new(14.0, egui::FontFamily::Proportional)),
                                );
                                ui.add_space(8.0);
                                ui.label(
                                    egui::RichText::new(&toast.message)
                                        .color(colors::TEXT_ON_PRIMARY)
                                        .font(egui::FontId::new(13.0, egui::FontFamily::Proportional)),
                                );
                                ui.add_space(12.0);
                            });
                        });
                    ui.add_space(8.0);
                }
            });
        });
}