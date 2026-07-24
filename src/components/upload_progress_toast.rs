//! 上传进度 toast — 右下角固定，SpringAnim 从右滑入。
//! 简单不定进度：primary 色横向填充条循环左右滑动。

use crate::theme::colors;
use egui::{Color32, CornerRadius, FontId, Rect, Stroke, Vec2, pos2};

/// 在屏幕右下角渲染上传中提示。
pub fn render(ctx: &egui::Context, file_name: &str) {
    let screen_rect = ctx.screen_rect();
    let toast_w = 320.0;
    let toast_h = 64.0;
    let margin = 24.0;
    let rect = Rect::from_min_size(
        pos2(
            screen_rect.right() - toast_w - margin,
            screen_rect.bottom() - toast_h - margin,
        ),
        Vec2::new(toast_w, toast_h),
    );
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("upload_toast"),
    ));
    painter.rect_filled(rect, CornerRadius::ZERO, colors::bg_card());
    painter.rect_stroke(
        rect,
        CornerRadius::ZERO,
        Stroke::new(1.0, colors::border()),
        egui::StrokeKind::Outside,
    );
    // 左 3px 强调色条
    painter.rect_filled(
        Rect::from_min_max(rect.left_top(), pos2(rect.left() + 3.0, rect.bottom())),
        CornerRadius::ZERO,
        colors::primary(),
    );

    // 文本
    painter.text(
        pos2(rect.left() + 16.0, rect.top() + 12.0),
        egui::Align2::LEFT_TOP,
        "上传中…",
        FontId::new(13.0, egui::FontFamily::Proportional),
        colors::text_primary(),
    );
    painter.text(
        pos2(rect.left() + 16.0, rect.top() + 30.0),
        egui::Align2::LEFT_TOP,
        file_name,
        FontId::new(11.0, egui::FontFamily::Proportional),
        colors::text_secondary(),
    );

    // 不定进度条：使用时间派生的 sine 位移
    let t = ctx.input(|i| i.time as f32);
    let bar_w = 60.0;
    let travel = rect.width() - bar_w - 32.0;
    let x = 16.0 + (0.5 + 0.5 * (t * 1.5).sin()) * travel;
    let bar_rect = Rect::from_min_size(
        pos2(rect.left() + x, rect.bottom() - 6.0),
        Vec2::new(bar_w, 3.0),
    );
    painter.rect_filled(bar_rect, CornerRadius::ZERO, colors::primary());
    let _ = Color32::WHITE;
    ctx.request_repaint(); // 保持 sine 动画
}
