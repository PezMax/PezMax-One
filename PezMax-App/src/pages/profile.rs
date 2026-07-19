// 个人中心页面

use crate::app::PezMaxApp;
use crate::theme::colors;
use egui::{FontId, CornerRadius, Vec2};

pub fn render(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    ui.add_space(16.0);
    ui.label(
        egui::RichText::new("👤 个人中心")
            .font(FontId::new(20.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_PRIMARY),
    );
    ui.add_space(24.0);

    // 用户信息卡片
    if let Some(ref user) = app.current_user {
        ui.horizontal(|ui| {
            // 头像区域
            egui::Frame::new()
                .fill(colors::PRIMARY)
                .corner_radius(CornerRadius::same(40))
                .show(ui, |ui| {
                    ui.allocate_space(Vec2::new(80.0, 80.0));
                    ui.allocate_ui_at_rect(ui.max_rect(), |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new(
                                    user.nick_name.chars().next().unwrap_or('?').to_string(),
                                )
                                .color(colors::TEXT_ON_PRIMARY)
                                .font(FontId::new(36.0, egui::FontFamily::Proportional)),
                            );
                        });
                    });
                });

            ui.add_space(24.0);

            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(&user.nick_name)
                        .font(FontId::new(22.0, egui::FontFamily::Proportional))
                        .color(colors::TEXT_PRIMARY),
                );
                ui.label(
                    egui::RichText::new(&user.user_name)
                        .font(FontId::new(14.0, egui::FontFamily::Proportional))
                        .color(colors::TEXT_SECONDARY),
                );
                ui.add_space(8.0);
                if ui.button("更换头像").clicked() {}
            });
        });
    }

    ui.add_space(24.0);

    // 信息编辑区
    egui::Frame::new()
        .fill(colors::BG_CARD)
        .corner_radius(CornerRadius::same(10))
        .stroke(egui::Stroke::new(1.0, colors::BORDER))
        .show(ui, |ui| {
            ui.add_space(16.0);
            ui.label(
                egui::RichText::new("基本信息")
                    .font(FontId::new(16.0, egui::FontFamily::Proportional))
                    .color(colors::TEXT_PRIMARY),
            );
            ui.add_space(12.0);
            ui.add_space(16.0);
        });
}