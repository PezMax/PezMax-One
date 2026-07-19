// 社区功能区
// 三个子标签：用户排行 / 贡献文件 / 举报记录

use crate::app::PezMaxApp;
use crate::theme::colors;
use egui::{CornerRadius, FontId, Vec2};

/// 用户排行榜（Phase 3 对接真实 API）
pub fn render_user_ranking(_app: &mut PezMaxApp, ui: &mut egui::Ui) {
    ui.add_space(16.0);
    ui.label(
        egui::RichText::new("用户排行")
            .font(FontId::new(22.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_PRIMARY),
    );
    ui.add_space(6.0);
    ui.label(
        egui::RichText::new("按贡献度排列的用户列表")
            .font(FontId::new(13.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_SECONDARY),
    );
    ui.add_space(16.0);

    // 模拟排行数据
    let rank_data = [
        (1, "张三", 128, 3_421),
        (2, "李四", 96, 2_108),
        (3, "王五", 74, 1_855),
        (4, "赵六", 51, 1_234),
        (5, "钱七", 42, 987),
    ];

    egui::ScrollArea::vertical()
        .id_salt("ranking_scroll")
        .show(ui, |ui| {
            for (rank, name, uploads, downloads) in &rank_data {
                let medal = match rank {
                    1 => "🥇",
                    2 => "🥈",
                    3 => "🥉",
                    _ => "  ",
                };
                let row_bg = if *rank <= 3 {
                    egui::Color32::from_rgb(0xFF, 0xF8, 0xE8)
                } else {
                    colors::BG_CARD
                };

                egui::Frame::new()
                    .fill(row_bg)
                    .corner_radius(CornerRadius::same(0))
                    .stroke(egui::Stroke::new(1.0, colors::BORDER))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.add_space(16.0);
                            ui.label(
                                egui::RichText::new(medal)
                                    .font(FontId::new(20.0, egui::FontFamily::Proportional)),
                            );
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new(format!("#{}", rank))
                                    .font(FontId::new(13.0, egui::FontFamily::Proportional))
                                    .color(colors::TEXT_SECONDARY),
                            );
                            ui.add_space(16.0);
                            ui.label(
                                egui::RichText::new(*name)
                                    .font(FontId::new(15.0, egui::FontFamily::Proportional))
                                    .color(colors::TEXT_PRIMARY),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.add_space(16.0);
                                    ui.label(
                                        egui::RichText::new(format!("📥 {}", downloads))
                                            .font(FontId::new(13.0, egui::FontFamily::Proportional))
                                            .color(colors::TEXT_SECONDARY),
                                    );
                                    ui.add_space(16.0);
                                    ui.label(
                                        egui::RichText::new(format!("📤 {} 份", uploads))
                                            .font(FontId::new(13.0, egui::FontFamily::Proportional))
                                            .color(colors::TEXT_SECONDARY),
                                    );
                                },
                            );
                        });
                        ui.add_space(6.0);
                    });
                ui.add_space(4.0);
            }
        });
}

/// 贡献文件（上传入口 + 元数据表单）
pub fn render_contribute_file(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    ui.add_space(16.0);
    ui.label(
        egui::RichText::new("贡献文件")
            .font(FontId::new(22.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_PRIMARY),
    );
    ui.add_space(6.0);
    ui.label(
        egui::RichText::new("上传试卷资源，帮助更多同学")
            .font(FontId::new(13.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_SECONDARY),
    );
    ui.add_space(20.0);

    egui::ScrollArea::vertical()
        .id_salt("contribute_scroll")
        .show(ui, |ui| {
            // ── 文件拖放区域 ──────────────────────────────────────
            egui::Frame::new()
                .fill(colors::BG_CARD)
                .corner_radius(CornerRadius::same(0))
                .stroke(egui::Stroke::new(2.0, colors::BORDER))
                .show(ui, |ui| {
                    ui.set_min_height(140.0);
                    ui.set_min_width(ui.available_width());
                    ui.vertical_centered(|ui| {
                        ui.add_space(28.0);
                        ui.label(
                            egui::RichText::new("📤")
                                .font(FontId::new(40.0, egui::FontFamily::Proportional)),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new("点击选择文件或拖放至此处")
                                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                .color(colors::TEXT_SECONDARY),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new("支持 PDF · 最大 50MB")
                                .font(FontId::new(12.0, egui::FontFamily::Proportional))
                                .color(colors::TEXT_SECONDARY),
                        );
                        ui.add_space(10.0);
                        let btn = egui::Button::new(
                            egui::RichText::new("  选择文件  ")
                                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                .color(colors::TEXT_ON_PRIMARY),
                        )
                        .fill(colors::PRIMARY)
                        .corner_radius(CornerRadius::same(0));
                        if ui.add(btn).clicked() {}
                        ui.add_space(20.0);
                    });
                });

            ui.add_space(20.0);

            // ── 元数据表单 ────────────────────────────────────────
            ui.label(
                egui::RichText::new("文件信息")
                    .font(FontId::new(16.0, egui::FontFamily::Proportional))
                    .color(colors::TEXT_PRIMARY),
            );
            ui.add_space(8.0);

            egui::Frame::new()
                .fill(colors::BG_CARD)
                .corner_radius(CornerRadius::same(0))
                .stroke(egui::Stroke::new(1.0, colors::BORDER))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.add_space(16.0);

                    contribute_field(ui, "学科", &mut app.contribute_subject, "如：数学");
                    ui.add_space(10.0);
                    contribute_field(ui, "学校", &mut app.contribute_school, "如：全国卷");
                    ui.add_space(10.0);
                    contribute_field(ui, "年份", &mut app.contribute_year, "如：2024");

                    ui.add_space(16.0);

                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        let can_submit = !app.contribute_subject.is_empty()
                            && !app.contribute_year.is_empty();
                        let btn = egui::Button::new(
                            egui::RichText::new("  提交上传  ")
                                .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                .color(colors::TEXT_ON_PRIMARY),
                        )
                        .fill(if can_submit {
                            colors::ACCENT_GREEN
                        } else {
                            colors::BG_HOVER
                        })
                        .corner_radius(CornerRadius::same(0));

                        if ui.add_enabled(can_submit, btn).clicked() {
                            // Phase 4: call API upload
                            app.contribute_subject.clear();
                            app.contribute_school.clear();
                            app.contribute_year.clear();
                        }
                    });
                    ui.add_space(16.0);
                });

            ui.add_space(20.0);

            // ── 上传历史 ──────────────────────────────────────────
            ui.label(
                egui::RichText::new("上传历史")
                    .font(FontId::new(16.0, egui::FontFamily::Proportional))
                    .color(colors::TEXT_PRIMARY),
            );
            ui.add_space(8.0);

            egui::Frame::new()
                .fill(colors::BG_CARD)
                .corner_radius(CornerRadius::same(0))
                .stroke(egui::Stroke::new(1.0, colors::BORDER))
                .show(ui, |ui| {
                    ui.set_min_height(80.0);
                    ui.set_min_width(ui.available_width());
                    ui.vertical_centered(|ui| {
                        ui.add_space(24.0);
                        ui.label(
                            egui::RichText::new("暂无上传记录")
                                .font(FontId::new(13.0, egui::FontFamily::Proportional))
                                .color(colors::TEXT_SECONDARY),
                        );
                        ui.add_space(24.0);
                    });
                });
        });
}

fn contribute_field(ui: &mut egui::Ui, label: &str, value: &mut String, hint: &str) {
    ui.horizontal(|ui| {
        ui.add_space(16.0);
        ui.add_sized(
            Vec2::new(60.0, 20.0),
            egui::Label::new(
                egui::RichText::new(label)
                    .font(FontId::new(14.0, egui::FontFamily::Proportional))
                    .color(colors::TEXT_SECONDARY),
            ),
        );
        ui.add_space(8.0);
        let edit = egui::TextEdit::singleline(value)
            .hint_text(hint)
            .desired_width(200.0)
            .font(FontId::new(14.0, egui::FontFamily::Proportional));
        ui.add(edit);
    });
}

/// 举报记录
pub fn render_report_record(_app: &mut PezMaxApp, ui: &mut egui::Ui) {
    ui.add_space(16.0);
    ui.label(
        egui::RichText::new("举报记录")
            .font(FontId::new(22.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_PRIMARY),
    );
    ui.add_space(6.0);
    ui.label(
        egui::RichText::new("举报违规内容，维护社区环境")
            .font(FontId::new(13.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_SECONDARY),
    );
    ui.add_space(20.0);

    // 快速举报按钮
    let btn = egui::Button::new(
        egui::RichText::new("  🚩 提交新举报  ")
            .font(FontId::new(14.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_ON_PRIMARY),
    )
    .fill(colors::ERROR)
    .corner_radius(CornerRadius::same(0));
    if ui.add(btn).clicked() {}

    ui.add_space(20.0);
    ui.label(
        egui::RichText::new("我的举报")
            .font(FontId::new(16.0, egui::FontFamily::Proportional))
            .color(colors::TEXT_PRIMARY),
    );
    ui.add_space(8.0);

    egui::Frame::new()
        .fill(colors::BG_CARD)
        .corner_radius(CornerRadius::same(0))
        .stroke(egui::Stroke::new(1.0, colors::BORDER))
        .show(ui, |ui| {
            ui.set_min_height(100.0);
            ui.set_min_width(ui.available_width());
            ui.vertical_centered(|ui| {
                ui.add_space(32.0);
                ui.label(
                    egui::RichText::new("暂无举报记录")
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(colors::TEXT_SECONDARY),
                );
                ui.add_space(32.0);
            });
        });
}
