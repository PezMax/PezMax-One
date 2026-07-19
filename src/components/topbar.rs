// 顶部栏：全局搜索 + 通知铃 + 用户名

use crate::app::{PezMaxApp, Section, Subsection};
use crate::theme::colors;
use egui::FontId;

pub fn render(app: &mut PezMaxApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("topbar")
        .min_height(48.0)
        .max_height(48.0)
        .frame(
            egui::Frame::new()
                .fill(colors::BG_CARD)
                .stroke(egui::Stroke::new(1.0, colors::BORDER)),
        )
        .show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(16.0);

                // 全局搜索框
                ui.add(
                    egui::TextEdit::singleline(&mut app.search_query)
                        .hint_text("🔍 搜索试卷、学科、学校...")
                        .desired_width(280.0)
                        .font(FontId::new(14.0, egui::FontFamily::Proportional)),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(16.0);

                    // 用户名
                    if let Some(ref user) = app.current_user {
                        ui.label(
                            egui::RichText::new(&user.nick_name)
                                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                .color(colors::TEXT_PRIMARY),
                        );
                        ui.add_space(8.0);
                    }

                    // 通知铃（点击跳转到个人 > 通知）
                    let bell_label = if app.unread_notifications > 0 {
                        format!("🔔 {}", app.unread_notifications)
                    } else {
                        "🔔".to_string()
                    };
                    let bell_resp = ui.label(
                        egui::RichText::new(bell_label)
                            .font(FontId::new(18.0, egui::FontFamily::Proportional))
                            .color(colors::TEXT_SECONDARY),
                    );
                    if bell_resp.interact(egui::Sense::click()).clicked() {
                        app.navigate_to(Section::Profile, Subsection::Notifications);
                    }
                    ui.add_space(8.0);
                });
            });
        });
}
