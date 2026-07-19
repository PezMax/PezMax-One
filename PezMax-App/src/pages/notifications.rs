// 通知中心页面

use crate::app::PezMaxApp;
use crate::theme::colors;
use egui::{FontId, CornerRadius};

pub fn render(_app: &mut PezMaxApp, ui: &mut egui::Ui) {
    ui.add_space(16.0);
    ui.label(
        egui::RichText::new("🔔 通知中心")
            .font(FontId::new(20.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_PRIMARY),
    );
    ui.add_space(16.0);

    let notifications = vec![
        ("系统通知", "您的账号已通过审核", "2024-06-15 10:30", false),
        ("下载通知", "您下载的「2024高考数学真题.pdf」已完成", "2024-06-14 15:20", true),
        ("收藏通知", "您收藏的试卷「2024高考语文真题.pdf」已更新", "2024-06-13 09:00", true),
    ];

    for (title, content, time, is_read) in &notifications {
        let bg = if *is_read { colors::BG_CARD } else { colors::BG_SELECTED };
        egui::Frame::new()
            .fill(bg)
            .corner_radius(CornerRadius::same(8))
            .stroke(egui::Stroke::new(1.0, colors::BORDER))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(12.0);
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(*title)
                                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                .color(colors::TEXT_PRIMARY),
                        );
                        ui.label(
                            egui::RichText::new(*content)
                                .font(FontId::new(12.0, egui::FontFamily::Proportional))
                                .color(colors::TEXT_SECONDARY),
                        );
                        ui.label(
                            egui::RichText::new(*time)
                                .font(FontId::new(11.0, egui::FontFamily::Proportional))
                                .color(colors::TEXT_SECONDARY),
                        );
                    });
                });
                ui.add_space(8.0);
            });
        ui.add_space(6.0);
    }
}