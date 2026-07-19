// 首页/仪表盘
// Metro Design tile 布局：统计卡片、快捷入口、最近文件

use crate::app::{Page, PezMaxApp};
use crate::theme::colors;
use egui::{FontId, CornerRadius, Vec2};

pub fn render(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    ui.add_space(24.0);

    // 欢迎标题
    let default_name = "用户".to_string();
    let nickname = app.current_user.as_ref().map(|u| &u.nick_name).unwrap_or(&default_name);
    ui.label(
        egui::RichText::new(format!("你好，{} 👋", nickname))
            .font(FontId::new(28.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_PRIMARY),
    );
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new("欢迎使用 PezMax 试卷资源管理系统")
            .font(FontId::new(14.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_SECONDARY),
    );

    ui.add_space(32.0);

    // 统计卡片（Metro Tile 风格）
    let stats = [
        ("📄", "试卷总数", "1,234", colors::PRIMARY),
        ("⭐", "我的收藏", "56", colors::ACCENT_ORANGE),
        ("📥", "下载次数", "189", colors::ACCENT_GREEN),
        ("🔖", "书签", "23", colors::ACCENT_PURPLE),
    ];

    ui.horizontal_wrapped(|ui| {
        for (icon, label, value, color) in &stats {
            egui::Frame::new()
                .fill(*color)
                .corner_radius(CornerRadius::same(12))
                .show(ui, |ui| {
                    ui.set_min_size(Vec2::new(180.0, 120.0));
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        ui.label(
                            egui::RichText::new(*icon)
                                .font(FontId::new(32.0, egui::FontFamily::Proportional)),
                        );
                    });
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new(*value)
                                    .font(FontId::new(28.0, egui::FontFamily::Proportional))
                                    .color(colors::TEXT_ON_PRIMARY),
                            );
                            ui.label(
                                egui::RichText::new(*label)
                                    .font(FontId::new(13.0, egui::FontFamily::Proportional))
                                    .color(colors::TEXT_ON_PRIMARY),
                            );
                        });
                    });
                });
            ui.add_space(12.0);
        }
    });

    ui.add_space(32.0);

    // 快捷入口（第二行 tile）
    ui.label(
        egui::RichText::new("快捷入口")
            .font(FontId::new(18.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_PRIMARY),
    );
    ui.add_space(12.0);

    let shortcuts = [
        ("📁", "浏览试卷", "查看所有试卷资源", Page::FileExplorer),
        ("⭐", "我的收藏", "管理收藏的试卷", Page::Favorites),
        ("📥", "下载记录", "查看下载历史", Page::Downloads),
        ("🔔", "通知中心", "查看系统通知", Page::Notifications),
    ];

    ui.horizontal_wrapped(|ui| {
        for (icon, title, desc, page) in &shortcuts {
            let response = egui::Frame::new()
                .fill(colors::BG_CARD)
                .corner_radius(CornerRadius::same(10))
                .stroke(egui::Stroke::new(1.0, colors::BORDER))
                .show(ui, |ui| {
                    ui.set_min_size(Vec2::new(200.0, 80.0));
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        ui.add_space(12.0);
                        ui.label(
                            egui::RichText::new(*icon)
                                .font(FontId::new(28.0, egui::FontFamily::Proportional)),
                        );
                        ui.add_space(12.0);
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new(*title)
                                    .font(FontId::new(15.0, egui::FontFamily::Proportional))
                                    .color(colors::TEXT_PRIMARY),
                            );
                            ui.label(
                                egui::RichText::new(*desc)
                                    .font(FontId::new(12.0, egui::FontFamily::Proportional))
                                    .color(colors::TEXT_SECONDARY),
                            );
                        });
                    });
                })
                .response
                .interact(egui::Sense::click());

            if response.clicked() {
                app.navigate(page.clone());
            }
            ui.add_space(12.0);
        }
    });
}