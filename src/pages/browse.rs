// 浏览功能区
// 三个子标签：资源管理 / 外部书签 / 我的收藏

use crate::app::PezMaxApp;
use crate::api::client::ApiClient;
use crate::api::models::*;
use crate::sokuou::map_range;
use crate::theme::colors;
use egui::{Color32, CornerRadius, FontId, Vec2};

static YEARS: &[(&str, Option<i32>)] = &[
    ("全部", None),
    ("2024", Some(2024)),
    ("2023", Some(2023)),
    ("2022", Some(2022)),
];

/// 资源管理：筛选器 + 文件网格  ↔  文件预览面板
pub fn render_resource_manager(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    // 自动加载数据
    if !app.file_list_data.is_loaded() && !app.file_list_data.is_loading() {
        app.trigger_load_file_list();
    }
    if !app.subjects_data.is_loaded() && !app.subjects_data.is_loading() {
        app.trigger_load_subjects();
    }
    if !app.schools_data.is_loaded() && !app.schools_data.is_loading() {
        app.trigger_load_schools();
    }

    // 预览面板优先显示
    if app.selected_file.is_some() {
        render_file_preview(app, ui);
        return;
    }

    // 加载骨架屏
    if app.file_list_data.is_loading() && !app.file_list_data.is_loaded() {
        render_skeleton(ui);
        return;
    }

    let active_sub = app.filters.subject.clone();
    let active_year = app.filters.year;
    let active_school = app.filters.school.clone();
    let search_q = app.search_query.to_lowercase();

    // ── 学科筛选 ──────────────────────────────────────────────
    ui.add_space(16.0);
    ui.label(
        egui::RichText::new("学科")
            .font(FontId::new(12.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_SECONDARY),
    );
    ui.add_space(6.0);

    let subjects = app.subjects_data.data.as_ref().map(|s| s.as_slice()).unwrap_or(&[]);
    let mut new_sub: Option<Option<String>> = None;
    ui.horizontal_wrapped(|ui| {
        let all_active = active_sub.is_none();
        if filter_chip(ui, "全部", all_active) {
            new_sub = Some(None);
        }
        ui.add_space(4.0);
        for sub in subjects {
            let sub_opt = Some(sub.clone());
            let active = active_sub == sub_opt;
            if filter_chip(ui, sub, active) {
                new_sub = Some(sub_opt);
            }
            ui.add_space(4.0);
        }
    });
    if let Some(s) = new_sub {
        app.filters.subject = s;
    }

    ui.add_space(10.0);

    // ── 年份筛选 ──────────────────────────────────────────────
    ui.label(
        egui::RichText::new("年份")
            .font(FontId::new(12.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_SECONDARY),
    );
    ui.add_space(6.0);

    let mut new_year: Option<Option<i32>> = None;
    ui.horizontal_wrapped(|ui| {
        for &(label, yr_val) in YEARS {
            let active = active_year == yr_val;
            if filter_chip(ui, label, active) {
                new_year = Some(yr_val);
            }
            ui.add_space(4.0);
        }
    });
    if let Some(y) = new_year {
        app.filters.year = y;
    }

    ui.add_space(10.0);

    // ── 学校筛选 ──────────────────────────────────────────────
    ui.label(
        egui::RichText::new("学校")
            .font(FontId::new(12.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_SECONDARY),
    );
    ui.add_space(6.0);

    let schools = app.schools_data.data.as_ref().map(|s| s.as_slice()).unwrap_or(&[]);
    let mut new_school: Option<Option<String>> = None;
    ui.horizontal_wrapped(|ui| {
        let all_active = active_school.is_none();
        if filter_chip(ui, "全部", all_active) {
            new_school = Some(None);
        }
        ui.add_space(4.0);
        for sch in schools {
            let sch_opt = Some(sch.clone());
            let active = active_school == sch_opt;
            if filter_chip(ui, sch, active) {
                new_school = Some(sch_opt);
            }
            ui.add_space(4.0);
        }
    });
    if let Some(s) = new_school {
        app.filters.school = s;
    }

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);

    // ── 过滤文件列表 ─────────────────────────────────────────
    let cur_sub = app.filters.subject.clone();
    let cur_year = app.filters.year;
    let cur_school = app.filters.school.clone();

    let files = app.file_list_data.data.as_ref().map(|f| f.as_slice()).unwrap_or(&[]);
    let filtered: Vec<&PaperFile> = files
        .iter()
        .filter(|f| {
            let sub_ok = cur_sub.as_deref().map_or(true, |s| s == f.file_subject);
            let yr_ok = cur_year.map_or(true, |y| y.to_string() == f.file_year);
            let sch_ok = cur_school.as_deref().map_or(true, |s| s == f.school_name);
            let q_ok = search_q.is_empty()
                || f.file_name.to_lowercase().contains(&search_q)
                || f.file_subject.to_lowercase().contains(&search_q);
            sub_ok && yr_ok && sch_ok && q_ok
        })
        .collect();

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("共 {} 份试卷", filtered.len()))
                .font(FontId::new(13.0, egui::FontFamily::Proportional))
                .color(colors::TEXT_SECONDARY),
        );
    });
    ui.add_space(12.0);

    // ── 文件卡片网格 ──────────────────────────────────────────
    let mut select_file: Option<PaperFile> = None;

    egui::ScrollArea::vertical()
        .id_salt("browse_scroll")
        .show(ui, |ui| {
            if filtered.is_empty() {
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("没有找到符合条件的试卷")
                            .font(FontId::new(15.0, egui::FontFamily::Proportional))
                            .color(colors::TEXT_SECONDARY),
                    );
                });
            } else {
                ui.horizontal_wrapped(|ui| {
                    for file in &filtered {
                        if file_card(ui, file, &app.api) {
                            select_file = Some((*file).clone());
                        }
                        ui.add_space(10.0);
                    }
                });
            }
        });

    if let Some(file) = select_file {
        app.selected_file = Some(file);
        app.preview_anim = crate::sokuou::SpringAnim::with_target(0.4, 0.8, 0.0, 0.0, 1.0);
    }
}

/// 文件预览面板（主从视图）
fn render_file_preview(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    let Some(ref file) = app.selected_file else {
        return;
    };

    let v = app.preview_anim.value();
    let y_offset = map_range(v, 16.0, 0.0) as f32;
    if y_offset > 0.1 {
        ui.add_space(y_offset);
    }

    // ── 顶部操作栏 ────────────────────────────────────────────
    ui.add_space(12.0);
    let mut go_back = false;

    ui.horizontal(|ui| {
        ui.add_space(16.0);
        let back_btn = egui::Button::new(
            egui::RichText::new("← 返回列表")
                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                .color(colors::PRIMARY),
        )
        .fill(Color32::TRANSPARENT)
        .corner_radius(CornerRadius::same(0));

        if ui.add(back_btn).clicked() {
            go_back = true;
        }
    });

    if go_back {
        app.selected_file = None;
        app.preview_anim.set_target(0.0);
        return;
    }

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(16.0);

    let size_str = if file.file_size > 0 {
        format!("{:.1} MB", file.file_size as f64 / 1048576.0)
    } else {
        "-".to_string()
    };

    egui::ScrollArea::vertical()
        .id_salt("preview_scroll")
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(16.0);

                // ── 左侧：元数据卡片 + 操作按钮 ──────────────────
                egui::Frame::new()
                    .fill(colors::BG_CARD)
                    .corner_radius(CornerRadius::same(0))
                    .stroke(egui::Stroke::new(1.0, colors::BORDER))
                    .show(ui, |ui| {
                        ui.set_min_size(Vec2::new(240.0, 320.0));
                        ui.set_max_width(240.0);
                        ui.add_space(20.0);

                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new("📄")
                                    .font(FontId::new(56.0, egui::FontFamily::Proportional)),
                            );
                        });
                        ui.add_space(12.0);

                        let meta = [
                            ("文件名", &file.file_name),
                            ("学科", &file.file_subject),
                            ("学校", &file.school_name),
                            ("年份", &file.file_year),
                            ("大小", &size_str),
                            ("上传者", &file.file_uploader),
                        ];

                        for (key, val) in &meta {
                            ui.horizontal(|ui| {
                                ui.add_space(16.0);
                                ui.label(
                                    egui::RichText::new(*key)
                                        .font(FontId::new(12.0, egui::FontFamily::Proportional))
                                        .color(colors::TEXT_SECONDARY),
                                );
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new(*val)
                                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                                        .color(colors::TEXT_PRIMARY),
                                );
                            });
                            ui.add_space(6.0);
                        }

                        ui.add_space(16.0);
                        ui.separator();
                        ui.add_space(12.0);

                        ui.vertical_centered(|ui| {
                            let dl_btn = egui::Button::new(
                                egui::RichText::new("  📥 下载文件  ")
                                    .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                    .color(colors::TEXT_ON_PRIMARY),
                            )
                            .fill(colors::PRIMARY)
                            .min_size(Vec2::new(200.0, 36.0))
                            .corner_radius(CornerRadius::same(0));
                            if ui.add(dl_btn).clicked() {
                                let api = app.api.clone();
                                let fid = file.file_id;
                                tokio::spawn(async move {
                                    let _ = api.download_paper(fid).await;
                                });
                            }
                            ui.add_space(8.0);

                            let fav_btn = egui::Button::new(
                                egui::RichText::new("  ⭐ 收藏  ")
                                    .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                    .color(colors::ACCENT_ORANGE),
                            )
                            .fill(colors::BG_HOVER)
                            .min_size(Vec2::new(200.0, 36.0))
                            .corner_radius(CornerRadius::same(0));
                            if ui.add(fav_btn).clicked() {
                                let api = app.api.clone();
                                let fid = file.file_id;
                                tokio::spawn(async move {
                                    let _ = api.add_favorite(fid).await;
                                });
                            }
                            ui.add_space(8.0);

                            let rep_btn = egui::Button::new(
                                egui::RichText::new("  🚩 举报  ")
                                    .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                    .color(colors::ERROR),
                            )
                            .fill(colors::BG_HOVER)
                            .min_size(Vec2::new(200.0, 36.0))
                            .corner_radius(CornerRadius::same(0));
                            if ui.add(rep_btn).clicked() {}
                        });

                        ui.add_space(20.0);
                    });

                ui.add_space(16.0);

                // ── 右侧：PDF 预览占位 ────────────────────────────
                egui::Frame::new()
                    .fill(Color32::from_gray(240))
                    .corner_radius(CornerRadius::same(0))
                    .stroke(egui::Stroke::new(1.0, colors::BORDER))
                    .show(ui, |ui| {
                        ui.set_min_size(Vec2::new(
                            ui.available_width().max(300.0),
                            320.0,
                        ));
                        ui.vertical_centered(|ui| {
                            ui.add_space(120.0);
                            ui.label(
                                egui::RichText::new("📋")
                                    .font(FontId::new(48.0, egui::FontFamily::Proportional)),
                            );
                            ui.add_space(12.0);
                            ui.label(
                                egui::RichText::new("PDF 预览")
                                    .font(FontId::new(15.0, egui::FontFamily::Proportional))
                                    .color(colors::TEXT_SECONDARY),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new("下载后可用系统阅读器打开")
                                    .font(FontId::new(12.0, egui::FontFamily::Proportional))
                                    .color(colors::TEXT_SECONDARY),
                            );
                        });
                    });
            });
        });
}

/// 骨架屏：加载时代替文件网格
fn render_skeleton(ui: &mut egui::Ui) {
    ui.add_space(16.0);
    ui.horizontal_wrapped(|ui| {
        for _ in 0..6 {
            egui::Frame::new()
                .fill(colors::BG_CARD)
                .corner_radius(CornerRadius::same(0))
                .stroke(egui::Stroke::new(1.0, colors::BORDER))
                .show(ui, |ui| {
                    ui.set_min_size(Vec2::new(200.0, 130.0));
                    ui.set_max_width(220.0);
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        ui.add_space(12.0);
                        egui::Frame::new()
                            .fill(Color32::from_gray(220))
                            .show(ui, |ui| {
                                ui.allocate_space(Vec2::new(28.0, 28.0));
                            });
                        ui.add_space(10.0);
                        ui.vertical(|ui| {
                            egui::Frame::new()
                                .fill(Color32::from_gray(215))
                                .show(ui, |ui| {
                                    ui.allocate_space(Vec2::new(120.0, 13.0));
                                });
                            ui.add_space(6.0);
                            egui::Frame::new()
                                .fill(Color32::from_gray(225))
                                .show(ui, |ui| {
                                    ui.allocate_space(Vec2::new(80.0, 12.0));
                                });
                            ui.add_space(4.0);
                            egui::Frame::new()
                                .fill(Color32::from_gray(225))
                                .show(ui, |ui| {
                                    ui.allocate_space(Vec2::new(60.0, 11.0));
                                });
                        });
                    });
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.add_space(12.0);
                        egui::Frame::new()
                            .fill(Color32::from_gray(220))
                            .show(ui, |ui| {
                                ui.allocate_space(Vec2::new(56.0, 20.0));
                            });
                        ui.add_space(4.0);
                        egui::Frame::new()
                            .fill(Color32::from_gray(220))
                            .show(ui, |ui| {
                                ui.allocate_space(Vec2::new(56.0, 20.0));
                            });
                    });
                    ui.add_space(8.0);
                });
            ui.add_space(10.0);
        }
    });
}

/// 外部书签管理（含新建表单）
pub fn render_bookmarks(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    // 自动加载书签
    if !app.bookmarks_data.is_loaded() && !app.bookmarks_data.is_loading() {
        app.trigger_load_bookmarks();
    }

    ui.add_space(16.0);
    ui.label(
        egui::RichText::new("外部书签")
            .font(FontId::new(22.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_PRIMARY),
    );
    ui.add_space(6.0);
    ui.label(
        egui::RichText::new("管理您收藏的外部书签链接")
            .font(FontId::new(13.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_SECONDARY),
    );
    ui.add_space(16.0);

    // ── 新建书签表单 ──────────────────────────────────────────
    egui::Frame::new()
        .fill(colors::BG_CARD)
        .corner_radius(CornerRadius::same(0))
        .stroke(egui::Stroke::new(1.0, colors::BORDER))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("添加新书签")
                        .font(FontId::new(15.0, egui::FontFamily::Proportional))
                        .color(colors::TEXT_PRIMARY),
                );
            });
            ui.add_space(12.0);

            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("名称")
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(colors::TEXT_SECONDARY),
                );
                ui.add_space(8.0);
                let name_edit = egui::TextEdit::singleline(&mut app.bookmark_form_name)
                    .hint_text("书签名称")
                    .desired_width(200.0)
                    .font(FontId::new(14.0, egui::FontFamily::Proportional));
                ui.add(name_edit);
            });
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("链接")
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(colors::TEXT_SECONDARY),
                );
                ui.add_space(8.0);
                let url_edit = egui::TextEdit::singleline(&mut app.bookmark_form_url)
                    .hint_text("https://...")
                    .desired_width(300.0)
                    .font(FontId::new(14.0, egui::FontFamily::Proportional));
                ui.add(url_edit);
            });
            ui.add_space(12.0);

            ui.horizontal(|ui| {
                ui.add_space(16.0);
                let can_submit =
                    !app.bookmark_form_name.is_empty() && !app.bookmark_form_url.is_empty();
                let btn = egui::Button::new(
                    egui::RichText::new("  添加  ")
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(colors::TEXT_ON_PRIMARY),
                )
                .fill(if can_submit {
                    colors::PRIMARY
                } else {
                    colors::BG_HOVER
                })
                .corner_radius(CornerRadius::same(0));

                if ui.add_enabled(can_submit, btn).clicked() {
                    let api = app.api.clone();
                    let name = app.bookmark_form_name.clone();
                    let url = app.bookmark_form_url.clone();
                    app.bookmark_form_name.clear();
                    app.bookmark_form_url.clear();
                    tokio::spawn(async move {
                        let bm = Bookmark {
                            bookmark_name: name,
                            bookmark_url: url,
                            ..Default::default()
                        };
                        let _ = api.create_bookmark(&bm).await;
                    });
                }
            });
            ui.add_space(16.0);
        });

    ui.add_space(16.0);

    // ── 书签列表 ─────────────────────────────────────────────
    if let Some(ref list) = app.bookmarks_data.data {
        for bookmark in list {
            egui::Frame::new()
                .fill(colors::BG_CARD)
                .corner_radius(CornerRadius::same(0))
                .stroke(egui::Stroke::new(1.0, colors::BORDER))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.add_space(12.0);
                        ui.label(
                            egui::RichText::new("🔖")
                                .font(FontId::new(18.0, egui::FontFamily::Proportional)),
                        );
                        ui.add_space(10.0);
                        ui.vertical(|ui| {
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new(&bookmark.bookmark_name)
                                    .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                    .color(colors::TEXT_PRIMARY),
                            );
                            ui.label(
                                egui::RichText::new(&bookmark.bookmark_url)
                                    .font(FontId::new(12.0, egui::FontFamily::Proportional))
                                    .color(colors::TEXT_SECONDARY),
                            );
                            ui.add_space(8.0);
                        });
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                ui.add_space(12.0);
                                if ui.small_button("删除").clicked() {
                                    let api = app.api.clone();
                                    let bid = bookmark.id;
                                    tokio::spawn(async move {
                                        let _ = api.delete_bookmark(bid).await;
                                    });
                                }
                            },
                        );
                    });
                });
            ui.add_space(4.0);
        }
    } else {
        egui::Frame::new()
            .fill(colors::BG_CARD)
            .corner_radius(CornerRadius::same(0))
            .stroke(egui::Stroke::new(1.0, colors::BORDER))
            .show(ui, |ui| {
                ui.set_min_height(120.0);
                ui.set_min_width(ui.available_width());
                ui.vertical_centered(|ui| {
                    ui.add_space(36.0);
                    ui.label(
                        egui::RichText::new("🔖")
                            .font(FontId::new(32.0, egui::FontFamily::Proportional)),
                    );
                    ui.add_space(6.0);
                    let msg = if app.bookmarks_data.is_loading() {
                        "加载中..."
                    } else {
                        "暂无书签"
                    };
                    ui.label(
                        egui::RichText::new(msg)
                            .font(FontId::new(14.0, egui::FontFamily::Proportional))
                            .color(colors::TEXT_SECONDARY),
                    );
                    ui.add_space(36.0);
                });
            });
    }
}

/// 我的收藏列表
pub fn render_favorites(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    // 自动加载收藏
    if !app.favorites_data.is_loaded() && !app.favorites_data.is_loading() {
        app.trigger_load_favorites();
    }

    ui.add_space(16.0);
    ui.label(
        egui::RichText::new("我的收藏")
            .font(FontId::new(22.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_PRIMARY),
    );
    ui.add_space(16.0);

    egui::ScrollArea::vertical()
        .id_salt("favorites_scroll")
        .show(ui, |ui| {
            if let Some(ref list) = app.favorites_data.data {
                for fav in list {
                    egui::Frame::new()
                        .fill(colors::BG_CARD)
                        .corner_radius(CornerRadius::same(0))
                        .stroke(egui::Stroke::new(1.0, colors::BORDER))
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.add_space(12.0);
                                ui.label(
                                    egui::RichText::new("⭐")
                                        .font(FontId::new(18.0, egui::FontFamily::Proportional)),
                                );
                                ui.add_space(10.0);
                                ui.vertical(|ui| {
                                    ui.add_space(8.0);
                                    ui.label(
                                        egui::RichText::new(&fav.file_name)
                                            .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                            .color(colors::TEXT_PRIMARY),
                                    );
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{} · 收藏于 {}",
                                            fav.file_subject, fav.create_time
                                        ))
                                        .font(FontId::new(12.0, egui::FontFamily::Proportional))
                                        .color(colors::TEXT_SECONDARY),
                                    );
                                    ui.add_space(8.0);
                                });
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.add_space(12.0);
                                        if ui.small_button("取消收藏").clicked() {
                                            let api = app.api.clone();
                                            let uid = app.current_user.as_ref().map(|u| u.user_id).unwrap_or(0);
                                            let fid = fav.file_id;
                                            tokio::spawn(async move {
                                                let _ = api.remove_favorite(uid, fid).await;
                                            });
                                        }
                                        ui.add_space(4.0);
                                        if ui.small_button("📥 下载").clicked() {}
                                    },
                                );
                            });
                        });
                    ui.add_space(6.0);
                }
            } else {
                let msg = if app.favorites_data.is_loading() {
                    "加载中..."
                } else {
                    "暂无收藏"
                };
                ui.label(
                    egui::RichText::new(msg)
                        .font(FontId::new(14.0, egui::FontFamily::Proportional))
                        .color(colors::TEXT_SECONDARY),
                );
            }
        });
}

// ── 内部组件 ─────────────────────────────────────────────────────────────────

/// 筛选器 chip 按钮（32px 高），返回是否点击
fn filter_chip(ui: &mut egui::Ui, label: &str, active: bool) -> bool {
    let (bg, fg) = if active {
        (colors::PRIMARY, colors::TEXT_ON_PRIMARY)
    } else {
        (colors::BG_HOVER, colors::TEXT_SECONDARY)
    };
    ui.add(
        egui::Button::new(
            egui::RichText::new(label)
                .font(FontId::new(13.0, egui::FontFamily::Proportional))
                .color(fg),
        )
        .fill(bg)
        .min_size(Vec2::new(0.0, 32.0))
        .corner_radius(CornerRadius::same(0)),
    )
    .clicked()
}

/// 文件卡片，返回是否被点击（打开预览）
fn file_card(ui: &mut egui::Ui, file: &PaperFile, api: &ApiClient) -> bool {
    let size_str = if file.file_size > 0 {
        format!("{:.1} MB", file.file_size as f64 / 1048576.0)
    } else {
        "-".to_string()
    };

    let resp = egui::Frame::new()
        .fill(colors::BG_CARD)
        .corner_radius(CornerRadius::same(0))
        .stroke(egui::Stroke::new(1.0, colors::BORDER))
        .show(ui, |ui| {
            ui.set_min_size(Vec2::new(200.0, 130.0));
            ui.set_max_width(220.0);
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new("📄")
                        .font(FontId::new(28.0, egui::FontFamily::Proportional)),
                );
                ui.add_space(10.0);
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(&file.file_name)
                            .font(FontId::new(13.0, egui::FontFamily::Proportional))
                            .color(colors::TEXT_PRIMARY),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(format!("{} · {}", file.file_subject, file.file_year))
                            .font(FontId::new(12.0, egui::FontFamily::Proportional))
                            .color(colors::TEXT_SECONDARY),
                    );
                    ui.label(
                        egui::RichText::new(format!("{} · {}", file.school_name, size_str))
                            .font(FontId::new(11.0, egui::FontFamily::Proportional))
                            .color(colors::TEXT_SECONDARY),
                    );
                });
            });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                if ui.small_button("📥 下载").clicked() {
                    let api = api.clone();
                    let fid = file.file_id;
                    tokio::spawn(async move {
                        let _ = api.download_paper(fid).await;
                    });
                }
                ui.add_space(4.0);
                if ui.small_button("⭐ 收藏").clicked() {
                    let api = api.clone();
                    let fid = file.file_id;
                    tokio::spawn(async move {
                        let _ = api.add_favorite(fid).await;
                    });
                }
            });
            ui.add_space(8.0);
        })
        .response
        .interact(egui::Sense::click());

    resp.clicked()
}