// 安全设置页面

use crate::app::PezMaxApp;
use crate::theme::colors;
use egui::{FontId, CornerRadius};

pub fn render(_app: &mut PezMaxApp, ui: &mut egui::Ui) {
    ui.add_space(16.0);
    ui.label(
        egui::RichText::new("🔒 安全设置")
            .font(FontId::new(20.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_PRIMARY),
    );
    ui.add_space(24.0);

    // 修改密码
    egui::Frame::new()
        .fill(colors::BG_CARD)
        .corner_radius(CornerRadius::same(0))
        .stroke(egui::Stroke::new(1.0, colors::BORDER))
        .show(ui, |ui| {
            ui.add_space(16.0);
            ui.label(
                egui::RichText::new("修改密码")
                    .font(FontId::new(16.0, egui::FontFamily::Proportional))
                    .color(colors::TEXT_PRIMARY),
            );
            ui.add_space(12.0);
            ui.label("输入旧密码和新密码");
            ui.add_space(16.0);
        });

    ui.add_space(12.0);

    // 密保设置
    egui::Frame::new()
        .fill(colors::BG_CARD)
        .corner_radius(CornerRadius::same(0))
        .stroke(egui::Stroke::new(1.0, colors::BORDER))
        .show(ui, |ui| {
            ui.add_space(16.0);
            ui.label(
                egui::RichText::new("密保问题")
                    .font(FontId::new(16.0, egui::FontFamily::Proportional))
                    .color(colors::TEXT_PRIMARY),
            );
            ui.add_space(12.0);
            ui.label("用于找回密码的安全验证");
            ui.add_space(16.0);
        });
}