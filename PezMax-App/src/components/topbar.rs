// 顶部栏：页面标题、搜索框、用户头像

use crate::app::{Page, PezMaxApp};
use crate::theme::colors;
use egui::{Color32, FontId, Rounding};

pub fn render(app: &mut PezMaxApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("topbar")
        .min_height(56.0)
        .frame(egui::Frame::none().fill(colors::BG_CARD))
        .show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                // 侧边栏折叠按钮
                let toggle_btn = egui::Button::new(
                    egui::RichText::new(if app.sidebar_open { "☰" } else { "☰" })
                        .font(FontId::new(20.0, egui::FontFamily::Proportional))
                        .color(colors::TEXT_PRIMARY),
                )
                .fill(colors::BG_CARD)
                .min_size(egui::Vec2::new(40.0, 40.0))
                .rounding(Rounding::same(6.0));
                if ui.add(toggle_btn).clicked() {
                    app.sidebar_open = !app.sidebar_open;
                }

                ui.add_space(12.0);

                // 返回按钮
                if !app.page_history.is_empty() {
                    let back_btn = egui::Button::new(
                        egui::RichText::new("←")
                            .font(FontId::new(18.0, egui::FontFamily::Proportional))
                            .color(colors::TEXT_PRIMARY),
                    )
                    .fill(colors::BG_CARD)
                    .min_size(egui::Vec2::new(40.0, 40.0))
                    .rounding(Rounding::same(6.0));
                    if ui.add(back_btn).clicked() {
                        app.go_back();
                    }
                    ui.add_space(12.0);
                }

                // 页面标题
                ui.label(
                    egui::RichText::new(app.page_title())
                        .font(FontId::new(20.0, egui::FontFamily::Proportional))
                        .color(colors::TEXT_PRIMARY),
                );

                // 弹性空间
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // 用户头像
                    if let Some(ref user) = app.current_user {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(&user.nick_name)
                                    .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                    .color(colors::TEXT_SECONDARY),
                            );
                            ui.add_space(4.0);
                            // 头像占位
                            egui::Frame::none()
                                .fill(colors::PRIMARY)
                                .rounding(Rounding::same(16.0))
                                .show(ui, |ui| {
                                    ui.allocate_space(egui::Vec2::new(32.0, 32.0));
                                    ui.allocate_ui_at_rect(ui.max_rect(), |ui| {
                                        ui.vertical_centered(|ui| {
                                            ui.label(
                                                egui::RichText::new(
                                                    user.nick_name.chars().next().unwrap_or('?').to_string(),
                                                )
                                                .color(colors::TEXT_ON_PRIMARY)
                                                .font(FontId::new(14.0, egui::FontFamily::Proportional)),
                                            );
                                        });
                                    });
                                });
                        });
                    }

                    ui.add_space(16.0);

                    // 搜索框（仅在试卷浏览页显示）
                    if app.current_page == Page::FileExplorer {
                        let search_response = ui.add(
                            egui::TextEdit::singleline(&mut app.search_query)
                                .hint_text("🔍 搜索试卷、学科...")
                                .desired_width(240.0)
                                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                .rounding(Rounding::same(20.0)),
                        );
                        if search_response.lost_focus() && !app.search_query.is_empty() {
                            app.navigate(Page::FileExplorer);
                        }
                    }
                });
            });
            ui.add_space(8.0);
        });
}