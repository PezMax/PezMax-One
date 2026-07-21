// 个人功能区 — Metro Design 重设计
// 四个子标签：个人中心 / 通知 / 下载记录 / 设置
//
// 设计语言：
//   - 方角纯色块（CornerRadius::ZERO）
//   - 左边缘 3px 强调色条装饰
//   - 大字号数字 + 小号标签
//   - 双色调文字（primary / secondary）
//   - 内容卡片 bg_card + 1px 边框
//   - 悬停叠加色（primary color + 低透明度）

use crate::app::PezMaxApp;
use crate::theme::colors;
use egui::{Color32, CornerRadius, FontId, Rect, Stroke, Vec2, pos2, StrokeKind};
use crate::theme::{ThemeMode, ACCENT_PRESETS};

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

// ── 个人中心 ─────────────────────────────────────────────────────────────────

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
            // ── 用户信息卡 ─────────────────────────────────────
            if let Some(ref user) = app.current_user {
                // 头像 + 信息
                let first_char = user.nick_name.chars().next().unwrap_or('?').to_string();
                let (rect, _) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), 96.0),
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

                // 头像色块
                let avatar_rect = Rect::from_min_size(
                    pos2(rect.left() + 20.0, rect.top() + 14.0),
                    Vec2::splat(68.0),
                );
                ui.painter().rect_filled(avatar_rect, CornerRadius::ZERO, colors::primary());
                ui.painter().text(
                    avatar_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &first_char,
                    FontId::new(32.0, egui::FontFamily::Proportional),
                    colors::text_on_primary(),
                );

                // 昵称
                ui.painter().text(
                    pos2(avatar_rect.right() + 16.0, rect.top() + 24.0),
                    egui::Align2::LEFT_CENTER,
                    &user.nick_name,
                    FontId::new(22.0, egui::FontFamily::Proportional),
                    colors::text_primary(),
                );
                // 用户名
                ui.painter().text(
                    pos2(avatar_rect.right() + 16.0, rect.top() + 52.0),
                    egui::Align2::LEFT_CENTER,
                    format!("@{}", user.user_name),
                    FontId::new(14.0, egui::FontFamily::Proportional),
                    colors::text_secondary(),
                );

                // 换头像按钮
                let btn_rect = Rect::from_min_size(
                    pos2(rect.right() - 100.0, rect.top() + 32.0),
                    Vec2::new(84.0, 32.0),
                );
                let btn_resp = ui.interact(btn_rect, ui.next_auto_id(), egui::Sense::click());
                ui.painter().rect_stroke(
                    btn_rect,
                    CornerRadius::ZERO,
                    Stroke::new(1.0, colors::primary()),
                    StrokeKind::Outside,
                );
                ui.painter().text(
                    btn_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "更换头像",
                    FontId::new(13.0, egui::FontFamily::Proportional),
                    colors::primary(),
                );
                if btn_resp.clicked() {
                    // 预留：触发头像上传
                }
                if btn_resp.hovered() {
                    let overlay = Color32::from_rgba_premultiplied(
                        colors::primary().r(), colors::primary().g(), colors::primary().b(), 24,
                    );
                    ui.painter().rect_filled(btn_rect, CornerRadius::ZERO, overlay);
                }

                ui.add_space(12.0);

                // ── 统计色块（3 个，匹配首页风格）───────────────
                let (fav, dl, ul) = app.user_stats.as_ref().map_or((0, 0, 0), |s| {
                    (s.favorite_count, s.download_count, s.upload_count)
                });

                let stats = [
                    (format!("{}", dl), "下载量", colors::primary()),
                    (format!("{}", fav), "收藏数", colors::accent_orange()),
                    (format!("{}", ul), "上传数", colors::accent_green()),
                ];

                let gap = 8.0;
                let block_w = (ui.available_width() - gap * 2.0) / 3.0;

                ui.horizontal(|ui| {
                    for (i, (value, label, color)) in stats.iter().enumerate() {
                        if i > 0 { ui.add_space(gap); }
                        stat_block(ui, value, label, *color, block_w);
                    }
                });

                ui.add_space(20.0);

                // ── 账号安全 ─────────────────────────────────────
                section_title(ui, "账号安全");
                ui.add_space(12.0);

                security_row(ui, "🔑 修改密码", "定期更换密码保护账号安全", colors::primary());
                ui.add_space(4.0);
                security_row(ui, "🔒 密保问题", "用于账号找回的安全验证", colors::accent_orange());
            } else {
                empty_state(ui, "👤", "用户信息加载中...");
            }

            ui.add_space(24.0);
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

/// 安全设置行（可点击）
fn security_row(ui: &mut egui::Ui, label: &str, desc: &str, accent: Color32) {
    let (rect, resp) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), 52.0),
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
        accent,
    );

    // 标题
    ui.painter().text(
        pos2(rect.left() + 18.0, rect.top() + 14.0),
        egui::Align2::LEFT_CENTER,
        label,
        FontId::new(15.0, egui::FontFamily::Proportional),
        colors::text_primary(),
    );
    // 描述
    ui.painter().text(
        pos2(rect.left() + 18.0, rect.top() + 36.0),
        egui::Align2::LEFT_CENTER,
        desc,
        FontId::new(12.0, egui::FontFamily::Proportional),
        colors::text_secondary(),
    );

    // 右箭头
    ui.painter().text(
        pos2(rect.right() - 14.0, rect.center().y),
        egui::Align2::RIGHT_CENTER,
        "›",
        FontId::new(22.0, egui::FontFamily::Proportional),
        colors::text_secondary(),
    );

    if resp.hovered() {
        let c = accent;
        let overlay = Color32::from_rgba_premultiplied(c.r(), c.g(), c.b(), 12);
        ui.painter().rect_filled(rect, CornerRadius::ZERO, overlay);
    }

    resp.on_hover_cursor(egui::CursorIcon::PointingHand);
}

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