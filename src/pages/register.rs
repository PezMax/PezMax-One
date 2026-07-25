//! 注册页 — 4 步向导 + 免责声明弹窗。
//!
//! 步骤：
//! 1. 账号信息（username / password / confirm）
//! 2. 密保问题 1
//! 3. 密保问题 2
//! 4. 密保问题 3 + 验证码 → 打开免责声明弹窗 → 提交

use crate::app::{AuthPage, PezMaxApp};
use crate::components::{disclaimer_dialog, step_indicator};
use crate::theme::colors;
use egui::{Color32, CornerRadius, FontId, Stroke};

const STEP_LABELS: [&str; 4] = ["账号信息", "密保 1", "密保 2", "密保 3 + 验证码"];

const DISCLAIMER_BODY: &str = "本软件为学生课余交流使用，与任何学校无官方关联。\n\n\
您通过本软件上传或分享的资料仅供个人学习交流之用，不得涉及商业用途。\n\n\
本软件不承担因用户上传内容引起的任何法律纠纷。\n\n\
若您继续注册，即表示您已阅读并同意本条款。";

pub fn render(app: &mut PezMaxApp, ctx: &egui::Context) {
    // 首次进入拉验证码
    if app.register_captcha_uuid.is_empty()
        && app.register_captcha_rx.is_none()
        && app.register_captcha_texture.is_none()
    {
        app.trigger_register_captcha();
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(colors::bg_white()))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.06);
                ui.label(
                    egui::RichText::new("创建账号")
                        .font(FontId::new(28.0, egui::FontFamily::Proportional))
                        .color(colors::text_primary()),
                );
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new("注册后即可浏览与下载试卷资源")
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary()),
                );

                ui.add_space(24.0);

                // 步骤指示器
                let step_val = app.auth_step_anim.value() as f32;
                step_indicator::render(ui, step_val, 4, &STEP_LABELS);

                ui.add_space(20.0);

                // 卡片
                egui::Frame::new()
                    .fill(colors::bg_card())
                    .corner_radius(CornerRadius::ZERO)
                    .stroke(Stroke::new(1.0, colors::border()))
                    .inner_margin(egui::Margin::symmetric(28, 24))
                    .show(ui, |ui| {
                        ui.set_max_width(420.0);
                        ui.set_min_width(420.0);
                        match app.register_step {
                            1 => render_step_account(app, ui),
                            2 => render_step_security(app, ui, 0),
                            3 => render_step_security(app, ui, 1),
                            4 => render_step_final(app, ui),
                            _ => {}
                        }
                    });

                ui.add_space(12.0);
                if !app.register_error.is_empty() {
                    ui.label(
                        egui::RichText::new(&app.register_error)
                            .font(FontId::new(12.0, egui::FontFamily::Proportional))
                            .color(Color32::from_rgb(200, 30, 30)),
                    );
                    ui.add_space(6.0);
                }

                // 返回登录
                ui.horizontal(|ui| {
                    ui.add_space(ui.available_width() / 2.0 - 60.0);
                    if link_btn(ui, "已有账号？返回登录").clicked() {
                        app.reset_register_flow();
                        app.auth_page = AuthPage::Login;
                    }
                });
            });
        });

    // 免责声明弹窗
    let (confirmed, closed) = disclaimer_dialog::render(
        ctx,
        app.register_disclaimer_open,
        &app.register_disclaimer_countdown,
        DISCLAIMER_BODY,
    );
    if confirmed {
        app.register_disclaimer_open = false;
        app.trigger_register();
    }
    if closed {
        app.register_disclaimer_open = false;
    }
}

// ── 步骤内容 ────────────────────────────────────────────────

fn render_step_account(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    label(ui, "用户名");
    ui.add(
        egui::TextEdit::singleline(&mut app.register_username)
            .hint_text("2-20 位字符，不要用手机号/学号")
            .desired_width(f32::INFINITY),
    );
    ui.add_space(10.0);
    label(ui, "密码");
    ui.add(
        egui::TextEdit::singleline(&mut app.register_password)
            .password(true)
            .hint_text("至少 6 位")
            .desired_width(f32::INFINITY),
    );
    ui.add_space(10.0);
    label(ui, "确认密码");
    ui.add(
        egui::TextEdit::singleline(&mut app.register_confirm_password)
            .password(true)
            .hint_text("再次输入密码")
            .desired_width(f32::INFINITY),
    );

    ui.add_space(16.0);
    ui.horizontal(|ui| {
        ui.add_space(ui.available_width() - 120.0);
        if primary_btn(ui, "下一步").clicked() && validate_step_account(app) {
            app.register_step = 2;
            app.auth_step_anim.set_target(1.0);
        }
    });
}

fn render_step_security(app: &mut PezMaxApp, ui: &mut egui::Ui, idx: usize) {
    let heading = match idx {
        0 => "密保问题 1",
        1 => "密保问题 2",
        _ => "密保问题 3",
    };
    ui.label(
        egui::RichText::new(heading)
            .font(FontId::new(15.0, egui::FontFamily::Proportional))
            .color(colors::text_primary())
            .strong(),
    );
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new("用于账号找回，请填写只有你自己知道的问题与答案")
            .font(FontId::new(11.0, egui::FontFamily::Proportional))
            .color(colors::text_secondary()),
    );
    ui.add_space(12.0);

    label(ui, "问题");
    ui.add(
        egui::TextEdit::singleline(&mut app.register_security_questions[idx].question)
            .hint_text("如：我的第一只宠物叫什么名字？")
            .desired_width(f32::INFINITY),
    );
    ui.add_space(10.0);
    label(ui, "答案");
    ui.add(
        egui::TextEdit::singleline(&mut app.register_security_questions[idx].answer)
            .hint_text("答案区分大小写")
            .desired_width(f32::INFINITY),
    );

    ui.add_space(16.0);
    ui.horizontal(|ui| {
        if ghost_btn(ui, "上一步").clicked() {
            app.register_step -= 1;
            app.auth_step_anim.set_target((app.register_step - 1) as f64);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if primary_btn(ui, "下一步").clicked() && validate_step_security(app, idx) {
                app.register_step += 1;
                app.auth_step_anim.set_target((app.register_step - 1) as f64);
            }
        });
    });
}

fn render_step_final(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new("最后一步 · 密保 3 + 验证码")
            .font(FontId::new(15.0, egui::FontFamily::Proportional))
            .color(colors::text_primary())
            .strong(),
    );
    ui.add_space(12.0);

    label(ui, "密保问题 3");
    ui.add(
        egui::TextEdit::singleline(&mut app.register_security_questions[2].question)
            .hint_text("请输入问题")
            .desired_width(f32::INFINITY),
    );
    ui.add_space(8.0);
    label(ui, "密保答案 3");
    ui.add(
        egui::TextEdit::singleline(&mut app.register_security_questions[2].answer)
            .hint_text("请输入答案")
            .desired_width(f32::INFINITY),
    );

    ui.add_space(14.0);

    if app.register_captcha_enabled {
        label(ui, "验证码");
        ui.horizontal(|ui| {
            let text_w = ui.available_width() - 132.0;
            ui.add(
                egui::TextEdit::singleline(&mut app.register_captcha)
                    .hint_text("请输入验证码")
                    .desired_width(text_w),
            );
            ui.add_space(8.0);
            if let Some(tex) = &app.register_captcha_texture {
                let img = egui::Image::new(tex)
                    .max_size(egui::vec2(120.0, 40.0))
                    .fit_to_exact_size(egui::vec2(120.0, 40.0));
                let resp = ui.add(img.sense(egui::Sense::click()));
                if resp.clicked() {
                    app.register_captcha.clear();
                    app.register_captcha_texture = None;
                    app.register_captcha_uuid.clear();
                    app.trigger_register_captcha();
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
        if ghost_btn(ui, "上一步").clicked() {
            app.register_step -= 1;
            app.auth_step_anim.set_target((app.register_step - 1) as f64);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let text = if app.register_loading { "注册中…" } else { "阅读免责声明并注册" };
            let resp = metro_button(
                ui,
                text,
                13.0,
                egui::vec2(180.0, 32.0),
                colors::primary(),
                colors::text_on_primary(),
                true,
                !app.register_loading,
            );
            if resp.clicked() && !app.register_loading && validate_step_final(app) {
                app.register_error.clear();
                app.register_disclaimer_open = true;
                app.register_disclaimer_countdown.jump_to(0.0);
                app.register_disclaimer_countdown.set_target(1.0);
            }
        });
    });
}

// ── 校验 ────────────────────────────────────────────────

fn validate_step_account(app: &mut PezMaxApp) -> bool {
    let u = app.register_username.trim();
    if u.len() < 2 || u.len() > 20 {
        app.register_error = "用户名长度需在 2-20 位".to_string();
        return false;
    }
    if app.register_password.len() < 6 {
        app.register_error = "密码至少 6 位".to_string();
        return false;
    }
    if app.register_password != app.register_confirm_password {
        app.register_error = "两次输入的密码不一致".to_string();
        return false;
    }
    app.register_error.clear();
    true
}

fn validate_step_security(app: &mut PezMaxApp, idx: usize) -> bool {
    let q = &app.register_security_questions[idx];
    if q.question.trim().is_empty() || q.answer.trim().is_empty() {
        app.register_error = "问题与答案均不能为空".to_string();
        return false;
    }
    app.register_error.clear();
    true
}

fn validate_step_final(app: &mut PezMaxApp) -> bool {
    if !validate_step_security(app, 2) { return false; }
    if app.register_captcha_enabled && app.register_captcha.trim().is_empty() {
        app.register_error = "请输入验证码".to_string();
        return false;
    }
    true
}

// ── 小组件 ──────────────────────────────────────────────

fn label(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .font(FontId::new(12.0, egui::FontFamily::Proportional))
            .color(colors::text_secondary()),
    );
    ui.add_space(2.0);
}

/// Metro 强调色主按钮。手绘背景 + 文本，按 hover / press 切换色调，
/// 因为 `egui::Button::fill()` 会覆盖 hover 视觉。
fn primary_btn(ui: &mut egui::Ui, text: &str) -> egui::Response {
    metro_button(
        ui,
        text,
        13.0,
        egui::vec2(120.0, 32.0),
        colors::primary(),
        colors::text_on_primary(),
        /* is_primary = */ true,
        /* enabled  = */ true,
    )
}

fn ghost_btn(ui: &mut egui::Ui, text: &str) -> egui::Response {
    metro_button(
        ui,
        text,
        13.0,
        egui::vec2(96.0, 32.0),
        colors::bg_input(),
        colors::text_secondary(),
        false,
        true,
    )
}

fn link_btn(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let font = FontId::new(12.0, egui::FontFamily::Proportional);
    let galley = ui.painter().layout_no_wrap(
        text.to_string(),
        font.clone(),
        colors::primary(),
    );
    let desired = egui::vec2(galley.size().x + 8.0, 20.0);
    let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::click());
    let color = if response.is_pointer_button_down_on() {
        colors::primary_dark()
    } else if response.hovered() {
        colors::primary_light()
    } else {
        colors::primary()
    };
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        font,
        color,
    );
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

/// 通用 Metro 按钮渲染：手绘背景 + 描边 + 文本，带 hover / active 反馈。
/// `is_primary=true` 用于强调色按钮（hover 加深，active 更深）；否则灰底按钮（hover/active 略深）。
pub(crate) fn metro_button(
    ui: &mut egui::Ui,
    text: &str,
    font_size: f32,
    size: egui::Vec2,
    base_fill: Color32,
    text_color: Color32,
    is_primary: bool,
    enabled: bool,
) -> egui::Response {
    let font = FontId::new(font_size, egui::FontFamily::Proportional);
    let sense = if enabled { egui::Sense::click() } else { egui::Sense::hover() };
    let (rect, response) = ui.allocate_exact_size(size, sense);

    let (fill, stroke_color) = if !enabled {
        // 禁用态：底色半透明，无描边
        let f = Color32::from_rgba_premultiplied(
            base_fill.r(), base_fill.g(), base_fill.b(), 90,
        );
        (f, Color32::TRANSPARENT)
    } else if response.is_pointer_button_down_on() {
        if is_primary {
            (colors::primary_dark(), colors::primary_dark())
        } else {
            (colors::bg_selected(), colors::border())
        }
    } else if response.hovered() {
        if is_primary {
            // hover 时略微加深并加亮描边
            (darken(base_fill, 0.88), colors::primary_light())
        } else {
            (colors::bg_hover(), colors::border())
        }
    } else {
        (base_fill, Color32::TRANSPARENT)
    };

    ui.painter().rect_filled(rect, CornerRadius::ZERO, fill);
    if stroke_color != Color32::TRANSPARENT {
        ui.painter().rect_stroke(
            rect,
            CornerRadius::ZERO,
            Stroke::new(1.0, stroke_color),
            egui::StrokeKind::Inside,
        );
    }
    let final_text_color = if enabled {
        text_color
    } else {
        Color32::from_rgba_premultiplied(text_color.r(), text_color.g(), text_color.b(), 150)
    };
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        font,
        final_text_color,
    );

    if enabled && response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

fn darken(c: Color32, factor: f32) -> Color32 {
    let f = factor.clamp(0.0, 1.0);
    Color32::from_rgb(
        (c.r() as f32 * f).round().clamp(0.0, 255.0) as u8,
        (c.g() as f32 * f).round().clamp(0.0, 255.0) as u8,
        (c.b() as f32 * f).round().clamp(0.0, 255.0) as u8,
    )
}
