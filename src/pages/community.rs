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
    // 提前触发学科/学校列表加载，用于表单自动补全 (#9)
    if !app.subjects_data.is_loaded() && !app.subjects_data.is_loading() {
        app.trigger_load_subjects();
    }
    if !app.schools_data.is_loaded() && !app.schools_data.is_loading() {
        app.trigger_load_schools();
    }

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
                            egui::RichText::new("  选择文件（可多选）  ")
                                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                .color(colors::text_on_primary()),
                        )
                        .fill(colors::primary())
                        .corner_radius(CornerRadius::same(0));
                        if ui.add(btn).clicked() {
                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                let files = rfd::FileDialog::new()
                                    .add_filter("PDF", &["pdf"])
                                    .pick_files();
                                if let Some(paths) = files {
                                    for path in paths {
                                        let name = path
                                            .file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or("")
                                            .to_string();
                                        let fmt = path
                                            .extension()
                                            .and_then(|e| e.to_str())
                                            .map(|s| s.to_lowercase())
                                            .unwrap_or_else(|| "pdf".to_string());
                                        let size = std::fs::metadata(&path).ok().map(|m| m.len()).unwrap_or(0);
                                        // 去重（按路径）
                                        let p_str = path.display().to_string();
                                        if !app.contribute_files.iter().any(|f| f.path == p_str) {
                                            app.contribute_files.push(crate::app::ContributeFile {
                                                path: p_str,
                                                name,
                                                format: fmt,
                                                size,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        ui.add_space(12.0);
                    });
                });

            // ── 已选文件列表 ────────────────────────────────────
            if !app.contribute_files.is_empty() {
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new(format!("已选 {} 个文件", app.contribute_files.len()))
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary()),
                );
                ui.add_space(6.0);
                // 上传进行中不能删条目
                let uploading = app.contribute_uploading;
                let mut remove_idx: Option<usize> = None;
                for (i, cf) in app.contribute_files.iter().enumerate() {
                    egui::Frame::new()
                        .fill(colors::bg_card())
                        .stroke(egui::Stroke::new(1.0, colors::border()))
                        .corner_radius(CornerRadius::same(0))
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.add_space(10.0);
                                let size_str = if cf.size < 1024 * 1024 {
                                    format!("{:.1} KB", cf.size as f64 / 1024.0)
                                } else {
                                    format!("{:.1} MB", cf.size as f64 / 1024.0 / 1024.0)
                                };
                                ui.vertical(|ui| {
                                    ui.add_space(6.0);
                                    ui.label(
                                        egui::RichText::new(&cf.name)
                                            .font(FontId::new(13.0, egui::FontFamily::Proportional))
                                            .color(colors::text_primary()),
                                    );
                                    ui.label(
                                        egui::RichText::new(size_str)
                                            .font(FontId::new(11.0, egui::FontFamily::Proportional))
                                            .color(colors::text_secondary()),
                                    );
                                    ui.add_space(6.0);
                                });
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.add_space(8.0);
                                    if ui
                                        .add_enabled(
                                            !uploading,
                                            egui::Button::new(
                                                egui::RichText::new("移除")
                                                    .font(FontId::new(12.0, egui::FontFamily::Proportional))
                                                    .color(colors::text_secondary()),
                                            )
                                            .fill(colors::bg_hover())
                                            .corner_radius(CornerRadius::same(0)),
                                        )
                                        .clicked()
                                    {
                                        remove_idx = Some(i);
                                    }
                                });
                            });
                        });
                    ui.add_space(4.0);
                }
                if let Some(i) = remove_idx {
                    app.contribute_files.remove(i);
                }
            }

            // 上传进度显示
            if app.contribute_uploading || !app.contribute_upload_errors.is_empty() {
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new(format!(
                        "已上传 {} / {}",
                        app.contribute_upload_done, app.contribute_upload_total
                    ))
                    .font(FontId::new(12.0, egui::FontFamily::Proportional))
                    .color(colors::primary()),
                );
                for (name, err) in &app.contribute_upload_errors {
                    ui.label(
                        egui::RichText::new(format!("✗ {} — {}", name, err))
                            .font(FontId::new(11.0, egui::FontFamily::Proportional))
                            .color(colors::error()),
                    );
                }
            }

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

                    // 学科：自动补全
                    let subjects: Vec<String> = app.subjects_data.data.clone().unwrap_or_default();
                    contribute_field_autocomplete(
                        ui,
                        "学科",
                        &mut app.contribute_subject,
                        "如：数学",
                        &subjects,
                    );
                    ui.add_space(10.0);
                    // 学校：自动补全
                    let schools: Vec<String> = app.schools_data.data.clone().unwrap_or_default();
                    contribute_field_autocomplete(
                        ui,
                        "学校",
                        &mut app.contribute_school,
                        "如：全国卷",
                        &schools,
                    );
                    ui.add_space(10.0);
                    contribute_field(ui, "年份", &mut app.contribute_year, "如：2024");

                    ui.add_space(16.0);

                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        let has_file = !app.contribute_files.is_empty();
                        let can_submit = has_file
                            && !app.contribute_subject.is_empty()
                            && !app.contribute_year.is_empty()
                            && !app.contribute_uploading;
                        let label = if app.contribute_uploading {
                            "  上传中…  "
                        } else if app.contribute_files.len() > 1 {
                            "  批量提交上传  "
                        } else {
                            "  提交上传  "
                        };
                        let btn = egui::Button::new(
                            egui::RichText::new(label)
                                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                .color(colors::text_on_primary()),
                        )
                        .fill(if can_submit { colors::primary() } else { colors::bg_hover() })
                        .corner_radius(CornerRadius::same(0));

                        if ui.add_enabled(can_submit, btn).clicked() {
                            app.trigger_contribute_upload();
                        }
                        if !has_file {
                            ui.add_space(12.0);
                            ui.label(
                                egui::RichText::new("请先选择要上传的文件")
                                    .font(FontId::new(11.0, egui::FontFamily::Proportional))
                                    .color(colors::text_secondary()),
                            );
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

/// 带自动补全的字段：候选来源 `options`，最多展示 6 条匹配项。
/// - 值命中已有条目 → 复用（分组归并）
/// - 值不为空且未命中 → 提示 "将新建 XX：YY"
fn contribute_field_autocomplete(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut String,
    hint: &str,
    options: &[String],
) {
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
        ui.vertical(|ui| {
            ui.scope(|ui| {
                crate::theme::apply_search_style(ui);
                ui.add(
                    egui::TextEdit::singleline(value)
                        .hint_text(hint)
                        .desired_width(240.0)
                        .font(FontId::new(14.0, egui::FontFamily::Proportional)),
                );
            });

            let v_trim = value.trim().to_string();
            let v_lower = v_trim.to_lowercase();
            let exact_match = !v_trim.is_empty()
                && options.iter().any(|o| o.eq_ignore_ascii_case(&v_trim));

            // 只在有输入 且没有精确命中时才展示候选
            if !v_lower.is_empty() && !exact_match {
                let matches: Vec<String> = options
                    .iter()
                    .filter(|o| o.to_lowercase().contains(&v_lower))
                    .take(6)
                    .cloned()
                    .collect();
                if !matches.is_empty() {
                    ui.add_space(4.0);
                    egui::Frame::new()
                        .fill(colors::bg_card())
                        .stroke(egui::Stroke::new(1.0, colors::border()))
                        .corner_radius(CornerRadius::same(0))
                        .show(ui, |ui| {
                            ui.set_min_width(240.0);
                            for opt in matches {
                                let resp = ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(&opt)
                                            .font(FontId::new(13.0, egui::FontFamily::Proportional))
                                            .color(colors::text_primary()),
                                    )
                                    .sense(egui::Sense::click()),
                                );
                                if resp.hovered() {
                                    let bg = colors::bg_hover();
                                    ui.painter().rect_filled(
                                        resp.rect,
                                        CornerRadius::ZERO,
                                        bg,
                                    );
                                    ui.painter().text(
                                        resp.rect.left_center() + egui::vec2(4.0, 0.0),
                                        egui::Align2::LEFT_CENTER,
                                        &opt,
                                        FontId::new(13.0, egui::FontFamily::Proportional),
                                        colors::text_primary(),
                                    );
                                }
                                if resp.clicked() {
                                    *value = opt.clone();
                                }
                            }
                        });
                }
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(format!("提示：将新建{}：{}", label, v_trim))
                        .font(FontId::new(11.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary()),
                );
            } else if exact_match {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(format!("✓ 已归入已有{}", label))
                        .font(FontId::new(11.0, egui::FontFamily::Proportional))
                        .color(colors::primary()),
                );
            }
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
        egui::RichText::new("查看你提交的所有举报及处理进度。想举报文件？打开文件详情，点举报按钮。")
            .font(FontId::new(13.0, egui::FontFamily::Proportional))
            .color(colors::text_secondary()),
    );
    ui.add_space(20.0);

    // ── 状态筛选 chip 行 ─────────────────────────────
    ui.horizontal(|ui| {
        let filters: [(Option<i64>, &str); 5] = [
            (None, "全部"),
            (Some(0), "待审核"),
            (Some(1), "已通过"),
            (Some(2), "已下架"),
            (Some(3), "已驳回"),
        ];
        for (val, label) in filters {
            let selected = app.report_status_filter == val;
            let btn = egui::Button::new(
                egui::RichText::new(label)
                    .font(FontId::new(12.0, egui::FontFamily::Proportional))
                    .color(if selected { colors::text_on_primary() } else { colors::text_secondary() }),
            )
            .fill(if selected { colors::primary() } else { colors::bg_input() })
            .stroke(egui::Stroke::NONE)
            .corner_radius(CornerRadius::ZERO)
            .min_size(egui::vec2(0.0, 26.0));
            if ui.add(btn).clicked() && !selected {
                app.report_status_filter = val;
            }
            ui.add_space(6.0);
        }
    });

    ui.add_space(16.0);

    // ── 记录列表 ────────────────────────────────────
    if app.my_reports_data.is_loading() && app.my_reports_data.data.is_none() {
        loading_placeholder(ui);
        ui.ctx().request_repaint();
        return;
    }

    let reports: Vec<Report> = app.my_reports_data
        .data
        .clone()
        .unwrap_or_default()
        .into_iter()
        .filter(|r| match app.report_status_filter {
            None => true,
            Some(v) => r.result.unwrap_or_else(|| r.status.parse().unwrap_or(0)) == v,
        })
        .collect();

    if reports.is_empty() {
        empty_placeholder(ui, "没有匹配的举报记录");
        return;
    }

    egui::ScrollArea::vertical()
        .id_salt("reports_scroll")
        .show(ui, |ui| {
            for report in &reports {
                let report_id = report.report_id;
                let (rect, resp) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 72.0),
                    egui::Sense::click(),
                );
                ui.painter().rect_filled(rect, CornerRadius::ZERO, colors::bg_card());
                ui.painter().rect_stroke(
                    rect,
                    CornerRadius::ZERO,
                    egui::Stroke::new(1.0, colors::border()),
                    egui::StrokeKind::Outside,
                );
                // 左 3px 强调色条
                let status = report.result.unwrap_or_else(|| report.status.parse().unwrap_or(0));
                let (status_label, status_color) = match status {
                    0 => ("待审核", colors::text_secondary()),
                    1 => ("已通过", colors::primary()),
                    2 => ("已下架", colors::primary()),
                    3 => ("已驳回", egui::Color32::from_rgb(200, 60, 60)),
                    _ => ("未知", colors::text_secondary()),
                };
                ui.painter().rect_filled(
                    egui::Rect::from_min_max(
                        rect.left_top(),
                        egui::pos2(rect.left() + 3.0, rect.bottom()),
                    ),
                    CornerRadius::ZERO,
                    status_color,
                );

                // 左侧：文件名 + 举报理由
                ui.painter().text(
                    egui::pos2(rect.left() + 20.0, rect.top() + 12.0),
                    egui::Align2::LEFT_TOP,
                    if report.file_name.is_empty() { format!("#{}", report.file_id) } else { report.file_name.clone() },
                    FontId::new(14.0, egui::FontFamily::Proportional),
                    colors::text_primary(),
                );
                let content_display = if report.content.is_empty() { "（未填写理由）" } else { &report.content };
                ui.painter().text(
                    egui::pos2(rect.left() + 20.0, rect.top() + 34.0),
                    egui::Align2::LEFT_TOP,
                    content_display,
                    FontId::new(11.0, egui::FontFamily::Proportional),
                    colors::text_secondary(),
                );
                ui.painter().text(
                    egui::pos2(rect.left() + 20.0, rect.top() + 52.0),
                    egui::Align2::LEFT_TOP,
                    &report.create_time,
                    FontId::new(11.0, egui::FontFamily::Proportional),
                    colors::text_secondary(),
                );

                // 右侧：状态徽标 + "查看进度 ›"
                ui.painter().text(
                    egui::pos2(rect.right() - 16.0, rect.top() + 16.0),
                    egui::Align2::RIGHT_TOP,
                    status_label,
                    FontId::new(12.0, egui::FontFamily::Proportional),
                    status_color,
                );
                ui.painter().text(
                    egui::pos2(rect.right() - 16.0, rect.bottom() - 18.0),
                    egui::Align2::RIGHT_BOTTOM,
                    "查看进度 ›",
                    FontId::new(11.0, egui::FontFamily::Proportional),
                    colors::primary(),
                );

                if resp.clicked() {
                    app.trigger_load_report_timeline(report_id);
                }
                ui.add_space(6.0);
            }

            // "加载更多"按钮：数据长度等于 pageSize * pageNum 就展示
            if app.report_has_more && !app.my_reports_data.is_loading() {
                ui.add_space(8.0);
                ui.vertical_centered(|ui| {
                    let btn = egui::Button::new(
                        egui::RichText::new("加载更多")
                            .font(FontId::new(12.0, egui::FontFamily::Proportional))
                            .color(colors::primary()),
                    )
                    .fill(colors::bg_input())
                    .stroke(egui::Stroke::NONE)
                    .corner_radius(CornerRadius::ZERO)
                    .min_size(egui::vec2(120.0, 28.0));
                    if ui.add(btn).clicked() {
                        app.report_page_num += 1;
                        app.my_reports_data.reset();
                    }
                });
            }
        });
}

fn loading_placeholder(ui: &mut egui::Ui) {
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
}

fn empty_placeholder(ui: &mut egui::Ui, msg: &str) {
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
                    egui::RichText::new(msg)
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary()),
                );
                ui.add_space(24.0);
            });
        });
}
