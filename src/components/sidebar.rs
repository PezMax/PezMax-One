// 可折叠汉堡菜单侧边栏
// SpringAnim 驱动宽度 48↔200px，sidebar_indicator_anim 驱动左侧高亮滑块

use crate::app::{PezMaxApp, Section};
use crate::sokuou::map_range_clamped;
use crate::theme::colors;
use egui::{Color32, CornerRadius, Frame, Rect, Sense, pos2};

pub fn render(app: &mut PezMaxApp, ctx: &egui::Context) {
    let anim_val = app.sidebar_anim.value().clamp(0.0, 1.0);
    let width = map_range_clamped(anim_val, 48.0, 200.0) as f32;
    // 导航标签渐入渐出 + 滑动
    // 0.3 → 0.7 之间从透明到不透明，同时从左侧滑入
    let label_alpha = ((anim_val - 0.3) / 0.4).clamp(0.0, 1.0) as f32;
    let label_offset = (1.0 - anim_val) as f32 * 6.0;

    egui::SidePanel::left("sidebar")
        .resizable(false)
        .min_width(width)
        .max_width(width)
        .frame(Frame::new().fill(colors::bg_sidebar()))
        .show(ctx, |ui| {
            ui.set_min_width(width);
            ui.add_space(8.0);

            // ☰ 汉堡按钮（用 Label + Sense::click 代替 Button，避免 button_padding 导致内容溢出）
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                let resp = ui.label(
                    egui::RichText::new("☰")
                        .font(egui::FontId::new(20.0, egui::FontFamily::Proportional))
                        .color(Color32::WHITE),
                )
                .interact(Sense::click())
                .on_hover_cursor(egui::CursorIcon::PointingHand);
                if resp.clicked() {
                    app.sidebar_open = !app.sidebar_open;
                    app.sidebar_anim
                        .set_target(if app.sidebar_open { 1.0 } else { 0.0 });
                }
            });

            ui.add_space(20.0);

            // ── 4 个导航项：收集 rect 以便绘制滑块 ──────────────
            let sections = [
                Section::Home,
                Section::Browse,
                Section::Community,
                Section::Profile,
            ];

            let mut item_rects: [Option<Rect>; 4] = [None; 4];

            for (i, section) in sections.iter().enumerate() {
                let is_active = app.current_section == *section;

                let resp = Frame::new()
                    .fill(Color32::TRANSPARENT)
                    .corner_radius(CornerRadius::same(0))
                    .show(ui, |ui| {
                        ui.set_min_width(width);
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            // 留出 3px 宽度给滑块指示器
                            ui.add_space(3.0);
                            ui.add_space(10.0);
                            ui.label(
                                egui::RichText::new(section.icon())
                                    .font(egui::FontId::new(22.0, egui::FontFamily::Proportional))
                                    .color(Color32::WHITE),
                            );
                            if label_alpha > 0.0 {
                                ui.add_space(10.0 + label_offset);
                                ui.set_opacity(label_alpha);
                                ui.label(
                                    egui::RichText::new(section.title())
                                        .font(egui::FontId::new(
                                            16.0,
                                            egui::FontFamily::Proportional,
                                        ))
                                        .color(if is_active {
                                            Color32::WHITE
                                        } else {
                                            Color32::from_gray(180)
                                        }),
                                );
                            }
                        });
                        ui.add_space(6.0);
                    })
                    .response
                    .interact(egui::Sense::click())
                    .on_hover_cursor(egui::CursorIcon::PointingHand);

                // Save what we need before on_hover_text consumes resp
                let nav_clicked = resp.clicked();
                let nav_rect = resp.rect;
                item_rects[i] = Some(nav_rect);

                // Tooltip 在折叠态显示功能名（on_hover_text consumes resp）
                if label_alpha < 0.5 {
                    resp.on_hover_text(section.title());
                }

                if nav_clicked && !is_active {
                    app.navigate_section(*section);
                }
            }

            // ── 滑块指示器（弹簧插值 y 位置）────────────────────
            let idx_f = app.sidebar_indicator_anim.value(); // 0.0 – 3.0
            let lo = idx_f.floor() as usize;
            let hi = (idx_f.ceil() as usize).min(3);
            let t = idx_f.fract() as f32;

            if let (Some(r_lo), Some(r_hi)) = (item_rects[lo], item_rects[hi]) {
                let y_top = egui::lerp(r_lo.top()..=r_hi.top(), t);
                let y_bot = egui::lerp(r_lo.bottom()..=r_hi.bottom(), t);
                let bar = Rect::from_min_max(pos2(r_lo.left(), y_top), pos2(r_lo.left() + 3.0, y_bot));
                ui.painter().rect_filled(bar, egui::CornerRadius::ZERO, colors::primary());
            }

            // ── 底部：退出登录 ────────────────────────────────────
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.add_space(8.0);
                let logout = Frame::new()
                    .corner_radius(CornerRadius::same(0))
                    .show(ui, |ui| {
                        ui.set_min_width(width);
                        ui.horizontal(|ui| {
                            ui.add_space(13.0);
                            ui.label(
                                egui::RichText::new("🔓")
                                    .font(egui::FontId::new(20.0, egui::FontFamily::Proportional))
                                    .color(Color32::WHITE),
                            );
                            if label_alpha > 0.0 {
                                ui.add_space(10.0 + label_offset);
                                ui.set_opacity(label_alpha);
                                ui.label(
                                    egui::RichText::new("退出登录")
                                        .font(egui::FontId::new(
                                            14.0,
                                            egui::FontFamily::Proportional,
                                        ))
                                        .color(Color32::from_gray(180)),
                                );
                            }
                        });
                        ui.add_space(4.0);
                    })
                    .response
                    .interact(egui::Sense::click())
                    .on_hover_cursor(egui::CursorIcon::PointingHand);

                let logout_clicked = logout.clicked();
                if label_alpha < 0.5 {
                    logout.on_hover_text("退出登录");
                }

                if logout_clicked {
                    app.logout();
                }
            });
        });
}
