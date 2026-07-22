// 仪表盘 — 最终版
// 欢迎语 → 色块（无 emoji）→ 双栏（快速入口 + 最近更新）

use crate::app::{PezMaxApp, Section, Subsection};
use crate::theme::colors;
use egui::{CornerRadius, FontId, Vec2, Color32, Stroke, pos2, UiBuilder};

const GAP: f32 = 12.0;

pub fn render(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical()
        .id_salt("home_scroll")
        .show(ui, |ui| {
            ui.add_space(8.0);
            render_welcome(app, ui);
            ui.add_space(16.0);
            render_metric_blocks(app, ui);
            ui.add_space(20.0);
            render_two_column(app, ui);
            ui.add_space(24.0);
        });
}

// ── 欢迎语 ────────────────────────────────────────────────────────────────────

fn render_welcome(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    // 提取数据，避免在闭包中借用 app
    let nickname = app.current_user.as_ref().map(|u| u.nick_name.clone()).unwrap_or("用户".to_string());

    let hour = {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        ((secs % 86400) / 3600 + 8) % 24
    };
    let greeting = if hour < 6 { "深夜好" } else if hour < 12 { "早上好" } else if hour < 18 { "下午好" } else { "晚上好" };

    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new(format!("{}，{}", greeting, nickname))
                    .font(FontId::new(28.0, egui::FontFamily::Proportional))
                    .color(colors::text_primary()),
            );
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new("欢迎使用 PezMax 试卷资源管理系统")
                    .font(FontId::new(14.0, egui::FontFamily::Proportional))
                    .color(colors::text_secondary()),
            );
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(16.0);
            let btn = egui::Button::new(
                egui::RichText::new("👤 个人中心")
                    .font(FontId::new(14.0, egui::FontFamily::Proportional))
                    .color(colors::primary()),
            )
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::new(1.0, colors::primary()))
            .corner_radius(CornerRadius::ZERO)
            .min_size(Vec2::new(100.0, 34.0));
            if ui.add(btn).clicked() {
                app.navigate_to(Section::Profile, Subsection::PersonalCenter);
            }
        });
    });
}

// ── 统计色块 ──────────────────────────────────────────────────────────────────
// 纯色色块，白字，无 emoji

fn render_metric_blocks(app: &PezMaxApp, ui: &mut egui::Ui) {
    let (fav, dl, ul) = app.user_stats.as_ref().map_or((0, 0, 0), |s| {
        (s.favorite_count, s.download_count, s.upload_count)
    });

    let metrics = [
        (format!("{}", dl), "下载量", colors::primary()),
        (format!("{}", fav), "收藏数", colors::accent_orange()),
        (format!("{}", ul), "上传数", colors::accent_green()),
    ];

    let block_w = 160.0;
    let block_h = 80.0;

    ui.horizontal(|ui| {
        for (i, (value, label, color)) in metrics.iter().enumerate() {
            if i > 0 {
                ui.add_space(GAP);
            }
            let (rect, _) = ui.allocate_exact_size(Vec2::new(block_w, block_h), egui::Sense::hover());
            ui.painter().rect_filled(rect, CornerRadius::ZERO, *color);

            let mut child_ui = ui.new_child(
                UiBuilder::new()
                    .max_rect(rect)
                    .layout(egui::Layout::top_down(egui::Align::Center)),
            );
            child_ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new(value)
                        .font(FontId::new(28.0, egui::FontFamily::Proportional))
                        .color(colors::text_on_primary())
                        .strong(),
                );
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new(*label)
                        .font(FontId::new(12.0, egui::FontFamily::Proportional))
                        .color(colors::text_on_primary()),
                );
            });
        }
    });
}

// ── 双栏 ──────────────────────────────────────────────────────────────────────

fn render_two_column(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    let avail_w = ui.available_width();
    let left_w = (avail_w - GAP) * 0.45;
    let right_w = (avail_w - GAP) * 0.55;

    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_min_width(left_w);
            ui.set_max_width(left_w);
            section_title(ui, "快速入口");
            ui.add_space(6.0);
            render_quick_actions(app, ui, left_w);
        });

        ui.add_space(GAP);

        ui.vertical(|ui| {
            ui.set_min_width(right_w);
            ui.set_max_width(right_w);
            section_title(ui, "最近更新");
            ui.add_space(6.0);
            render_recent_files(app, ui);
        });
    });
}

fn section_title(ui: &mut egui::Ui, text: &str) {
    ui.horizontal(|ui| {
        egui::Frame::new()
            .fill(colors::primary())
            .corner_radius(CornerRadius::ZERO)
            .show(ui, |ui| {
                ui.set_min_size(Vec2::new(3.0, 16.0));
                ui.set_max_size(Vec2::new(3.0, 16.0));
            });
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(text)
                .font(FontId::new(16.0, egui::FontFamily::Proportional))
                .color(colors::text_primary())
                .strong(),
        );
    });
}

// ── 快速入口（2×2 等宽网格）─────────────────────────────────────────────────

fn render_quick_actions(app: &mut PezMaxApp, ui: &mut egui::Ui, parent_w: f32) {
    let mut nav_to: Option<(Section, Subsection)> = None;

    let actions = [
        ("📁", "浏览试卷", "查看全部资源", Section::Browse, Subsection::ResourceManager, colors::primary()),
        ("📤", "贡献文件", "上传分享试卷", Section::Community, Subsection::ContributeFile, colors::accent_green()),
        ("🔖", "我的收藏", "快速访问收藏", Section::Browse, Subsection::MyFavorites, colors::accent_orange()),
        ("📥", "下载记录", "历史下载回溯", Section::Profile, Subsection::DownloadHistory, colors::accent_teal()),
    ];

    let card_w = (parent_w - GAP) / 2.0;
    let card_h = 88.0;

    for row in 0..2 {
        ui.horizontal(|ui| {
            for col in 0..2 {
                let idx = row * 2 + col;
                if idx >= actions.len() { break; }
                let (icon, title, desc, section, sub, color) = actions[idx];

                let (rect, resp) = ui.allocate_exact_size(Vec2::new(card_w, card_h), egui::Sense::click());
                ui.painter().rect_filled(rect, CornerRadius::ZERO, colors::bg_card());

                ui.painter().rect_filled(
                    egui::Rect::from_min_max(pos2(rect.left(), rect.top()), pos2(rect.left() + 3.0, rect.bottom())),
                    CornerRadius::ZERO,
                    color,
                );

                let mut child_ui = ui.new_child(
                    UiBuilder::new()
                        .max_rect(rect)
                        .layout(egui::Layout::top_down(egui::Align::Center)),
                );
                child_ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new(icon)
                            .font(FontId::new(24.0, egui::FontFamily::Proportional)),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new(title)
                            .font(FontId::new(14.0, egui::FontFamily::Proportional))
                            .color(colors::text_primary())
                            .strong(),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new(desc)
                            .font(FontId::new(11.0, egui::FontFamily::Proportional))
                            .color(colors::text_secondary()),
                    );
                });

                if resp.hovered() {
                    let overlay = Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 18);
                    ui.painter().rect_filled(rect, CornerRadius::ZERO, overlay);
                    ui.painter().rect_filled(
                        egui::Rect::from_min_max(pos2(rect.left(), rect.top()), pos2(rect.left() + 4.0, rect.bottom())),
                        CornerRadius::ZERO,
                        color,
                    );
                }

                if resp.clicked() {
                    nav_to = Some((section, sub));
                }

                let _ = resp.on_hover_cursor(egui::CursorIcon::PointingHand);

                ui.add_space(GAP);
            }
        });
        ui.add_space(GAP);
    }

    if let Some((s, sub)) = nav_to {
        app.navigate_to(s, sub);
    }
}

// ── 最近文件 ──────────────────────────────────────────────────────────────────

fn render_recent_files(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    let files = app.recent_files.data.clone();
    let is_loading = app.recent_files.is_loading();

    if let Some(files) = files {
        if files.is_empty() {
            centered_text(ui, "暂无最近更新");
            return;
        }

        for file in files.iter().take(6) {
            let avail_w = ui.available_width();
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(avail_w, 40.0), egui::Sense::click());
            ui.painter().rect_filled(rect, CornerRadius::ZERO, colors::bg_card());

            ui.painter().text(
                pos2(rect.left() + 14.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                "📄",
                FontId::new(18.0, egui::FontFamily::Proportional),
                colors::text_primary(),
            );

            ui.painter().text(
                pos2(rect.left() + 14.0 + 28.0, rect.center().y - 6.0),
                egui::Align2::LEFT_CENTER,
                &file.file_name,
                FontId::new(14.0, egui::FontFamily::Proportional),
                colors::text_primary(),
            );

            let meta = format!("{} · {} · {}", file.file_subject, file.file_year, file.create_by);
            ui.painter().text(
                pos2(rect.left() + 14.0 + 28.0, rect.center().y + 8.0),
                egui::Align2::LEFT_CENTER,
                &meta,
                FontId::new(11.0, egui::FontFamily::Proportional),
                colors::text_secondary(),
            );

            ui.painter().line_segment(
                [pos2(rect.left() + 14.0, rect.bottom()), pos2(rect.right() - 14.0, rect.bottom())],
                Stroke::new(1.0, colors::border()),
            );

            if resp.hovered() {
                let c = colors::primary();
                let overlay = Color32::from_rgba_premultiplied(c.r(), c.g(), c.b(), 12);
                ui.painter().rect_filled(rect, CornerRadius::ZERO, overlay);
            }

            let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);

            if resp.clicked() {
                app.navigate_to(Section::Browse, Subsection::ResourceManager);
            }
        }
    } else if is_loading {
        centered_text(ui, "⏳ 加载中...");
    } else {
        let btn = egui::Button::new(
            egui::RichText::new("加载最近更新")
                .font(FontId::new(13.0, egui::FontFamily::Proportional))
                .color(colors::primary()),
        )
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::new(1.0, colors::primary()))
        .corner_radius(CornerRadius::ZERO)
        .min_size(Vec2::new(120.0, 32.0));
        if ui.add(btn).clicked() {
            app.trigger_load_recent_files();
        }
    }
}

// ── 工具 ──────────────────────────────────────────────────────────────────────

fn centered_text(ui: &mut egui::Ui, text: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(32.0);
        ui.label(
            egui::RichText::new(text)
                .font(FontId::new(13.0, egui::FontFamily::Proportional))
                .color(colors::text_secondary()),
        );
        ui.add_space(32.0);
    });
}