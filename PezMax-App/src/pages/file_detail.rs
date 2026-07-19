// 文件详情页面

use crate::app::PezMaxApp;
use crate::theme::colors;
use egui::FontId;

pub fn render(_app: &mut PezMaxApp, ui: &mut egui::Ui, file_id: i64) {
    ui.add_space(16.0);
    ui.label(
        egui::RichText::new(format!("文件详情 #{}", file_id))
            .font(FontId::new(20.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_PRIMARY),
    );
    ui.add_space(16.0);
    ui.label("详细信息区域（待实现）...");
}