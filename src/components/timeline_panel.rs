//! 举报时间线弹窗 — 3 节点 + 状态描述。参考 `ReporTimeLinePanel.vue`。
//! 数据形态：`serde_json::Value`，字段 { submitTime, reviewTime, resolveTime, status, remark }。

use crate::app::PezMaxApp;
use crate::sokuou::{map_range_clamped, SpringAnim};
use crate::theme::colors;
use egui::{CornerRadius, FontId, Stroke, Vec2, pos2};

/// 返回 close_clicked。
pub fn render(ctx: &egui::Context, app: &mut PezMaxApp) -> bool {
    if !app.show_report_timeline {
        return false;
    }
    let mut close = false;

    // 动画：使用 report_timeline_anim（0=隐藏, 1=完全展示）
    let t = app.report_timeline_anim.value() as f32;
    let dy = map_range_clamped(app.report_timeline_anim.value(), 24.0, 0.0) as f32;
    let alpha = (t * 255.0).clamp(0.0, 255.0) as u8;

    egui::Window::new("举报时间线")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, dy])
        .fixed_size(egui::vec2(520.0, 480.0))
        .title_bar(false)
        .frame(egui::Frame::new()
            .fill(colors::bg_card().gamma_multiply(alpha as f32 / 255.0))
            .corner_radius(CornerRadius::ZERO)
            .stroke(Stroke::new(1.0, colors::border())))
        .show(ctx, |ui| {
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    ui.max_rect().left_top(),
                    Vec2::new(3.0, ui.max_rect().height()),
                ),
                CornerRadius::ZERO,
                colors::primary(),
            );

            ui.add_space(20.0);
            ui.horizontal(|ui| {
                ui.add_space(24.0);
                ui.label(
                    egui::RichText::new("举报进度")
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
                            .min_size(Vec2::new(24.0, 24.0)),
                        )
                    }).inner.clicked() {
                        close = true;
                    }
                });
            });

            ui.add_space(18.0);

            // 时间线数据
            let (submit_time, review_time, resolve_time, status, remark) = extract_timeline(&app.report_timeline_data);

            let nodes: Vec<(&str, &str, bool)> = vec![
                ("提交举报", submit_time.as_deref().unwrap_or("—"), true),
                ("审核处理", review_time.as_deref().unwrap_or("—"), review_time.is_some()),
                (status_label(status), resolve_time.as_deref().unwrap_or("—"), resolve_time.is_some()),
            ];

            for (i, (title, time, done)) in nodes.iter().enumerate() {
                render_node(ui, title, time, *done, i == nodes.len() - 1);
            }

            if !remark.is_empty() {
                ui.add_space(18.0);
                ui.horizontal(|ui| {
                    ui.add_space(24.0);
                    ui.label(
                        egui::RichText::new("处理备注")
                            .font(FontId::new(13.0, egui::FontFamily::Proportional))
                            .color(colors::text_primary())
                            .strong(),
                    );
                });
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.add_space(24.0);
                    ui.label(
                        egui::RichText::new(&remark)
                            .font(FontId::new(12.0, egui::FontFamily::Proportional))
                            .color(colors::text_secondary()),
                    );
                });
            }
        });

    close
}

fn render_node(ui: &mut egui::Ui, title: &str, time: &str, done: bool, is_last: bool) {
    let color = if done { colors::primary() } else { colors::border() };
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), 56.0),
        egui::Sense::hover(),
    );
    // 圆点
    let dot_center = pos2(rect.left() + 36.0, rect.top() + 12.0);
    ui.painter().circle_filled(dot_center, 6.0, color);
    // 连接线到下一个（除非是最后一个）
    if !is_last {
        ui.painter().line_segment(
            [pos2(dot_center.x, dot_center.y + 8.0), pos2(dot_center.x, rect.bottom())],
            Stroke::new(2.0, colors::border()),
        );
    }
    // 文字
    ui.painter().text(
        pos2(rect.left() + 56.0, rect.top() + 6.0),
        egui::Align2::LEFT_TOP,
        title,
        FontId::new(14.0, egui::FontFamily::Proportional),
        colors::text_primary(),
    );
    ui.painter().text(
        pos2(rect.left() + 56.0, rect.top() + 26.0),
        egui::Align2::LEFT_TOP,
        time,
        FontId::new(11.0, egui::FontFamily::Proportional),
        colors::text_secondary(),
    );
}

fn extract_timeline(v: &Option<serde_json::Value>) -> (Option<String>, Option<String>, Option<String>, i64, String) {
    let obj = match v {
        Some(serde_json::Value::Object(m)) => m,
        _ => return (None, None, None, 0, String::new()),
    };
    let sub = obj.get("submitTime").or_else(|| obj.get("createTime")).and_then(|x| x.as_str()).map(String::from);
    let rev = obj.get("reviewTime").and_then(|x| x.as_str()).map(String::from);
    let res = obj.get("resolveTime").or_else(|| obj.get("handleTime")).and_then(|x| x.as_str()).map(String::from);
    let status = obj.get("status").or_else(|| obj.get("result")).and_then(|x| x.as_i64()).unwrap_or(0);
    let remark = obj.get("remark").and_then(|x| x.as_str()).map(String::from).unwrap_or_default();
    (sub, rev, res, status, remark)
}

fn status_label(status: i64) -> &'static str {
    match status {
        0 => "待审核",
        1 => "已通过 · 处理完成",
        2 => "已下架 · 处理完成",
        3 => "已驳回",
        _ => "已处理",
    }
}

// 保留一个别名让调用方无需 import SpringAnim
#[allow(dead_code)]
pub fn spring_new() -> SpringAnim {
    SpringAnim::new(0.4, 0.8, 0.0)
}
