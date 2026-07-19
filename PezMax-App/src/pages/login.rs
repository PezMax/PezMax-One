// 登录页面
// Metro Design 风格：居中卡片、大标题、扁平输入框

use crate::app::{Page, PezMaxApp, ToastLevel};
use crate::theme::colors;
use egui::{FontId, CornerRadius};

pub fn render(app: &mut PezMaxApp, ctx: &egui::Context) {
    // 登录状态
    let mut username = String::new();
    let mut password = String::new();

    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(colors::BG_WHITE))
        .show(ctx, |ui| {
            // 垂直居中
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.15);

                // Logo
                ui.label(
                    egui::RichText::new("PezMax")
                        .font(FontId::new(42.0, egui::FontFamily::Proportional))
                        .color(colors::PRIMARY),
                );
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("试卷资源管理系统")
                        .font(FontId::new(16.0, egui::FontFamily::Proportional))
                        .color(colors::TEXT_SECONDARY),
                );

                ui.add_space(40.0);

                // 登录卡片
                egui::Frame::new()
                    .fill(colors::BG_CARD)
                    .corner_radius(CornerRadius::same(12))
                    .stroke(egui::Stroke::new(1.0, colors::BORDER))
                    .show(ui, |ui| {
                        ui.set_max_width(380.0);
                        ui.set_min_width(320.0);

                        ui.add_space(32.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new("欢迎回来")
                                    .font(FontId::new(24.0, egui::FontFamily::Proportional))
                                    .color(colors::TEXT_PRIMARY),
                            );
                        });
                        ui.add_space(24.0);

                        egui::Grid::new("login_form")
                            .min_col_width(80.0)
                            .max_col_width(80.0)
                            .show(ui, |ui| {
                                ui.label("用户名");
                                ui.add(
                                    egui::TextEdit::singleline(&mut username)
                                        .hint_text("请输入用户名")
                                        .desired_width(220.0)
                                        .font(FontId::new(14.0, egui::FontFamily::Proportional)),
                                );
                                ui.end_row();

                                ui.label("密码");
                                ui.add(
                                    egui::TextEdit::singleline(&mut password)
                                        .hint_text("请输入密码")
                                        .password(true)
                                        .desired_width(220.0)
                                        .font(FontId::new(14.0, egui::FontFamily::Proportional)),
                                );
                                ui.end_row();
                            });

                        ui.add_space(24.0);

                        // 登录按钮
                        let btn = egui::Button::new(
                            egui::RichText::new("登  录")
                                .font(FontId::new(16.0, egui::FontFamily::Proportional))
                                .color(colors::TEXT_ON_PRIMARY),
                        )
                        .fill(colors::PRIMARY)
                        .min_size(egui::Vec2::new(280.0, 44.0))
                        .corner_radius(CornerRadius::same(8));

                        if ui.add(btn).clicked() {
                            // 模拟登录
                            if !username.is_empty() && !password.is_empty() {
                                app.is_logged_in = true;
                                app.token = Some("mock_token".to_string());
                                app.current_user = Some(crate::api::UserInfo {
                                    user_id: 1,
                                    user_name: username.clone(),
                                    nick_name: username.clone(),
                                    avatar: String::new(),
                                    email: String::new(),
                                    phonenumber: String::new(),
                                    sex: String::new(),
                                });
                                app.navigate(Page::Home);
                                app.add_toast("登录成功，欢迎回来！", ToastLevel::Success);
                            } else {
                                app.add_toast("请输入用户名和密码", ToastLevel::Warning);
                            }
                        }

                        ui.add_space(12.0);

                        // 注册/忘记密码链接
                        ui.horizontal(|ui| {
                            ui.add_space(40.0);
                            let register_link = egui::Link::new("注册账号");
                            if ui.add(register_link).clicked() {
                                app.navigate(Page::Register);
                            }
                            ui.add_space(40.0);
                            let forget_link = egui::Link::new("忘记密码");
                            if ui.add(forget_link).clicked() {
                                app.navigate(Page::ForgetPassword);
                            }
                        });

                        ui.add_space(32.0);
                    });
            });
        });
}