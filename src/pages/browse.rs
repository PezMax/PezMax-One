// 浏览功能区
// 三个子标签：资源管理 / 外部书签 / 我的收藏

use crate::app::PezMaxApp;
use crate::sokuou::map_range;
use crate::theme::colors;
use egui::{Color32, CornerRadius, FontId, Vec2};

static SUBJECTS: &[(&str, Option<&str>)] = &[
    ("全部", None),
    ("数学", Some("数学")),
    ("语文", Some("语文")),
    ("英语", Some("英语")),
    ("物理", Some("物理")),
    ("化学", Some("化学")),
    ("生物", Some("生物")),
    ("历史", Some("历史")),
    ("地理", Some("地理")),
];

static YEARS: &[(&str, Option<i32>)] = &[
    ("全部", None),
    ("2024", Some(2024)),
    ("2023", Some(2023)),
    ("2022", Some(2022)),
];

static SCHOOLS: &[(&str, Option<&str>)] = &[
    ("全部", None),
    ("清华附中", Some("清华附中")),
    ("北师大附中", Some("北师大附中")),
    ("人大附中", Some("人大附中")),
    ("全国卷", Some("全国卷")),
];

// 6-tuple: (文件名, 学科, 年份, 大小, 上传者, 学校)
static MOCK_FILES: &[(&str, &str, &str, &str, &str, &str)] = &[
    ("2024高考数学真题", "数学", "2024", "2.3MB", "张三", "全国卷"),
    ("2024高考语文真题", "语文", "2024", "1.8MB", "李四", "全国卷"),
    ("2024高考英语真题", "英语", "2024", "1.5MB", "王五", "全国卷"),
    ("2023高考物理真题", "物理", "2023", "2.1MB", "赵六", "全国卷"),
    ("2023高考化学真题", "化学", "2023", "1.9MB", "钱七", "清华附中"),
    ("2024模拟试卷·数学", "数学", "2024", "3.2MB", "孙八", "清华附中"),
    ("2022数学期末联考", "数学", "2022", "1.7MB", "周九", "北师大附中"),
    ("2023英语听力专项", "英语", "2023", "0.8MB", "吴十", "人大附中"),
];

/// 资源管理：筛选器 + 文件网格  ↔  文件预览面板
pub fn render_resource_manager(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    // 预览面板优先显示
    if app.browse_selected_idx.is_some() {
        render_file_preview(app, ui);
        return;
    }

    // 加载骨架屏
    if app.is_loading {
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

    let mut new_sub: Option<Option<String>> = None;
    ui.horizontal_wrapped(|ui| {
        for &(label, sub_val) in SUBJECTS {
            let sub_opt: Option<String> = sub_val.map(String::from);
            let active = active_sub == sub_opt;
            if filter_chip(ui, label, active) {
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

    let mut new_school: Option<Option<String>> = None;
    ui.horizontal_wrapped(|ui| {
        for &(label, sch_val) in SCHOOLS {
            let sch_opt: Option<String> = sch_val.map(String::from);
            let active = active_school == sch_opt;
            if filter_chip(ui, label, active) {
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

    // ── 过滤并建立 (mock_idx, data) 列表 ─────────────────────
    let cur_sub = app.filters.subject.clone();
    let cur_year = app.filters.year;
    let cur_school = app.filters.school.clone();

    let filtered: Vec<(usize, &(&str, &str, &str, &str, &str, &str))> = MOCK_FILES
        .iter()
        .enumerate()
        .filter(|(_, (name, subj, yr, _, _, sch))| {
            let sub_ok = cur_sub.as_deref().map_or(true, |s| s == *subj);
            let yr_ok = cur_year.map_or(true, |y| y.to_string().as_str() == *yr);
            let sch_ok = cur_school.as_deref().map_or(true, |s| s == *sch);
            let q_ok = search_q.is_empty()
                || name.to_lowercase().contains(&search_q)
                || subj.to_lowercase().contains(&search_q);
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
    let mut select_idx: Option<usize> = None;

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
                    for (mock_idx, &(name, subject, year, size, _uploader, school)) in &filtered {
                        if file_card(ui, name, subject, year, size, school) {
                            select_idx = Some(*mock_idx);
                        }
                        ui.add_space(10.0);
                    }
                });
            }
        });

    if let Some(idx) = select_idx {
        app.browse_selected_idx = Some(idx);
        app.preview_anim = crate::sokuou::SpringAnim::with_target(0.4, 0.8, 0.0, 0.0, 1.0);
    }
}

/// 文件预览面板（主从视图）
fn render_file_preview(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    let Some(idx) = app.browse_selected_idx else {
        return;
    };
    let (name, subject, year, size, uploader, school) = MOCK_FILES[idx];

    let v = app.preview_anim.value();
    let y_offset = map_range(v, 16.0, 0.0) as f32;
    if y_offset > 0.1 {
        ui.add_space(y_offset);
    }

    // ── 顶部操作栏 ────────────────────────────────────────────
    ui.add_space(12.0);
    let mut go_back = false;
    let mut prev_idx: Option<usize> = None;
    let mut next_idx: Option<usize> = None;

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

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(16.0);
            let total = MOCK_FILES.len();

            let next_btn = egui::Button::new(
                egui::RichText::new("→")
                    .font(FontId::new(16.0, egui::FontFamily::Proportional)),
            )
            .fill(colors::BG_HOVER)
            .corner_radius(CornerRadius::same(0));
            if ui.add_enabled(idx + 1 < total, next_btn).clicked() {
                next_idx = Some(idx + 1);
            }

            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(format!("{} / {}", idx + 1, total))
                    .font(FontId::new(13.0, egui::FontFamily::Proportional))
                    .color(colors::TEXT_SECONDARY),
            );
            ui.add_space(8.0);

            let prev_btn = egui::Button::new(
                egui::RichText::new("←")
                    .font(FontId::new(16.0, egui::FontFamily::Proportional)),
            )
            .fill(colors::BG_HOVER)
            .corner_radius(CornerRadius::same(0));
            if ui.add_enabled(idx > 0, prev_btn).clicked() {
                prev_idx = Some(idx - 1);
            }
        });
    });

    if go_back {
        app.browse_selected_idx = None;
        app.preview_anim.set_target(0.0);
        return;
    }
    if let Some(new_idx) = next_idx.or(prev_idx) {
        app.browse_selected_idx = Some(new_idx);
        app.preview_anim = crate::sokuou::SpringAnim::with_target(0.4, 0.8, 0.0, 0.0, 1.0);
        return;
    }

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(16.0);

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
                            ("文件名", name),
                            ("学科", subject),
                            ("学校", school),
                            ("年份", year),
                            ("大小", size),
                            ("上传者", uploader),
                        ];

                        for (key, val) in &meta {
                            ui.horizontal(|ui| {
                                ui.add_space(16.0);
                                ui.label(
                                    egui::RichText::new(*key)
                                        .font(FontId::new(
                                            12.0,
                                            egui::FontFamily::Proportional,
                                        ))
                                        .color(colors::TEXT_SECONDARY),
                                );
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new(*val)
                                        .font(FontId::new(
                                            13.0,
                                            egui::FontFamily::Proportional,
                                        ))
                                        .color(colors::TEXT_PRIMARY),
                                );
                            });
                            ui.add_space(6.0);
                        }

                        ui.add_space(16.0);
                        ui.separator();
                        ui.add_space(12.0);

                        ui.vertical_centered(|ui| {
                            // 下载按钮
                            let dl_btn = egui::Button::new(
                                egui::RichText::new("  📥 下载文件  ")
                                    .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                    .color(colors::TEXT_ON_PRIMARY),
                            )
                            .fill(colors::PRIMARY)
                            .min_size(Vec2::new(200.0, 36.0))
                            .corner_radius(CornerRadius::same(0));
                            if ui.add(dl_btn).clicked() {
                                // Phase: 接入 rfd + reqwest 下载
                            }
                            ui.add_space(8.0);

                            // 收藏按钮
                            let fav_btn = egui::Button::new(
                                egui::RichText::new("  ⭐ 收藏  ")
                                    .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                    .color(colors::ACCENT_ORANGE),
                            )
                            .fill(colors::BG_HOVER)
                            .min_size(Vec2::new(200.0, 36.0))
                            .corner_radius(CornerRadius::same(0));
                            if ui.add(fav_btn).clicked() {}
                            ui.add_space(8.0);

                            // 举报按钮
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
                                    .font(FontId::new(
                                        15.0,
                                        egui::FontFamily::Proportional,
                                    ))
                                    .color(colors::TEXT_SECONDARY),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new("下载后可用系统阅读器打开")
                                    .font(FontId::new(
                                        12.0,
                                        egui::FontFamily::Proportional,
                                    ))
                                    .color(colors::TEXT_SECONDARY),
                            );
                        });
                    });
            });
        });
}

/// 骨架屏：is_loading 时替代文件网格
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
                        // 图标占位
                        egui::Frame::new()
                            .fill(Color32::from_gray(220))
                            .show(ui, |ui| {
                                ui.allocate_space(Vec2::new(28.0, 28.0));
                            });
                        ui.add_space(10.0);
                        ui.vertical(|ui| {
                            // 标题占位
                            egui::Frame::new()
                                .fill(Color32::from_gray(215))
                                .show(ui, |ui| {
                                    ui.allocate_space(Vec2::new(120.0, 13.0));
                                });
                            ui.add_space(6.0);
                            // 副标题占位
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
                    app.bookmark_form_name.clear();
                    app.bookmark_form_url.clear();
                }
            });
            ui.add_space(16.0);
        });

    ui.add_space(16.0);

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
                ui.label(
                    egui::RichText::new("暂无书签")
                        .font(FontId::new(14.0, egui::FontFamily::Proportional))
                        .color(colors::TEXT_SECONDARY),
                );
                ui.add_space(36.0);
            });
        });
}

/// 我的收藏列表
pub fn render_favorites(_app: &mut PezMaxApp, ui: &mut egui::Ui) {
    ui.add_space(16.0);
    ui.label(
        egui::RichText::new("我的收藏")
            .font(FontId::new(22.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_PRIMARY),
    );
    ui.add_space(16.0);

    let favorites = [
        ("2024高考数学真题", "数学", "2024-06-15"),
        ("2024高考物理真题", "物理", "2024-06-10"),
    ];

    egui::ScrollArea::vertical()
        .id_salt("favorites_scroll")
        .show(ui, |ui| {
            for (name, subject, date) in &favorites {
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
                                    egui::RichText::new(*name)
                                        .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                        .color(colors::TEXT_PRIMARY),
                                );
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{} · 收藏于 {}",
                                        subject, date
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
                                    if ui.small_button("取消收藏").clicked() {}
                                    ui.add_space(4.0);
                                    if ui.small_button("📥 下载").clicked() {}
                                },
                            );
                        });
                    });
                ui.add_space(6.0);
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
fn file_card(ui: &mut egui::Ui, name: &str, subject: &str, year: &str, size: &str, school: &str) -> bool {
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
                        egui::RichText::new(name)
                            .font(FontId::new(13.0, egui::FontFamily::Proportional))
                            .color(colors::TEXT_PRIMARY),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(format!("{} · {}", subject, year))
                            .font(FontId::new(12.0, egui::FontFamily::Proportional))
                            .color(colors::TEXT_SECONDARY),
                    );
                    ui.label(
                        egui::RichText::new(format!("{} · {}", school, size))
                            .font(FontId::new(11.0, egui::FontFamily::Proportional))
                            .color(colors::TEXT_SECONDARY),
                    );
                });
            });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                if ui.small_button("📥 下载").clicked() {}
                ui.add_space(4.0);
                if ui.small_button("⭐ 收藏").clicked() {}
            });
            ui.add_space(8.0);
        })
        .response
        .interact(egui::Sense::click());

    resp.clicked()
}
