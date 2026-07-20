// 顶部栏：全局搜索 + 通知铃 + 用户名

use crate::app::{PezMaxApp, Section, Subsection};
use crate::sokuou::map_range;
use crate::theme::colors;
use egui::FontId;

pub fn render(app: &mut PezMaxApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("topbar")
        .min_height(56.0)
        .max_height(56.0)
        .resizable(false)
        .frame(
            egui::Frame::new()
                .fill(colors::bg_white())
                .inner_margin(egui::Margin::ZERO)
                .stroke(egui::Stroke::NONE),
        )
        .show_separator_line(false)
        .show(ctx, |ui| {
            // 手动垂直居中：用 available_height 计算偏移
            let avail_h = ui.available_height();
            let box_h = 40.0_f32;
            let top_pad = ((avail_h - box_h) / 2.0).max(0.0);
            ui.add_space(top_pad);

            ui.horizontal(|ui| {
                ui.add_space(32.0);

                // 全局搜索框
                // 待机：🔍 在搜索框左边缘
                // 聚焦：🔍 平滑左滑出场，文字正常输入
                let search_bg = colors::bg_search();
                let resp = ui.scope(|ui| {
                    ui.visuals_mut().extreme_bg_color = search_bg;
                    ui.visuals_mut().widgets.noninteractive.bg_fill = search_bg;
                    ui.visuals_mut().widgets.noninteractive.weak_bg_fill = search_bg;
                    ui.visuals_mut().widgets.inactive.bg_fill = search_bg;
                    ui.visuals_mut().widgets.inactive.bg_stroke = egui::Stroke::NONE;
                    ui.visuals_mut().widgets.hovered.bg_fill = search_bg;
                    ui.visuals_mut().widgets.hovered.bg_stroke  = egui::Stroke::NONE;
                    ui.visuals_mut().widgets.active.bg_fill = search_bg;
                    ui.visuals_mut().widgets.active.bg_stroke   = egui::Stroke::NONE;
                    ui.add_sized(
                        egui::vec2(420.0, box_h),
                        egui::TextEdit::singleline(&mut app.search_query)
                            .font(FontId::new(18.0, egui::FontFamily::Proportional))
                            .margin(egui::Margin::symmetric(14, 10)),
                    )
                });

                // 追踪焦点变化，触发 🔍 滑出/滑入
                let now_focused = resp.inner.has_focus();
                if now_focused != app.search_was_focused {
                    app.search_was_focused = now_focused;
                    app.search_hint_anim.set_target(if now_focused { 1.0 } else { 0.0 });
                }

                // 绘制 🔍 图标（仅搜索框为空时显示，弹簧驱动左滑出场）
                let r = resp.inner.rect;
                let center_y = r.center().y;
                let p = app.search_hint_anim.value();
                if app.search_query.is_empty() {
                    // p=0（待机）：🔍 在框内左边缘；p=1（聚焦）：🔍 左滑出场
                    let icon_x = map_range(p, (r.left() + 4.0) as f64, (r.left() - 30.0) as f64) as f32;

                    // 用 scope 约束绘制区域，确保 🔍 滑出框外时被裁剪
                    ui.scope(|ui| {
                        ui.set_clip_rect(r);
                        ui.painter().text(
                            egui::pos2(icon_x, center_y),
                            egui::Align2::LEFT_CENTER,
                            "🔍",
                            FontId::new(18.0, egui::FontFamily::Proportional),
                            colors::text_secondary(),
                        );
                    });
                }

                // 占位文字（仅在搜索框为空时显示）
                if app.search_query.is_empty() {
                    let text = "搜索试卷、学科、学校...";
                    let font_id = FontId::new(18.0, egui::FontFamily::Proportional);

                    // 计算文字宽度
                    let text_width = ui.fonts(|f| {
                        f.layout_no_wrap(text.to_owned(), font_id.clone(), egui::Color32::WHITE)
                            .size()
                            .x
                    });

                    // p=0：文字右端在 🔍 处（整段隐藏）；p=1：文字左端到 TextEdit 边距处
                    let text_x = map_range(
                        p,
                        (r.left() - 2.0 - text_width) as f64,
                        (r.left() + 14.0) as f64,
                    ) as f32;

                    ui.scope(|ui| {
                        ui.set_clip_rect(r);
                        ui.painter().text(
                            egui::pos2(text_x, center_y),
                            egui::Align2::LEFT_CENTER,
                            text,
                            font_id,
                            colors::text_secondary(),
                        );
                    });
                }

                // 搜索框回车 → 跳转到「浏览-资源管理」
                if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                    app.navigate_to(Section::Browse, Subsection::ResourceManager);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(24.0);

                    // 用户名
                    if let Some(ref user) = app.current_user {
                        ui.label(
                            egui::RichText::new(&user.nick_name)
                                .font(FontId::new(15.0, egui::FontFamily::Proportional))
                                .color(colors::text_primary()),
                        );
                        ui.add_space(12.0);
                    }

                    // 通知铃（点击跳转到个人 > 通知）
                    let bell_label = if app.unread_notifications > 0 {
                        format!("🔔 {}", app.unread_notifications)
                    } else {
                        "🔔".to_string()
                    };
                    let bell_resp = ui.label(
                        egui::RichText::new(bell_label)
                            .font(FontId::new(20.0, egui::FontFamily::Proportional))
                            .color(colors::text_secondary()),
                    );
                    if bell_resp.interact(egui::Sense::click()).clicked() {
                        app.navigate_to(Section::Profile, Subsection::Notifications);
                    }
                    ui.add_space(8.0);
                });
            });
        });
}