// 登录页面
// Metro Design 风格：居中卡片、大标题、扁平输入框
// 支持验证码、异步登录、封禁检查

use crate::app::{AuthPage, PezMaxApp};
use crate::theme::colors;
use egui::{CornerRadius, FontId};

pub fn render(app: &mut PezMaxApp, ctx: &egui::Context) {
    // 首次渲染时自动加载验证码
    if !app.captcha_loaded && app.captcha_rx.is_none() {
        app.trigger_captcha_load();
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(colors::BG_WHITE))
        .show(ctx, |ui| {
            // 垂直居中
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.12);

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

                ui.add_space(36.0);

                // 登录卡片
                egui::Frame::new()
                    .fill(colors::BG_CARD)
                    .corner_radius(CornerRadius::same(0))
                    .stroke(egui::Stroke::new(1.0, colors::BORDER))
                    .show(ui, |ui| {
                        ui.set_max_width(380.0);
                        ui.set_min_width(320.0);

                        ui.add_space(28.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new("欢迎回来")
                                    .font(FontId::new(24.0, egui::FontFamily::Proportional))
                                    .color(colors::TEXT_PRIMARY),
                            );
                        });
                        ui.add_space(20.0);

                        // ── 用户名 ──────────────────────────────────────
                        ui.horizontal(|ui| {
                            ui.add_space(24.0);
                            ui.label(
                                egui::RichText::new("用户名")
                                    .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                    .color(colors::TEXT_PRIMARY),
                            );
                            ui.add(
                                egui::TextEdit::singleline(&mut app.login_username)
                                    .hint_text("请输入用户名")
                                    .desired_width(200.0)
                                    .font(FontId::new(14.0, egui::FontFamily::Proportional)),
                            );
                        });

                        ui.add_space(12.0);

                        // ── 密码 ────────────────────────────────────────
                        ui.horizontal(|ui| {
                            ui.add_space(24.0);
                            ui.label(
                                egui::RichText::new("密  码")
                                    .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                    .color(colors::TEXT_PRIMARY),
                            );
                            ui.add(
                                egui::TextEdit::singleline(&mut app.login_password)
                                    .hint_text("请输入密码")
                                    .password(true)
                                    .desired_width(200.0)
                                    .font(FontId::new(14.0, egui::FontFamily::Proportional)),
                            );
                        });

                        // ── 验证码 ──────────────────────────────────────
                        if app.login_captcha_enabled {
                            ui.add_space(12.0);
                            ui.horizontal(|ui| {
                                ui.add_space(24.0);
                                ui.label(
                                    egui::RichText::new("验证码")
                                        .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                        .color(colors::TEXT_PRIMARY),
                                );
                                ui.add(
                                    egui::TextEdit::singleline(&mut app.login_captcha)
                                        .hint_text("验证码")
                                        .desired_width(100.0)
                                        .font(FontId::new(14.0, egui::FontFamily::Proportional)),
                                );
                                ui.add_space(8.0);

                                // 验证码图片
                                if let Some(texture) = &app.login_captcha_texture {
                                    let img = egui::Image::new(texture)
                                        .max_height(36.0)
                                        .max_width(100.0);
                                    if ui.add(img).clicked() {
                                        // 点击刷新验证码
                                        app.captcha_loaded = false;
                                        app.login_captcha_texture = None;
                                        ctx.request_repaint();
                                    }
                                } else if app.captcha_loaded {
                                    ui.label(
                                        egui::RichText::new("验证码加载失败")
                                            .color(colors::TEXT_SECONDARY)
                                            .font(FontId::new(12.0, egui::FontFamily::Proportional)),
                                    );
                                } else {
                                    ui.label(
                                        egui::RichText::new("加载中...")
                                            .color(colors::TEXT_SECONDARY)
                                            .font(FontId::new(12.0, egui::FontFamily::Proportional)),
                                    );
                                }
                            });
                        }

                        ui.add_space(20.0);

                        // ── 错误提示 ────────────────────────────────────
                        if !app.login_error.is_empty() {
                            ui.horizontal(|ui| {
                                ui.add_space(24.0);
                                ui.label(
                                    egui::RichText::new(&app.login_error)
                                        .color(colors::ERROR)
                                        .font(FontId::new(13.0, egui::FontFamily::Proportional)),
                                );
                            });
                            ui.add_space(12.0);
                        }

                        // ── 登录按钮 ────────────────────────────────────
                        let btn_label = if app.login_loading {
                            "登录中..."
                        } else {
                            "登  录"
                        };
                        let btn = egui::Button::new(
                            egui::RichText::new(btn_label)
                                .font(FontId::new(16.0, egui::FontFamily::Proportional))
                                .color(colors::TEXT_ON_PRIMARY),
                        )
                        .fill(colors::PRIMARY)
                        .min_size(egui::Vec2::new(280.0, 44.0))
                        .corner_radius(CornerRadius::same(0));

                        if ui.add(btn).clicked() && !app.login_loading {
                            if app.login_username.is_empty() || app.login_password.is_empty() {
                                app.login_error = "请输入用户名和密码".to_string();
                            } else if app.login_captcha_enabled && app.login_captcha.is_empty() {
                                app.login_error = "请输入验证码".to_string();
                            } else {
                                app.trigger_login();
                            }
                        }

                        // 按 Enter 键触发登录
                        if ui.input(|i| i.key_pressed(egui::Key::Enter)) && !app.login_loading {
                            if app.login_username.is_empty() || app.login_password.is_empty() {
                                app.login_error = "请输入用户名和密码".to_string();
                            } else if app.login_captcha_enabled && app.login_captcha.is_empty() {
                                app.login_error = "请输入验证码".to_string();
                            } else {
                                app.trigger_login();
                            }
                        }

                        ui.add_space(12.0);

                        // ── 注册 / 忘记密码 ─────────────────────────────
                        ui.horizontal(|ui| {
                            ui.add_space(36.0);
                            let register_link = egui::Link::new("注册账号");
                            if ui.add(register_link).clicked() {
                                app.set_auth_page(AuthPage::Register);
                                app.login_error.clear();
                            }
                            ui.add_space(48.0);
                            let forget_link = egui::Link::new("忘记密码");
                            if ui.add(forget_link).clicked() {
                                app.set_auth_page(AuthPage::ForgetPassword);
                                app.login_error.clear();
                            }
                        });

                        ui.add_space(28.0);
                    });
            });
        });
}