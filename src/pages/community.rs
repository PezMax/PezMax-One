// 社区功能区
// 三个子标签：用户排行 / 贡献文件 / 举报记录

use crate::app::PezMaxApp;
use crate::api::models::*;
use crate::theme::colors;
use egui::{CornerRadius, FontId, Vec2};

/// 用户排行榜（对接 /datum/user/rank）
pub fn render_user_ranking(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    app.trigger_load_user_rank();

    ui.add_space(16.0);
    ui.label(
        egui::RichText::new("用户排行")
            .font(FontId::new(22.0, egui::FontFamily::Proportional))
            .color(colors::text_primary()),
    );
    ui.add_space(6.0);
    ui.label(
        egui::RichText::new("按贡献度排列的用户列表")
            .font(FontId::new(13.0, egui::FontFamily::Proportional))
            .color(colors::text_secondary()),
    );
    ui.add_space(16.0);

    if app.user_rank_data.is_loading() {
        ui.vertical_centered(|ui| {
            ui.add_space(32.0);
            ui.label(
                egui::RichText::new("加载中…")
                    .font(FontId::new(14.0, egui::FontFamily::Proportional))
                    .color(colors::text_secondary()),
            );
        });
        ui.ctx().request_repaint();
        return;
    }

    if let Some(err) = &app.user_rank_data.error.clone() {
        ui.vertical_centered(|ui| {
            ui.add_space(32.0);
            ui.label(
                egui::RichText::new(format!("加载失败：{}", err))
                    .font(FontId::new(13.0, egui::FontFamily::Proportional))
                    .color(colors::error()),
            );
            ui.add_space(8.0);
            if ui.button("重试").clicked() {
                app.user_rank_data.reset();
            }
        });
        return;
    }

    let items: Vec<UserRankItem> = app
        .user_rank_data
        .data
        .clone()
        .unwrap_or_default();

    if items.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(32.0);
            ui.label(
                egui::RichText::new("暂无排行数据")
                    .font(FontId::new(13.0, egui::FontFamily::Proportional))
                    .color(colors::text_secondary()),
            );
        });
        return;
    }

    egui::ScrollArea::vertical()
        .id_salt("ranking_scroll")
        .show(ui, |ui| {
            for (idx, item) in items.iter().enumerate() {
                let rank = idx + 1;
                let medal = match rank {
                    1 => "🥇",
                    2 => "🥈",
                    3 => "🥉",
                    _ => "  ",
                };
                let row_bg = if rank <= 3 {
                    egui::Color32::from_rgb(0xFF, 0xF8, 0xE8)
                } else {
                    colors::bg_card()
                };

                egui::Frame::new()
                    .fill(row_bg)
                    .corner_radius(CornerRadius::same(0))
                    .stroke(egui::Stroke::new(1.0, colors::border()))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.add_space(16.0);
                            ui.label(
                                egui::RichText::new(medal)
                                    .font(FontId::new(20.0, egui::FontFamily::Proportional)),
                            );
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new(format!("#{}", rank))
                                    .font(FontId::new(13.0, egui::FontFamily::Proportional))
                                    .color(colors::text_secondary()),
                            );
                            ui.add_space(16.0);
                            ui.label(
                                egui::RichText::new(item.display_name())
                                    .font(FontId::new(15.0, egui::FontFamily::Proportional))
                                    .color(colors::text_primary()),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.add_space(16.0);
                                    ui.label(
                                        egui::RichText::new(format!("📥 {}", item.download_count))
                                            .font(FontId::new(13.0, egui::FontFamily::Proportional))
                                            .color(colors::text_secondary()),
                                    );
                                    ui.add_space(16.0);
                                    ui.label(
                                        egui::RichText::new(format!("📤 {} 份", item.upload_count))
                                            .font(FontId::new(13.0, egui::FontFamily::Proportional))
                                            .color(colors::text_secondary()),
                                    );
                                },
                            );
                        });
                        ui.add_space(6.0);
                    });
                ui.add_space(4.0);
            }
        });
}

/// 贡献文件（上传入口 + 元数据表单）
pub fn render_contribute_file(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    ui.add_space(16.0);
    ui.label(
        egui::RichText::new("贡献文件")
            .font(FontId::new(22.0, egui::FontFamily::Proportional))
            .color(colors::text_primary()),
    );
    ui.add_space(6.0);
    ui.label(
        egui::RichText::new("上传试卷资源，帮助更多同学")
            .font(FontId::new(13.0, egui::FontFamily::Proportional))
            .color(colors::text_secondary()),
    );
    ui.add_space(20.0);

    egui::ScrollArea::vertical()
        .id_salt("contribute_scroll")
        .show(ui, |ui| {
            // ── 文件拖放区域 ──────────────────────────────────────
            egui::Frame::new()
                .fill(colors::bg_card())
                .corner_radius(CornerRadius::same(0))
                .stroke(egui::Stroke::new(2.0, colors::border()))
                .show(ui, |ui| {
                    ui.set_min_height(140.0);
                    ui.set_min_width(ui.available_width());
                    ui.vertical_centered(|ui| {
                        ui.add_space(28.0);
                        ui.label(
                            egui::RichText::new("📤")
                                .font(FontId::new(40.0, egui::FontFamily::Proportional)),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new("点击选择文件或拖放至此处")
                                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                .color(colors::text_secondary()),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new("支持 PDF · 最大 50MB")
                                .font(FontId::new(12.0, egui::FontFamily::Proportional))
                                .color(colors::text_secondary()),
                        );
                        ui.add_space(10.0);
                        let btn = egui::Button::new(
                            egui::RichText::new("  选择文件  ")
                                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                .color(colors::text_on_primary()),
                        )
                        .fill(colors::primary())
                        .corner_radius(CornerRadius::same(0));
                        if ui.add(btn).clicked() {
                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                let file = rfd::FileDialog::new()
                                    .add_filter("PDF", &["pdf"])
                                    .pick_file();
                                if let Some(path) = file {
                                    app.contribute_file_path = Some(path.display().to_string());
                                }
                            }
                        }
                        ui.add_space(20.0);
                    });
                });

            ui.add_space(20.0);

            // ── 元数据表单 ────────────────────────────────────────
            ui.label(
                egui::RichText::new("文件信息")
                    .font(FontId::new(16.0, egui::FontFamily::Proportional))
                    .color(colors::text_primary()),
            );
            ui.add_space(8.0);

            egui::Frame::new()
                .fill(colors::bg_card())
                .corner_radius(CornerRadius::same(0))
                .stroke(egui::Stroke::new(1.0, colors::border()))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.add_space(16.0);

                    contribute_field(ui, "学科", &mut app.contribute_subject, "如：数学");
                    ui.add_space(10.0);
                    contribute_field(ui, "学校", &mut app.contribute_school, "如：全国卷");
                    ui.add_space(10.0);
                    contribute_field(ui, "年份", &mut app.contribute_year, "如：2024");

                    ui.add_space(16.0);

                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        let can_submit = !app.contribute_subject.is_empty()
                            && !app.contribute_year.is_empty();
                        let btn = egui::Button::new(
                            egui::RichText::new("  提交上传  ")
                                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                .color(colors::text_on_primary()),
                        )
                        .fill(if can_submit {
                            colors::accent_green()
                        } else {
                            colors::bg_hover()
                        })
                        .corner_radius(CornerRadius::same(0));

                        if ui.add_enabled(can_submit, btn).clicked() {
                            let api = app.api.clone();
                            let subject = app.contribute_subject.clone();
                            let school = app.contribute_school.clone();
                            let year = app.contribute_year.clone();
                            app.contribute_subject.clear();
                            app.contribute_school.clear();
                            app.contribute_year.clear();
                            tokio::spawn(async move {
                                let file = PaperFile {
                                    file_subject: subject,
                                    school_name: school,
                                    file_year: year.parse().unwrap_or(0),
                                    ..Default::default()
                                };
                                let _ = api.create_file(&file).await;
                            });
                        }
                    });
                    ui.add_space(16.0);
                });

            ui.add_space(20.0);

            // ── 上传统计 ──────────────────────────────────────────
            ui.label(
                egui::RichText::new("我的贡献")
                    .font(FontId::new(16.0, egui::FontFamily::Proportional))
                    .color(colors::text_primary()),
            );
            ui.add_space(8.0);

            egui::Frame::new()
                .fill(colors::bg_card())
                .corner_radius(CornerRadius::same(0))
                .stroke(egui::Stroke::new(1.0, colors::border()))
                .show(ui, |ui| {
                    ui.set_min_height(60.0);
                    ui.set_min_width(ui.available_width());
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        let upload_count = app
                            .user_stats_data
                            .data
                            .as_ref()
                            .map(|s| s.upload_count)
                            .unwrap_or(0);
                        ui.label(
                            egui::RichText::new(format!("已上传 {} 份试卷", upload_count))
                                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                .color(colors::text_primary()),
                        );
                    });
                    ui.add_space(16.0);
                });
        });
}

fn contribute_field(ui: &mut egui::Ui, label: &str, value: &mut String, hint: &str) {
    ui.horizontal(|ui| {
        ui.add_space(16.0);
        ui.add_sized(
            Vec2::new(60.0, 20.0),
            egui::Label::new(
                egui::RichText::new(label)
                    .font(FontId::new(14.0, egui::FontFamily::Proportional))
                    .color(colors::text_secondary()),
            ),
        );
        ui.add_space(8.0);
        let edit = egui::TextEdit::singleline(value)
            .hint_text(hint)
            .desired_width(200.0)
            .font(FontId::new(14.0, egui::FontFamily::Proportional));
        ui.add(edit);
    });
}

/// 举报记录（对接 /datum/report/list）
pub fn render_report_record(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    app.trigger_load_my_reports();

    ui.add_space(16.0);
    ui.label(
        egui::RichText::new("举报记录")
            .font(FontId::new(22.0, egui::FontFamily::Proportional))
            .color(colors::text_primary()),
    );
    ui.add_space(6.0);
    ui.label(
        egui::RichText::new("举报违规内容，维护社区环境")
            .font(FontId::new(13.0, egui::FontFamily::Proportional))
            .color(colors::text_secondary()),
    );
    ui.add_space(20.0);

    // 举报表单
    egui::Frame::new()
        .fill(colors::bg_card())
        .corner_radius(CornerRadius::same(0))
        .stroke(egui::Stroke::new(1.0, colors::border()))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.add_space(16.0);

            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("举报内容")
                        .font(FontId::new(14.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary()),
                );
                ui.add_space(8.0);
                let edit = egui::TextEdit::singleline(&mut app.report_content)
                    .hint_text("请描述违规内容")
                    .desired_width(300.0)
                    .font(FontId::new(14.0, egui::FontFamily::Proportional));
                ui.add(edit);
            });
            ui.add_space(12.0);

            ui.horizontal(|ui| {
                ui.add_space(16.0);
                let btn = egui::Button::new(
                    egui::RichText::new("  🚩 提交举报  ")
                        .font(FontId::new(14.0, egui::FontFamily::Proportional))
                        .color(colors::text_on_primary()),
                )
                .fill(if !app.report_content.is_empty() {
                    colors::error()
                } else {
                    colors::bg_hover()
                })
                .corner_radius(CornerRadius::same(0));

                if ui.add_enabled(!app.report_content.is_empty(), btn).clicked() {
                    let api = app.api.clone();
                    let content = app.report_content.clone();
                    app.report_content.clear();
                    // 提交后重置列表，以便下次进入时重新加载
                    app.my_reports_data.reset();
                    tokio::spawn(async move {
                        let report = Report {
                            content,
                            report_type: "file".to_string(),
                            ..Default::default()
                        };
                        let _ = api.create_report(&report).await;
                    });
                }
            });
            ui.add_space(16.0);
        });

    ui.add_space(20.0);
    ui.label(
        egui::RichText::new("我的举报")
            .font(FontId::new(16.0, egui::FontFamily::Proportional))
            .color(colors::text_primary()),
    );
    ui.add_space(8.0);

    if app.my_reports_data.is_loading() {
        egui::Frame::new()
            .fill(colors::bg_card())
            .corner_radius(CornerRadius::same(0))
            .stroke(egui::Stroke::new(1.0, colors::border()))
            .show(ui, |ui| {
                ui.set_min_height(80.0);
                ui.set_min_width(ui.available_width());
                ui.vertical_centered(|ui| {
                    ui.add_space(24.0);
                    ui.label(
                        egui::RichText::new("加载中…")
                            .font(FontId::new(13.0, egui::FontFamily::Proportional))
                            .color(colors::text_secondary()),
                    );
                    ui.add_space(24.0);
                });
            });
        ui.ctx().request_repaint();
        return;
    }

    let reports: Vec<Report> = app.my_reports_data.data.clone().unwrap_or_default();

    if reports.is_empty() {
        egui::Frame::new()
            .fill(colors::bg_card())
            .corner_radius(CornerRadius::same(0))
            .stroke(egui::Stroke::new(1.0, colors::border()))
            .show(ui, |ui| {
                ui.set_min_height(80.0);
                ui.set_min_width(ui.available_width());
                ui.vertical_centered(|ui| {
                    ui.add_space(24.0);
                    ui.label(
                        egui::RichText::new("暂无举报记录")
                            .font(FontId::new(13.0, egui::FontFamily::Proportional))
                            .color(colors::text_secondary()),
                    );
                    ui.add_space(24.0);
                });
            });
        return;
    }

    egui::ScrollArea::vertical()
        .id_salt("reports_scroll")
        .show(ui, |ui| {
            for report in &reports {
                egui::Frame::new()
                    .fill(colors::bg_card())
                    .corner_radius(CornerRadius::same(0))
                    .stroke(egui::Stroke::new(1.0, colors::border()))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.add_space(16.0);
                            ui.label(
                                egui::RichText::new(&report.content)
                                    .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                    .color(colors::text_primary()),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.add_space(16.0);
                                    let status_text = match report.status.as_str() {
                                        "0" => ("待处理", colors::text_secondary()),
                                        "1" => ("已处理", colors::accent_green()),
                                        "2" => ("已驳回", colors::error()),
                                        _ => ("未知", colors::text_secondary()),
                                    };
                                    ui.label(
                                        egui::RichText::new(status_text.0)
                                            .font(FontId::new(12.0, egui::FontFamily::Proportional))
                                            .color(status_text.1),
                                    );
                                    ui.add_space(8.0);
                                    ui.label(
                                        egui::RichText::new(&report.create_time)
                                            .font(FontId::new(12.0, egui::FontFamily::Proportional))
                                            .color(colors::text_secondary()),
                                    );
                                },
                            );
                        });
                        ui.add_space(10.0);
                    });
                ui.add_space(4.0);
            }
        });
}
