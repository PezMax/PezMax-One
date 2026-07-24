//! 步骤条：Metro 风格短横杠 + 强调色进度填充。
//! 由 `SpringAnim`（0..=total-1 的浮点值）驱动指示位置。

use crate::theme::colors;
use egui::{Color32, CornerRadius, FontId, Rect, Stroke, StrokeKind, Vec2, pos2};

/// 渲染一行 `total` 个步骤指示器。
/// `current_anim_value` 通常来自 `SpringAnim::value() as f32`，范围 [0, total-1]。
/// `labels` 长度需等于 total（例如 ["账号", "密保", "昵称", "验证码"]）。
pub fn render(ui: &mut egui::Ui, current_anim_value: f32, total: usize, labels: &[&str]) {
    let width = ui.available_width().min(420.0);
    let step_gap = 8.0;
    let step_w = (width - step_gap * (total as f32 - 1.0)) / total as f32;
    let bar_h = 4.0;

    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(width, bar_h + 22.0),
        egui::Sense::hover(),
    );

    // 每个 step 分段
    for i in 0..total {
        let x0 = rect.left() + i as f32 * (step_w + step_gap);
        let bar_rect = Rect::from_min_size(pos2(x0, rect.top()), Vec2::new(step_w, bar_h));

        // 计算这一段的填充比例
        let fill_frac = (current_anim_value - i as f32).clamp(0.0, 1.0);
        // 底色
        ui.painter().rect_filled(bar_rect, CornerRadius::ZERO, colors::bg_input());
        // 填充色
        if fill_frac > 0.0 {
            let fill_rect = Rect::from_min_size(bar_rect.min, Vec2::new(step_w * fill_frac, bar_h));
            ui.painter().rect_filled(fill_rect, CornerRadius::ZERO, colors::primary());
        }
        // 完成 tick 圈（当前段已完全填满）
        if fill_frac >= 1.0 {
            let dot_center = pos2(x0 + step_w - 4.0, bar_rect.center().y);
            ui.painter().circle_filled(dot_center, 3.5, colors::primary());
        }

        // 标签
        if let Some(label) = labels.get(i) {
            let active = current_anim_value >= i as f32 - 0.15;
            let color = if active { colors::text_primary() } else { colors::text_secondary() };
            ui.painter().text(
                pos2(x0 + step_w / 2.0, rect.top() + bar_h + 10.0),
                egui::Align2::CENTER_TOP,
                *label,
                FontId::new(11.0, egui::FontFamily::Proportional),
                color,
            );
        }
    }
    // 用一次 rect_stroke 让整条外框更清爽（可选）
    let _ = Stroke::NONE;
    let _ = StrokeKind::Outside;
    let _ = Color32::WHITE;
}
