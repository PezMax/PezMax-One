//! 找回密码 — 3 步向导：用户名+验证码 → 3 个密保答题 → 新密码。

use crate::app::{AuthPage, PezMaxApp};
use crate::components::step_indicator;
use crate::theme::colors;
use egui::{Color32, CornerRadius, FontId, Stroke};

const STEP_LABELS: [&str; 3] = ["身份核对", "密保答题", "重置密码"];

pub fn render(app: &mut PezMaxApp, ctx: &egui::Context) {
    if app.forget_captcha_uuid.is_empty()
        && app.forget_captcha_rx.is_none()
        && app.forget_captcha_texture.is_none()
    {
        app.trigger_forget_captcha();
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(colors::bg_white()))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.08);
                ui.label(
                    egui::RichText::new("找回密码")
                        .font(FontId::new(28.0, egui::FontFamily::Proportional))
                        .color(colors::text_primary()),
                );
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new("通过密保问题重置您的密码")
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary()),
                );

                ui.add_space(24.0);

                let step_val = app.auth_step_anim.value() as f32;
                step_indicator::render(ui, step_val, 3, &STEP_LABELS);

                ui.add_space(20.0);

                egui::Frame::new()
                    .fill(colors::bg_card())
                    .corner_radius(CornerRadius::ZERO)
                    .stroke(Stroke::new(1.0, colors::border()))
                    .inner_margin(egui::Margin::symmetric(28, 24))
                    .show(ui, |ui| {
                        ui.set_max_width(440.0);
                        ui.set_min_width(440.0);
                        match app.forget_step {
                            1 => render_step_username(app, ui),
                            2 => render_step_answers(app, ui),
                            3 => render_step_new_password(app, ui),
                            _ => {}
                        }
                    });

                ui.add_space(12.0);
                if !app.forget_error.is_empty() {
                    ui.label(
                        egui::RichText::new(&app.forget_error)
                            .font(FontId::new(12.0, egui::FontFamily::Proportional))
                            .color(Color32::from_rgb(200, 30, 30)),
                    );
                    ui.add_space(6.0);
                }

                ui.horizontal(|ui| {
                    ui.add_space(ui.available_width() / 2.0 - 50.0);
                    if link_btn(ui, "返回登录").clicked() {
                        app.reset_forget_flow();
                        app.auth_page = AuthPage::Login;
                    }
                });
            });
        });
}

fn render_step_username(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    label(ui, "用户名");
    ui.add(
        egui::TextEdit::singleline(&mut app.forget_username)
            .hint_text("请输入注册时的用户名")
            .desired_width(f32::INFINITY),
    );
    ui.add_space(10.0);

    if app.forget_captcha_enabled {
        label(ui, "验证码");
        ui.horizontal(|ui| {
            let text_w = ui.available_width() - 132.0;
            ui.add(
                egui::TextEdit::singleline(&mut app.forget_captcha)
                    .hint_text("请输入验证码")
                    .desired_width(text_w),
            );
            ui.add_space(8.0);
            if let Some(tex) = &app.forget_captcha_texture {
                let img = egui::Image::new(tex)
                    .max_size(egui::vec2(120.0, 40.0))
                    .fit_to_exact_size(egui::vec2(120.0, 40.0));
                let resp = ui.add(img.sense(egui::Sense::click()));
                if resp.clicked() {
                    app.forget_captcha.clear();
                    app.forget_captcha_texture = None;
                    app.forget_captcha_uuid.clear();
                    app.trigger_forget_captcha();
                }
            } else {
                ui.label(
                    egui::RichText::new("加载中…")
                        .font(FontId::new(11.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary()),
                );
            }
        });
    }

    ui.add_space(16.0);
    ui.horizontal(|ui| {
        ui.add_space(ui.available_width() - 160.0);
        let text = if app.forget_loading { "加载密保…" } else { "下一步" };
        let btn = egui::Button::new(
            egui::RichText::new(text)
                .font(FontId::new(13.0, egui::FontFamily::Proportional))
                .color(colors::text_on_primary()),
        )
        .fill(colors::primary())
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::ZERO)
        .min_size(egui::vec2(148.0, 32.0));
        if ui.add_enabled(!app.forget_loading, btn).clicked() {
            if app.forget_username.trim().is_empty() {
                app.forget_error = "请输入用户名".to_string();
            } else if app.forget_captcha_enabled && app.forget_captcha.trim().is_empty() {
                app.forget_error = "请输入验证码".to_string();
            } else {
                app.forget_error.clear();
                app.trigger_forget_load_questions();
            }
        }
    });
}

fn render_step_answers(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new("请回答您注册时设置的 3 个密保问题")
            .font(FontId::new(13.0, egui::FontFamily::Proportional))
            .color(colors::text_secondary()),
    );
    ui.add_space(12.0);

    let n = app.forget_questions.len();
    for i in 0..n {
        let q_text = format!("问题 {}：{}", i + 1, app.forget_questions[i].question);
        label(ui, &q_text);
        ui.add(
            egui::TextEdit::singleline(&mut app.forget_questions[i].answer)
                .hint_text("请输入答案")
                .desired_width(f32::INFINITY),
        );
        ui.add_space(8.0);
    }

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if ghost_btn(ui, "上一步").clicked() {
            app.forget_step = 1;
            app.auth_step_anim.set_target(0.0);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if primary_btn(ui, "下一步").clicked() {
                if app.forget_questions.iter().any(|q| q.answer.trim().is_empty()) {
                    app.forget_error = "请回答全部 3 个密保问题".to_string();
                } else {
                    app.forget_error.clear();
                    app.forget_step = 3;
                    app.auth_step_anim.set_target(2.0);
                }
            }
        });
    });
}

fn render_step_new_password(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    ui.push_id("forget_new_pwd_scope", |ui| {
        label(ui, "新密码");
        ui.add(
            egui::TextEdit::singleline(&mut app.forget_new_password)
                .password(true)
                .hint_text("至少 6 位")
                .desired_width(f32::INFINITY),
        );
    });
    ui.add_space(10.0);
    ui.push_id("forget_confirm_pwd_scope", |ui| {
        label(ui, "确认新密码");
        ui.add(
            egui::TextEdit::singleline(&mut app.forget_confirm_password)
                .password(true)
                .hint_text("再次输入")
                .desired_width(f32::INFINITY),
        );
    });

    ui.add_space(16.0);
    ui.horizontal(|ui| {
        if ghost_btn(ui, "上一步").clicked() {
            app.forget_step = 2;
            app.auth_step_anim.set_target(1.0);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let text = if app.forget_loading { "提交中…" } else { "重置密码" };
            let btn = egui::Button::new(
                egui::RichText::new(text)
                    .font(FontId::new(13.0, egui::FontFamily::Proportional))
                    .color(colors::text_on_primary()),
            )
            .fill(colors::primary())
            .stroke(Stroke::NONE)
            .corner_radius(CornerRadius::ZERO)
            .min_size(egui::vec2(148.0, 32.0));
            if ui.add_enabled(!app.forget_loading, btn).clicked() {
                let new_pwd = app.forget_new_password.trim();
                let confirm_pwd = app.forget_confirm_password.trim();
                if new_pwd.chars().count() < 6 {
                    app.forget_error = "密码至少 6 位".to_string();
                } else if new_pwd != confirm_pwd {
                    app.forget_error = "两次输入的密码不一致".to_string();
                } else {
                    app.forget_error.clear();
                    app.trigger_forget_reset();
                }
            }
        });
    });
}

fn label(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .font(FontId::new(12.0, egui::FontFamily::Proportional))
            .color(colors::text_secondary()),
    );
    ui.add_space(2.0);
}

fn primary_btn(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(
            egui::RichText::new(text)
                .font(FontId::new(13.0, egui::FontFamily::Proportional))
                .color(colors::text_on_primary()),
        )
        .fill(colors::primary())
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::ZERO)
        .min_size(egui::vec2(120.0, 32.0)),
    )
}

fn ghost_btn(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(
            egui::RichText::new(text)
                .font(FontId::new(13.0, egui::FontFamily::Proportional))
                .color(colors::text_secondary()),
        )
        .fill(colors::bg_input())
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::ZERO)
        .min_size(egui::vec2(96.0, 32.0)),
    )
}

fn link_btn(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.scope(|ui| {
        ui.visuals_mut().widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
        ui.add(
            egui::Button::new(
                egui::RichText::new(text)
                    .font(FontId::new(12.0, egui::FontFamily::Proportional))
                    .color(colors::primary()),
            )
            .stroke(Stroke::NONE)
            .corner_radius(CornerRadius::ZERO),
        )
    }).inner
}
