// 首页/仪表盘
// 三段布局：统计卡片 / 快速操作 / 最近更新

use crate::app::{PezMaxApp, Section, Subsection};
use crate::theme::colors;
use egui::{CornerRadius, FontId, Vec2};

pub fn render(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical()
        .id_salt("home_scroll")
        .show(ui, |ui| {
            ui.add_space(24.0);

            // ── 欢迎标题 ───────────────────────────────────────────
            let default_name = "用户".to_string();
            let nickname = app
                .current_user
                .as_ref()
                .map(|u| &u.nick_name)
                .unwrap_or(&default_name);

            let hour = {
                use std::time::{SystemTime, UNIX_EPOCH};
                let secs = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                ((secs % 86400) / 3600 + 8) % 24 // UTC+8 近似
            };
            let greeting = if hour < 6 {
                "深夜好"
            } else if hour < 12 {
                "早上好"
            } else if hour < 18 {
                "下午好"
            } else {
                "晚上好"
            };

            ui.label(
                egui::RichText::new(format!("{}，{}", greeting, nickname))
                    .font(FontId::new(28.0, egui::FontFamily::Proportional))
                    .color(colors::text_primary()),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("欢迎使用 PezMax 试卷资源管理系统")
                    .font(FontId::new(14.0, egui::FontFamily::Proportional))
                    .color(colors::text_secondary()),
            );

            ui.add_space(28.0);

            // ── 第一段：统计卡片 ────────────────────────────────────
            let (fav, dl, ul) = if let Some(ref s) = app.user_stats {
                (s.favorite_count, s.download_count, s.upload_count)
            } else {
                (0, 0, 0)
            };

            let stat_tiles = [
                ("⭐", "我的收藏", fav, colors::accent_orange()),
                ("📥", "下载次数", dl, colors::primary()),
                ("📤", "上传贡献", ul, colors::accent_green()),
            ];

            ui.horizontal_wrapped(|ui| {
                for (icon, label, value, color) in &stat_tiles {
                    egui::Frame::new()
                        .fill(*color)
                        .corner_radius(CornerRadius::same(0))
                        .show(ui, |ui| {
                            ui.set_min_size(Vec2::new(160.0, 110.0));
                            ui.add_space(16.0);
                            ui.horizontal(|ui| {
                                ui.add_space(16.0);
                                ui.label(
                                    egui::RichText::new(*icon)
                                        .font(FontId::new(28.0, egui::FontFamily::Proportional)),
                                );
                            });
                            ui.add_space(6.0);
                            ui.horizontal(|ui| {
                                ui.add_space(16.0);
                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new(value.to_string())
                                            .font(FontId::new(
                                                26.0,
                                                egui::FontFamily::Proportional,
                                            ))
                                            .color(colors::text_on_primary()),
                                    );
                                    ui.label(
                                        egui::RichText::new(*label)
                                            .font(FontId::new(
                                                13.0,
                                                egui::FontFamily::Proportional,
                                            ))
                                            .color(colors::text_on_primary()),
                                    );
                                });
                            });
                        });
                    ui.add_space(12.0);
                }
            });

            ui.add_space(28.0);

            // ── 第二段：快速操作 ────────────────────────────────────
            ui.label(
                egui::RichText::new("快速操作")
                    .font(FontId::new(16.0, egui::FontFamily::Proportional))
                    .color(colors::text_secondary()),
            );
            ui.add_space(8.0);

            let mut nav_to: Option<(Section, Subsection)> = None;

            ui.horizontal_wrapped(|ui| {
                let quick_actions = [
                    (
                        "📁",
                        "浏览试卷",
                        "查看并下载试卷资源",
                        Section::Browse,
                        Subsection::ResourceManager,
                        colors::primary(),
                    ),
                    (
                        "📤",
                        "贡献文件",
                        "上传你的试卷，帮助大家",
                        Section::Community,
                        Subsection::ContributeFile,
                        colors::accent_green(),
                    ),
                ];

                for (icon, title, desc, section, sub, color) in quick_actions {
                    let resp = egui::Frame::new()
                        .fill(colors::bg_card())
                        .corner_radius(CornerRadius::same(0))
                        .stroke(egui::Stroke::new(1.5, color))
                        .show(ui, |ui| {
                            ui.set_min_size(Vec2::new(220.0, 72.0));
                            ui.add_space(12.0);
                            ui.horizontal(|ui| {
                                ui.add_space(14.0);
                                egui::Frame::new()
                                    .fill(color)
                                    .corner_radius(CornerRadius::same(0))
                                    .show(ui, |ui| {
                                        ui.set_min_size(Vec2::new(40.0, 40.0));
                                        ui.set_max_size(Vec2::new(40.0, 40.0));
                                        ui.vertical_centered(|ui| {
                                            ui.add_space(4.0);
                                            ui.label(
                                                egui::RichText::new(icon).font(FontId::new(
                                                    22.0,
                                                    egui::FontFamily::Proportional,
                                                )),
                                            );
                                        });
                                    });
                                ui.add_space(12.0);
                                ui.vertical(|ui| {
                                    ui.add_space(4.0);
                                    ui.label(
                                        egui::RichText::new(title)
                                            .font(FontId::new(
                                                15.0,
                                                egui::FontFamily::Proportional,
                                            ))
                                            .color(colors::text_primary()),
                                    );
                                    ui.label(
                                        egui::RichText::new(desc)
                                            .font(FontId::new(
                                                12.0,
                                                egui::FontFamily::Proportional,
                                            ))
                                            .color(colors::text_secondary()),
                                    );
                                });
                            });
                        })
                        .response
                        .interact(egui::Sense::click());

                    if resp.clicked() {
                        nav_to = Some((section, sub));
                    }
                    ui.add_space(12.0);
                }
            });

            if let Some((s, sub)) = nav_to {
                app.navigate_to(s, sub);
            }

            ui.add_space(28.0);

            // ── 第三段：最近更新 ────────────────────────────────────
            ui.label(
                egui::RichText::new("最近更新")
                    .font(FontId::new(16.0, egui::FontFamily::Proportional))
                    .color(colors::text_secondary()),
            );
            ui.add_space(8.0);

            if let Some(ref files) = app.recent_files.data {
                for file in files.iter().take(10) {
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
                                        egui::RichText::new(&file.file_name)
                                            .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                            .color(colors::text_primary()),
                                    );
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{} · {} · by {}",
                                            file.file_subject, file.create_time, file.create_by
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
                                        if ui.small_button("📥 下载").clicked() {}
                                    },
                                );
                            });
                        });
                    ui.add_space(4.0);
                }
            } else {
                ui.label(
                    egui::RichText::new(if app.recent_files.is_loading() { "加载中..." } else { "暂无最近更新" })
                        .font(FontId::new(14.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary()),
                );
            }

            ui.add_space(16.0);
        });
}
