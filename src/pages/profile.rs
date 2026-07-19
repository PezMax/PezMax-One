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
                    .fill(colors::BG_CARD)
                    .corner_radius(CornerRadius::same(0))
                    .stroke(egui::Stroke::new(1.0, colors::BORDER))
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
                                .fill(colors::PRIMARY)
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
                                                .color(colors::TEXT_ON_PRIMARY),
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
                                        .color(colors::TEXT_PRIMARY),
                                );
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new(format!("@{}", user.user_name))
                                        .font(FontId::new(
                                            14.0,
                                            egui::FontFamily::Proportional,
                                        ))
                                        .color(colors::TEXT_SECONDARY),
                                );
                                ui.add_space(12.0);
                                let btn = egui::Button::new(
                                    egui::RichText::new("更换头像")
                                        .font(FontId::new(
                                            13.0,
                                            egui::FontFamily::Proportional,
                                        )),
                                )
                                .fill(colors::BG_HOVER)
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
                    ("⭐", "收藏", fav, colors::ACCENT_ORANGE),
                    ("📥", "下载", dl, colors::PRIMARY),
                    ("📤", "上传", ul, colors::ACCENT_GREEN),
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
                                            .color(colors::TEXT_ON_PRIMARY),
                                    );
                                    ui.label(
                                        egui::RichText::new(*label)
                                            .font(FontId::new(
                                                12.0,
                                                egui::FontFamily::Proportional,
                                            ))
                                            .color(colors::TEXT_ON_PRIMARY),
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
                    .color(colors::TEXT_PRIMARY),
            );
            ui.add_space(8.0);

            security_row(ui, "修改密码", "定期更换密码保护账号安全");
            ui.add_space(4.0);
            security_row(ui, "密保问题", "用于账号找回的安全验证");
        });
}

/// 通知列表
pub fn render_notifications(_app: &mut PezMaxApp, ui: &mut egui::Ui) {
    ui.add_space(16.0);
    ui.label(
        egui::RichText::new("通知")
            .font(FontId::new(22.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_PRIMARY),
    );
    ui.add_space(16.0);

    let notifications = [
        ("系统通知", "您的账号已通过审核", "2024-06-15 10:30", false),
        (
            "下载完成",
            "「2024高考数学真题」已下载完成",
            "2024-06-14 15:20",
            true,
        ),
        (
            "收藏更新",
            "「2024高考语文真题」内容已更新",
            "2024-06-13 09:00",
            true,
        ),
    ];

    egui::ScrollArea::vertical()
        .id_salt("notif_scroll")
        .show(ui, |ui| {
            for (title, content, time, is_read) in &notifications {
                let bg = if *is_read {
                    colors::BG_CARD
                } else {
                    colors::BG_SELECTED
                };
                egui::Frame::new()
                    .fill(bg)
                    .corner_radius(CornerRadius::same(0))
                    .stroke(egui::Stroke::new(1.0, colors::BORDER))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.horizontal(|ui| {
                            // 未读指示条
                            if !*is_read {
                                egui::Frame::new()
                                    .fill(colors::PRIMARY)
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
                                    egui::RichText::new(*title)
                                        .font(FontId::new(
                                            14.0,
                                            egui::FontFamily::Proportional,
                                        ))
                                        .color(colors::TEXT_PRIMARY),
                                );
                                ui.label(
                                    egui::RichText::new(*content)
                                        .font(FontId::new(
                                            13.0,
                                            egui::FontFamily::Proportional,
                                        ))
                                        .color(colors::TEXT_SECONDARY),
                                );
                                ui.label(
                                    egui::RichText::new(*time)
                                        .font(FontId::new(
                                            11.0,
                                            egui::FontFamily::Proportional,
                                        ))
                                        .color(colors::TEXT_SECONDARY),
                                );
                                ui.add_space(8.0);
                            });
                        });
                    });
                ui.add_space(4.0);
            }
        });
}

/// 下载记录
pub fn render_download_history(_app: &mut PezMaxApp, ui: &mut egui::Ui) {
    ui.add_space(16.0);
    ui.label(
        egui::RichText::new("下载记录")
            .font(FontId::new(22.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_PRIMARY),
    );
    ui.add_space(16.0);

    let records = [
        ("2024高考数学真题", "数学", "2024-06-15", "2.3MB"),
        ("2024高考语文真题", "语文", "2024-06-14", "1.8MB"),
        ("2023高考英语真题", "英语", "2024-06-10", "1.5MB"),
    ];

    egui::ScrollArea::vertical()
        .id_salt("download_scroll")
        .show(ui, |ui| {
            for (name, subject, date, size) in &records {
                egui::Frame::new()
                    .fill(colors::BG_CARD)
                    .corner_radius(CornerRadius::same(0))
                    .stroke(egui::Stroke::new(1.0, colors::BORDER))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.add_space(12.0);
                            ui.label(
                                egui::RichText::new("📄")
                                    .font(FontId::new(
                                        20.0,
                                        egui::FontFamily::Proportional,
                                    )),
                            );
                            ui.add_space(10.0);
                            ui.vertical(|ui| {
                                ui.add_space(8.0);
                                ui.label(
                                    egui::RichText::new(*name)
                                        .font(FontId::new(
                                            14.0,
                                            egui::FontFamily::Proportional,
                                        ))
                                        .color(colors::TEXT_PRIMARY),
                                );
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{} · {} · {}",
                                        subject, date, size
                                    ))
                                    .font(FontId::new(
                                        12.0,
                                        egui::FontFamily::Proportional,
                                    ))
                                    .color(colors::TEXT_SECONDARY),
                                );
                                ui.add_space(8.0);
                            });
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.add_space(12.0);
                                    if ui.small_button("隐藏").clicked() {}
                                },
                            );
                        });
                    });
                ui.add_space(4.0);
            }
        });
}

/// 应用设置
pub fn render_app_settings(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    ui.add_space(16.0);
    ui.label(
        egui::RichText::new("设置")
            .font(FontId::new(22.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_PRIMARY),
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
            settings_info_row(ui, "主题", "浅色（深色模式开发中）");

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
        .fill(colors::BG_CARD)
        .corner_radius(CornerRadius::same(0))
        .stroke(egui::Stroke::new(1.0, colors::BORDER))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.vertical(|ui| {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(label)
                            .font(FontId::new(15.0, egui::FontFamily::Proportional))
                            .color(colors::TEXT_PRIMARY),
                    );
                    ui.label(
                        egui::RichText::new(desc)
                            .font(FontId::new(12.0, egui::FontFamily::Proportional))
                            .color(colors::TEXT_SECONDARY),
                    );
                    ui.add_space(8.0);
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(16.0);
                    ui.label(
                        egui::RichText::new("›")
                            .font(FontId::new(20.0, egui::FontFamily::Proportional))
                            .color(colors::TEXT_SECONDARY),
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
            .color(colors::TEXT_SECONDARY),
    );
    ui.add_space(4.0);
}

/// 带开关的设置行
fn toggle_row(ui: &mut egui::Ui, label: &str, desc: &str, value: &mut bool) {
    egui::Frame::new()
        .fill(colors::BG_CARD)
        .corner_radius(CornerRadius::same(0))
        .stroke(egui::Stroke::new(1.0, colors::BORDER))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.vertical(|ui| {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(label)
                            .font(FontId::new(15.0, egui::FontFamily::Proportional))
                            .color(colors::TEXT_PRIMARY),
                    );
                    ui.label(
                        egui::RichText::new(desc)
                            .font(FontId::new(12.0, egui::FontFamily::Proportional))
                            .color(colors::TEXT_SECONDARY),
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
        .fill(colors::BG_CARD)
        .corner_radius(CornerRadius::same(0))
        .stroke(egui::Stroke::new(1.0, colors::BORDER))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.vertical(|ui| {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(label)
                            .font(FontId::new(15.0, egui::FontFamily::Proportional))
                            .color(colors::TEXT_PRIMARY),
                    );
                    ui.add_space(8.0);
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(16.0);
                    ui.label(
                        egui::RichText::new("›")
                            .font(FontId::new(20.0, egui::FontFamily::Proportional))
                            .color(colors::TEXT_SECONDARY),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(value)
                            .font(FontId::new(13.0, egui::FontFamily::Proportional))
                            .color(colors::TEXT_SECONDARY),
                    );
                });
            });
        });
}

/// 可点击操作行（返回 Response）
fn action_row(ui: &mut egui::Ui, label: &str, desc: &str) -> egui::Response {
    egui::Frame::new()
        .fill(colors::BG_CARD)
        .corner_radius(CornerRadius::same(0))
        .stroke(egui::Stroke::new(1.0, colors::BORDER))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.vertical(|ui| {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(label)
                            .font(FontId::new(15.0, egui::FontFamily::Proportional))
                            .color(colors::TEXT_PRIMARY),
                    );
                    ui.label(
                        egui::RichText::new(desc)
                            .font(FontId::new(12.0, egui::FontFamily::Proportional))
                            .color(colors::TEXT_SECONDARY),
                    );
                    ui.add_space(8.0);
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(16.0);
                    ui.label(
                        egui::RichText::new("›")
                            .font(FontId::new(20.0, egui::FontFamily::Proportional))
                            .color(colors::TEXT_SECONDARY),
                    );
                });
            });
        })
        .response
        .interact(egui::Sense::click())
}
