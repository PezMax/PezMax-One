// 找回密码页面

use crate::app::{Page, PezMaxApp};
use crate::theme::colors;
use egui::{FontId, Rounding};

pub fn render(app: &mut PezMaxApp, ctx: &egui::Context) {
    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(colors::BG_WHITE))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.2);

                ui.label(
                    egui::RichText::new("找回密码")
                        .font(FontId::new(28.0, egui::FontFamily::Proportional))
                        .color(colors::TEXT_PRIMARY),
                );
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("通过密保问题重置您的密码")
                        .font(FontId::new(14.0, egui::FontFamily::Proportional))
                        .color(colors::TEXT_SECONDARY),
                );

                ui.add_space(32.0);

                egui::Frame::none()
                    .fill(colors::BG_CARD)
                    .rounding(Rounding::same(12.0))
                    .stroke(egui::Stroke::new(1.0, colors::BORDER))
                    .show(ui, |ui| {
                        ui.set_max_width(400.0);
                        ui.add_space(24.0);
                        ui.vertical_centered(|ui| {
                            ui.label("输入用户名开始找回密码");
                        });
                        ui.add_space(24.0);
                        ui.add_space(24.0);
                    });

                ui.add_space(16.0);
                if ui.link("返回登录").clicked() {
                    app.navigate(Page::Login);
                }
            });
        });
}