// 顶部栏：全局搜索 + 通知铃 + 用户名

use crate::app::{PezMaxApp, Section, Subsection};
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
                let search_bg = colors::bg_search();
                ui.scope(|ui| {
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
                            .hint_text("🔍 搜索试卷、学科、学校...")
                            .font(FontId::new(18.0, egui::FontFamily::Proportional))
                            .margin(egui::Margin::symmetric(14, 10)),
                    );
                });

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