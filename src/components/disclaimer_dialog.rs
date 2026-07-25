//! 免责声明弹窗 — Metro Design。
//! 使用 Sokuou `Progress` 做 1s 倒计时门：`progress.value() >= 1.0` 才可点确认。

use crate::pages::register::metro_button;
use crate::sokuou::Progress;
use crate::theme::colors;
use egui::{CornerRadius, FontId, Stroke};


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
            // ── 布局：把弹窗内部的固定 rect 显式切成 [标题 + 正文] / [按钮行] 两块 ──
            //    这样按钮行必然贴在色条底部，不再依赖 `available_height()` 的运行时估算。
            let full_rect = ui.max_rect();
            let button_row_h  = 32.0;
            let bottom_pad    = 20.0;
            let top_pad       = 20.0;
            let side_pad      = 24.0;

            // 左侧强调色条 —— 与 full_rect 完整同高
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    full_rect.left_top(),
                    egui::vec2(3.0, full_rect.height()),
                ),
                CornerRadius::ZERO,
                colors::primary(),
            );

            // 按钮行 rect（贴底）
            let btn_rect = egui::Rect::from_min_max(
                egui::pos2(full_rect.left(), full_rect.bottom() - bottom_pad - button_row_h),
                egui::pos2(full_rect.right(), full_rect.bottom() - bottom_pad),
            );
            // 正文 rect（顶部 → 按钮行上方）
            let body_rect = egui::Rect::from_min_max(
                egui::pos2(full_rect.left(), full_rect.top() + top_pad),
                egui::pos2(full_rect.right(), btn_rect.top() - 12.0),
            );

            // ── 正文区（标题 + 滚动正文，填满 body_rect）────────────────────
            let mut body_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(body_rect)
                    .layout(egui::Layout::top_down(egui::Align::LEFT)),
            );
            body_ui.horizontal(|ui| {
                ui.add_space(side_pad);
                ui.label(
                    egui::RichText::new("免责声明与用户协议")
                        .font(FontId::new(16.0, egui::FontFamily::Proportional))
                        .color(colors::text_primary())
                        .strong(),
                );
            });
            body_ui.add_space(12.0);
            let scroll_h = body_ui.available_height();
            egui::ScrollArea::vertical()
                .id_salt("disclaimer_scroll")
                .max_height(scroll_h)
                .min_scrolled_height(scroll_h)
                .auto_shrink([false, false])
                .show(&mut body_ui, |ui| {
                    ui.set_min_height(scroll_h);
                    ui.horizontal(|ui| {
                        ui.add_space(side_pad);
                        ui.vertical(|ui| {
                            ui.set_max_width(ui.available_width() - side_pad);
                            ui.label(
                                egui::RichText::new(body_text)
                                    .font(FontId::new(12.0, egui::FontFamily::Proportional))
                                    .color(colors::text_secondary()),
                            );
                        });
                    });
                });

            // ── 按钮行（贴底）────────────────────────────────────────────────
            let mut btn_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(btn_rect)
                    .layout(egui::Layout::left_to_right(egui::Align::Center)),
            );
            btn_ui.add_space(side_pad);
            if metro_button(
                &mut btn_ui,
                "取消",
                13.0,
                egui::vec2(88.0, button_row_h),
                colors::bg_input(),
                colors::text_secondary(),
                false,
                true,
            ).clicked() {
                closed = true;
            }

            btn_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(side_pad);
                let label = if can_confirm {
                    "我已阅读并同意".to_string()
                } else {
                    format!("我已阅读并同意 ({:.0}s)", remain.ceil())
                };
                let resp = metro_button(
                    ui,
                    &label,
                    13.0,
                    egui::vec2(180.0, button_row_h),
                    colors::primary(),
                    colors::text_on_primary(),
                    true,
                    can_confirm,
                );
                if resp.clicked() && can_confirm {
                    confirmed = true;
                }
            });

            // 强制把这个 fixed_size 窗口的内部游标推到底部，防止 egui 因内容不足
            // 而在色条与实际布局之间留白（关键的一行 —— 之前 `available_height` 之所以
            // 算不准，就是因为它读的是当时的游标，而游标还没到底）。
            ui.allocate_rect(full_rect, egui::Sense::hover());
        });

    (confirmed, closed)
}
