//! 举报对话框 — 采集 fileId/userId/reason/remark 四字段。
//! 参考 `ReportFileDialog.vue`。

use crate::app::PezMaxApp;
use crate::theme::colors;
use egui::{CornerRadius, FontId, Stroke};

/// 返回 (submit_clicked, close_clicked)。
pub fn render(ctx: &egui::Context, app: &mut PezMaxApp) -> (bool, bool) {
    if !app.show_report_dialog {
        return (false, false);
    }

    let mut submit = false;
    let mut close = false;

    egui::Window::new("举报")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .fixed_size(egui::vec2(500.0, 460.0))
        .title_bar(false)
        .frame(egui::Frame::new()
            .fill(colors::bg_card())
            .corner_radius(CornerRadius::ZERO)
            .stroke(Stroke::new(1.0, colors::border())))
        .show(ctx, |ui| {
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    ui.max_rect().left_top(),
                    egui::vec2(3.0, ui.max_rect().height()),
                ),
                CornerRadius::ZERO,
                colors::primary(),
            );

            ui.add_space(20.0);
            ui.horizontal(|ui| {
                ui.add_space(24.0);
                ui.label(
                    egui::RichText::new("举报文件")
                        .font(FontId::new(16.0, egui::FontFamily::Proportional))
                        .color(colors::text_primary())
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(20.0);
                    if ui.scope(|ui| {
                        ui.visuals_mut().widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
                        ui.add(
                            egui::Button::new(
                                egui::RichText::new("×")
                                    .font(FontId::new(20.0, egui::FontFamily::Proportional))
                                    .color(colors::text_secondary()),
                            )
                            .stroke(Stroke::NONE)
                            .min_size(egui::vec2(24.0, 24.0)),
                        )
                    }).inner.clicked() {
                        close = true;
                    }
                });
            });

            ui.add_space(14.0);

            // 目标文件信息（只读）
            ui.horizontal(|ui| {
                ui.add_space(24.0);
                ui.label(
                    egui::RichText::new(format!("目标文件: {}", app.report_target_file_name))
                        .font(FontId::new(12.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary()),
                );
            });
            ui.add_space(10.0);

            // 举报理由
            ui.horizontal(|ui| {
                ui.add_space(24.0);
                ui.label(
                    egui::RichText::new("举报理由")
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(colors::text_primary()),
                );
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space(24.0);
                ui.add(
                    egui::TextEdit::multiline(&mut app.report_reason)
                        .desired_rows(3)
                        .desired_width(ui.available_width() - 24.0)
                        .hint_text("请说明违规类型：色情低俗 / 广告垃圾 / 侵权 / 政治敏感 / 其它"),
                );
            });

            ui.add_space(12.0);
            // 补充说明
            ui.horizontal(|ui| {
                ui.add_space(24.0);
                ui.label(
                    egui::RichText::new("补充说明（可选）")
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(colors::text_primary()),
                );
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space(24.0);
                ui.add(
                    egui::TextEdit::multiline(&mut app.report_remark)
                        .desired_rows(4)
                        .desired_width(ui.available_width() - 24.0)
                        .hint_text("如有链接、时间戳或截图信息，请附在这里"),
                );
            });

            ui.add_space(18.0);

            // 按钮行
            ui.horizontal(|ui| {
                ui.add_space(24.0);
                let cancel = egui::Button::new(
                    egui::RichText::new("取消")
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary()),
                )
                .fill(colors::bg_input())
                .stroke(Stroke::NONE)
                .corner_radius(CornerRadius::ZERO)
                .min_size(egui::vec2(96.0, 32.0));
                if ui.add(cancel).clicked() {
                    close = true;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(24.0);
                    let can_submit = !app.report_reason.trim().is_empty() && !app.report_submit_rx.is_some();
                    let submit_btn = egui::Button::new(
                        egui::RichText::new(if app.report_submit_rx.is_some() { "提交中…" } else { "提交举报" })
                            .font(FontId::new(13.0, egui::FontFamily::Proportional))
                            .color(colors::text_on_primary()),
                    )
                    .fill(colors::primary())
                    .stroke(Stroke::NONE)
                    .corner_radius(CornerRadius::ZERO)
                    .min_size(egui::vec2(120.0, 32.0));
                    if ui.add_enabled(can_submit, submit_btn).clicked() && can_submit {
                        submit = true;
                    }
                });
            });
        });

    (submit, close)
}
