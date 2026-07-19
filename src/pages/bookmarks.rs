// 书签管理页面

use crate::app::PezMaxApp;
use crate::theme::colors;
use egui::{FontId, CornerRadius};

pub fn render(_app: &mut PezMaxApp, ui: &mut egui::Ui) {
    ui.add_space(16.0);
    ui.label(
        egui::RichText::new("🔖 书签管理")
            .font(FontId::new(20.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_PRIMARY),
    );
    ui.add_space(8.0);
    ui.label(
        egui::RichText::new("管理您收藏的外部书签链接")
            .font(FontId::new(13.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_SECONDARY),
    );
    ui.add_space(16.0);

    // 书签列表占位
    egui::Frame::new()
        .fill(colors::BG_CARD)
        .corner_radius(CornerRadius::same(10))
        .stroke(egui::Stroke::new(1.0, colors::BORDER))
        .show(ui, |ui| {
            ui.set_min_height(200.0);
            ui.vertical_centered(|ui| {
                ui.add_space(80.0);
                ui.label(
                    egui::RichText::new("暂无书签")
                        .font(FontId::new(16.0, egui::FontFamily::Proportional))
                        .color(colors::TEXT_SECONDARY),
                );
                ui.add_space(80.0);
            });
        });
}