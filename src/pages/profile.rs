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

use crate::app::{AccountEditSection, PezMaxApp, ToastLevel};
use crate::api::models::SecurityQuestion;
use crate::components::animated_counter::render_odometer_value;
use crate::pdf::ViewMode;
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

/// 设置页小节标题：3px 强调色条 + 13px 二级色文字
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
                .font(FontId::new(13.0, egui::FontFamily::Proportional))
                .color(colors::text_secondary())
                .strong(),
        );
    });
    ui.add_space(8.0);
}

/// 统一设置卡片：左 3px 强调色条 + 边框 + 标签/描述 + 右侧控件
/// right_width 决定右侧控件区宽度；返回卡片的 hover/click Response
fn setting_card(
    ui: &mut egui::Ui,
    label: &str,
    desc: &str,
    right_width: f32,
    add_right: impl FnOnce(&mut egui::Ui),
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), 60.0),
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
    ui.painter().rect_filled(
        Rect::from_min_max(pos2(rect.left(), rect.top()), pos2(rect.left() + 3.0, rect.bottom())),
        CornerRadius::ZERO,
        colors::primary(),
    );

    // 左侧：标签 + 描述
    let left_rect = Rect::from_min_max(
        pos2(rect.left() + 20.0, rect.top() + 10.0),
        pos2(rect.right() - right_width - 24.0, rect.bottom() - 10.0),
    );
    ui.allocate_ui_at_rect(left_rect, |ui| {
        ui.label(
            egui::RichText::new(label)
                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                .color(colors::text_primary()),
        );
        ui.label(
            egui::RichText::new(desc)
                .font(FontId::new(11.0, egui::FontFamily::Proportional))
                .color(colors::text_secondary()),
        );
    });

    // 右侧：控件区（垂直居中，右对齐）
    let right_rect = Rect::from_min_size(
        pos2(rect.right() - right_width - 12.0, rect.top() + 14.0),
        Vec2::new(right_width, 32.0),
    );
    ui.allocate_ui_at_rect(right_rect, |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            add_right(ui);
        });
    });

    resp
}

/// Metro 风格开关：直角方形 + 白色滑块。
/// 动画由 Sokuou `Progress` 驱动：滑块位置在 off/on 之间以 EaseOutCubic 平滑过渡；
/// 背景色随 progress 由 bg_input 插值到 primary。每个开关按 `id` 独立存储动画状态。
fn render_toggle_switch(ui: &mut egui::Ui, id_source: &str, value: &mut bool) {
    use crate::sokuou::{Easing, Progress};

    let (rect, resp) = ui.allocate_exact_size(
        Vec2::new(40.0, 24.0),
        egui::Sense::click(),
    );

    if resp.clicked() {
        *value = !*value;
    }

    // ── 每-开关的 Progress 缓存到 egui 临时数据 ──
    let anim_id = egui::Id::new(("toggle_progress", id_source));
    let target = if *value { 1.0f64 } else { 0.0 };
    let mut anim = ui.ctx().data_mut(|d| {
        d.get_temp::<Progress>(anim_id).unwrap_or_else(|| {
            let mut p = Progress::with_easing(0.22, Easing::EaseOutCubic);
            p.jump_to(target); // 首次渲染直接落到当前状态，不播开场动画
            p
        })
    });
    if (anim.target() - target).abs() > f64::EPSILON {
        anim.set_target(target);
    }
    let dt = ui.input(|i| i.stable_dt) as f64;
    anim.update(dt);
    let t = anim.value().clamp(0.0, 1.0) as f32;
    if !anim.is_steady() {
        ui.ctx().request_repaint();
    }
    ui.ctx().data_mut(|d| d.insert_temp(anim_id, anim));

    // ── 背景色：bg_input → primary 插值 ──
    let bg_off = colors::bg_input();
    let bg_on = colors::primary();
    let bg = Color32::from_rgb(
        lerp_u8(bg_off.r(), bg_on.r(), t),
        lerp_u8(bg_off.g(), bg_on.g(), t),
        lerp_u8(bg_off.b(), bg_on.b(), t),
    );
    ui.painter().rect_filled(rect, CornerRadius::ZERO, bg);

    // ── 滑块横向位置：4px → (rect.width - 20) 线性映射 ──
    let knob_left = rect.left() + 4.0;
    let knob_right = rect.right() - 20.0;
    let knob_x = knob_left + (knob_right - knob_left) * t;
    let knob_rect = Rect::from_min_size(
        pos2(knob_x, rect.top() + 4.0),
        Vec2::splat(16.0),
    );
    ui.painter().rect_filled(knob_rect, CornerRadius::ZERO, Color32::WHITE);

    if resp.hovered() {
        let c = if *value { colors::primary() } else { colors::border() };
        let overlay = Color32::from_rgba_premultiplied(c.r(), c.g(), c.b(), 30);
        ui.painter().rect_filled(rect, CornerRadius::ZERO, overlay);
    }
}

#[inline]
fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).clamp(0.0, 255.0) as u8
}

/// 小号方角文字按钮（选中态填充强调色，未选中透明）
fn render_choice_button(ui: &mut egui::Ui, label: &str, selected: bool) -> egui::Response {
    let btn = egui::Button::new(
        egui::RichText::new(label)
            .font(FontId::new(12.0, egui::FontFamily::Proportional))
            .color(if selected {
                colors::text_on_primary()
            } else {
                colors::text_secondary()
            }),
    )
    .fill(if selected { colors::primary() } else { colors::bg_input() })
    .corner_radius(CornerRadius::ZERO)
    .stroke(Stroke::NONE)
    .min_size(Vec2::new(0.0, 26.0));
    ui.add(btn)
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

                // 右半区：统计列（电表风格数显）
                let stat_x = rect.right() - 140.0;
                let stat_items = [
                    (&app.dl_anim, "下载量"),
                    (&app.fav_anim, "收藏数"),
                    (&app.ul_anim, "上传数"),
                ];
                let stat_gap = 28.0;
                let stat_start_y = rect.top() + (card_height - (stat_items.len() as f32 * stat_gap)) / 2.0 + stat_gap / 2.0;
                for (i, (counter, label)) in stat_items.iter().enumerate() {
                    let y = stat_start_y + i as f32 * stat_gap;
                    // 数值（电表滚动）
                    let font_size = 18.0;
                    render_odometer_value(
                        ui,
                        pos2(stat_x, y - font_size * 0.6),
                        counter,
                        font_size,
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

    ui.add_space(4.0);

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
            let clicked = ui.scope(|ui| {
                ui.visuals_mut().widgets.inactive.bg_fill = Color32::TRANSPARENT;
                ui.add(
                    egui::Button::new(
                        egui::RichText::new("退出")
                            .font(FontId::new(13.0, egui::FontFamily::Proportional))
                            .color(colors::primary()),
                    )
                    .corner_radius(CornerRadius::ZERO)
                    .min_size(Vec2::new(56.0, 28.0))
                    .stroke(Stroke::new(1.0, colors::primary())),
                )
            }).inner.clicked();
            if clicked {
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

/// 编辑按钮（小号方角，hover/click 有填充色）
fn edit_button(ui: &mut egui::Ui, text: &str) -> bool {
    ui.scope(|ui| {
        ui.visuals_mut().widgets.inactive.bg_fill = Color32::TRANSPARENT;
        ui.add(
            egui::Button::new(
                egui::RichText::new(text)
                    .font(FontId::new(13.0, egui::FontFamily::Proportional))
                    .color(colors::primary()),
            )
            .corner_radius(CornerRadius::ZERO)
            .min_size(Vec2::new(56.0, 28.0))
            .stroke(Stroke::new(1.0, colors::primary())),
        )
    }).inner.clicked()
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

/// 次要按钮（hover/click 有填充色）
fn secondary_button(ui: &mut egui::Ui, text: &str) -> bool {
    ui.scope(|ui| {
        ui.visuals_mut().widgets.inactive.bg_fill = Color32::TRANSPARENT;
        ui.add(
            egui::Button::new(
                egui::RichText::new(text)
                    .font(FontId::new(13.0, egui::FontFamily::Proportional))
                    .color(colors::text_secondary()),
            )
            .corner_radius(CornerRadius::ZERO)
            .min_size(Vec2::new(56.0, 28.0))
            .stroke(Stroke::new(1.0, colors::border())),
        )
    }).inner.clicked()
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
            setting_section_title(ui, "常规");
            setting_card(ui, "开机自启", "登录时自动启动 PezMax", 60.0, |ui| {
                render_toggle_switch(ui, "auto_launch", &mut app.setting_auto_launch);
            });

            ui.add_space(16.0);

            // ── 外观 ──────────────────────────────────────────────
            setting_section_title(ui, "外观");
            setting_card(ui, "外观模式", "跟随系统 / 深色 / 浅色", 240.0, |ui| {
                // right_to_left 布局：Light → Dark → System
                for (variant, label) in [
                    (ThemeMode::Light,  "浅色"),
                    (ThemeMode::Dark,   "深色"),
                    (ThemeMode::System, "跟随系统"),
                ] {
                    let selected = app.theme_mode == variant;
                    if render_choice_button(ui, label, selected).clicked() {
                        app.theme_mode = variant;
                    }
                    ui.add_space(4.0);
                }
            });
            ui.add_space(6.0);
            setting_card(ui, "强调色", ACCENT_PRESETS[app.accent_idx].name, 260.0, |ui| {
                for (i, preset) in ACCENT_PRESETS.iter().enumerate().rev() {
                    let selected = app.accent_idx == i;
                    let color = egui::Color32::from_rgb(preset.r, preset.g, preset.b);
                    let (crect, cresp) = ui.allocate_exact_size(
                        Vec2::splat(28.0),
                        egui::Sense::click(),
                    );
                    if cresp.clicked() {
                        app.accent_idx = i;
                    }
                    ui.painter().rect_filled(crect, CornerRadius::ZERO, color);
                    if selected {
                        ui.painter().text(
                            crect.center(),
                            egui::Align2::CENTER_CENTER,
                            "✓",
                            FontId::new(14.0, egui::FontFamily::Proportional),
                            Color32::WHITE,
                        );
                    }
                    if cresp.hovered() {
                        ui.painter().rect_stroke(
                            crect,
                            CornerRadius::ZERO,
                            Stroke::new(2.0, Color32::WHITE),
                            StrokeKind::Outside,
                        );
                    }
                    ui.add_space(6.0);
                }
            });

            ui.add_space(16.0);

            // ── PDF ──────────────────────────────────────────────
            setting_section_title(ui, "PDF");
            setting_card(ui, "视图模式", "网格 / 单页 / 双页", 240.0, |ui| {
                let modes: Vec<ViewMode> = ViewMode::all().iter().rev().copied().collect();
                for variant in modes {
                    let selected = app.setting_pdf_view_mode == variant;
                    if render_choice_button(ui, variant.label(), selected).clicked() {
                        app.setting_pdf_view_mode = variant;
                    }
                    ui.add_space(4.0);
                }
            });
            ui.add_space(6.0);
            setting_card(ui, "默认缩放", "PDF 打开时的默认显示比例", 130.0, |ui| {
                // 预设缩放档位
                const SCALES: &[f32] = &[0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0];
                let current = app.setting_pdf_scale;
                let selected_text = format!("{:.0}%", current * 100.0);
                egui::ComboBox::from_id_salt("pdf_default_scale")
                    .selected_text(
                        egui::RichText::new(&selected_text)
                            .font(FontId::new(13.0, egui::FontFamily::Proportional))
                            .color(colors::primary()),
                    )
                    .width(96.0)
                    .show_ui(ui, |ui| {
                        for &s in SCALES {
                            let label = format!("{:.0}%", s * 100.0);
                            let active = (current - s).abs() < 0.001;
                            if ui.selectable_label(active, label).clicked() {
                                app.setting_pdf_scale = s;
                            }
                        }
                    });
            });

            ui.add_space(16.0);

            // ── 下载 ──────────────────────────────────────────────
            setting_section_title(ui, "下载");
            setting_card(ui, "静默下载", "跳过保存路径选择，直接存入默认目录", 60.0, |ui| {
                render_toggle_switch(ui, "silent_download", &mut app.setting_silent_download);
            });
            ui.add_space(6.0);
            render_download_path_card(app, ui);

            ui.add_space(16.0);

            // ── 隐私 ──────────────────────────────────────────────
            setting_section_title(ui, "隐私");
            let cache_size = format_cache_size(compute_cache_size(&app.cache_manager));
            setting_card(ui, "缓存大小", "头像、PDF 页面、书签封面等本地缓存占用", 100.0, |ui| {
                ui.label(
                    egui::RichText::new(&cache_size)
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary()),
                );
            });
            ui.add_space(6.0);
            if setting_card(ui, "清理缓存", "释放本地缓存空间", 80.0, |ui| {
                ui.label(
                    egui::RichText::new("清理 ›")
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(colors::primary()),
                );
            }).clicked() {
                if rfd::MessageDialog::new()
                    .set_title("清除缓存")
                    .set_description("确定要清除所有本地缓存吗？\n此操作将清除头像、PDF 页面和书签封面等缓存数据。")
                    .set_buttons(rfd::MessageButtons::OkCancel)
                    .show()
                    == rfd::MessageDialogResult::Ok
                {
                    app.clear_cache();
                }
            }

            ui.add_space(16.0);

            // ── 关于 ──────────────────────────────────────────────
            setting_section_title(ui, "关于");
            if setting_card(ui, "关于 PezMax One", "版本 · 许可证 · 联系方式", 80.0, |ui| {
                ui.label(
                    egui::RichText::new("查看 ›")
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(colors::primary()),
                );
            }).clicked() {
                app.show_about_dialog = true;
            }

            ui.add_space(24.0);
        });

    // 关于弹窗
    render_about_dialog(app, ui);
}

// ── 内部组件 ──────────────────────────────────────────────────────────────────

/// 默认下载路径卡片 — 高度加大以容纳完整路径，整卡片可点击打开系统文件夹选择器
fn render_download_path_card(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    let path = app.setting_download_dir.clone();
    let (rect, resp) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), 80.0),
        egui::Sense::click(),
    );
    ui.painter().rect_filled(rect, CornerRadius::ZERO, colors::bg_card());
    ui.painter().rect_stroke(
        rect,
        CornerRadius::ZERO,
        Stroke::new(1.0, colors::border()),
        StrokeKind::Outside,
    );
    // 左 3px 强调色条
    ui.painter().rect_filled(
        Rect::from_min_max(pos2(rect.left(), rect.top()), pos2(rect.left() + 3.0, rect.bottom())),
        CornerRadius::ZERO,
        colors::primary(),
    );

    let content_rect = Rect::from_min_max(
        pos2(rect.left() + 20.0, rect.top() + 10.0),
        pos2(rect.right() - 12.0, rect.bottom() - 10.0),
    );
    ui.allocate_ui_at_rect(content_rect, |ui| {
        // 第一行：标签 + 右侧"更改 ›"
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("默认下载路径")
                    .font(FontId::new(14.0, egui::FontFamily::Proportional))
                    .color(colors::text_primary()),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new("更改 ›")
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(colors::primary()),
                );
            });
        });
        ui.add_space(4.0);
        // 第二行：路径（超长时截断，避免撑破卡片）
        ui.add(
            egui::Label::new(
                egui::RichText::new(&path)
                    .font(FontId::new(11.0, egui::FontFamily::Monospace))
                    .color(colors::text_secondary()),
            )
            .truncate(),
        );
    });

    if resp.clicked() {
        let start_dir = std::path::PathBuf::from(&path);
        let mut dlg = rfd::FileDialog::new().set_title("选择默认下载文件夹");
        if start_dir.exists() {
            dlg = dlg.set_directory(&start_dir);
        }
        if let Some(chosen) = dlg.pick_folder() {
            app.setting_download_dir = chosen.to_string_lossy().to_string();
        }
    }
}

/// 计算缓存目录大小（字节）
fn compute_cache_size(cm: &crate::cache::CacheManager) -> u64 {
    let cache_dir = cm.cache_dir();
    if !cache_dir.exists() {
        return 0;
    }
    fn walk_dir(dir: &std::path::Path) -> u64 {
        let mut total = 0u64;
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    total += walk_dir(&path);
                } else if let Ok(meta) = entry.metadata() {
                    total += meta.len();
                }
            }
        }
        total
    }
    walk_dir(cache_dir)
}

/// 格式化缓存大小（B/KB/MB）
fn format_cache_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// 关于弹窗 — Metro Design 风格，居中显示完整信息
fn render_about_dialog(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    if !app.show_about_dialog {
        return;
    }

    let license_text = "MIT License

Copyright (c) 2026 Takahashi_Rinta

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the \"Software\"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.";

    // 内容左侧内边距（过 3px 强调色条 + 呼吸空间）
    const LEFT_PAD: f32 = 24.0;
    const RIGHT_PAD: f32 = 20.0;

    egui::Window::new("关于")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .fixed_size(egui::vec2(500.0, 560.0))
        .title_bar(false)
        .frame(egui::Frame::new()
            .fill(colors::bg_card())
            .corner_radius(egui::CornerRadius::ZERO)
            .stroke(egui::Stroke::new(1.0, colors::border())))
        .show(ui.ctx(), |ui| {
            // 左边缘 3px 强调色条
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    ui.max_rect().left_top(),
                    egui::vec2(3.0, ui.max_rect().height()),
                ),
                egui::CornerRadius::ZERO,
                colors::primary(),
            );

            let content_left = ui.max_rect().left() + LEFT_PAD;
            let content_right = ui.max_rect().right() - RIGHT_PAD;

            ui.add_space(18.0);

            // ── 标题栏 ─────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.add_space(LEFT_PAD);
                ui.label(
                    egui::RichText::new("关于")
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary())
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(RIGHT_PAD - 12.0);
                    let close_clicked = ui.scope(|ui| {
                        ui.visuals_mut().widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
                        ui.add(
                            egui::Button::new(
                                egui::RichText::new("×")
                                    .font(FontId::new(22.0, egui::FontFamily::Proportional))
                                    .color(colors::text_secondary()),
                            )
                            .stroke(egui::Stroke::NONE)
                            .min_size(egui::vec2(28.0, 28.0)),
                        )
                    }).inner.clicked();
                    if close_clicked {
                        app.show_about_dialog = false;
                    }
                });
            });

            ui.add_space(14.0);

            // ── 应用名称 + 版本号 ──────────────────────────────
            ui.horizontal(|ui| {
                ui.add_space(LEFT_PAD);
                ui.label(
                    egui::RichText::new("PezMax One")
                        .font(FontId::new(24.0, egui::FontFamily::Proportional))
                        .color(colors::primary())
                        .strong(),
                );
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("v0.1.0")
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary()),
                );
            });
            ui.horizontal(|ui| {
                ui.add_space(LEFT_PAD);
                ui.label(
                    egui::RichText::new("拼图满绩 · 绫")
                        .font(FontId::new(14.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary()),
                );
            });

            ui.add_space(18.0);

            // ── 分割线 ────────────────────────────────────────
            let sep_y = ui.cursor().top();
            ui.painter().line_segment(
                [egui::pos2(content_left, sep_y), egui::pos2(content_right, sep_y)],
                egui::Stroke::new(1.0, colors::border()),
            );
            ui.add_space(16.0);

            // ── 信息列表（key 列固定 84px 宽度对齐）─────────
            let info_items: &[(&str, &str)] = &[
                ("版本号", "v0.1.0 (build 2026-07)"),
                ("作者", "Takahashi Rinta"),
                ("QQ交流群", "1077605719"),
                ("许可证", "MIT License"),
            ];

            for (key, val) in info_items {
                ui.horizontal(|ui| {
                    ui.add_space(LEFT_PAD);
                    ui.allocate_ui_with_layout(
                        egui::vec2(84.0, 22.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.label(
                                egui::RichText::new(*key)
                                    .font(FontId::new(12.0, egui::FontFamily::Proportional))
                                    .color(colors::text_secondary()),
                            );
                        },
                    );
                    ui.label(
                        egui::RichText::new(*val)
                            .font(FontId::new(13.0, egui::FontFamily::Proportional))
                            .color(colors::text_primary()),
                    );
                    if *key == "QQ交流群" {
                        ui.add_space(8.0);
                        let copy_btn = ui.scope(|ui| {
                            ui.visuals_mut().widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
                            ui.add(
                                egui::Button::new(
                                    egui::RichText::new("复制")
                                        .font(FontId::new(11.0, egui::FontFamily::Proportional))
                                        .color(colors::primary()),
                                )
                                .corner_radius(egui::CornerRadius::ZERO)
                                .min_size(egui::vec2(40.0, 22.0))
                                .stroke(egui::Stroke::new(1.0, colors::primary())),
                            )
                        }).inner.clicked();
                        if copy_btn {
                            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                let _ = clipboard.set_text("1077605719");
                            }
                            app.add_toast("QQ群号已复制到剪贴板", ToastLevel::Success);
                        }
                    }
                });
                ui.add_space(8.0);
            }

            ui.add_space(6.0);
            let sep_y2 = ui.cursor().top();
            ui.painter().line_segment(
                [egui::pos2(content_left, sep_y2), egui::pos2(content_right, sep_y2)],
                egui::Stroke::new(1.0, colors::border()),
            );
            ui.add_space(12.0);

            // ── 许可证 ────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.add_space(LEFT_PAD);
                ui.label(
                    egui::RichText::new("MIT License")
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(colors::text_primary())
                        .strong(),
                );
            });
            ui.add_space(6.0);

            egui::ScrollArea::vertical()
                .id_salt("about_license_scroll")
                .max_height(180.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(LEFT_PAD);
                        ui.vertical(|ui| {
                            ui.set_max_width(content_right - content_left);
                            ui.label(
                                egui::RichText::new(license_text)
                                    .font(FontId::new(11.0, egui::FontFamily::Monospace))
                                    .color(colors::text_secondary()),
                            );
                        });
                    });
                });

            ui.add_space(14.0);

            // ── 关闭按钮（真居中）─────────────────────────────
            ui.vertical_centered(|ui| {
                let close_btn = egui::Button::new(
                    egui::RichText::new("关闭")
                        .font(FontId::new(14.0, egui::FontFamily::Proportional))
                        .color(colors::text_on_primary()),
                )
                .fill(colors::primary())
                .stroke(egui::Stroke::NONE)
                .corner_radius(egui::CornerRadius::ZERO)
                .min_size(egui::vec2(96.0, 32.0));
                if ui.add(close_btn).clicked() {
                    app.show_about_dialog = false;
                }
            });
        });
}