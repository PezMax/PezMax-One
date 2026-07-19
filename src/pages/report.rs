// 举报中心页面

use crate::app::PezMaxApp;
use crate::theme::colors;
use egui::{FontId, CornerRadius};

pub fn render(_app: &mut PezMaxApp, ui: &mut egui::Ui) {
    ui.add_space(16.0);
    ui.label(
        egui::RichText::new("🚩 举报中心")
            .font(FontId::new(20.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_PRIMARY),
    );
    ui.add_space(8.0);
    ui.label(
        egui::RichText::new("举报违规内容或试卷文件")
            .font(FontId::new(13.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_SECONDARY),
    );
    ui.add_space(16.0);

    // 举报表单占位
    egui::Frame::new()
        .fill(colors::BG_CARD)
        .corner_radius(CornerRadius::same(10))
        .stroke(egui::Stroke::new(1.0, colors::BORDER))
        .show(ui, |ui| {
            ui.set_min_height(200.0);
            ui.vertical_centered(|ui| {
                ui.add_space(80.0);
                ui.label("举报表单（待实现）");
                ui.add_space(80.0);
            });
        });
}