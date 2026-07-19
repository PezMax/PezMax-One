// 下载记录页面

use crate::app::PezMaxApp;
use crate::theme::colors;
use egui::{FontId, CornerRadius};

pub fn render(_app: &mut PezMaxApp, ui: &mut egui::Ui) {
    ui.add_space(16.0);
    ui.label(
        egui::RichText::new("📥 下载记录")
            .font(FontId::new(20.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_PRIMARY),
    );
    ui.add_space(16.0);

    // 下载记录列表
    let records = vec![
        ("2024高考数学真题.pdf", "2024-06-15", "数学", "2.3MB"),
        ("2024高考语文真题.pdf", "2024-06-14", "语文", "1.8MB"),
        ("2023高考英语真题.pdf", "2024-06-10", "英语", "1.5MB"),
    ];

    for (name, date, subject, size) in &records {
        egui::Frame::new()
            .fill(colors::BG_CARD)
            .corner_radius(CornerRadius::same(0))
            .stroke(egui::Stroke::new(1.0, colors::BORDER))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(12.0);
                    ui.label(
                        egui::RichText::new("📄")
                            .font(FontId::new(20.0, egui::FontFamily::Proportional)),
                    );
                    ui.add_space(10.0);
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(*name)
                                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                .color(colors::TEXT_PRIMARY),
                        );
                        ui.label(
                            egui::RichText::new(format!("{} | {} | {}", date, subject, size))
                                .font(FontId::new(12.0, egui::FontFamily::Proportional))
                                .color(colors::TEXT_SECONDARY),
                        );
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("隐藏").clicked() {}
                    });
                });
                ui.add_space(8.0);
            });
        ui.add_space(6.0);
    }
}