// 个人功能区
// 四个子标签：个人中心 / 通知 / 下载记录 / 设置

use crate::app::PezMaxApp;
use crate::theme::colors;
use egui::{CornerRadius, FontId, Vec2};

/// 个人中心：用户信息卡片 + 统计数据 + 账号安全设置

pub fn render_personal_center(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    ui.add_space(16.0);

    egui::ScrollArea::vertical()
        .id_salt("profile_scroll")
        .show(ui, |ui| {
            // ── 用户信息卡 ─────────────────────────────────────
            if let Some(ref user) = app.current_user {
                egui::Frame::new()
                    .fill(colors::bg_card())
                    .corner_radius(CornerRadius::same(0))
                    .stroke(egui::Stroke::new(1.0, colors::border()))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.add_space(24.0);
                        ui.horizontal(|ui| {
                            ui.add_space(24.0);

                            // 头像
                            let first_char = user
                                .nick_name
                                .chars()
                                .next()
                                .unwrap_or('?')
                                .to_string();
                            egui::Frame::new()
                                .fill(colors::primary())
                                .corner_radius(CornerRadius::same(0))
                                .show(ui, |ui| {
                                    ui.set_min_size(Vec2::new(72.0, 72.0));
                                    ui.set_max_size(Vec2::new(72.0, 72.0));
                                    ui.vertical_centered(|ui| {
                                        ui.add_space(14.0);
                                        ui.label(
                                            egui::RichText::new(first_char)
                                                .font(FontId::new(
                                                    36.0,
                                                    egui::FontFamily::Proportional,
                                                ))
                                                .color(colors::text_on_primary()),
                                        );
                                    });
                                });

                            ui.add_space(20.0);
                            ui.vertical(|ui| {
                                ui.add_space(8.0);
                                ui.label(
                                    egui::RichText::new(&user.nick_name)
                                        .font(FontId::new(
                                            22.0,
                                            egui::FontFamily::Proportional,
                                        ))
                                        .color(colors::text_primary()),
                                );
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new(format!("@{}", user.user_name))
                                        .font(FontId::new(
                                            14.0,
                                            egui::FontFamily::Proportional,
                                        ))
                                        .color(colors::text_secondary()),
                                );
                                ui.add_space(12.0);
                                let btn = egui::Button::new(
                                    egui::RichText::new("更换头像")
                                        .font(FontId::new(
                                            13.0,
                                            egui::FontFamily::Proportional,
                                        )),
                                )
                                .fill(colors::bg_hover())
                                .corner_radius(CornerRadius::same(0));
                                if ui.add(btn).clicked() {}
                            });
                        });
                        ui.add_space(24.0);
                    });

                ui.add_space(16.0);

                // ── 统计卡片 ───────────────────────────────────
                let (fav, dl, ul) = if let Some(ref s) = app.user_stats {
                    (s.favorite_count, s.download_count, s.upload_count)
                } else {
                    (0, 0, 0)
                };

                let stat_items = [
                    ("⭐", "收藏", fav, colors::accent_orange()),
                    ("📥", "下载", dl, colors::primary()),
                    ("📤", "上传", ul, colors::accent_green()),
                ];

                ui.horizontal(|ui| {
                    for (icon, label, value, color) in &stat_items {
                        egui::Frame::new()
                            .fill(*color)
                            .corner_radius(CornerRadius::same(0))
                            .show(ui, |ui| {
                                ui.set_min_size(Vec2::new(120.0, 80.0));
                                ui.add_space(10.0);
                                ui.vertical_centered(|ui| {
                                    ui.label(
                                        egui::RichText::new(*icon).font(FontId::new(
                                            22.0,
                                            egui::FontFamily::Proportional,
                                        )),
                                    );
                                    ui.label(
                                        egui::RichText::new(value.to_string())
                                            .font(FontId::new(
                                                20.0,
                                                egui::FontFamily::Proportional,
                                            ))
                                            .color(colors::text_on_primary()),
                                    );
                                    ui.label(
                                        egui::RichText::new(*label)
                                            .font(FontId::new(
                                                12.0,
                                                egui::FontFamily::Proportional,
                                            ))
                                            .color(colors::text_on_primary()),
                                    );
                                });
                                ui.add_space(10.0);
                            });
                        ui.add_space(8.0);
                    }
                });

                ui.add_space(16.0);
            }

            // ── 账号安全 ───────────────────────────────────────
            ui.label(
                egui::RichText::new("账号安全")
                    .font(FontId::new(16.0, egui::FontFamily::Proportional))
                    .color(colors::text_primary()),
            );
            ui.add_space(8.0);

            security_row(ui, "修改密码", "定期更换密码保护账号安全");
            ui.add_space(4.0);
            security_row(ui, "密保问题", "用于账号找回的安全验证");
        });
}

/// 通知列表
pub fn render_notifications(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    // 自动加载通知
    if !app.notifications.is_loaded() && !app.notifications.is_loading() {
        app.trigger_load_notifications();
    }

    ui.add_space(16.0);
    ui.label(
        egui::RichText::new("通知")
            .font(FontId::new(22.0, egui::FontFamily::Proportional))
            .color(colors::text_primary()),
    );
    ui.add_space(16.0);

    egui::ScrollArea::vertical()
        .id_salt("notif_scroll")
        .show(ui, |ui| {
            if let Some(ref list) = app.notifications.data {
                for notif in list {
                    let is_read = notif.status == "1";
                    let bg = if is_read {
                        colors::bg_card()
                    } else {
                        colors::bg_selected()
                    };
                    egui::Frame::new()
                        .fill(bg)
                        .corner_radius(CornerRadius::same(0))
                        .stroke(egui::Stroke::new(1.0, colors::border()))
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.horizontal(|ui| {
                                // 未读指示条
                                if !is_read {
                                    egui::Frame::new()
                                        .fill(colors::primary())
                                        .show(ui, |ui| {
                                            ui.allocate_space(Vec2::new(3.0, 48.0));
                                        });
                                } else {
                                    ui.add_space(3.0);
                                }
                                ui.add_space(12.0);
                                ui.vertical(|ui| {
                                    ui.add_space(8.0);
                                    ui.label(
                                        egui::RichText::new(&notif.title)
                                            .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                            .color(colors::text_primary()),
                                    );
                                    ui.label(
                                        egui::RichText::new(&notif.content)
                                            .font(FontId::new(13.0, egui::FontFamily::Proportional))
                                            .color(colors::text_secondary()),
                                    );
                                    ui.label(
                                        egui::RichText::new(&notif.create_time)
                                            .font(FontId::new(11.0, egui::FontFamily::Proportional))
                                            .color(colors::text_secondary()),
                                    );
                                    ui.add_space(8.0);
                                });
                            });
                        });
                    ui.add_space(4.0);
                }
            } else {
                let msg = if app.notifications.is_loading() {
                    "加载中..."
                } else {
                    "暂无通知"
                };
                ui.label(
                    egui::RichText::new(msg)
                        .font(FontId::new(14.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary()),
                );
            }
        });
}

/// 下载记录
pub fn render_download_history(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    // 自动加载下载记录
    if !app.download_records.is_loaded() && !app.download_records.is_loading() {
        app.trigger_load_download_records();
    }

    ui.add_space(16.0);
    ui.label(
        egui::RichText::new("下载记录")
            .font(FontId::new(22.0, egui::FontFamily::Proportional))
            .color(colors::text_primary()),
    );
    ui.add_space(16.0);

    egui::ScrollArea::vertical()
        .id_salt("download_scroll")
        .show(ui, |ui| {
            if let Some(ref list) = app.download_records.data {
                for record in list {
                    egui::Frame::new()
                        .fill(colors::bg_card())
                        .corner_radius(CornerRadius::same(0))
                        .stroke(egui::Stroke::new(1.0, colors::border()))
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.add_space(12.0);
                                ui.label(
                                    egui::RichText::new("📄")
                                        .font(FontId::new(20.0, egui::FontFamily::Proportional)),
                                );
                                ui.add_space(10.0);
                                ui.vertical(|ui| {
                                    ui.add_space(8.0);
                                    ui.label(
                                        egui::RichText::new(&record.file_name)
                                            .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                            .color(colors::text_primary()),
                                    );
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{} · {}",
                                            record.file_format, record.download_time
                                        ))
                                        .font(FontId::new(12.0, egui::FontFamily::Proportional))
                                        .color(colors::text_secondary()),
                                    );
                                    ui.add_space(8.0);
                                });
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.add_space(12.0);
                                        if ui.small_button("隐藏").clicked() {
                                            // 隐藏下载记录
                                            if let Some(ref user) = app.current_user {
                                                // 异步执行
                                                let api = app.api.clone();
                                                let uid = user.user_id;
                                                let fid = record.file_id;
                                                tokio::spawn(async move {
                                                    let _ = api.hide_download(uid, fid).await;
                                                });
                                            }
                                        }
                                    },
                                );
                            });
                        });
                    ui.add_space(4.0);
                }
            } else {
                let msg = if app.download_records.is_loading() {
                    "加载中..."
                } else {
                    "暂无下载记录"
                };
                ui.label(
                    egui::RichText::new(msg)
                        .font(FontId::new(14.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary()),
                );
            }
        });
}

/// 应用设置
pub fn render_app_settings(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    ui.add_space(16.0);
    ui.label(
        egui::RichText::new("设置")
            .font(FontId::new(22.0, egui::FontFamily::Proportional))
            .color(colors::text_primary()),
    );
    ui.add_space(16.0);

    egui::ScrollArea::vertical()
        .id_salt("settings_scroll")
        .show(ui, |ui| {
            // ── 常规 ──────────────────────────────────────────────
            settings_group_label(ui, "常规");

            toggle_row(
                ui,
                "开机自启",
                "登录时自动启动 PezMax",
                &mut app.setting_auto_launch,
            );
            ui.add_space(2.0);
            toggle_row(ui, "深色模式", "切换深色 / 浅色外观", &mut app.dark_mode);

            ui.add_space(12.0);

            // ── 下载 ──────────────────────────────────────────────
            settings_group_label(ui, "下载");

            toggle_row(
                ui,
                "静默下载",
                "跳过保存路径选择，直接存入默认目录",
                &mut app.setting_silent_download,
            );
            ui.add_space(2.0);
            settings_info_row(ui, "默认路径", "~/Downloads/PezMax");

            ui.add_space(12.0);

            // ── 外观 ──────────────────────────────────────────────
            settings_group_label(ui, "外观");
            settings_info_row(ui, "强调色", "蓝色（更多颜色开发中）");
            ui.add_space(2.0);
            settings_info_row(ui, "壁纸", "无");

            ui.add_space(12.0);

            // ── 隐私 ──────────────────────────────────────────────
            settings_group_label(ui, "隐私");
            let resp = action_row(ui, "清理缓存", "释放本地缓存空间");
            if resp.clicked() {}

            ui.add_space(12.0);

            // ── 关于 ──────────────────────────────────────────────
            settings_group_label(ui, "关于");
            settings_info_row(ui, "版本", "PezMax 0.1.0 · egui 版");
        });
}

// ── 内部组件 ──────────────────────────────────────────────────────────────────

/// 安全设置行（可点击）
fn security_row(ui: &mut egui::Ui, label: &str, desc: &str) {
    let resp = egui::Frame::new()
        .fill(colors::bg_card())
        .corner_radius(CornerRadius::same(0))
        .stroke(egui::Stroke::new(1.0, colors::border()))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.vertical(|ui| {
                    ui.add_space(8.0);
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
                    ui.add_space(8.0);
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(16.0);
                    ui.label(
                        egui::RichText::new("›")
                            .font(FontId::new(20.0, egui::FontFamily::Proportional))
                            .color(colors::text_secondary()),
                    );
                });
            });
        })
        .response
        .interact(egui::Sense::click());
    if resp.clicked() {}
}

fn settings_group_label(ui: &mut egui::Ui, title: &str) {
    ui.label(
        egui::RichText::new(title)
            .font(FontId::new(13.0, egui::FontFamily::Proportional))
            .color(colors::text_secondary()),
    );
    ui.add_space(4.0);
}

/// 带开关的设置行
fn toggle_row(ui: &mut egui::Ui, label: &str, desc: &str, value: &mut bool) {
    egui::Frame::new()
        .fill(colors::bg_card())
        .corner_radius(CornerRadius::same(0))
        .stroke(egui::Stroke::new(1.0, colors::border()))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.vertical(|ui| {
                    ui.add_space(8.0);
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
                    ui.add_space(8.0);
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(16.0);
                    ui.checkbox(value, "");
                });
            });
        });
}

/// 只读信息行（带右箭头，无交互）
fn settings_info_row(ui: &mut egui::Ui, label: &str, value: &str) {
    egui::Frame::new()
        .fill(colors::bg_card())
        .corner_radius(CornerRadius::same(0))
        .stroke(egui::Stroke::new(1.0, colors::border()))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.vertical(|ui| {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(label)
                            .font(FontId::new(15.0, egui::FontFamily::Proportional))
                            .color(colors::text_primary()),
                    );
                    ui.add_space(8.0);
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(16.0);
                    ui.label(
                        egui::RichText::new("›")
                            .font(FontId::new(20.0, egui::FontFamily::Proportional))
                            .color(colors::text_secondary()),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(value)
                            .font(FontId::new(13.0, egui::FontFamily::Proportional))
                            .color(colors::text_secondary()),
                    );
                });
            });
        });
}

/// 可点击操作行（返回 Response）
fn action_row(ui: &mut egui::Ui, label: &str, desc: &str) -> egui::Response {
    egui::Frame::new()
        .fill(colors::bg_card())
        .corner_radius(CornerRadius::same(0))
        .stroke(egui::Stroke::new(1.0, colors::border()))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.vertical(|ui| {
                    ui.add_space(8.0);
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
                    ui.add_space(8.0);
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(16.0);
                    ui.label(
                        egui::RichText::new("›")
                            .font(FontId::new(20.0, egui::FontFamily::Proportional))
                            .color(colors::text_secondary()),
                    );
                });
            });
        })
        .response
        .interact(egui::Sense::click())
}
