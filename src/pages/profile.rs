// 个人功能区 — Metro Design 重设计
// 五个子标签：个人中心 / 账号设置 / 通知 / 下载记录 / 设置
//
// 设计语言：
//   - 方角纯色块（CornerRadius::ZERO）
//   - 左边缘 3px 强调色条装饰
//   - 大字号数字 + 小号标签
//   - 双色调文字（primary / secondary）
//   - 内容卡片 bg_card + 1px 边框
//   - 悬停叠加色（primary color + 低透明度）

use crate::app::{AccountEditSection, PezMaxApp};
use crate::api::models::SecurityQuestion;
use crate::theme::colors;
use crate::theme::{ThemeMode, ACCENT_PRESETS};
use egui::{Color32, CornerRadius, FontId, Rect, Stroke, Vec2, pos2, StrokeKind};

// ── 公共组件 ─────────────────────────────────────────────────────────────────────

/// Metro 风格小节标题：3px 强调色竖条 + 标题文字
fn section_title(ui: &mut egui::Ui, text: &str) {
    ui.horizontal(|ui| {
        egui::Frame::new()
            .fill(colors::primary())
            .corner_radius(CornerRadius::ZERO)
            .show(ui, |ui| {
                ui.set_min_size(Vec2::new(3.0, 18.0));
                ui.set_max_size(Vec2::new(3.0, 18.0));
            });
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(text)
                .font(FontId::new(18.0, egui::FontFamily::Proportional))
                .color(colors::text_primary())
                .strong(),
        );
    });
}

/// 带小一号标题的 setting 小节标题
fn setting_section_title(ui: &mut egui::Ui, text: &str) {
    ui.horizontal(|ui| {
        egui::Frame::new()
            .fill(colors::primary())
            .corner_radius(CornerRadius::ZERO)
            .show(ui, |ui| {
                ui.set_min_size(Vec2::new(3.0, 14.0));
                ui.set_max_size(Vec2::new(3.0, 14.0));
            });
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(text)
                .font(FontId::new(15.0, egui::FontFamily::Proportional))
                .color(colors::text_primary())
                .strong(),
        );
    });
}

/// 纯色统计色块（匹配首页 render_metric_blocks 风格）
fn stat_block(ui: &mut egui::Ui, value: &str, label: &str, color: Color32, width: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 80.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, CornerRadius::ZERO, color);

    ui.painter().text(
        pos2(rect.center().x, rect.top() + 16.0),
        egui::Align2::CENTER_CENTER,
        value,
        FontId::new(28.0, egui::FontFamily::Proportional),
        colors::text_on_primary(),
    );
    ui.painter().text(
        pos2(rect.center().x, rect.bottom() - 14.0),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::new(12.0, egui::FontFamily::Proportional),
        colors::text_on_primary(),
    );
}

/// 空状态占位文字
fn empty_state(ui: &mut egui::Ui, icon: &str, text: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(48.0);
        ui.label(
            egui::RichText::new(icon)
                .font(FontId::new(36.0, egui::FontFamily::Proportional)),
        );
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(text)
                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                .color(colors::text_secondary()),
        );
        ui.add_space(48.0);
    });
}

// ── 个人中心 ─────────────────────────────────────────────────────────────────

pub fn render_personal_center(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    ui.add_space(8.0);
    section_title(ui, "个人中心");
    ui.add_space(16.0);

    egui::ScrollArea::vertical()
        .id_salt("profile_scroll")
        .show(ui, |ui| {
            if let Some(ref user) = app.current_user {
                let display_name = if user.nick_name.is_empty() { &user.user_name } else { &user.nick_name };
                let first_char = display_name.chars().next().unwrap_or('?').to_string();
                let (fav, dl, ul) = app.user_stats.as_ref().map_or((0, 0, 0), |s| {
                    (s.favorite_count, s.download_count, s.upload_count)
                });

                // ── 顶部信息卡：头像 + 信息（左）| 统计（右）─────────
                let card_height = 100.0;
                let (rect, _) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), card_height),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(rect, CornerRadius::ZERO, colors::bg_card());
                ui.painter().rect_stroke(
                    rect,
                    CornerRadius::ZERO,
                    Stroke::new(1.0, colors::border()),
                    StrokeKind::Outside,
                );
                // 左边缘 3px 强调色条
                ui.painter().rect_filled(
                    Rect::from_min_max(pos2(rect.left(), rect.top()), pos2(rect.left() + 3.0, rect.bottom())),
                    CornerRadius::ZERO,
                    colors::primary(),
                );

                // 左半区：头像 + 文字
                let avatar_size = 72.0;
                let avatar_rect = Rect::from_min_size(
                    pos2(rect.left() + 20.0, rect.top() + (card_height - avatar_size) / 2.0),
                    Vec2::splat(avatar_size),
                );
                if let Some(tex) = &app.avatar_texture {
                    let uv = calc_center_crop_uv(app.avatar_image_size, avatar_size);
                    ui.painter().image(tex.id(), avatar_rect, uv, Color32::WHITE);
                } else {
                    ui.painter().rect_filled(avatar_rect, CornerRadius::ZERO, colors::primary());
                    ui.painter().text(
                        avatar_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        &first_char,
                        FontId::new(34.0, egui::FontFamily::Proportional),
                        colors::text_on_primary(),
                    );
                }

                // 显示名称
                ui.painter().text(
                    pos2(avatar_rect.right() + 16.0, rect.top() + 28.0),
                    egui::Align2::LEFT_CENTER,
                    display_name,
                    FontId::new(24.0, egui::FontFamily::Proportional),
                    colors::text_primary(),
                );
                // 用户名
                ui.painter().text(
                    pos2(avatar_rect.right() + 16.0, rect.top() + 60.0),
                    egui::Align2::LEFT_CENTER,
                    format!("@{}", user.user_name),
                    FontId::new(14.0, egui::FontFamily::Proportional),
                    colors::text_secondary(),
                );

                // 右半区：统计列
                let stat_x = rect.right() - 140.0;
                let stat_items = [
                    (format!("{}", dl), "下载量"),
                    (format!("{}", fav), "收藏数"),
                    (format!("{}", ul), "上传数"),
                ];
                let stat_gap = 28.0;
                let stat_start_y = rect.top() + (card_height - (stat_items.len() as f32 * stat_gap)) / 2.0 + stat_gap / 2.0;
                for (i, (value, label)) in stat_items.iter().enumerate() {
                    let y = stat_start_y + i as f32 * stat_gap;
                    // 数值
                    ui.painter().text(
                        pos2(stat_x, y),
                        egui::Align2::LEFT_CENTER,
                        value,
                        FontId::new(18.0, egui::FontFamily::Proportional),
                        colors::text_primary(),
                    );
                    // 标签
                    ui.painter().text(
                        pos2(stat_x + 52.0, y),
                        egui::Align2::LEFT_CENTER,
                        *label,
                        FontId::new(13.0, egui::FontFamily::Proportional),
                        colors::text_secondary(),
                    );
                }

                ui.add_space(16.0);

                // ── 账号设置区域 ─────────────────────────────────────
                let section_label = match app.account_edit_section {
                    AccountEditSection::None => "账号设置",
                    AccountEditSection::Avatar => "修改头像",
                    AccountEditSection::Username => "修改用户名",
                    AccountEditSection::Security => "修改密保问题",
                    AccountEditSection::Password => "修改登录密码",
                };
                section_title(ui, section_label);
                ui.add_space(12.0);

                // 成功/错误提示
                if !app.account_edit_error.is_empty() {
                    ui.label(
                        egui::RichText::new(&app.account_edit_error)
                            .font(FontId::new(13.0, egui::FontFamily::Proportional))
                            .color(Color32::RED),
                    );
                    ui.add_space(4.0);
                }
                if !app.account_edit_success.is_empty() {
                    ui.label(
                        egui::RichText::new(&app.account_edit_success)
                            .font(FontId::new(13.0, egui::FontFamily::Proportional))
                            .color(Color32::GREEN),
                    );
                    ui.add_space(4.0);
                }

                // 当前编辑区域
                match app.account_edit_section {
                    AccountEditSection::None => render_account_settings_list(app, ui),
                    AccountEditSection::Avatar => render_avatar_edit(app, ui),
                    AccountEditSection::Username => render_username_edit(app, ui),
                    AccountEditSection::Security => render_security_edit(app, ui),
                    AccountEditSection::Password => render_password_edit(app, ui),
                }
            } else {
                empty_state(ui, "👤", "用户信息加载中...");
            }

            ui.add_space(24.0);
        });
}


/// 账号设置主列表
fn render_account_settings_list(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    let user = match &app.current_user {
        Some(u) => u,
        None => {
            empty_state(ui, "👤", "用户信息加载中...");
            return;
        }
    };

    // 头像
    settings_card_row(ui, "头像", "个人头像，展示在个人中心和各页面", |_ui| {}, |ui| {
        if edit_button(ui, "更换") {
            app.account_edit_section = AccountEditSection::Avatar;
        }
    });

    ui.add_space(4.0);

    // 用户名
    settings_card_row(ui, "用户名", "用于登录和个人信息展示", |ui| {
        ui.label(
            egui::RichText::new(&user.user_name)
                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                .color(colors::text_secondary()),
        );
    }, |ui| {
        if edit_button(ui, "修改") {
            app.account_edit_username = user.user_name.clone();
            app.account_edit_section = AccountEditSection::Username;
        }
    });

    ui.add_space(4.0);

    // 密保问题
    settings_card_row(ui, "密保问题", "用于账号找回的安全验证", |_| {}, |ui| {
        if edit_button(ui, "修改") {
            app.account_edit_section = AccountEditSection::Security;
        }
    });

    ui.add_space(4.0);

    // 登录密码
    settings_card_row(ui, "登录密码", "定期更换密码保护账号安全", |_| {}, |ui| {
        if edit_button(ui, "修改") {
            app.account_edit_section = AccountEditSection::Password;
        }
    });

    ui.add_space(12.0);

    // ── 退出登录 ─────────────────────────────────────
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), 64.0),
        egui::Sense::hover(),
    );
    ui.painter().rect_filled(rect, CornerRadius::ZERO, colors::bg_card());
    ui.painter().rect_stroke(
        rect,
        CornerRadius::ZERO,
        Stroke::new(1.0, colors::border()),
        StrokeKind::Outside,
    );
    // 左边缘 3px 强调色条
    ui.painter().rect_filled(
        Rect::from_min_max(pos2(rect.left(), rect.top()), pos2(rect.left() + 3.0, rect.bottom())),
        CornerRadius::ZERO,
        colors::primary(),
    );

    // 左侧：标签 + 描述
    let left_rect = Rect::from_min_max(
        pos2(rect.left() + 20.0, rect.top() + 6.0),
        pos2(rect.right() - 76.0, rect.bottom()),
    );
    ui.allocate_ui_at_rect(left_rect, |ui| {
        ui.label(
            egui::RichText::new("退出登录")
                .font(FontId::new(15.0, egui::FontFamily::Proportional))
                .color(colors::text_primary()),
        );
        ui.label(
            egui::RichText::new("退出当前账号，返回登录页面")
                .font(FontId::new(12.0, egui::FontFamily::Proportional))
                .color(colors::text_secondary()),
        );
    });

    // 右侧：退出按钮
    let btn_rect = Rect::from_min_size(
        pos2(rect.right() - 70.0, rect.top() + 14.0),
        Vec2::new(60.0, 28.0),
    );
    ui.allocate_ui_at_rect(btn_rect, |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let btn = egui::Button::new(
                egui::RichText::new("退出")
                    .font(FontId::new(13.0, egui::FontFamily::Proportional))
                    .color(colors::primary()),
            )
            .fill(Color32::TRANSPARENT)
            .corner_radius(CornerRadius::ZERO)
            .min_size(Vec2::new(56.0, 28.0))
            .stroke(Stroke::new(1.0, colors::primary()));
            if ui.add(btn).clicked() {
                app.logout();
            }
        });
    });
}

/// 设置项卡片行（带左强调色条 + 标签 + 左侧内容 + 右侧按钮）
fn settings_card_row(
    ui: &mut egui::Ui,
    label: &str,
    desc: &str,
    add_left: impl FnOnce(&mut egui::Ui),
    add_right: impl FnOnce(&mut egui::Ui),
) {
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), 64.0),
        egui::Sense::hover(),
    );
    ui.painter().rect_filled(rect, CornerRadius::ZERO, colors::bg_card());
    ui.painter().rect_stroke(
        rect,
        CornerRadius::ZERO,
        Stroke::new(1.0, colors::border()),
        StrokeKind::Outside,
    );
    // 左边缘 3px 强调色条
    ui.painter().rect_filled(
        Rect::from_min_max(pos2(rect.left(), rect.top()), pos2(rect.left() + 3.0, rect.bottom())),
        CornerRadius::ZERO,
        colors::primary(),
    );

    // 左侧：标签 + 描述 + 值
    let left_rect = Rect::from_min_max(
        pos2(rect.left() + 20.0, rect.top() + 6.0),
        pos2(rect.right() - 76.0, rect.bottom()),
    );
    ui.allocate_ui_at_rect(left_rect, |ui| {
        ui.horizontal(|ui| {
            // 标签
            ui.label(
                egui::RichText::new(label)
                    .font(FontId::new(15.0, egui::FontFamily::Proportional))
                    .color(colors::text_primary()),
            );
            ui.add_space(8.0);
            // 值（左侧内容）
            add_left(ui);
        });
        // 描述
        ui.label(
            egui::RichText::new(desc)
                .font(FontId::new(12.0, egui::FontFamily::Proportional))
                .color(colors::text_secondary()),
        );
    });

    // 右侧：按钮区
    let btn_rect = Rect::from_min_size(
        pos2(rect.right() - 70.0, rect.top() + 14.0),
        Vec2::new(60.0, 28.0),
    );
    ui.allocate_ui_at_rect(btn_rect, |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            add_right(ui);
        });
    });
}

/// 编辑按钮（小号方角）
fn edit_button(ui: &mut egui::Ui, text: &str) -> bool {
    let btn = egui::Button::new(
        egui::RichText::new(text)
            .font(FontId::new(13.0, egui::FontFamily::Proportional))
            .color(colors::primary()),
    )
    .fill(Color32::TRANSPARENT)
    .corner_radius(CornerRadius::ZERO)
    .min_size(Vec2::new(56.0, 28.0))
    .stroke(Stroke::new(1.0, colors::primary()));
    ui.add(btn).clicked()
}

/// 计算居中裁剪的 UV 坐标，使任意比例图片以正方形居中显示
fn calc_center_crop_uv(image_size: Option<(usize, usize)>, target_size: f32) -> egui::Rect {
    let (w, h) = match image_size {
        Some((w, h)) if w > 0 && h > 0 => (w as f32, h as f32),
        _ => return egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
    };

    let aspect = w / h;
    if aspect > 1.0 {
        // 图片宽 > 高：左右裁剪
        let crop = (1.0 - 1.0 / aspect) / 2.0;
        egui::Rect::from_min_max(pos2(crop, 0.0), pos2(1.0 - crop, 1.0))
    } else if aspect < 1.0 {
        // 图片高 > 宽：上下裁剪
        let crop = (1.0 - aspect) / 2.0;
        egui::Rect::from_min_max(pos2(0.0, crop), pos2(1.0, 1.0 - crop))
    } else {
        // 正方形：完整显示
        egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0))
    }
}

/// 主要操作按钮（强调色填充）
fn primary_button(ui: &mut egui::Ui, text: &str, loading: bool) -> bool {
    let label = if loading {
        format!("⏳ {}", text)
    } else {
        text.to_string()
    };
    let btn = egui::Button::new(
        egui::RichText::new(label)
            .font(FontId::new(13.0, egui::FontFamily::Proportional))
            .color(colors::text_on_primary()),
    )
    .fill(if loading { colors::bg_input() } else { colors::primary() })
    .corner_radius(CornerRadius::ZERO)
    .min_size(Vec2::new(80.0, 32.0));
    if loading { return false; }
    ui.add(btn).clicked()
}

/// 次要按钮
fn secondary_button(ui: &mut egui::Ui, text: &str) -> bool {
    let btn = egui::Button::new(
        egui::RichText::new(text)
            .font(FontId::new(13.0, egui::FontFamily::Proportional))
            .color(colors::text_secondary()),
    )
    .fill(Color32::TRANSPARENT)
    .corner_radius(CornerRadius::ZERO)
    .min_size(Vec2::new(56.0, 28.0))
    .stroke(Stroke::new(1.0, colors::border()));
    ui.add(btn).clicked()
}

/// 编辑表单容器
fn edit_form(ui: &mut egui::Ui, title: &str, add_content: impl FnOnce(&mut egui::Ui)) {
    ui.add_space(4.0);
    egui::Frame::new()
        .fill(colors::bg_card())
        .corner_radius(CornerRadius::ZERO)
        .stroke(Stroke::new(1.0, colors::border()))
        .inner_margin(egui::Margin::symmetric(20, 16))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(
                egui::RichText::new(title)
                    .font(FontId::new(16.0, egui::FontFamily::Proportional))
                    .color(colors::text_primary())
                    .strong(),
            );
            ui.add_space(12.0);
            add_content(ui);
        });
}

// ── 头像编辑 ────────────────────────────────────────────────────────────────

fn render_avatar_edit(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    edit_form(ui, "修改头像", |ui| {
        // 当前头像显示
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("当前头像")
                    .font(FontId::new(14.0, egui::FontFamily::Proportional))
                    .color(colors::text_primary()),
            );
            let avatar_size = 64.0;
            if let Some(tex) = &app.avatar_texture {
                let (r, _) = ui.allocate_exact_size(Vec2::splat(avatar_size), egui::Sense::hover());
                ui.painter().image(
                    tex.id(),
                    r,
                    egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else if let Some(ref user) = app.current_user {
                let display_name = if user.nick_name.is_empty() { &user.user_name } else { &user.nick_name };
                let first_char = display_name.chars().next().unwrap_or('?').to_string();
                let (r, _) = ui.allocate_exact_size(Vec2::splat(avatar_size), egui::Sense::hover());
                ui.painter().rect_filled(r, CornerRadius::ZERO, colors::primary());
                ui.painter().text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    &first_char,
                    FontId::new(28.0, egui::FontFamily::Proportional),
                    colors::text_on_primary(),
                );
            }
        });

        ui.add_space(12.0);

        // 上传按钮
        if primary_button(ui, "选择图片并上传", false) {
            app.account_edit_error.clear();
            app.account_edit_success.clear();
            app.account_edit_message_timer = 0.0;

            let api = app.api.clone();
            tokio::spawn(async move {
                // 使用 rfd 打开文件选择对话框
                let file = rfd::AsyncFileDialog::new()
                    .add_filter("图片", &["jpg", "jpeg", "png", "gif"])
                    .pick_file()
                    .await;
                if let Some(file) = file {
                    let path = file.path().to_string_lossy().to_string();
                    match api.upload_avatar(&path).await {
                        Ok(resp) => {
                            log::info!("头像上传成功: {:?}", resp);
                        }
                        Err(e) => {
                            log::error!("头像上传失败: {}", e);
                        }
                    }
                }
            });
        }

        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("支持 JPG / PNG / GIF 格式，文件大小不超过 2MB")
                .font(FontId::new(12.0, egui::FontFamily::Proportional))
                .color(colors::text_secondary()),
        );

        ui.add_space(16.0);
        if secondary_button(ui, "返回") {
            app.account_edit_section = AccountEditSection::None;
            app.account_edit_loading = false;
            app.account_edit_error.clear();
            app.account_edit_success.clear();
            app.account_edit_message_timer = 0.0;
        }
    });
}

// ── 用户名编辑 ──────────────────────────────────────────────────────────────

fn render_username_edit(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    edit_form(ui, "修改用户名", |ui| {
        ui.label(
            egui::RichText::new("用户名将用于登录和个人信息展示")
                .font(FontId::new(12.0, egui::FontFamily::Proportional))
                .color(colors::text_secondary()),
        );
        ui.add_space(8.0);

        let resp = ui.add(
            egui::TextEdit::singleline(&mut app.account_edit_username)
                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                .text_color(colors::text_primary())
                .desired_width(240.0)
                .margin(egui::Vec2::new(8.0, 6.0))
                .hint_text("请输入新用户名"),
        );
        // 设置背景色
        let bg_rect = resp.rect;
        ui.painter().rect_filled(bg_rect, CornerRadius::ZERO, colors::bg_input());
        ui.painter().rect_stroke(
            bg_rect,
            CornerRadius::ZERO,
            Stroke::new(1.0, colors::border()),
            StrokeKind::Outside,
        );

        ui.add_space(16.0);

        ui.horizontal(|ui| {
            if primary_button(ui, "保存用户名", app.account_edit_loading) {
                let new_name = app.account_edit_username.trim().to_string();
                if new_name.len() < 2 || new_name.len() > 30 {
                    app.account_edit_error = "用户名长度应为 2-30 个字符".to_string();
                    app.account_edit_message_timer = 3.0;
                    return;
                }
                app.account_edit_error.clear();
                app.account_edit_success.clear();

                // 异步调用 API
                let api = app.api.clone();
                let name = new_name.clone();
                tokio::spawn(async move {
                    match api.update_username(&name).await {
                        Ok(resp) => {
                            if resp.code == 200 {
                                log::info!("用户名更新成功");
                            } else {
                                log::error!("用户名更新失败: {} {}", resp.code, resp.msg);
                            }
                        }
                        Err(e) => {
                            log::error!("用户名更新失败: {}", e);
                        }
                    }
                });
                // 本地更新
                if let Some(ref mut user) = app.current_user {
                    user.user_name = new_name;
                }
                app.account_edit_success = "用户名更新成功".to_string();
                app.account_edit_message_timer = 3.0;
                app.account_edit_section = AccountEditSection::None;
            }

            if secondary_button(ui, "取消") {
                app.account_edit_section = AccountEditSection::None;
                app.account_edit_error.clear();
                app.account_edit_success.clear();
            }
        });
    });
}

// ── 昵称编辑 ────────────────────────────────────────────────────────────────

// ── 密保问题编辑 ────────────────────────────────────────────────────────────

fn render_security_edit(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    // 初始化 3 组空密保问题
    if app.account_edit_security_questions.is_empty() {
        for _ in 0..3 {
            app.account_edit_security_questions.push(SecurityQuestion {
                question: String::new(),
                answer: String::new(),
            });
        }
    }

    edit_form(ui, "修改密保问题", |ui| {
        ui.label(
            egui::RichText::new("设置 3 组密保问题，用于账号找回时的安全验证")
                .font(FontId::new(12.0, egui::FontFamily::Proportional))
                .color(colors::text_secondary()),
        );
        ui.add_space(16.0);

        // 3 组密保问题输入
        for i in 0..3 {
            let q_item = &mut app.account_edit_security_questions[i];

            ui.label(
                egui::RichText::new(format!("密保 {}", i + 1))
                    .font(FontId::new(14.0, egui::FontFamily::Proportional))
                    .color(colors::text_primary())
                    .strong(),
            );
            ui.add_space(4.0);

            ui.label(
                egui::RichText::new("问题")
                    .font(FontId::new(12.0, egui::FontFamily::Proportional))
                    .color(colors::text_secondary()),
            );
            ui.add(
                egui::TextEdit::singleline(&mut q_item.question)
                    .font(FontId::new(14.0, egui::FontFamily::Proportional))
                    .text_color(colors::text_primary())
                    .desired_width(360.0)
                    .margin(egui::Vec2::new(8.0, 6.0))
                    .hint_text("请输入密保问题"),
            );

            ui.label(
                egui::RichText::new("答案")
                    .font(FontId::new(12.0, egui::FontFamily::Proportional))
                    .color(colors::text_secondary()),
            );
            ui.add(
                egui::TextEdit::singleline(&mut q_item.answer)
                    .font(FontId::new(14.0, egui::FontFamily::Proportional))
                    .text_color(colors::text_primary())
                    .desired_width(360.0)
                    .margin(egui::Vec2::new(8.0, 6.0))
                    .hint_text("请输入密保答案"),
            );

            ui.add_space(12.0);
        }

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            if primary_button(ui, "保存密保", app.account_edit_loading) {
                // 验证所有字段
                let all_filled = app.account_edit_security_questions.iter().all(|q| {
                    !q.question.trim().is_empty() && !q.answer.trim().is_empty()
                });
                if !all_filled {
                    app.account_edit_error = "请填写完整的 3 组密保问题和答案".to_string();
                    app.account_edit_message_timer = 3.0;
                    return;
                }

                app.account_edit_error.clear();
                app.account_edit_success.clear();

                let api = app.api.clone();
                let qs = app.account_edit_security_questions.clone();
                let data = serde_json::json!({
                    "securityQuestionOne": qs[0].question,
                    "securityAnswerOne": qs[0].answer,
                    "securityQuestionTwo": qs[1].question,
                    "securityAnswerTwo": qs[1].answer,
                    "securityQuestionThree": qs[2].question,
                    "securityAnswerThree": qs[2].answer,
                });
                tokio::spawn(async move {
                    match api.update_security(&data).await {
                        Ok(resp) => {
                            if resp.code == 200 {
                                log::info!("密保更新成功");
                            } else {
                                log::error!("密保更新失败: {} {}", resp.code, resp.msg);
                            }
                        }
                        Err(e) => {
                            log::error!("密保更新失败: {}", e);
                        }
                    }
                });
                app.account_edit_success = "密保问题已更新".to_string();
                app.account_edit_message_timer = 3.0;
                app.account_edit_section = AccountEditSection::None;
            }

            if secondary_button(ui, "取消") {
                app.account_edit_section = AccountEditSection::None;
                app.account_edit_security_questions.clear();
                app.account_edit_error.clear();
                app.account_edit_success.clear();
                app.account_edit_message_timer = 0.0;
            }
        });
    });
}

// ── 密码修改 ────────────────────────────────────────────────────────────────

fn render_password_edit(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    edit_form(ui, "修改登录密码", |ui| {
        ui.label(
            egui::RichText::new("请输入旧密码并设置新密码")
                .font(FontId::new(12.0, egui::FontFamily::Proportional))
                .color(colors::text_secondary()),
        );
        ui.add_space(12.0);

        // 旧密码
        ui.label(
            egui::RichText::new("旧密码")
                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                .color(colors::text_primary()),
        );
        ui.add(
            egui::TextEdit::singleline(&mut app.account_edit_old_password)
                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                .text_color(colors::text_primary())
                .desired_width(240.0)
                .margin(egui::Vec2::new(8.0, 6.0))
                .password(true)
                .hint_text("请输入旧密码"),
        );

        ui.add_space(8.0);

        // 新密码
        ui.label(
            egui::RichText::new("新密码")
                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                .color(colors::text_primary()),
        );
        ui.add(
            egui::TextEdit::singleline(&mut app.account_edit_new_password)
                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                .text_color(colors::text_primary())
                .desired_width(240.0)
                .margin(egui::Vec2::new(8.0, 6.0))
                .password(true)
                .hint_text("请输入新密码"),
        );

        ui.add_space(8.0);

        // 确认新密码
        ui.label(
            egui::RichText::new("确认新密码")
                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                .color(colors::text_primary()),
        );
        ui.add(
            egui::TextEdit::singleline(&mut app.account_edit_confirm_password)
                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                .text_color(colors::text_primary())
                .desired_width(240.0)
                .margin(egui::Vec2::new(8.0, 6.0))
                .password(true)
                .hint_text("请再次输入新密码"),
        );

        ui.add_space(16.0);

        ui.horizontal(|ui| {
            if primary_button(ui, "保存密码", app.account_edit_loading) {
                let old = app.account_edit_old_password.trim();
                let new = app.account_edit_new_password.trim();
                let confirm = app.account_edit_confirm_password.trim();

                if old.is_empty() || new.is_empty() || confirm.is_empty() {
                    app.account_edit_error = "请填写所有密码字段".to_string();
                    app.account_edit_message_timer = 3.0;
                    return;
                }
                if new != confirm {
                    app.account_edit_error = "两次输入的新密码不一致".to_string();
                    app.account_edit_message_timer = 3.0;
                    return;
                }
                if new.len() < 6 {
                    app.account_edit_error = "新密码长度不能少于 6 位".to_string();
                    app.account_edit_message_timer = 3.0;
                    return;
                }

                app.account_edit_error.clear();
                app.account_edit_success.clear();

                let api = app.api.clone();
                let old_pwd = old.to_string();
                let new_pwd = new.to_string();
                tokio::spawn(async move {
                    match api.update_password(&old_pwd, &new_pwd).await {
                        Ok(resp) => {
                            if resp.code == 200 {
                                log::info!("密码更新成功");
                            } else {
                                log::error!("密码更新失败: {} {}", resp.code, resp.msg);
                            }
                        }
                        Err(e) => {
                            log::error!("密码更新失败: {}", e);
                        }
                    }
                });
                app.account_edit_success = "密码已更新".to_string();
                app.account_edit_message_timer = 3.0;
                app.account_edit_old_password.clear();
                app.account_edit_new_password.clear();
                app.account_edit_confirm_password.clear();
                app.account_edit_section = AccountEditSection::None;
            }

            if secondary_button(ui, "取消") {
                app.account_edit_section = AccountEditSection::None;
                app.account_edit_old_password.clear();
                app.account_edit_new_password.clear();
                app.account_edit_confirm_password.clear();
                app.account_edit_error.clear();
                app.account_edit_success.clear();
            }
        });
    });
}

// ── 通知列表 ───────────────────────────────────────────────────────────────

pub fn render_notifications(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    // 自动加载通知
    if !app.notifications.is_loaded() && !app.notifications.is_loading() {
        app.trigger_load_notifications();
    }

    ui.add_space(8.0);
    section_title(ui, "通知");
    ui.add_space(16.0);

    egui::ScrollArea::vertical()
        .id_salt("notif_scroll")
        .show(ui, |ui| {
            if let Some(ref list) = app.notifications.data {
                if list.is_empty() {
                    empty_state(ui, "🔔", "暂无通知");
                    return;
                }

                for notif in list {
                    let is_read = notif.status == "1";
                    let accent = if is_read { colors::border() } else { colors::primary() };

                    let (rect, resp) = ui.allocate_exact_size(
                        Vec2::new(ui.available_width(), 80.0),
                        egui::Sense::click(),
                    );

                    let bg = if is_read { colors::bg_card() } else { colors::bg_selected() };
                    ui.painter().rect_filled(rect, CornerRadius::ZERO, bg);
                    ui.painter().rect_stroke(
                        rect,
                        CornerRadius::ZERO,
                        Stroke::new(1.0, colors::border()),
                        StrokeKind::Outside,
                    );

                    // 左边缘：未读用强调色，已读用边框色
                    let bar_w = if is_read { 2.0 } else { 3.0 };
                    ui.painter().rect_filled(
                        Rect::from_min_max(pos2(rect.left(), rect.top()), pos2(rect.left() + bar_w, rect.bottom())),
                        CornerRadius::ZERO,
                        accent,
                    );

                    // 标题
                    ui.painter().text(
                        pos2(rect.left() + 18.0, rect.top() + 16.0),
                        egui::Align2::LEFT_CENTER,
                        &notif.title,
                        FontId::new(15.0, egui::FontFamily::Proportional),
                        if is_read { colors::text_secondary() } else { colors::text_primary() },
                    );

                    // 内容
                    let content = if notif.content.len() > 60 {
                        format!("{}...", &notif.content[..60])
                    } else {
                        notif.content.clone()
                    };
                    ui.painter().text(
                        pos2(rect.left() + 18.0, rect.top() + 40.0),
                        egui::Align2::LEFT_CENTER,
                        &content,
                        FontId::new(13.0, egui::FontFamily::Proportional),
                        colors::text_secondary(),
                    );

                    // 时间
                    ui.painter().text(
                        pos2(rect.right() - 14.0, rect.top() + 16.0),
                        egui::Align2::RIGHT_CENTER,
                        &notif.create_time,
                        FontId::new(11.0, egui::FontFamily::Proportional),
                        colors::text_secondary(),
                    );

                    // 未读标记点
                    if !is_read {
                        let dot = Rect::from_min_size(
                            pos2(rect.left() + 18.0, rect.bottom() - 14.0),
                            Vec2::splat(6.0),
                        );
                        ui.painter().rect_filled(dot, CornerRadius::ZERO, colors::primary());
                    }

                    // 悬停效果
                    if resp.hovered() {
                        let c = colors::primary();
                        let overlay = Color32::from_rgba_premultiplied(c.r(), c.g(), c.b(), 10);
                        ui.painter().rect_filled(rect, CornerRadius::ZERO, overlay);
                    }

                    if resp.clicked() {
                        // 预留：点击通知跳转详情
                    }

                    ui.add_space(4.0);
                }
            } else if app.notifications.is_loading() {
                empty_state(ui, "⏳", "加载中...");
            } else {
                empty_state(ui, "🔔", "暂无通知");
            }
        });
}

// ── 下载记录 ───────────────────────────────────────────────────────────────

pub fn render_download_history(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    // 自动加载下载记录
    if !app.download_records.is_loaded() && !app.download_records.is_loading() {
        app.trigger_load_download_records();
    }

    ui.add_space(8.0);
    section_title(ui, "下载记录");
    ui.add_space(16.0);

    egui::ScrollArea::vertical()
        .id_salt("download_scroll")
        .show(ui, |ui| {
            if let Some(ref list) = app.download_records.data {
                if list.is_empty() {
                    empty_state(ui, "📥", "暂无下载记录");
                    return;
                }

                for record in list {
                    let (rect, resp) = ui.allocate_exact_size(
                        Vec2::new(ui.available_width(), 64.0),
                        egui::Sense::click(),
                    );

                    ui.painter().rect_filled(rect, CornerRadius::ZERO, colors::bg_card());
                    ui.painter().rect_stroke(
                        rect,
                        CornerRadius::ZERO,
                        Stroke::new(1.0, colors::border()),
                        StrokeKind::Outside,
                    );

                    // 左边缘 3px 强调色条
                    let accent = colors::primary();
                    ui.painter().rect_filled(
                        Rect::from_min_max(pos2(rect.left(), rect.top()), pos2(rect.left() + 3.0, rect.bottom())),
                        CornerRadius::ZERO,
                        accent,
                    );

                    // 文件图标
                    ui.painter().text(
                        pos2(rect.left() + 20.0, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        "📄",
                        FontId::new(20.0, egui::FontFamily::Proportional),
                        colors::text_primary(),
                    );

                    // 文件名
                    ui.painter().text(
                        pos2(rect.left() + 50.0, rect.top() + 16.0),
                        egui::Align2::LEFT_CENTER,
                        &record.file_name,
                        FontId::new(14.0, egui::FontFamily::Proportional),
                        colors::text_primary(),
                    );

                    // 格式标签 + 时间
                    let meta = format!("{} · {}", record.file_format, record.download_time);
                    ui.painter().text(
                        pos2(rect.left() + 50.0, rect.top() + 40.0),
                        egui::Align2::LEFT_CENTER,
                        &meta,
                        FontId::new(12.0, egui::FontFamily::Proportional),
                        colors::text_secondary(),
                    );

                    // 隐藏按钮
                    let hide_rect = Rect::from_min_size(
                        pos2(rect.right() - 68.0, rect.top() + 18.0),
                        Vec2::new(56.0, 28.0),
                    );
                    let hide_resp = ui.interact(hide_rect, ui.next_auto_id(), egui::Sense::click());
                    ui.painter().rect_stroke(
                        hide_rect,
                        CornerRadius::ZERO,
                        Stroke::new(1.0, colors::border()),
                        StrokeKind::Outside,
                    );
                    ui.painter().text(
                        hide_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "隐藏",
                        FontId::new(12.0, egui::FontFamily::Proportional),
                        colors::text_secondary(),
                    );
                    if hide_resp.hovered() {
                        ui.painter().rect_filled(hide_rect, CornerRadius::ZERO, colors::bg_hover());
                    }
                    if hide_resp.clicked() {
                        if let Some(ref user) = app.current_user {
                            let api = app.api.clone();
                            let uid = user.user_id;
                            let fid = record.file_id;
                            tokio::spawn(async move {
                                let _ = api.hide_download(uid, fid).await;
                            });
                        }
                    }

                    // 悬停效果
                    if resp.hovered() {
                        let c = colors::primary();
                        let overlay = Color32::from_rgba_premultiplied(c.r(), c.g(), c.b(), 10);
                        ui.painter().rect_filled(rect, CornerRadius::ZERO, overlay);
                    }

                    ui.add_space(4.0);
                }
            } else if app.download_records.is_loading() {
                empty_state(ui, "⏳", "加载中...");
            } else {
                empty_state(ui, "📥", "暂无下载记录");
            }
        });
}

// ── 应用设置 ───────────────────────────────────────────────────────────────

pub fn render_app_settings(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    ui.add_space(8.0);
    section_title(ui, "设置");
    ui.add_space(16.0);

    egui::ScrollArea::vertical()
        .id_salt("settings_scroll")
        .show(ui, |ui| {
            // ── 常规 ──────────────────────────────────────────────
            settings_group(ui, "常规", |ui| {
                toggle_row(ui, "开机自启", "登录时自动启动 PezMax", &mut app.setting_auto_launch);
            });

            ui.add_space(12.0);

            // ── 外观 ──────────────────────────────────────────────
            settings_group(ui, "外观", |ui| {
                theme_mode_row(ui, &mut app.theme_mode);
                ui.add_space(4.0);
                accent_color_row(ui, &mut app.accent_idx);
            });

            ui.add_space(12.0);

            // ── 下载 ──────────────────────────────────────────────
            settings_group(ui, "下载", |ui| {
                toggle_row(ui, "静默下载", "跳过保存路径选择，直接存入默认目录", &mut app.setting_silent_download);
                ui.add_space(4.0);
                info_row(ui, "默认路径", "~/Downloads/PezMax");
            });

            ui.add_space(12.0);

            // ── 隐私 ──────────────────────────────────────────────
            settings_group(ui, "隐私", |ui| {
                action_row(ui, "清理缓存", "释放本地缓存空间");
            });

            ui.add_space(12.0);

            // ── 关于 ──────────────────────────────────────────────
            settings_group(ui, "关于", |ui| {
                info_row(ui, "版本", "PezMax 0.1.0 · egui 版");
            });

            ui.add_space(24.0);
        });
}

// ── 内部组件 ──────────────────────────────────────────────────────────────────

/// 设置分组容器（带分组标题）
fn settings_group(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        egui::Frame::new()
            .fill(colors::primary())
            .corner_radius(CornerRadius::ZERO)
            .show(ui, |ui| {
                ui.set_min_size(Vec2::new(3.0, 14.0));
                ui.set_max_size(Vec2::new(3.0, 14.0));
            });
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(title)
                .font(FontId::new(13.0, egui::FontFamily::Proportional))
                .color(colors::text_secondary())
                .strong(),
        );
    });
    ui.add_space(6.0);

    // 内容卡片容器
    egui::Frame::new()
        .fill(colors::bg_card())
        .corner_radius(CornerRadius::ZERO)
        .stroke(egui::Stroke::new(1.0, colors::border()))
        .inner_margin(egui::Margin::symmetric(16, 8))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            add_contents(ui);
        });
}

/// 带开关的设置行
fn toggle_row(ui: &mut egui::Ui, label: &str, desc: &str, value: &mut bool) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new(label)
                    .font(FontId::new(15.0, egui::FontFamily::Proportional))
                    .color(colors::text_primary()),
            );
            ui.label(
                egui::RichText::new(desc)
                    .font(FontId::new(12.0, egui::FontFamily::Proportional))
                    .color(colors::text_secondary()),
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Metro 风格开关：用强调色填充
            let toggle_size = 28.0;
            let (rect, resp) = ui.allocate_exact_size(
                Vec2::new(44.0, toggle_size),
                egui::Sense::click(),
            );

            let bg = if *value { colors::primary() } else { colors::bg_input() };
            ui.painter().rect_filled(rect, CornerRadius::ZERO, bg);

            // 滑块
            let knob_x = if *value { rect.right() - 24.0 } else { rect.left() + 4.0 };
            let knob_rect = Rect::from_min_size(
                pos2(knob_x, rect.top() + 4.0),
                Vec2::splat(20.0),
            );
            ui.painter().rect_filled(knob_rect, CornerRadius::ZERO, Color32::WHITE);

            if resp.clicked() {
                *value = !*value;
            }
            if resp.hovered() {
                let c = if *value { colors::primary() } else { colors::border() };
                let overlay = Color32::from_rgba_premultiplied(c.r(), c.g(), c.b(), 30);
                ui.painter().rect_filled(rect, CornerRadius::ZERO, overlay);
            }
        });
    });
}

/// 只读信息行
fn info_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(label)
                .font(FontId::new(15.0, egui::FontFamily::Proportional))
                .color(colors::text_primary()),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(value)
                    .font(FontId::new(13.0, egui::FontFamily::Proportional))
                    .color(colors::text_secondary()),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("›")
                    .font(FontId::new(20.0, egui::FontFamily::Proportional))
                    .color(colors::text_secondary()),
            );
        });
    });
}

/// 可点击操作行
fn action_row(ui: &mut egui::Ui, label: &str, desc: &str) -> egui::Response {
    ui.horizontal(|ui| {
        let resp = ui.vertical(|ui| {
            ui.label(
                egui::RichText::new(label)
                    .font(FontId::new(15.0, egui::FontFamily::Proportional))
                    .color(colors::primary()),
            );
            ui.label(
                egui::RichText::new(desc)
                    .font(FontId::new(12.0, egui::FontFamily::Proportional))
                    .color(colors::text_secondary()),
            );
        })
        .response
        .interact(egui::Sense::click());

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new("›")
                    .font(FontId::new(20.0, egui::FontFamily::Proportional))
                    .color(colors::primary()),
            );
        });

        resp
    })
    .inner
}

/// 外观模式三态选择行
fn theme_mode_row(ui: &mut egui::Ui, mode: &mut ThemeMode) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("外观模式")
                .font(FontId::new(15.0, egui::FontFamily::Proportional))
                .color(colors::text_primary()),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            for (variant, label) in [
                (ThemeMode::System, "跟随系统"),
                (ThemeMode::Dark,   "深色"),
                (ThemeMode::Light,  "浅色"),
            ] {
                let selected = *mode == variant;
                let btn = egui::Button::new(
                    egui::RichText::new(label)
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(if selected {
                            colors::text_on_primary()
                        } else {
                            colors::text_secondary()
                        }),
                )
                .fill(if selected { colors::primary() } else { colors::bg_input() })
                .corner_radius(CornerRadius::same(0))
                .min_size(Vec2::new(0.0, 28.0));
                if ui.add(btn).clicked() {
                    *mode = variant;
                }
                ui.add_space(4.0);
            }
        });
    });
}

/// 强调色色块选择行
fn accent_color_row(ui: &mut egui::Ui, accent_idx: &mut usize) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("强调色")
                .font(FontId::new(15.0, egui::FontFamily::Proportional))
                .color(colors::text_primary()),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            for (i, preset) in ACCENT_PRESETS.iter().enumerate().rev() {
                let selected = *accent_idx == i;
                let color = egui::Color32::from_rgb(preset.r, preset.g, preset.b);
                let (rect, resp) = ui.allocate_exact_size(
                    Vec2::splat(28.0),
                    egui::Sense::click(),
                );
                if resp.clicked() {
                    *accent_idx = i;
                }

                // 色块
                ui.painter().rect_filled(rect, 0.0, color);

                // 选中时：白色对勾
                if selected {
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "✓",
                        FontId::new(16.0, egui::FontFamily::Proportional),
                        egui::Color32::WHITE,
                    );
                }
                // 悬停时：白色边框
                if resp.hovered() {
                    ui.painter().rect_stroke(
                        rect,
                        0.0,
                        egui::Stroke::new(2.0, egui::Color32::WHITE),
                        egui::StrokeKind::Outside,
                    );
                }
                ui.add_space(6.0);
            }
            // 当前颜色名称
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(ACCENT_PRESETS[*accent_idx].name)
                    .font(FontId::new(13.0, egui::FontFamily::Proportional))
                    .color(colors::text_secondary()),
            );
        });
    });
}