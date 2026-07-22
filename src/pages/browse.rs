// 浏览功能区
// 三个子标签：资源管理 / 外部书签 / 我的收藏

use crate::app::PezMaxApp;
use crate::api::models::*;
use crate::components::action_bar::Action;
use crate::pdf;
use crate::sokuou::map_range;
use crate::theme::colors;
use crate::app::ToastLevel;
use egui::{Color32, CornerRadius, FontId, Vec2};
use tokio::sync::oneshot;

/// 资源管理：筛选器（从数据派生，三级级联）+ 文件纵向列表
pub fn render_resource_manager(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    if !app.file_list_data.is_loaded() && !app.file_list_data.is_loading() {
        app.trigger_load_file_list();
    }

    if app.selected_file.is_some() {
        render_file_preview(app, ui);
        return;
    }

    if app.file_list_data.is_loading() {
        render_skeleton(ui);
        ui.ctx().request_repaint();
        return;
    }

    if let Some(ref err) = app.file_list_data.error.clone() {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new(format!("加载失败：{}", err))
                    .font(FontId::new(14.0, egui::FontFamily::Proportional))
                    .color(colors::error()),
            );
            ui.add_space(12.0);
            if ui
                .button(egui::RichText::new("重新加载").font(FontId::new(13.0, egui::FontFamily::Proportional)))
                .clicked()
            {
                app.file_list_data.reset();
            }
        });
        return;
    }

    // ── 从文件列表派生级联标签 ───────────────────────────────
    // Phase 1: 读取当前筛选状态（clone 避免后续借用冲突）
    let active_sub   = app.filters.subject.clone();
    let active_school = app.filters.school.clone();
    let search_q     = app.search_query.to_lowercase();

    // 从数据中提取已审核文件，并派生筛选选项
    let (subjects, schools, filtered_files) = {
        let all = app.file_list_data.data.as_deref().unwrap_or(&[]);
        let approved: Vec<&PaperFile> = all
            .iter()
            .filter(|f| !matches!(f.file_status, Some(0) | Some(2)))
            .collect();

        // 学科：全量
        let subjects: Vec<String> = {
            let mut set = std::collections::BTreeSet::new();
            for f in &approved {
                if !f.file_subject.is_empty() {
                    set.insert(f.file_subject.clone());
                }
            }
            set.into_iter().collect()
        };

        // 学校：随学科过滤
        let schools: Vec<String> = {
            let mut set = std::collections::BTreeSet::new();
            for f in &approved {
                let ok = active_sub.as_deref().map_or(true, |s| s == f.file_subject);
                if ok && !f.school_name.is_empty() {
                    set.insert(f.school_name.clone());
                }
            }
            set.into_iter().collect()
        };

        // 最终过滤结果
        let filtered: Vec<PaperFile> = approved
            .into_iter()
            .filter(|f| {
                let sub_ok = active_sub.as_deref().map_or(true, |s| s == f.file_subject);
                let sch_ok = active_school.as_deref().map_or(true, |s| s == f.school_name);
                let q_ok   = search_q.is_empty()
                    || f.file_name.to_lowercase().contains(&search_q)
                    || f.file_subject.to_lowercase().contains(&search_q);
                sub_ok && sch_ok && q_ok
            })
            .cloned()
            .collect();

        (subjects, schools, filtered)
    };

    // Phase 2: 级联重置（上级变化时清空下级选项）
    if let Some(ref sch) = active_school {
        if !schools.contains(sch) {
            app.filters.school = None;
        }
    }

    // Phase 3: 渲染筛选器，收集用户操作
    let mut new_sub: Option<Option<String>> = None;
    let mut new_school: Option<Option<String>> = None;

    ui.add_space(8.0);

    // ── 紧凑水平筛选栏 ────────────────────────────────────
    ui.horizontal(|ui| {
        // 学科组
        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new("学科")
                    .font(FontId::new(12.0, egui::FontFamily::Proportional))
                    .color(colors::text_secondary()),
            );
            let sub_label = active_sub.as_deref().unwrap_or("全部");
            egui::ComboBox::from_id_salt("subject_filter")
                .selected_text(
                    egui::RichText::new(sub_label)
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(if active_sub.is_some() { colors::primary() } else { colors::text_primary() }),
                )
                .width(120.0)
                .show_ui(ui, |ui| {
                    if ui.selectable_label(active_sub.is_none(), "全部").clicked() {
                        new_sub = Some(None);
                    }
                    for sub in &subjects {
                        let active = active_sub.as_deref() == Some(sub.as_str());
                        if ui.selectable_label(active, sub).clicked() {
                            new_sub = Some(Some(sub.clone()));
                        }
                    }
                });
        });

        ui.add_space(16.0);

        // 学校组
        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new("学校")
                    .font(FontId::new(12.0, egui::FontFamily::Proportional))
                    .color(colors::text_secondary()),
            );
            let sch_label = active_school.as_deref().unwrap_or("全部");
            egui::ComboBox::from_id_salt("school_filter")
                .selected_text(
                    egui::RichText::new(sch_label)
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(if active_school.is_some() { colors::primary() } else { colors::text_primary() }),
                )
                .width(140.0)
                .show_ui(ui, |ui| {
                    if ui.selectable_label(active_school.is_none(), "全部").clicked() {
                        new_school = Some(None);
                    }
                    for sch in &schools {
                        let active = active_school.as_deref() == Some(sch.as_str());
                        if ui.selectable_label(active, sch).clicked() {
                            new_school = Some(Some(sch.clone()));
                        }
                    }
                });
        });

        ui.add_space(16.0);

        // 筛选计数 + 清除按钮
        let active_count = app.filters.subject.is_some() as i32
            + app.filters.school.is_some() as i32;
        if active_count > 0 {
            ui.label(
                egui::RichText::new(format!("已选 {} 项", active_count))
                    .font(FontId::new(12.0, egui::FontFamily::Proportional))
                    .color(colors::primary()),
            );
            ui.add_space(8.0);
            let clear_btn = egui::Button::new(
                egui::RichText::new("✕ 清除")
                    .font(FontId::new(12.0, egui::FontFamily::Proportional))
                    .color(colors::text_secondary()),
            )
            .fill(egui::Color32::TRANSPARENT)
            .corner_radius(egui::CornerRadius::same(0));
            if ui.add(clear_btn).clicked() {
                new_sub = Some(None);
                new_school = Some(None);
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("共 {} 份试卷", filtered_files.len()))
                    .font(FontId::new(12.0, egui::FontFamily::Proportional))
                    .color(colors::text_secondary()),
            );
        });
    });

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);

    // Phase 4: 纵向文件列表
    let mut select_file: Option<PaperFile> = None;
    egui::ScrollArea::vertical()
        .id_salt("browse_scroll")
        .show(ui, |ui| {
            if filtered_files.is_empty() {
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("没有找到符合条件的试卷")
                            .font(FontId::new(15.0, egui::FontFamily::Proportional))
                            .color(colors::text_secondary()),
                    );
                });
            } else {
                for file in &filtered_files {
                    if file_row(ui, file, app) {
                        select_file = Some(file.clone());
                    }
                    ui.add_space(4.0);
                }
            }
        });

    // Phase 5: 应用状态变更
    if let Some(s) = new_sub {
        if s != app.filters.subject {
            app.filters.school = None;
        }
        app.filters.subject = s;
    }
    if let Some(sch) = new_school {
        if sch != app.filters.school {
        }
        app.filters.school = sch;
    }
    if let Some(file) = select_file {
        let fid = file.file_id;
        app.selected_file = Some(file);
        app.preview_anim = crate::sokuou::SpringAnim::with_target(0.4, 0.8, 0.0, 0.0, 1.0);
        // 直接启动 PDF 加载，跳过预览按钮中间步骤
        app.trigger_load_pdf_bytes(fid);
    }
}

/// 文件预览面板（全屏 PDF 阅读器 + 底部操作栏由 app.rs 渲染）
fn render_file_preview(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    let Some(ref file) = app.selected_file else {
        return;
    };

    let file_id = file.file_id;
    let file_name = file.file_name.clone();
    let file_subject = file.file_subject.clone();
    let school_name = file.school_name.clone();
    let file_size = file.file_size;
    let create_by = file.create_by.clone();

    let v = app.preview_anim.value();
    let y_offset = map_range(v, 16.0, 0.0) as f32;
    if y_offset > 0.1 {
        ui.add_space(y_offset);
    }

    // 处理 app.rs 渲染的操作栏点击
    match app.preview_bar_action {
        Action::Back => {
            app.selected_file = None;
            app.preview_anim.set_target(0.0);
            return;
        }
        Action::Download => {
            let api = app.api.clone();
            let fid = file_id;
            let fname = file_name.clone();
            tokio::spawn(async move {
                // 选择保存路径
                let file = rfd::AsyncFileDialog::new()
                    .set_file_name(&fname)
                    .add_filter("PDF", &["pdf"])
                    .save_file()
                    .await;
                if let Some(file) = file {
                    match api.download_paper(fid).await {
                        Ok(bytes) => {
                            let _ = std::fs::write(file.path(), &bytes);
                        }
                        Err(e) => {
                            log::error!("下载失败: {}", e);
                        }
                    }
                }
            });
        }
        Action::Favorite => {
            let api = app.api.clone();
            let fid = file_id;
            let uid = app.current_user.as_ref().map(|u| u.user_id).unwrap_or(0);
            let is_fav = app.favorite_file_ids.contains(&fid);
            let msg = if is_fav {
                format!("已取消收藏 {}", file_name)
            } else {
                format!("已收藏 {}", file_name)
            };
            if is_fav {
                // 取消收藏
                app.favorite_file_ids.remove(&fid);
                // 乐观更新统计数据
                if let Some(ref mut stats) = app.user_stats {
                    stats.favorite_count = (stats.favorite_count - 1).max(0);
                }
                tokio::spawn(async move {
                    match api.remove_favorite(uid, fid).await {
                        Ok(_) => log::info!("取消收藏成功: {}", fid),
                        Err(e) => log::error!("取消收藏失败: {}", e),
                    }
                });
            } else {
                // 添加收藏
                app.favorite_file_ids.insert(fid);
                // 乐观更新统计数据
                if let Some(ref mut stats) = app.user_stats {
                    stats.favorite_count += 1;
                }
                tokio::spawn(async move {
                    match api.add_favorite(uid, fid).await {
                        Ok(_) => log::info!("收藏成功: {}", fid),
                        Err(e) => log::error!("收藏失败: {}", e),
                    }
                });
            }
            app.add_toast(msg, ToastLevel::Success);
            // 后台刷新统计（确保下次打开时数据一致）
            app.trigger_load_user_stats();
        }
        Action::Report => {
            app.show_report_dialog = true;
            app.report_content.clear();
            app.report_type = "侵权".to_string();
        }
        Action::ToggleInfo => {
            app.show_info_dialog = true;
        }
        Action::None => {}
    }

    // ── 主内容：PDF 阅读器（全宽渲染） ────────────────────────
    if app.pdf_viewer.loaded {
        let engine = app.pdf_engine.clone();
        pdf::render_pdf_viewer(ui, &mut app.pdf_viewer, &engine);
    } else {
        render_pdf_placeholder(ui, &file_name, &file_subject, &school_name, file_id, app);
    }

    // ── 文件信息弹窗（Metro Design） ──────────────────────────
    if app.show_info_dialog {
        let size_str = if file_size > 0 {
            format!("{:.1} MB", file_size as f64 / 1048576.0)
        } else {
            "-".to_string()
        };

        // 消除窗口阴影（Metro Design 无阴影）
        let mut style = (*ui.ctx().style()).clone();
        style.visuals.window_shadow = egui::Shadow::NONE;
        ui.ctx().set_style(style);

        egui::Window::new("文件信息")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                ui.set_min_width(360.0);

                let info_rect = ui.available_rect_before_wrap();

                // 左侧强调色条（Metro Design 标志）
                let accent_bar = egui::Rect::from_min_size(
                    egui::pos2(info_rect.left() - ui.style().spacing.window_margin.left as f32, info_rect.top()),
                    egui::vec2(3.0, ui.min_rect().height().max(260.0)),
                );
                ui.painter().rect_filled(accent_bar, egui::CornerRadius::ZERO, colors::primary());

                ui.vertical_centered(|ui| {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("📄")
                            .font(FontId::new(36.0, egui::FontFamily::Proportional)),
                    );
                    ui.add_space(10.0);
                });

                let info_rows: &[(&str, &str)] = &[
                    ("文件名", &file_name),
                    ("学科", &file_subject),
                    ("学校", &school_name),
                    ("大小", &size_str),
                    ("上传者", &create_by),
                    ("文件ID", &file_id.to_string()),
                ];

                for (key, val) in info_rows {
                    ui.horizontal(|ui| {
                        ui.add_space(20.0);
                        ui.label(
                            egui::RichText::new(format!("{}:", key))
                                .font(FontId::new(12.0, egui::FontFamily::Proportional))
                                .color(colors::text_secondary()),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(*val)
                                .font(FontId::new(13.0, egui::FontFamily::Proportional))
                                .color(colors::text_primary()),
                        );
                    });
                    ui.add_space(5.0);
                }

                ui.add_space(12.0);

                // 关闭按钮 — Metro 纯色块风格
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let close_btn = egui::Button::new(
                        egui::RichText::new("  关闭  ")
                            .font(FontId::new(14.0, egui::FontFamily::Proportional))
                            .color(colors::text_on_primary()),
                    )
                    .fill(colors::primary())
                    .stroke(egui::Stroke::NONE)
                    .corner_radius(egui::CornerRadius::ZERO)
                    .min_size(egui::vec2(80.0, 32.0));
                    if ui.add(close_btn).clicked() {
                        app.show_info_dialog = false;
                    }
                });
                ui.add_space(4.0);
            });

        // 恢复窗口阴影（避免影响其他窗口）
        let mut style = (*ui.ctx().style()).clone();
        style.visuals.window_shadow = egui::Shadow::default();
        ui.ctx().set_style(style);
    }

    // ── 举报对话框 ────────────────────────────────────────
    if app.show_report_dialog {
        egui::Window::new("举报试卷")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                ui.set_min_width(360.0);
                ui.label(
                    egui::RichText::new("举报原因")
                        .font(FontId::new(14.0, egui::FontFamily::Proportional)),
                );
                ui.add_space(8.0);
                egui::ComboBox::from_id_salt("report_type")
                    .selected_text(&app.report_type)
                    .show_ui(ui, |ui| {
                        for t in &["侵权", "色情", "暴力", "广告", "其他"] {
                            ui.selectable_value(&mut app.report_type, t.to_string(), *t);
                        }
                    });
                ui.add_space(8.0);
                ui.add(
                    egui::TextEdit::multiline(&mut app.report_content)
                        .hint_text("补充说明（可选）")
                        .desired_rows(3)
                        .desired_width(340.0),
                );
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui.button("取消").clicked() {
                        app.show_report_dialog = false;
                    }
                    ui.add_space(8.0);
                    let can_submit = !app.report_type.is_empty();
                    if ui.add_enabled(can_submit, egui::Button::new("提交举报")).clicked() {
                        let api = app.api.clone();
                        let r#type = app.report_type.clone();
                        let content = app.report_content.clone();
                        tokio::spawn(async move {
                            let report = Report {
                                report_type: r#type,
                                content,
                                ..Default::default()
                            };
                            match api.create_report(&report).await {
                                Ok(_) => log::info!("举报成功"),
                                Err(e) => log::error!("举报失败: {}", e),
                            }
                        });
                        app.add_toast("举报已提交", ToastLevel::Success);
                        app.show_report_dialog = false;
                    }
                });
            });
    }
}

/// PDF 未加载时的占位（加载中动画）
fn render_pdf_placeholder(
    ui: &mut egui::Ui,
    _file_name: &str,
    _file_subject: &str,
    _school_name: &str,
    file_id: i64,
    app: &mut PezMaxApp,
) {
    if app.pdf_loading || app.pdf_viewer.is_loading() {
        egui::Frame::new()
            .fill(colors::bg_card())
            .stroke(egui::Stroke::NONE)
            .show(ui, |ui| {
                ui.set_min_size(egui::vec2(ui.available_width(), ui.available_height()));
                ui.vertical_centered(|ui| {
                    ui.add_space(160.0);
                    ui.label(egui::RichText::new("加载中...")
                        .font(FontId::new(15.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary()));
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("正在下载并解析 PDF 文件")
                        .font(FontId::new(12.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary()));
                });
            });
        ui.ctx().request_repaint();
    } else {
        // 刚选择文件，后台加载尚未开始（极短窗口）→ 直接触发加载
        // 此分支只在点击文件行后一帧内出现，立即显示加载状态
        app.trigger_load_pdf_bytes(file_id);
    }
}

/// 骨架屏
fn render_skeleton(ui: &mut egui::Ui) {
    ui.add_space(16.0);
    for _ in 0..6 {
        egui::Frame::new()
            .fill(colors::bg_card())
            .corner_radius(CornerRadius::same(0))
            .stroke(egui::Stroke::new(1.0, colors::border()))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.set_min_height(64.0);
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.add_space(12.0);
                    egui::Frame::new().fill(colors::skeleton_base()).show(ui, |ui| {
                        ui.allocate_space(Vec2::new(28.0, 28.0));
                    });
                    ui.add_space(12.0);
                    ui.vertical(|ui| {
                        ui.add_space(4.0);
                        egui::Frame::new().fill(colors::skeleton_line()).show(ui, |ui| {
                            ui.allocate_space(Vec2::new(280.0, 13.0));
                        });
                        ui.add_space(6.0);
                        egui::Frame::new().fill(colors::skeleton_line_alt()).show(ui, |ui| {
                            ui.allocate_space(Vec2::new(180.0, 11.0));
                        });
                    });
                });
                ui.add_space(12.0);
            });
        ui.add_space(4.0);
    }
}

/// 外部书签管理（含新建表单）
pub fn render_bookmarks(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    if app.selected_bookmark.is_some() {
        render_bookmark_detail(app, ui);
        return;
    }
    render_bookmark_list(app, ui);
}

/// ── 资源类型 → 图标 + 颜色 配置表 ─────────────────────────────────────────
const RESOURCE_TYPES: &[(&str, &str, (u8, u8, u8))] = &[
    ("course",        "📹",  (0xF5, 0x6C, 0x6C)),
    ("blog",          "📝",  (0x67, 0xC2, 0x3A)),
    ("paper",         "📄",  (0xE6, 0xA2, 0x3C)),
    ("tool",          "🛠️",  (0x90, 0x93, 0x99)),
    ("entertainment", "🎬",  (0xB3, 0x7F, 0xEB)),
];

fn resource_type_info(key: &str) -> (&str, Color32) {
    RESOURCE_TYPES
        .iter()
        .find(|(k, ..)| *k == key)
        .map(|(_, icon, rgb)| (*icon, Color32::from_rgb(rgb.0, rgb.1, rgb.2)))
        .unwrap_or(("🔖", colors::text_secondary()))
}

/// ── 书签列表主视图（扁平列表，无搜索框）──────────────────────────────────────
fn render_bookmark_list(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    if !app.bookmarks_data.is_loaded() && !app.bookmarks_data.is_loading() {
        app.trigger_load_bookmarks();
    }

    // 标题区
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("外部书签")
                .font(FontId::new(22.0, egui::FontFamily::Proportional))
                .color(colors::text_primary()),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let adding = app.show_bookmark_form;
            let btn_text = if adding { "− 收起" } else { "+ 添加书签" };
            let btn = egui::Button::new(
                egui::RichText::new(btn_text)
                    .font(FontId::new(13.0, egui::FontFamily::Proportional))
                    .color(if adding { colors::text_secondary() } else { colors::text_on_primary() }),
            )
            .fill(if adding { colors::bg_hover() } else { colors::primary() })
            .corner_radius(CornerRadius::same(0));
            if ui.add(btn).clicked() {
                app.show_bookmark_form = !app.show_bookmark_form;
                if app.show_bookmark_form {
                    app.bookmark_edit_target = None;
                    app.bookmark_form_name.clear();
                    app.bookmark_form_url.clear();
                    app.bookmark_form_resource_type.clear();
                    app.bookmark_form_collection.clear();
                    app.bookmark_form_subject.clear();
                    app.bookmark_form_description.clear();
                }
            }
        });
    });
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new("管理您收藏的外部书签链接")
            .font(FontId::new(13.0, egui::FontFamily::Proportional))
            .color(colors::text_secondary()),
    );
    ui.add_space(12.0);

    // 创建/编辑表单
    if app.show_bookmark_form {
        render_bookmark_form(app, ui);
        ui.add_space(12.0);
    }

    // 加载中
    if app.bookmarks_data.is_loading() {
        render_skeleton(ui);
        ui.ctx().request_repaint();
        return;
    }

    // 数据为空
    let list_clone: Vec<Bookmark> = match &app.bookmarks_data.data {
        Some(list) => list.clone(),
        None => {
            egui::Frame::new()
                .fill(colors::bg_card())
                .corner_radius(CornerRadius::same(0))
                .stroke(egui::Stroke::new(1.0, colors::border()))
                .show(ui, |ui| {
                    ui.set_min_height(120.0);
                    ui.set_min_width(ui.available_width());
                    ui.vertical_centered(|ui| {
                        ui.add_space(36.0);
                        ui.label(
                            egui::RichText::new("📄")
                                .font(FontId::new(32.0, egui::FontFamily::Proportional)),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new("暂无书签")
                                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                .color(colors::text_secondary()),
                        );
                        ui.add_space(36.0);
                    });
                });
            return;
        }
    };

    if list_clone.is_empty() {
        ui.add_space(20.0);
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("暂无书签")
                    .font(FontId::new(14.0, egui::FontFamily::Proportional))
                    .color(colors::text_secondary()),
            );
        });
        return;
    }

    // ── 扁平列表 ──────────────────────────────────────────────────────
    egui::ScrollArea::vertical()
        .id_salt("bookmark_list_scroll")
        .show(ui, |ui| {
            for bm in &list_clone {
                let row_clicked = render_bookmark_row(ui, bm, app);
                if row_clicked {
                    app.selected_bookmark = Some(bm.clone());
                    app.bookmark_detail_anim = crate::sokuou::SpringAnim::with_target(
                        0.4, 0.8, 0.0, 0.0, 1.0,
                    );
                }
                ui.add_space(4.0);
            }
        });
}

/// ── 单行书签项（纯文本入口卡片，点击进入详情）────────────────────────────────
fn render_bookmark_row(ui: &mut egui::Ui, bm: &Bookmark, _app: &mut PezMaxApp) -> bool {
    let (_icon, type_color) = resource_type_info(&bm.resource_type);

    let domain = bm
        .url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("");

    let cover_w = 56.0_f32;
    let cover_h = 36.0_f32;
    let card_h = 64.0_f32;
    let has_cover = !bm.cover_url.is_empty();
    let tex_loaded = has_cover && _app.bookmark_cover_textures.contains_key(&bm.id);

    let frame_resp = egui::Frame::new()
        .fill(colors::bg_card())
        .corner_radius(CornerRadius::same(0))
        .stroke(egui::Stroke::new(1.0, colors::border()))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.set_min_height(card_h);

            ui.horizontal(|ui| {
                ui.add_space(16.0);

                // 封面区域：申请一个高度 = card_h 的空间，在其中垂直居中 36px 封面
                let cover_area = ui.allocate_exact_size(
                    egui::vec2(cover_w, card_h),
                    egui::Sense::hover(),
                ).0;
                let cover_y = cover_area.center().y - cover_h / 2.0;
                let cover_rect = egui::Rect::from_min_size(
                    egui::pos2(cover_area.left(), cover_y),
                    egui::vec2(cover_w, cover_h),
                );

                if tex_loaded {
                    if let Some(tex_handle) = _app.bookmark_cover_textures.get(&bm.id) {
                        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                        ui.painter().image(tex_handle.id(), cover_rect, uv, egui::Color32::WHITE);
                    }
                } else {
                    ui.painter().rect_filled(
                        cover_rect,
                        CornerRadius::ZERO,
                        Color32::from_rgba_unmultiplied(type_color.r(), type_color.g(), type_color.b(), 38),
                    );
                }

                ui.add_space(12.0);

                // 标题 + 域名/专栏
                ui.vertical(|ui| {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(&bm.title)
                            .font(FontId::new(14.0, egui::FontFamily::Proportional))
                            .color(colors::text_primary()),
                    );
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(domain.to_string())
                                .font(FontId::new(11.0, egui::FontFamily::Proportional))
                                .color(colors::text_secondary()),
                        );
                        if !bm.collection.is_empty() {
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new(format!("· {}", bm.collection))
                                    .font(FontId::new(11.0, egui::FontFamily::Proportional))
                                    .color(colors::text_secondary()),
                            );
                        }
                    });
                    ui.add_space(8.0);
                });
            });
        });

    // ── 左侧类型色条（精确对齐 frame rect 全高） ──
    let full_rect = frame_resp.response.rect;
    let bar_rect = egui::Rect::from_min_size(
        egui::pos2(full_rect.left() + 1.0, full_rect.top()),
        egui::vec2(4.0, full_rect.height()),
    );
    ui.painter().rect_filled(bar_rect, egui::CornerRadius::ZERO, type_color);

    frame_resp.response.interact(egui::Sense::click()).clicked()
}

/// ── 书签创建/编辑表单 ─────────────────────────────────────────────────────
fn render_bookmark_form(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    let is_edit = app.bookmark_edit_target.is_some();
    let title = if is_edit { "编辑书签" } else { "添加新书签" };

    egui::Frame::new()
        .fill(colors::bg_card())
        .corner_radius(CornerRadius::same(0))
        .stroke(egui::Stroke::new(1.0, colors::primary_light()))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new(title)
                        .font(FontId::new(15.0, egui::FontFamily::Proportional))
                        .color(colors::text_primary()),
                );
            });
            ui.add_space(10.0);

            // Row 1: 名称 + 链接
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("名称")
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary()),
                );
                ui.add_space(8.0);
                ui.scope(|ui| {
                    crate::theme::apply_search_style(ui);
                    ui.add(
                        egui::TextEdit::singleline(&mut app.bookmark_form_name)
                            .hint_text("书签名称")
                            .desired_width(180.0)
                            .font(FontId::new(14.0, egui::FontFamily::Proportional)),
                    );
                });
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new("链接")
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary()),
                );
                ui.add_space(8.0);
                ui.scope(|ui| {
                    crate::theme::apply_search_style(ui);
                    ui.add(
                        egui::TextEdit::singleline(&mut app.bookmark_form_url)
                            .hint_text("https://...")
                            .desired_width(ui.available_width().max(200.0))
                            .font(FontId::new(14.0, egui::FontFamily::Proportional)),
                    );
                });
            });
            ui.add_space(8.0);

            // Row 2: 资源类型 + 专栏
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("类型")
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary()),
                );
                ui.add_space(8.0);
                let rt_label = RESOURCE_TYPES
                    .iter()
                    .find(|(k, ..)| *k == app.bookmark_form_resource_type)
                    .map(|(_, label, _)| *label)
                    .unwrap_or("选择类型");
                egui::ComboBox::from_id_salt("bm_form_type")
                    .selected_text(
                        egui::RichText::new(rt_label)
                            .font(FontId::new(13.0, egui::FontFamily::Proportional)),
                    )
                    .width(130.0)
                    .show_ui(ui, |ui| {
                        for (key, label, _) in RESOURCE_TYPES {
                            ui.selectable_value(
                                &mut app.bookmark_form_resource_type,
                                key.to_string(),
                                *label,
                            );
                        }
                        ui.selectable_value(
                            &mut app.bookmark_form_resource_type,
                            "other".to_string(),
                            "📌 其他",
                        );
                    });
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new("专栏")
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary()),
                );
                ui.add_space(8.0);
                ui.scope(|ui| {
                    crate::theme::apply_search_style(ui);
                    ui.add(
                        egui::TextEdit::singleline(&mut app.bookmark_form_collection)
                            .hint_text("所属专栏（可选）")
                            .desired_width(ui.available_width().max(140.0))
                            .font(FontId::new(14.0, egui::FontFamily::Proportional)),
                    );
                });
            });
            ui.add_space(8.0);

            // Row 3: 科目 + 描述
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("科目")
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary()),
                );
                ui.add_space(8.0);
                ui.scope(|ui| {
                    crate::theme::apply_search_style(ui);
                    ui.add(
                        egui::TextEdit::singleline(&mut app.bookmark_form_subject)
                            .hint_text("关联科目（可选）")
                            .desired_width(140.0)
                            .font(FontId::new(14.0, egui::FontFamily::Proportional)),
                    );
                });
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new("描述")
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary()),
                );
                ui.add_space(8.0);
                ui.scope(|ui| {
                    crate::theme::apply_search_style(ui);
                    ui.add(
                        egui::TextEdit::singleline(&mut app.bookmark_form_description)
                            .hint_text("内容摘要（可选）")
                            .desired_width(ui.available_width().max(140.0))
                            .font(FontId::new(14.0, egui::FontFamily::Proportional)),
                    );
                });
            });
            ui.add_space(12.0);

            // 操作按钮
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                let can_submit =
                    !app.bookmark_form_name.is_empty() && !app.bookmark_form_url.is_empty();
                let btn_text = if is_edit { "  保存修改  " } else { "  添加书签  " };
                let btn = egui::Button::new(
                    egui::RichText::new(btn_text)
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(if can_submit {
                            colors::text_on_primary()
                        } else {
                            colors::text_secondary()
                        }),
                )
                .fill(if can_submit {
                    colors::primary()
                } else {
                    colors::bg_hover()
                })
                .corner_radius(CornerRadius::same(0));
                if ui.add_enabled(can_submit, btn).clicked() {
                    let api = app.api.clone();
                    if let Some(ref target) = app.bookmark_edit_target {
                        // 编辑模式
                        let mut bm = target.clone();
                        bm.title = app.bookmark_form_name.clone();
                        bm.url = app.bookmark_form_url.clone();
                        bm.resource_type = app.bookmark_form_resource_type.clone();
                        bm.collection = app.bookmark_form_collection.clone();
                        bm.subject = app.bookmark_form_subject.clone();
                        bm.description = app.bookmark_form_description.clone();
                        tokio::spawn(async move {
                            let _ = api.update_bookmark(&bm).await;
                        });
                        app.add_toast("书签已更新".to_string(), ToastLevel::Success);
                    } else {
                        // 新建模式
                        let bm = Bookmark {
                            title: app.bookmark_form_name.clone(),
                            url: app.bookmark_form_url.clone(),
                            resource_type: app.bookmark_form_resource_type.clone(),
                            collection: app.bookmark_form_collection.clone(),
                            subject: app.bookmark_form_subject.clone(),
                            description: app.bookmark_form_description.clone(),
                            ..Default::default()
                        };
                        tokio::spawn(async move {
                            let _ = api.create_bookmark(&bm).await;
                        });
                        app.add_toast("书签已添加".to_string(), ToastLevel::Success);
                    }
                    // 清空表单并刷新
                    app.show_bookmark_form = false;
                    app.bookmark_edit_target = None;
                    app.bookmark_form_name.clear();
                    app.bookmark_form_url.clear();
                    app.bookmark_form_resource_type.clear();
                    app.bookmark_form_collection.clear();
                    app.bookmark_form_subject.clear();
                    app.bookmark_form_description.clear();
                    app.bookmarks_data = crate::app::AsyncData::new();
                }
                ui.add_space(8.0);
                if ui
                    .add(
                        egui::Button::new("取消")
                            .fill(egui::Color32::TRANSPARENT)
                            .corner_radius(CornerRadius::same(0)),
                    )
                    .clicked()
                {
                    app.show_bookmark_form = false;
                    app.bookmark_edit_target = None;
                }
            });
            ui.add_space(12.0);
        });
}

/// ── 书签详情页（Bento Layout + 真实封面图） ──────────────────────────────────
fn render_bookmark_detail(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    // Clone all bookmark fields upfront to avoid borrowing conflicts
    let bm_id = app.selected_bookmark.as_ref().map(|b| b.id).unwrap_or(0);
    let bm_title = app.selected_bookmark.as_ref().map(|b| b.title.clone()).unwrap_or_default();
    let bm_url = app.selected_bookmark.as_ref().map(|b| b.url.clone()).unwrap_or_default();
    let bm_description = app.selected_bookmark.as_ref().map(|b| b.description.clone()).unwrap_or_default();
    let bm_resource_type = app.selected_bookmark.as_ref().map(|b| b.resource_type.clone()).unwrap_or_default();
    let bm_collection = app.selected_bookmark.as_ref().map(|b| b.collection.clone()).unwrap_or_default();
    let bm_subject = app.selected_bookmark.as_ref().map(|b| b.subject.clone()).unwrap_or_default();
    let bm_remark = app.selected_bookmark.as_ref().map(|b| b.remark.clone()).unwrap_or_default();
    let bm_create_by = app.selected_bookmark.as_ref().map(|b| b.create_by.clone()).unwrap_or_default();
    let bm_create_time = app.selected_bookmark.as_ref().map(|b| b.create_time.clone()).unwrap_or_default();
    let bm_cover_url = app.selected_bookmark.as_ref().map(|b| b.cover_url.clone()).unwrap_or_default();

    let v = app.bookmark_detail_anim.value();
    let y_offset = map_range(v, 20.0, 0.0) as f32;
    let _alpha = map_range(v, 0.4, 1.0) as f32;
    if y_offset > 0.1 {
        ui.add_space(y_offset);
    }

    // ── 返回按钮（主题色适配） ───────────────────────────────────────────
    ui.horizontal(|ui| {
        let back_text = egui::RichText::new("← 返回书签列表")
            .font(FontId::new(13.0, egui::FontFamily::Proportional))
            .color(colors::text_secondary());
        if ui
            .add(
                egui::Button::new(back_text)
                    .fill(colors::bg_card())
                    .stroke(egui::Stroke::new(1.0, colors::border()))
                    .corner_radius(CornerRadius::same(0)),
            )
            .clicked()
        {
            app.selected_bookmark = None;
            app.bookmark_detail_anim.set_target(0.0);
            return;
        }
    });
    ui.add_space(12.0);

    let (_icon, type_color) = resource_type_info(&bm_resource_type);
    let type_label = RESOURCE_TYPES
        .iter()
        .find(|(k, ..)| *k == bm_resource_type)
        .map(|(_, label, _)| *label)
        .unwrap_or("📌 其他");

    // ── Hero Card（真实封面图） ──────────────────────────────────────────
    egui::Frame::new()
        .fill(colors::bg_card())
        .corner_radius(CornerRadius::same(0))
        .stroke(egui::Stroke::new(1.0, colors::border()))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                ui.add_space(16.0);

                // 左侧封面图（宽形，匹配 Electron hero-cover 220×140 ≈ 1.57:1）
                let cover_w = 160.0_f32;
                let cover_h = 102.0_f32;
                let cover_pos = ui.next_widget_position();
                let cover_rect =
                    egui::Rect::from_min_size(cover_pos, Vec2::new(cover_w, cover_h));

                let has_cover = !bm_cover_url.is_empty();
                let has_cover_texture = if has_cover {
                    app.bookmark_cover_textures.contains_key(&bm_id)
                } else {
                    false
                };

                if has_cover_texture {
                    // 显示真实封面纹理
                    if let Some(tex_handle) = app.bookmark_cover_textures.get(&bm_id) {
                        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                        ui.painter().image(
                            tex_handle.id(),
                            cover_rect,
                            uv,
                            egui::Color32::WHITE,
                        );
                        ui.allocate_rect(cover_rect, egui::Sense::hover());
                    }
                } else {
                    // 占位：类型色底（无 emoji 图标，无圆角）
                    ui.painter().rect_filled(
                        cover_rect,
                        CornerRadius::ZERO,
                        Color32::from_rgba_unmultiplied(type_color.r(), type_color.g(), type_color.b(), 38),
                    );
                    ui.allocate_rect(cover_rect, egui::Sense::hover());
                }

                // 触发封面加载
                if has_cover && !has_cover_texture {
                    let api = app.api.clone();
                    let id = bm_id;
                    let url = bm_cover_url.clone();
                    // 用 once 标记避免重复触发
                    if !app.bookmark_cover_requested.contains(&id) {
                        app.bookmark_cover_requested.insert(id);
                        let (tx, rx) = oneshot::channel();
                        tokio::spawn(async move {
                            let result = api.download_bytes(&url).await;
                            tx.send(result).ok();
                        });
                        app.bookmark_cover_rx = Some(rx);
                        app.bookmark_cover_pending_id = Some(id);
                    }
                }

                ui.add_space(16.0);

                // 右侧内容
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    // 类型 Badge + 时间
                    ui.horizontal(|ui| {
                        egui::Frame::new()
                            .fill(type_color)
                            .corner_radius(CornerRadius::same(0))
                            .show(ui, |ui| {
                                ui.add_space(6.0);
                                ui.label(
                                    egui::RichText::new(type_label)
                                        .font(FontId::new(11.0, egui::FontFamily::Proportional))
                                        .color(egui::Color32::WHITE),
                                );
                                ui.add_space(6.0);
                            });
                        ui.add_space(8.0);
                        let time_display = if bm_create_time.is_empty() { "-" } else { &bm_create_time };
                        ui.label(
                            egui::RichText::new(format!("·  {}", time_display))
                                .font(FontId::new(12.0, egui::FontFamily::Proportional))
                                .color(colors::text_secondary()),
                        );
                    });
                    ui.add_space(6.0);

                    // 标题
                    ui.label(
                        egui::RichText::new(&bm_title)
                            .font(FontId::new(20.0, egui::FontFamily::Proportional))
                            .color(colors::text_primary()),
                    );
                    ui.add_space(2.0);

                    // URL
                    if !bm_url.is_empty() {
                        ui.label(
                            egui::RichText::new(format!("🔗 {}", &bm_url))
                                .font(FontId::new(12.0, egui::FontFamily::Proportional))
                                .color(colors::primary()),
                        );
                    }
                    ui.add_space(8.0);

                    // ── 操作按钮行（主题色适配） ──────────────────────────
                    ui.horizontal(|ui| {
                        // 立即前往（主色按钮）
                        let goto_btn = egui::Button::new(
                            egui::RichText::new("  立即前往  ")
                                .font(FontId::new(13.0, egui::FontFamily::Proportional))
                                .color(colors::text_on_primary()),
                        )
                            .fill(colors::primary())
                            .stroke(egui::Stroke::NONE)
                            .corner_radius(CornerRadius::same(0))
                            .min_size(egui::vec2(80.0, 32.0));
                        if ui.add(goto_btn).clicked() {
                            let url = bm_url.clone();
                            let _ = webbrowser::open(&url);
                        }

                        ui.add_space(8.0);

                        // 复制链接
                        let copy_btn = egui::Button::new(
                            egui::RichText::new("📋 复制链接")
                                .font(FontId::new(13.0, egui::FontFamily::Proportional))
                                .color(colors::text_primary()),
                        )
                            .fill(colors::bg_hover())
                            .stroke(egui::Stroke::new(1.0, colors::border()))
                            .corner_radius(CornerRadius::same(0))
                            .min_size(egui::vec2(80.0, 32.0));
                        if ui.add(copy_btn).clicked() {
                            ui.ctx().copy_text(bm_url.clone());
                            app.add_toast("链接已复制".to_string(), ToastLevel::Success);
                        }

                        ui.add_space(8.0);

                        // 收藏（真实实现）
                        let is_fav = app.favorite_bookmark_ids.contains(&bm_id);
                        let fav_btn = egui::Button::new(
                            egui::RichText::new(if is_fav { "⭐ 已收藏" } else { "☆ 收藏" })
                                .font(FontId::new(13.0, egui::FontFamily::Proportional))
                                .color(if is_fav { colors::primary() } else { colors::text_primary() }),
                        )
                            .fill(colors::bg_hover())
                            .stroke(egui::Stroke::new(1.0, colors::border()))
                            .corner_radius(CornerRadius::same(0))
                            .min_size(egui::vec2(80.0, 32.0));
                        if ui.add(fav_btn).clicked() {
                            let target_id = bm_id;
                            if is_fav {
                                app.favorite_bookmark_ids.remove(&target_id);
                                app.bookmark_fav_data.push((target_id, false));
                                app.add_toast("已取消收藏".to_string(), ToastLevel::Success);
                            } else {
                                app.favorite_bookmark_ids.insert(target_id);
                                app.bookmark_fav_data.push((target_id, true));
                                app.add_toast("已收藏".to_string(), ToastLevel::Success);
                            }
                        }

                        ui.add_space(8.0);

                        // 举报
                        let report_btn = egui::Button::new(
                            egui::RichText::new("🚩 举报")
                                .font(FontId::new(13.0, egui::FontFamily::Proportional))
                                .color(colors::text_primary()),
                        )
                            .fill(colors::bg_hover())
                            .stroke(egui::Stroke::new(1.0, colors::border()))
                            .corner_radius(CornerRadius::same(0))
                            .min_size(egui::vec2(80.0, 32.0));
                        if ui.add(report_btn).clicked() {
                            app.show_report_dialog = true;
                            app.report_type = "侵权".to_string();
                            app.report_content.clear();
                        }
                    });
                });
                ui.add_space(16.0);
            });
            ui.add_space(16.0);
        });

    ui.add_space(12.0);

    // ── 举报对话框 ────────────────────────────────────────────────────
    if app.show_report_dialog {
        egui::Window::new("举报书签")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                ui.set_min_width(360.0);
                ui.label(
                    egui::RichText::new("举报原因")
                        .font(FontId::new(14.0, egui::FontFamily::Proportional))
                        .color(colors::text_primary()),
                );
                ui.add_space(8.0);
                egui::ComboBox::from_id_salt("bm_detail_report_type")
                    .selected_text(&app.report_type)
                    .show_ui(ui, |ui| {
                        for t in &["侵权", "色情", "暴力", "广告", "其他"] {
                            ui.selectable_value(&mut app.report_type, t.to_string(), *t);
                        }
                    });
                ui.add_space(8.0);
                ui.add(
                    egui::TextEdit::multiline(&mut app.report_content)
                        .hint_text("补充说明（可选）")
                        .desired_rows(3)
                        .desired_width(340.0),
                );
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui.button("取消").clicked() {
                        app.show_report_dialog = false;
                    }
                    ui.add_space(8.0);
                    let can_submit = !app.report_type.is_empty();
                    if ui
                        .add_enabled(can_submit, egui::Button::new("提交举报"))
                        .clicked()
                    {
                        let api = app.api.clone();
                        let r#type = app.report_type.clone();
                        let content = app.report_content.clone();
                        tokio::spawn(async move {
                            let report = Report {
                                report_type: r#type,
                                content,
                                ..Default::default()
                            };
                            match api.create_report(&report).await {
                                Ok(_) => log::info!("举报成功"),
                                Err(e) => log::error!("举报失败: {}", e),
                            }
                        });
                        app.add_toast("举报已提交".to_string(), ToastLevel::Success);
                        app.show_report_dialog = false;
                    }
                });
            });
    }

    // ── Bento Grid: 2列 ────────────────────────────────────────────────
    ui.add_space(8.0);

    // Row 1: 内容摘要（全宽）
    if !bm_description.is_empty() {
        bento_card_full(ui, "📋 内容摘要", &bm_description);
        ui.add_space(8.0);
    }

    // Row 2: 所属专栏 | 关联科目
    ui.horizontal(|ui| {
        let col_text = if bm_collection.is_empty() {
            "未分类".to_string()
        } else {
            bm_collection.clone()
        };
        bento_card_half(ui, "📂 所属专栏", &col_text);

        ui.add_space(8.0);

        let sub_text = if bm_subject.is_empty() {
            "未分类".to_string()
        } else {
            bm_subject.clone()
        };
        bento_card_half(ui, "📘 关联科目", &sub_text);
    });
    ui.add_space(8.0);

    // Row 3: 上传者（全宽）
    if !bm_create_by.is_empty() {
        let time_disp = if bm_create_time.is_empty() { "-" } else { &bm_create_time };
        let info = format!("@{} · {}", bm_create_by, time_disp);
        bento_card_full(ui, "👤 上传者", &info);
        ui.add_space(8.0);
    }

    // Row 4: 备注（仅当有 remark）
    if !bm_remark.is_empty() {
        bento_card_full(ui, "💬 备注", &bm_remark);
        ui.add_space(8.0);
    }
}

/// Bento 全宽卡片
fn bento_card_full(ui: &mut egui::Ui, title: &str, content: &str) {
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
                    egui::RichText::new(title)
                        .font(FontId::new(12.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary()),
                );
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new(content)
                        .font(FontId::new(14.0, egui::FontFamily::Proportional))
                        .color(colors::text_primary()),
                );
            });
            ui.add_space(10.0);
        });
}

/// Bento 半宽卡片
fn bento_card_half(ui: &mut egui::Ui, title: &str, content: &str) {
    let half_w = (ui.available_width() - 8.0) / 2.0;
    egui::Frame::new()
        .fill(colors::bg_card())
        .corner_radius(CornerRadius::same(0))
        .stroke(egui::Stroke::new(1.0, colors::border()))
        .show(ui, |ui| {
            ui.set_min_width(half_w);
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new(title)
                        .font(FontId::new(12.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary()),
                );
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new(content)
                        .font(FontId::new(14.0, egui::FontFamily::Proportional))
                        .color(colors::text_primary()),
                );
            });
            ui.add_space(10.0);
        });
}

/// 我的收藏列表
pub fn render_favorites(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    if !app.favorites_data.is_loaded() && !app.favorites_data.is_loading() {
        app.trigger_load_favorites();
    }

    ui.add_space(16.0);
    ui.label(
        egui::RichText::new("我的收藏")
            .font(FontId::new(22.0, egui::FontFamily::Proportional))
            .color(colors::text_primary()),
    );
    ui.add_space(16.0);

    egui::ScrollArea::vertical()
        .id_salt("favorites_scroll")
        .show(ui, |ui| {
            if let Some(ref list) = app.favorites_data.data {
                for fav in list {
                    egui::Frame::new()
                        .fill(colors::bg_card())
                        .corner_radius(CornerRadius::same(0))
                        .stroke(egui::Stroke::new(1.0, colors::border()))
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.add_space(12.0);
                                ui.label(egui::RichText::new("⭐").font(FontId::new(18.0, egui::FontFamily::Proportional)));
                                ui.add_space(10.0);
                                ui.vertical(|ui| {
                                    ui.add_space(8.0);
                                    ui.label(
                                        egui::RichText::new(&fav.file_name)
                                            .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                            .color(colors::text_primary()),
                                    );
                                    ui.label(
                                        egui::RichText::new(format!("{} · 收藏于 {}", fav.file_subject, fav.create_time))
                                            .font(FontId::new(12.0, egui::FontFamily::Proportional))
                                            .color(colors::text_secondary()),
                                    );
                                    ui.add_space(8.0);
                                });
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.add_space(12.0);
                                    if ui.small_button("取消收藏").clicked() {
                                        let api = app.api.clone();
                                        let uid = app.current_user.as_ref().map(|u| u.user_id).unwrap_or(0);
                                        let fid = fav.file_id;
                                        app.favorite_file_ids.remove(&fid);
                                        if let Some(ref mut stats) = app.user_stats {
                                            stats.favorite_count = (stats.favorite_count - 1).max(0);
                                        }
                                        tokio::spawn(async move { let _ = api.remove_favorite(uid, fid).await; });
                                    }
                                    ui.add_space(4.0);
                                    if ui.small_button("📥 下载").clicked() {
                                        let api = app.api.clone();
                                        let fid = fav.file_id;
                                        tokio::spawn(async move { let _ = api.download_paper(fid).await; });
                                    }
                                });
                            });
                        });
                    ui.add_space(6.0);
                }
            } else {
                ui.label(
                    egui::RichText::new(if app.favorites_data.is_loading() { "加载中..." } else { "暂无收藏" })
                        .font(FontId::new(14.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary()),
                );
            }
        });
}

// ── 内部组件 ─────────────────────────────────────────────────────────────────

/// 全宽纵向文件行，返回是否点击（打开预览）
fn file_row(ui: &mut egui::Ui, file: &PaperFile, app: &PezMaxApp) -> bool {
    let size_str = if file.file_size > 0 {
        format!("{:.1} MB", file.file_size as f64 / 1048576.0)
    } else {
        "-".to_string()
    };

    let resp = egui::Frame::new()
        .fill(colors::bg_card())
        .corner_radius(CornerRadius::same(0))
        .stroke(egui::Stroke::new(1.0, colors::border()))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.add_space(14.0);
                ui.label(
                    egui::RichText::new("📄")
                        .font(FontId::new(26.0, egui::FontFamily::Proportional)),
                );
                ui.add_space(12.0);
                ui.vertical(|ui| {
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(&file.file_name)
                            .font(FontId::new(14.0, egui::FontFamily::Proportional))
                            .color(colors::text_primary()),
                    );
                    // 副标题：学科 · 学校 · 大小
                    let parts: Vec<&str> = [
                        file.file_subject.as_str(),
                        file.school_name.as_str(),
                        size_str.as_str(),
                    ]
                    .iter()
                    .copied()
                    .filter(|s| !s.is_empty())
                    .collect();
                    ui.label(
                        egui::RichText::new(parts.join(" · "))
                            .font(FontId::new(12.0, egui::FontFamily::Proportional))
                            .color(colors::text_secondary()),
                    );
                    ui.add_space(10.0);
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(14.0);
                    if ui.small_button("⭐ 收藏").clicked() {
                        let api = app.api.clone();
                        let fid = file.file_id;
                        let uid = app.current_user.as_ref().map(|u| u.user_id).unwrap_or(0);
                        tokio::spawn(async move { let _ = api.add_favorite(uid, fid).await; });
                    }
                    ui.add_space(6.0);
                    if ui.small_button("📥 下载").clicked() {
                        let api = app.api.clone();
                        let fid = file.file_id;
                        tokio::spawn(async move { let _ = api.download_paper(fid).await; });
                    }
                });
            });
        })
        .response
        .interact(egui::Sense::click());

    resp.clicked()
}
