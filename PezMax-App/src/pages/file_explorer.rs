// 试卷浏览页面
// 文件树 + 列表视图，类似资源管理器

use crate::app::{Page, PezMaxApp};
use crate::theme::colors;
use egui::{FontId, Rounding};

pub fn render(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    ui.add_space(16.0);

    // 当前路径指示
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("全部试卷")
                .font(FontId::new(18.0, egui::FontFamily::Proportional))
                .color(colors::TEXT_PRIMARY),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("🔄 刷新").clicked() {
                // 刷新列表
            }
        });
    });

    ui.add_space(12.0);

    // 分割布局：左侧文件树 + 右侧文件列表
    egui::SidePanel::left("file_tree")
        .resizable(true)
        .default_width(220.0)
        .min_width(160.0)
        .frame(egui::Frame::none().fill(colors::BG_CARD))
        .show_inside(ui, |ui| {
            ui.add_space(12.0);
            ui.label(
                egui::RichText::new("📂 分类")
                    .font(FontId::new(14.0, egui::FontFamily::Proportional))
                    .color(colors::TEXT_SECONDARY),
            );
            ui.add_space(8.0);

            // 树形分类
            let categories = [
                ("全部", "📁"),
                ("数学", "📐"),
                ("语文", "📝"),
                ("英语", "🔤"),
                ("物理", "⚛"),
                ("化学", "🧪"),
                ("生物", "🧬"),
                ("历史", "📜"),
                ("地理", "🌍"),
            ];

            for (name, icon) in &categories {
                let response = egui::Frame::none()
                    .rounding(Rounding::same(6.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new(*icon)
                                    .font(FontId::new(16.0, egui::FontFamily::Proportional)),
                            );
                            ui.add_space(6.0);
                            ui.label(
                                egui::RichText::new(*name)
                                    .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                    .color(colors::TEXT_PRIMARY),
                            );
                        });
                    })
                    .response
                    .interact(egui::Sense::click());

                if response.clicked() {
                    // 按分类筛选
                }
                ui.add_space(2.0);
            }
        });

    // 右侧文件列表
    egui::Frame::none()
        .fill(colors::BG_WHITE)
        .show(ui, |ui| {
            ui.add_space(8.0);

            // 文件卡片网格
            ui.horizontal_wrapped(|ui| {
                let sample_files = [
                    ("2024高考数学真题.pdf", "数学", "2024", "2.3MB", "张三"),
                    ("2024高考语文真题.pdf", "语文", "2024", "1.8MB", "李四"),
                    ("2024高考英语真题.pdf", "英语", "2024", "1.5MB", "王五"),
                    ("2023高考物理真题.pdf", "物理", "2023", "2.1MB", "赵六"),
                    ("2023高考化学真题.pdf", "化学", "2023", "1.9MB", "钱七"),
                    ("2024模拟试卷-数学.pdf", "数学", "2024", "3.2MB", "孙八"),
                ];

                for (name, subject, year, size, uploader) in &sample_files {
                    let response = egui::Frame::none()
                        .fill(colors::BG_CARD)
                        .rounding(Rounding::same(10.0))
                        .stroke(egui::Stroke::new(1.0, colors::BORDER))
                        .show(ui, |ui| {
                            ui.set_min_size(egui::Vec2::new(240.0, 140.0));
                            ui.add_space(12.0);
                            ui.horizontal(|ui| {
                                ui.add_space(12.0);
                                // 文件图标
                                egui::Frame::none()
                                    .fill(colors::PRIMARY_LIGHT)
                                    .rounding(Rounding::same(8.0))
                                    .show(ui, |ui| {
                                        ui.allocate_space(egui::Vec2::new(40.0, 48.0));
                                        ui.allocate_ui_at_rect(ui.max_rect(), |ui| {
                                            ui.vertical_centered(|ui| {
                                                ui.label(
                                                    egui::RichText::new("📄")
                                                        .font(FontId::new(24.0, egui::FontFamily::Proportional)),
                                                );
                                            });
                                        });
                                    });
                                ui.add_space(10.0);
                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new(*name)
                                            .font(FontId::new(14.0, egui::FontFamily::Proportional))
                                            .color(colors::TEXT_PRIMARY),
                                    );
                                    ui.add_space(4.0);
                                    ui.label(
                                        egui::RichText::new(format!("{} | {} | {}", subject, year, size))
                                            .font(FontId::new(12.0, egui::FontFamily::Proportional))
                                            .color(colors::TEXT_SECONDARY),
                                    );
                                    ui.label(
                                        egui::RichText::new(format!("上传者: {}", uploader))
                                            .font(FontId::new(11.0, egui::FontFamily::Proportional))
                                            .color(colors::TEXT_SECONDARY),
                                    );
                                });
                            });
                            ui.add_space(8.0);
                            // 操作按钮
                            ui.horizontal(|ui| {
                                ui.add_space(12.0);
                                let dl_btn = egui::Button::new(
                                    egui::RichText::new("📥 下载")
                                        .font(FontId::new(12.0, egui::FontFamily::Proportional)),
                                )
                                .fill(colors::PRIMARY)
                                .rounding(Rounding::same(4.0));
                                if ui.add(dl_btn).clicked() {
                                    // 下载
                                }
                                ui.add_space(4.0);
                                let fav_btn = egui::Button::new(
                                    egui::RichText::new("⭐ 收藏")
                                        .font(FontId::new(12.0, egui::FontFamily::Proportional)),
                                )
                                .fill(colors::BG_CARD)
                                .rounding(Rounding::same(4.0));
                                if ui.add(fav_btn).clicked() {
                                    // 收藏
                                }
                            });
                        })
                        .response
                        .interact(egui::Sense::click());

                    if response.clicked() {
                        // 打开文件详情
                    }
                    ui.add_space(10.0);
                }
            });
        });
}