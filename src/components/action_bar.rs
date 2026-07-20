// 试卷详情页底部操作栏
// 水平全宽条，TopBottomPanel 级别，左右贴边

use crate::theme::colors;
use egui::{Color32, FontId, Sense, Vec2};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    None,
    Back,
    Download,
    Favorite,
    Report,
    ToggleInfo,
}

/// 渲染底部操作栏（TopBottomPanel 级别，左右贴边）
pub fn render_bar(ctx: &egui::Context, file_name: &str) -> Action {
    let mut action = Action::None;
    let bar_bg = Color32::from_rgb(0x1E, 0x1E, 0x2E);

    egui::TopBottomPanel::bottom("preview_action_bar")
        .min_height(48.0)
        .max_height(48.0)
        .frame(
            egui::Frame::new()
                .fill(bar_bg)
                .inner_margin(egui::Margin::ZERO)
                .stroke(egui::Stroke::NONE),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(0.0);

                if bar_btn(ui, "← 返回列表", 14.0) {
                    action = Action::Back;
                }

                ui.add_space(24.0);

                ui.label(
                    egui::RichText::new(file_name)
                        .font(FontId::new(14.0, egui::FontFamily::Proportional))
                        .color(Color32::from_gray(200)),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(0.0);
                    if bar_btn(ui, "ℹ️ 信息", 14.0) { action = Action::ToggleInfo; }
                    ui.add_space(8.0);
                    if bar_btn(ui, "🚩 举报", 14.0) { action = Action::Report; }
                    ui.add_space(8.0);
                    if bar_btn(ui, "⭐ 收藏", 14.0) { action = Action::Favorite; }
                    ui.add_space(8.0);
                    if bar_btn(ui, "📥 下载", 14.0) { action = Action::Download; }
                });
            });
        });

    action
}

fn bar_btn(ui: &mut egui::Ui, text: &str, font_size: f32) -> bool {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(80.0, 48.0), Sense::click());
    let clicked = response.clicked();
    let hovered = response.hovered();
    let pressed = response.is_pointer_button_down_on();
    response.on_hover_text(text);

    if ui.is_rect_visible(rect) {
        if hovered || pressed {
            let bg = if pressed { colors::bg_selected() } else { colors::bg_hover() };
            ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, bg);
        }
        ui.painter().text(
            egui::pos2(rect.left() + 12.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            text,
            FontId::new(font_size, egui::FontFamily::Proportional),
            Color32::WHITE,
        );
    }
    clicked
}