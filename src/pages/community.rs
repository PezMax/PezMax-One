// 社区功能区
// 三个子标签：用户排行 / 贡献文件 / 举报记录

use crate::app::PezMaxApp;
use crate::api::models::*;
use crate::theme::colors;
use egui::{Color32, CornerRadius, FontId, Rect, StrokeKind, Vec2, pos2};

// ── 头像颜色预设（12 种 Metro 色板，基于 user_id 循环分配）───────────────
const AVATAR_PALETTE: [(u8, u8, u8); 12] = [
    (0x3B, 0x82, 0xF6), // 钴蓝
    (0x1D, 0xB9, 0x54), // 云杉绿
    (0xEF, 0x44, 0x44), // 绯红
    (0xF5, 0x9E, 0x0B), // 琥珀
    (0x8B, 0x5C, 0xF6), // 堇紫
    (0x00, 0xBC, 0x70), // 翡翠
    (0xE0, 0x67, 0xC9), // 粉紫
    (0x00, 0xB7, 0xC3), // 青碧
    (0xF7, 0x63, 0x00), // 橙
    (0x54, 0x6E, 0x7A), // 钢蓝
    (0x9C, 0x27, 0xB0), // 深紫
    (0x4C, 0xAF, 0x50), // 草绿
];

fn avatar_color(user_id: i64) -> (u8, u8, u8) {
    AVATAR_PALETTE[(user_id as usize) % 12]
}

fn avatar_initial(name: &str) -> String {
    name.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_else(|| "?".to_string())
}

/// 绘制头像：优先真实头像（方形），备用色块+首字母
fn draw_avatar(
    ui: &mut egui::Ui,
    app: &mut PezMaxApp,
    item: &UserRankItem,
    avatar_rect: Rect,
    avatar_size: f32,
    r: u8, g: u8, b: u8,
    initial: &str,
) {
    if let Some(textures) = app.rank_avatar_textures.get(&item.user_id) {
        if !textures.is_empty() {
            let tex_size = textures[0].size();
            let (tw, th) = (tex_size[0] as f32, tex_size[1] as f32);
            let uv_rect = if (tw - th).abs() > 1.0 {
                if tw > th {
                    let crop = (1.0 - th / tw) / 2.0;
                    Rect::from_min_max(pos2(crop, 0.0), pos2(1.0 - crop, 1.0))
                } else {
                    let crop = (1.0 - tw / th) / 2.0;
                    Rect::from_min_max(pos2(0.0, crop), pos2(1.0, 1.0 - crop))
                }
            } else {
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0))
            };
            let frame_idx = app.rank_avatar_frame_idx.get(&item.user_id).copied().unwrap_or(0);
            let tex_idx = frame_idx.min(textures.len() - 1);
            // 方形色块垫底 + 图片覆盖
            ui.painter().rect_filled(avatar_rect, CornerRadius::ZERO, Color32::from_rgb(r, g, b));
            ui.painter().image(textures[tex_idx].id(), avatar_rect, uv_rect, Color32::WHITE);
            return;
        }
    }
    // 备用色块
    fallback_avatar(ui, avatar_rect, r, g, b, initial);
}

/// 备用色块头像（方块）
fn fallback_avatar(ui: &mut egui::Ui, rect: Rect, r: u8, g: u8, b: u8, initial: &str) {
    ui.painter().rect_filled(rect, CornerRadius::ZERO, egui::Color32::from_rgb(r, g, b));
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        initial,
        egui::FontId::new(20.0, egui::FontFamily::Proportional),
        egui::Color32::WHITE,
    );
}

/// 奖牌色：与 rank 1-3 对应的左强调条颜色
fn medal_bar_color(rank: usize) -> egui::Color32 {
    match rank {
        1 => egui::Color32::from_rgb(0xFF, 0xBF, 0x00), // 金
        2 => egui::Color32::from_rgb(0xC0, 0xC0, 0xC0), // 银
        3 => egui::Color32::from_rgb(0xCD, 0x7F, 0x32), // 铜
        _ => colors::primary(),
    }
}

/// 用户排行榜（对接 /datum/user/rank）
pub fn render_user_ranking(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    app.trigger_load_user_rank();

    ui.add_space(16.0);
    ui.label(
        egui::RichText::new("🏆 用户排行")
            .font(FontId::new(24.0, egui::FontFamily::Proportional))
            .color(colors::text_primary()),
    );
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new("按贡献度排列的用户列表")
            .font(FontId::new(13.0, egui::FontFamily::Proportional))
            .color(colors::text_secondary()),
    );
    ui.add_space(20.0);

    if app.user_rank_data.is_loading() {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
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
            ui.add_space(40.0);
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
            ui.add_space(40.0);
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
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // 关键：清除一切间距，卡片依次紧挨
            ui.spacing_mut().item_spacing.y = 0.0;

            let card_width = ui.available_width();

            for (idx, item) in items.iter().enumerate() {
                let rank = idx + 1;
                let (rank_label, rank_label_color) = match rank {
                    1 => ("#1", Color32::from_rgb(0xD4, 0x8B, 0x0A)),
                    2 => ("#2", Color32::from_rgb(0x80, 0x80, 0x80)),
                    3 => ("#3", Color32::from_rgb(0xAD, 0x6B, 0x2B)),
                    _ => ("", Color32::BLACK),
                };

                let bar_color = medal_bar_color(rank);
                let (r, g, b) = avatar_color(item.user_id);
                let initial = avatar_initial(item.display_name());

                let rank_text_color = if rank <= 3 {
                    if crate::theme::is_dark() { Color32::WHITE } else { Color32::BLACK }
                } else {
                    colors::text_primary()
                };
                let rank_secondary_color = if rank <= 3 {
                    if crate::theme::is_dark() { Color32::from_gray(200) } else { Color32::from_gray(60) }
                } else {
                    colors::text_secondary()
                };
                let row_bg = if rank == 1 {
                    if crate::theme::is_dark() { Color32::from_rgb(0x3A, 0x30, 0x10) } else { Color32::from_rgb(0xFF, 0xF8, 0xE0) }
                } else if rank == 2 {
                    if crate::theme::is_dark() { Color32::from_rgb(0x30, 0x30, 0x30) } else { Color32::from_rgb(0xF5, 0xF5, 0xF5) }
                } else if rank == 3 {
                    if crate::theme::is_dark() { Color32::from_rgb(0x33, 0x2B, 0x1E) } else { Color32::from_rgb(0xFD, 0xF5, 0xEB) }
                } else {
                    colors::bg_card()
                };

                // 卡片之间无间距，紧挨排列
                let card_rect = ui.allocate_exact_size(
                    Vec2::new(card_width, 80.0),
                    egui::Sense::hover(),
                ).0;

                let cy = card_rect.center().y;

                // 背景 + 边框（使用 Inside 避免边框重叠）
                ui.painter().rect(card_rect, CornerRadius::ZERO, row_bg, egui::Stroke::new(1.0, colors::border()), StrokeKind::Inside);
                // 左强调色条
                ui.painter().rect_filled(
                    Rect::from_min_max(pos2(card_rect.left(), card_rect.top()), pos2(card_rect.left() + 4.0, card_rect.bottom())),
                    CornerRadius::ZERO, bar_color,
                );

                // 头像（48px）
                let avatar_size = 48.0;
                let avatar_rect = Rect::from_center_size(
                    pos2(card_rect.left() + 4.0 + 16.0 + avatar_size / 2.0, cy),
                    Vec2::splat(avatar_size),
                );

                if let Some(textures) = app.rank_avatar_textures.get(&item.user_id) {
                    if !textures.is_empty() {
                        // 方形头像：色块垫底 + 图片覆盖
                        draw_avatar(ui, app, item, avatar_rect, avatar_size, r, g, b, &initial);
                    } else {
                        fallback_avatar(ui, avatar_rect, r, g, b, &initial);
                    }
                } else {
                    fallback_avatar(ui, avatar_rect, r, g, b, &initial);
                }

                // 中间文本
                let text_x = avatar_rect.right() + 16.0;
                let mut line_x = text_x;

                if rank <= 3 {
                    let r = ui.painter().text(pos2(line_x, cy), egui::Align2::LEFT_CENTER, rank_label, FontId::new(16.0, egui::FontFamily::Proportional), rank_label_color);
                    line_x += r.width() + 10.0;
                } else {
                    ui.painter().text(pos2(line_x, cy), egui::Align2::LEFT_CENTER, format!("#{}", rank), FontId::new(14.0, egui::FontFamily::Proportional), colors::text_secondary());
                    line_x += 30.0;
                }
                ui.painter().text(pos2(line_x, cy), egui::Align2::LEFT_CENTER, item.display_name(), FontId::new(17.0, egui::FontFamily::Proportional), rank_text_color);

                // 右侧上传数量
                let upload_clr = if rank == 1 { Color32::from_rgb(0xE6, 0x7E, 0x22) } else { colors::primary() };
                let right_x = card_rect.right() - 80.0;
                ui.painter().text(pos2(right_x, cy - 14.0), egui::Align2::CENTER_CENTER, format!("{}", item.upload_count), FontId::new(26.0, egui::FontFamily::Proportional), upload_clr);
                ui.painter().text(pos2(right_x, cy + 14.0), egui::Align2::CENTER_CENTER, "份上传", FontId::new(12.0, egui::FontFamily::Proportional), rank_secondary_color);
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
        ui.scope(|ui| {
            crate::theme::apply_search_style(ui);
            ui.add(
                egui::TextEdit::singleline(value)
                    .hint_text(hint)
                    .desired_width(200.0)
                    .font(FontId::new(14.0, egui::FontFamily::Proportional)),
            );
        });
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
                ui.scope(|ui| {
                    crate::theme::apply_search_style(ui);
                    ui.add(
                        egui::TextEdit::singleline(&mut app.report_content)
                            .hint_text("请描述违规内容")
                            .desired_width(300.0)
                            .font(FontId::new(14.0, egui::FontFamily::Proportional)),
                    );
                });
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
