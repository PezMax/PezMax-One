// PezMax egui Desktop — 高性能 Metro Design 试卷资源管理客户端
// 入口文件：初始化日志、tokio runtime、eframe 窗口

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod app;
mod components;
mod pages;
mod sokuou;
mod theme;

use app::PezMaxApp;
use eframe::NativeOptions;
use std::sync::Arc;

fn main() -> Result<(), eframe::Error> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    // 创建 Tokio 运行时，使 API 层可以使用 tokio::spawn
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let _guard = rt.enter(); // 将运行时设为当前线程的默认运行时

    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../resources/icon.png").as_slice())
        .unwrap_or_default();

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(Arc::new(icon))
            .with_title("PezMax"),
        ..Default::default()
    };

    eframe::run_native(
        "PezMax — 试卷资源管理系统",
        options,
        Box::new(|cc| Ok(Box::new(PezMaxApp::new(cc)))),
    )
}