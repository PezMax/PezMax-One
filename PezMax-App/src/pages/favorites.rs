// 收藏页面

use crate::app::PezMaxApp;
use crate::theme::colors;
use egui::{FontId, Rounding};

pub fn render(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    ui.add_space(16.0);
    ui.label(
        egui::RichText::new("⭐ 我的收藏")
            .font(FontId::new(20.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_PRIMARY),
    );
    ui.add_space(16.0);

    let favorites = vec![
        ("2024高考数学真题.pdf", "数学", "2024-06-15"),
        ("2024高考物理真题.pdf", "物理", "2024-06-10"),
    ];

    for (name, subject, date) in &favorites {
        egui::Frame::none()
            .fill(colors::BG_CARD)
            .rounding(Rounding::same(8.0))
            .stroke(egui::Stroke::new(1.0, colors::BORDER))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(12.0);
                    ui.label("⭐");
                    ui.add_space(10.0);
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(*name)
                                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                .color(colors::TEXT_PRIMARY),
                        );
                        ui.label(
                            egui::RichText::new(format!("{} | 收藏于 {}", subject, date))
                                .font(FontId::new(12.0, egui::FontFamily::Proportional))
                                .color(colors::TEXT_SECONDARY),
                        );
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("取消收藏").clicked() {}
                    });
                });
                ui.add_space(8.0);
            });
        ui.add_space(6.0);
    }
}