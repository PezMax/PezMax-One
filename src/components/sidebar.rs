// Metro Design 侧边栏
// 深色背景，大图标+文字导航，类似 Windows 11 开始菜单风格

use crate::app::{Page, PezMaxApp};
use crate::theme::colors;
use egui::{Color32, Frame, CornerRadius, Vec2};

/// 导航项定义
struct NavItem {
    label: &'static str,
    icon: &'static str, // Unicode 图标
    page: Page,
    badge: Option<i32>, // 通知角标
}

const NAV_ITEMS: &[NavItem] = &[
    NavItem { label: "首页", icon: "🏠", page: Page::Home, badge: None },
    NavItem { label: "试卷浏览", icon: "📁", page: Page::FileExplorer, badge: None },
    NavItem { label: "书签管理", icon: "🔖", page: Page::Bookmarks, badge: None },
    NavItem { label: "我的收藏", icon: "⭐", page: Page::Favorites, badge: None },
    NavItem { label: "下载记录", icon: "📥", page: Page::Downloads, badge: None },
    NavItem { label: "通知中心", icon: "🔔", page: Page::Notifications, badge: Some(0) },
    NavItem { label: "个人中心", icon: "👤", page: Page::Profile, badge: None },
    NavItem { label: "安全设置", icon: "🔒", page: Page::Security, badge: None },
    NavItem { label: "举报中心", icon: "🚩", page: Page::Report, badge: None },
    NavItem { label: "系统设置", icon: "⚙️", page: Page::Settings, badge: None },
];

pub fn render(app: &mut PezMaxApp, ctx: &egui::Context) {
    let sidebar_width = if app.sidebar_open { 220.0 } else { 60.0 };

    egui::SidePanel::left("sidebar")
        .resizable(false)
        .default_width(sidebar_width)
        .min_width(sidebar_width)
        .max_width(sidebar_width)
        .frame(Frame::new().fill(colors::BG_SIDEBAR))
        .show(ctx, |ui| {
            // 间距
            ui.add_space(16.0);

            // Logo / 标题区域
            ui.horizontal(|ui| {
                ui.add_space(if app.sidebar_open { 16.0 } else { 12.0 });
                ui.label(
                    egui::RichText::new("P")
                        .font(egui::FontId::new(28.0, egui::FontFamily::Proportional))
                        .color(colors::ACCENT_ORANGE),
                );
                if app.sidebar_open {
                    ui.label(
                        egui::RichText::new("ezMax")
                            .font(egui::FontId::new(20.0, egui::FontFamily::Proportional))
                            .color(colors::TEXT_ON_PRIMARY),
                    );
                }
            });

            ui.add_space(24.0);

            // 用户信息摘要
            if app.sidebar_open {
                if let Some(ref user) = app.current_user {
                    ui.horizontal(|ui| {
                        ui.add_space(12.0);
                        // 头像占位
                        let avatar_size = 40.0;
                        egui::Frame::new()
                            .fill(colors::PRIMARY)
                            .corner_radius(CornerRadius::same(0))
                            .show(ui, |ui| {
                                ui.allocate_space(Vec2::new(avatar_size, avatar_size));
                                ui.allocate_ui_at_rect(
                                    ui.max_rect(),
                                    |ui| {
                                        ui.vertical_centered(|ui| {
                                            ui.label(
                                                egui::RichText::new(
                                                    &user.nick_name.chars().next().unwrap_or('?').to_string(),
                                                )
                                                .color(colors::TEXT_ON_PRIMARY)
                                                .font(egui::FontId::new(18.0, egui::FontFamily::Proportional)),
                                            );
                                        });
                                    },
                                );
                            });
                        ui.add_space(8.0);
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new(&user.nick_name)
                                    .color(colors::TEXT_ON_PRIMARY)
                                    .font(egui::FontId::new(14.0, egui::FontFamily::Proportional)),
                            );
                            ui.label(
                                egui::RichText::new(&user.user_name)
                                    .color(Color32::from_gray(160))
                                    .font(egui::FontId::new(11.0, egui::FontFamily::Proportional)),
                            );
                        });
                    });
                    ui.add_space(20.0);
                }
            }

            // 导航项
            for item in NAV_ITEMS {
                let is_active = app.current_page == item.page;
                let bg = if is_active {
                    colors::PRIMARY
                } else {
                    Color32::TRANSPARENT
                };

                let response = egui::Frame::new()
                    .fill(bg)
                    .corner_radius(CornerRadius::same(0))
                    .show(ui, |ui| {
                        ui.add_space(2.0);
                        ui.horizontal(|ui| {
                            ui.add_space(if app.sidebar_open { 12.0 } else { 8.0 });
                            // 图标
                            ui.label(
                                egui::RichText::new(item.icon)
                                    .font(egui::FontId::new(18.0, egui::FontFamily::Proportional)),
                            );
                            if app.sidebar_open {
                                ui.add_space(8.0);
                                ui.label(
                                    egui::RichText::new(item.label)
                                        .color(colors::TEXT_ON_PRIMARY)
                                        .font(egui::FontId::new(14.0, egui::FontFamily::Proportional)),
                                );
                                // 角标
                                if let Some(count) = item.badge {
                                    if count > 0 {
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            egui::Frame::new()
                                                .fill(colors::ERROR)
                                                .corner_radius(CornerRadius::same(0))
                                                .show(ui, |ui| {
                                                    ui.add_space(4.0);
                                                    ui.label(
                                                        egui::RichText::new(count.to_string())
                                                            .color(colors::TEXT_ON_PRIMARY)
                                                            .font(egui::FontId::new(11.0, egui::FontFamily::Proportional)),
                                                    );
                                                    ui.add_space(4.0);
                                                });
                                        });
                                    }
                                }
                            }
                        });
                        ui.add_space(2.0);
                    })
                    .response
                    .interact(egui::Sense::click());

                if response.clicked() && app.current_page != item.page {
                    app.navigate(item.page.clone());
                }

                // hover 效果
                if response.hovered() && !is_active {
                    // 简化为交互反馈 (egui 的 interact 自带 hover 高亮)
                }
            }

            // 底部：登出按钮
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.add_space(8.0);
                let logout_btn = egui::Frame::new()
                    .corner_radius(CornerRadius::same(0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.add_space(if app.sidebar_open { 12.0 } else { 8.0 });
                            ui.label(
                                egui::RichText::new("🚪")
                                    .font(egui::FontId::new(18.0, egui::FontFamily::Proportional)),
                            );
                            if app.sidebar_open {
                                ui.add_space(8.0);
                                ui.label(
                                    egui::RichText::new("退出登录")
                                        .color(Color32::from_gray(180))
                                        .font(egui::FontId::new(14.0, egui::FontFamily::Proportional)),
                                );
                            }
                        });
                    })
                    .response
                    .interact(egui::Sense::click());

                if logout_btn.clicked() {
                    app.is_logged_in = false;
                    app.token = None;
                    app.current_user = None;
                    app.navigate(Page::Login);
                }
            });
        });
}