// 注册页面
// 带密保问题的三步注册流程

use crate::app::{Page, PezMaxApp, ToastLevel};
use crate::theme::colors;
use egui::{FontId, Rounding};

pub fn render(app: &mut PezMaxApp, ctx: &egui::Context) {
    let mut step = 1;

    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(colors::BG_WHITE))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.1);

                ui.label(
                    egui::RichText::new("创建账号")
                        .font(FontId::new(28.0, egui::FontFamily::Proportional))
                        .color(colors::TEXT_PRIMARY),
                );
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("注册后即可浏览和下载试卷资源")
                        .font(FontId::new(14.0, egui::FontFamily::Proportional))
                        .color(colors::TEXT_SECONDARY),
                );

                ui.add_space(32.0);

                // 步骤指示器
                ui.horizontal(|ui| {
                    for i in 1..=3 {
                        let is_active = i == step;
                        let is_done = i < step;
                        let color = if is_done { colors::SUCCESS } else if is_active { colors::PRIMARY } else { colors::BORDER };

                        egui::Frame::none()
                            .fill(color)
                            .rounding(Rounding::same(12.0))
                            .show(ui, |ui| {
                                ui.allocate_space(egui::Vec2::new(24.0, 24.0));
                                ui.allocate_ui_at_rect(ui.max_rect(), |ui| {
                                    ui.vertical_centered(|ui| {
                                        ui.label(
                                            egui::RichText::new(if is_done { "✓" } else { &i.to_string() })
                                                .color(colors::TEXT_ON_PRIMARY)
                                                .font(FontId::new(12.0, egui::FontFamily::Proportional)),
                                        );
                                    });
                                });
                            });
                        if i < 3 {
                            ui.add_space(40.0);
                        }
                    }
                });

                ui.add_space(32.0);

                // 注册表单卡片
                egui::Frame::none()
                    .fill(colors::BG_CARD)
                    .rounding(Rounding::same(12.0))
                    .stroke(egui::Stroke::new(1.0, colors::BORDER))
                    .show(ui, |ui| {
                        ui.set_max_width(400.0);
                        ui.add_space(24.0);

                        match step {
                            1 => {
                                ui.vertical_centered(|ui| {
                                    ui.label("基本信息");
                                });
                                ui.add_space(16.0);
                                // 用户名 / 密码 / 昵称 输入
                                ui.add_space(8.0);
                            }
                            2 => {
                                ui.vertical_centered(|ui| {
                                    ui.label("密保问题设置");
                                });
                                ui.add_space(16.0);
                                ui.add_space(8.0);
                            }
                            3 => {
                                ui.vertical_centered(|ui| {
                                    ui.label("验证码确认");
                                });
                                ui.add_space(16.0);
                                ui.add_space(8.0);
                            }
                            _ => {}
                        }

                        ui.add_space(24.0);

                        ui.horizontal(|ui| {
                            ui.add_space(60.0);
                            if step > 1 {
                                if ui.button("上一步").clicked() {
                                    step -= 1;
                                }
                                ui.add_space(16.0);
                            }
                            let next_label = if step < 3 { "下一步" } else { "完成注册" };
                            let btn = egui::Button::new(
                                egui::RichText::new(next_label)
                                    .color(colors::TEXT_ON_PRIMARY),
                            )
                            .fill(colors::PRIMARY)
                            .min_size(egui::Vec2::new(120.0, 40.0))
                            .rounding(Rounding::same(8.0));

                            if ui.add(btn).clicked() {
                                if step < 3 {
                                    step += 1;
                                } else {
                                    app.add_toast("注册成功！请登录", ToastLevel::Success);
                                    app.navigate(Page::Login);
                                }
                            }
                        });

                        ui.add_space(24.0);
                    });

                ui.add_space(16.0);
                if ui.link("已有账号？去登录").clicked() {
                    app.navigate(Page::Login);
                }
            });
        });
}