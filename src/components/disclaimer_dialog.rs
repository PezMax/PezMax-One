//! 免责声明弹窗 — Metro Design。
//! 使用 Sokuou `Progress` 做 1s 倒计时门：`progress.value() >= 1.0` 才可点确认。

use crate::sokuou::Progress;
use crate::theme::colors;
use egui::{Color32, CornerRadius, FontId, Stroke};

/// 返回 (是否点了确认, 是否点了关闭)。
pub fn render(
    ctx: &egui::Context,
    open: bool,
    countdown: &Progress,
    body_text: &str,
) -> (bool, bool) {
    if !open {
        return (false, false);
    }

    let mut confirmed = false;
    let mut closed = false;
    let remain = (1.0 - countdown.value()).max(0.0);
    let can_confirm = countdown.value() >= 1.0;

    egui::Window::new("免责声明")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .fixed_size(egui::vec2(480.0, 420.0))
        .title_bar(false)
        .frame(egui::Frame::new()
            .fill(colors::bg_card())
            .corner_radius(CornerRadius::ZERO)
            .stroke(Stroke::new(1.0, colors::border())))
        .show(ctx, |ui| {
            // 左 3px 强调色条
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
                    egui::RichText::new("免责声明与用户协议")
                        .font(FontId::new(16.0, egui::FontFamily::Proportional))
                        .color(colors::text_primary())
                        .strong(),
                );
            });
            ui.add_space(12.0);

            // 正文
            egui::ScrollArea::vertical()
                .id_salt("disclaimer_scroll")
                .max_height(260.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(24.0);
                        ui.vertical(|ui| {
                            ui.set_max_width(ui.available_width() - 24.0);
                            ui.label(
                                egui::RichText::new(body_text)
                                    .font(FontId::new(12.0, egui::FontFamily::Proportional))
                                    .color(colors::text_secondary()),
                            );
                        });
                    });
                });

            ui.add_space(16.0);

            // 按钮行
            ui.horizontal(|ui| {
                ui.add_space(24.0);
                let close_btn = egui::Button::new(
                    egui::RichText::new("取消")
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary()),
                )
                .fill(colors::bg_input())
                .stroke(Stroke::NONE)
                .corner_radius(CornerRadius::ZERO)
                .min_size(egui::vec2(88.0, 32.0));
                if ui.add(close_btn).clicked() {
                    closed = true;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(24.0);
                    let label = if can_confirm {
                        "我已阅读并同意".to_string()
                    } else {
                        format!("我已阅读并同意 ({:.0}s)", remain.ceil())
                    };
                    // 未达倒计时时按钮变灰不可点
                    let fill = if can_confirm { colors::primary() } else {
                        let p = colors::primary();
                        Color32::from_rgba_premultiplied(p.r(), p.g(), p.b(), 90)
                    };
                    let btn = egui::Button::new(
                        egui::RichText::new(label)
                            .font(FontId::new(13.0, egui::FontFamily::Proportional))
                            .color(colors::text_on_primary()),
                    )
                    .fill(fill)
                    .stroke(Stroke::NONE)
                    .corner_radius(CornerRadius::ZERO)
                    .min_size(egui::vec2(180.0, 32.0));
                    let resp = ui.add_enabled(can_confirm, btn);
                    if resp.clicked() && can_confirm {
                        confirmed = true;
                    }
                });
            });
        });

    (confirmed, closed)
}
