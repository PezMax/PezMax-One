// PezMax egui Desktop — 高性能 Metro Design 试卷资源管理客户端
// 入口文件：初始化日志、tokio runtime、eframe 窗口

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod app;
mod app_menu;
mod cache;
mod components;
mod db;
mod pages;
mod pdf;
mod settings;
mod sokuou;
mod theme;
mod updater;

use app::PezMaxApp;
use eframe::NativeOptions;
use pdf::PdfEngine;
use std::sync::Arc;

fn main() -> Result<(), eframe::Error> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    // 创建 Tokio 运行时，使 API 层可以使用 tokio::spawn
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let _guard = rt.enter(); // 将运行时设为当前线程的默认运行时

    // 初始化 PDF 引擎
    let pdf_engine = Arc::new(PdfEngine::new());
    if !pdf_engine.is_available() {
        log::warn!("PDF engine unavailable: {:?}", pdf_engine.error());
    }

    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../resources/icon.png").as_slice())
        .unwrap_or_default();

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(Arc::new(icon))
            // Wayland compositor 用 app_id 匹配 .desktop 文件；不设置则任务栏无图标。
            // 该值同时会被 .desktop 里的 StartupWMClass 引用。
            .with_app_id("io.github.pezmax.one")
            .with_title("PezMax One · 拼图满绩·绫"),
        ..Default::default()
    };

    eframe::run_native(
        "PezMax One · 拼图满绩·绫",
        options,
        Box::new(|cc| {
            // 安装平台菜单（macOS NSMenu / Linux Plasma Global Menu）
            let (menu_rx, menu_backend) = app_menu::install(cc);
            Ok(Box::new(PezMaxApp::new(cc, pdf_engine, menu_rx, menu_backend)))
        }),
    )
}