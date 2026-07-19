// 系统设置页面

use crate::app::PezMaxApp;
use crate::theme::colors;
use egui::{FontId, CornerRadius};

pub fn render(_app: &mut PezMaxApp, ui: &mut egui::Ui) {
    ui.add_space(16.0);
    ui.label(
        egui::RichText::new("⚙️ 系统设置")
            .font(FontId::new(20.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_PRIMARY),
    );
    ui.add_space(24.0);

    let settings = [
        ("常规", "主题、语言、启动等基础设置"),
        ("下载", "下载目录、并发数、自动下载"),
        ("通知", "通知开关、提醒方式"),
        ("缓存", "缓存管理、数据清理"),
        ("关于", "版本信息、更新检查"),
    ];

    for (title, desc) in &settings {
        let response = egui::Frame::new()
            .fill(colors::BG_CARD)
            .corner_radius(CornerRadius::same(0))
            .stroke(egui::Stroke::new(1.0, colors::BORDER))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    ui.vertical(|ui| {
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(*title)
                                .font(FontId::new(15.0, egui::FontFamily::Proportional))
                                .color(colors::TEXT_PRIMARY),
                        );
                        ui.label(
                            egui::RichText::new(*desc)
                                .font(FontId::new(12.0, egui::FontFamily::Proportional))
                                .color(colors::TEXT_SECONDARY),
                        );
                        ui.add_space(8.0);
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(">");
                    });
                    ui.add_space(12.0);
                });
            })
            .response
            .interact(egui::Sense::click());

        if response.clicked() {}
        ui.add_space(6.0);
    }
}