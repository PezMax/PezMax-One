//! PDF 渲染引擎：基于 pdfium-render（Chrome 同款引擎）
//!
//! 架构：
//! - `PdfEngine`：全局单例，持有 Arc<Pdfium>（Send + Sync）
//! - `PdfViewer`：每个 PDF 文档的查看状态（页码、缩放、纹理缓存）
//! - 渲染在后台线程完成：传入 PDF 字节 + 页码，加载文档→渲染页面→返回纹理
//!   （避免 PdfDocument 的 lifetime 问题，因为文档在后台线程内创建和销毁）

use crate::sokuou::{map_range, SpringAnim};
use egui::Context;
use egui::{ColorImage, TextureHandle, TextureOptions, Vec2};
use pdfium_render::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::oneshot;

// ── 常量 ──────────────────────────────────────────────────────────────────

/// 渲染 DPI 倍率（pdfium 默认 72 DPI，我们提高到 200 DPI）
const RENDER_SCALE: f32 = 200.0 / 72.0;

/// 纹理缓存最大页数
const MAX_CACHED_PAGES: usize = 10;

/// 缩放步进
const ZOOM_STEP: f32 = 0.25;
const MIN_SCALE: f32 = 0.5;
const MAX_SCALE: f32 = 4.0;

// ── PdfEngine ─────────────────────────────────────────────────────────────

/// 全局 PDF 引擎，持有 Arc<Pdfium>（Send + Sync，可跨线程）
pub struct PdfEngine {
    inner: Option<Arc<Pdfium>>,
    error: Option<String>,
}

impl PdfEngine {
    /// 初始化 Pdfium 引擎。
    ///
    /// 查找顺序：
    /// 1. 可执行文件同目录下的 pdfium.dll / libpdfium.so / libpdfium.dylib
    /// 2. 系统已安装的 pdfium 库
    pub fn new() -> Self {
        let library_path = Pdfium::pdfium_platform_library_name_at_path("./");
        match Pdfium::bind_to_library(&library_path) {
            Ok(bindings) => {
                log::info!("PDF engine: bound to library at {:?}", library_path);
                Self {
                    inner: Some(Arc::new(Pdfium::new(bindings))),
                    error: None,
                }
            }
            Err(e1) => {
                match Pdfium::bind_to_system_library() {
                    Ok(bindings) => {
                        log::info!("PDF engine: bound to system library");
                        Self {
                            inner: Some(Arc::new(Pdfium::new(bindings))),
                            error: None,
                        }
                    }
                    Err(e2) => {
                        let msg = format!(
                            "PDF 引擎初始化失败：\n1. 本地: {}\n2. 系统: {}\n\n\
                             请将 pdfium.dll 放在程序目录下。\n\
                             下载地址：https://github.com/bblanchon/pdfium-binaries/releases",
                            e1, e2
                        );
                        log::warn!("{}", msg);
                        Self { inner: None, error: Some(msg) }
                    }
                }
            }
        }
    }

    pub fn is_available(&self) -> bool {
        self.inner.is_some()
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// 后台渲染一页 PDF：传入 PDF 字节 + 页码 + 缩放，返回 ColorImage
    /// 文档在后台线程内创建和销毁，无 lifetime 问题
    pub fn render_page(
        self: &Arc<Self>,
        pdf_bytes: Vec<u8>,
        page_idx: usize,
        scale: f32,
    ) -> Result<ColorImage, String> {
        let pdfium = self.inner.as_ref().ok_or("PDF 引擎未初始化")?;
        let doc = pdfium
            .load_pdf_from_byte_vec(pdf_bytes, None)
            .map_err(|e| format!("加载 PDF 失败: {}", e))?;

        let pages = doc.pages();
        let count = pages.len() as usize;
        if page_idx >= count {
            return Err(format!("页码越界: {} >= {}", page_idx, count));
        }
        let page = pages.get(page_idx as u16).map_err(|e| format!("获取第 {} 页失败: {}", page_idx, e))?;

        let w = (page.width().value * scale) as usize;
        let h = (page.height().value * scale) as usize;

        let bitmap = page
            .render_with_config(
                &PdfRenderConfig::new()
                    .set_target_width(w as i32)
                    .set_target_height(h as i32),
            )
            .map_err(|e| format!("渲染第 {} 页失败: {}", page_idx, e))?;

        let pixels = bitmap.as_bytes().to_vec();
        Ok(ColorImage::from_rgba_unmultiplied([w, h], &pixels))
    }

    /// 获取 PDF 页数（在后台线程执行）
    pub fn get_page_count(self: &Arc<Self>, pdf_bytes: Vec<u8>) -> Result<usize, String> {
        let pdfium = self.inner.as_ref().ok_or("PDF 引擎未初始化")?;
        let doc = pdfium
            .load_pdf_from_byte_vec(pdf_bytes, None)
            .map_err(|e| format!("加载 PDF 失败: {}", e))?;
        Ok(doc.pages().len() as usize)
    }

    /// 获取第一页尺寸（用于初始缩放适配）
    pub fn get_first_page_size(self: &Arc<Self>, pdf_bytes: Vec<u8>) -> Result<(f32, f32), String> {
        let pdfium = self.inner.as_ref().ok_or("PDF 引擎未初始化")?;
        let doc = pdfium
            .load_pdf_from_byte_vec(pdf_bytes, None)
            .map_err(|e| format!("加载 PDF 失败: {}", e))?;
        let page = doc.pages().get(0).map_err(|e| format!("获取首页失败: {}", e))?;
        Ok((page.width().value, page.height().value))
    }
}

// ── PdfViewer ─────────────────────────────────────────────────────────────

/// 异步渲染任务
struct RenderTask {
    rx: oneshot::Receiver<(usize, ColorImage)>,
    page_idx: usize,
}

/// PDF 查看器状态
pub struct PdfViewer {
    /// PDF 原始字节（用于后台渲染）
    pdf_bytes: Option<Vec<u8>>,

    // 文档元数据
    page_count: usize,
    page_width: f32,
    page_height: f32,

    // 视图状态
    pub current_page: usize,
    pub scale: f32,

    // 动画
    pub page_enter_anim: SpringAnim,
    pub page_exit_anim: SpringAnim,
    is_animating_out: bool,

    // 纹理缓存
    textures: HashMap<usize, TextureHandle>,

    // 异步渲染
    pending_render: Option<RenderTask>,

    // 文档是否已加载
    pub loaded: bool,
    pub error: Option<String>,
}

impl PdfViewer {
    pub fn new() -> Self {
        Self {
            pdf_bytes: None,
            page_count: 0,
            page_width: 595.0,
            page_height: 842.0,
            current_page: 0,
            scale: 1.0,
            page_enter_anim: SpringAnim::new(0.3, 0.8, 0.0),
            page_exit_anim: SpringAnim::new(0.2, 0.8, 0.0),
            is_animating_out: false,
            textures: HashMap::new(),
            pending_render: None,
            loaded: false,
            error: None,
        }
    }

    /// 加载文档
    pub fn load_document(
        &mut self,
        engine: &Arc<PdfEngine>,
        pdf_bytes: Vec<u8>,
        ctx: &Context,
    ) {
        self.pdf_bytes = Some(pdf_bytes);
        self.loaded = false;
        self.error = None;
        self.textures.clear();
        self.pending_render = None;
        self.current_page = 0;
        self.scale = 1.0;

        // 同步获取文档元数据（pdfium 加载很快，只是渲染慢）
        let bytes = self.pdf_bytes.as_ref().unwrap().clone();
        match engine.get_page_count(bytes.clone()) {
            Ok(count) => {
                self.page_count = count;
                if let Ok((w, h)) = engine.get_first_page_size(bytes) {
                    self.page_width = w;
                    self.page_height = h;
                }
                self.loaded = true;
            }
            Err(e) => {
                self.error = Some(e);
                self.loaded = true;
            }
        }

        // 请求渲染第一页
        self.request_render(&engine, 0, ctx);
    }

    /// 请求渲染第 `page_idx` 页（后台线程）
    fn request_render(&mut self, engine: &Arc<PdfEngine>, page_idx: usize, _ctx: &Context) {
        if page_idx >= self.page_count {
            return;
        }
        if self.textures.contains_key(&page_idx) {
            return;
        }
        // 忽略重复请求
        if let Some(ref task) = self.pending_render {
            if task.page_idx == page_idx {
                return;
            }
        }

        let bytes = match &self.pdf_bytes {
            Some(b) => b.clone(),
            None => return,
        };
        let engine = engine.clone();
        let scale = RENDER_SCALE * self.scale;
        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            let result = engine.render_page(bytes, page_idx, scale);
            match result {
                Ok(img) => { tx.send((page_idx, img)).ok(); }
                Err(e) => {
                    log::error!("PDF render page {}: {}", page_idx, e);
                    // 发送一个空白占位图
                    let placeholder = ColorImage::new([1, 1], egui::Color32::RED);
                    tx.send((page_idx, placeholder)).ok();
                }
            }
        });

        self.pending_render = Some(RenderTask { rx, page_idx });
    }

    /// 每帧轮询渲染结果
    pub fn poll_render(&mut self, _engine: &Arc<PdfEngine>, ctx: &Context) {
        if let Some(task) = &mut self.pending_render {
            if let Ok((idx, img)) = task.rx.try_recv() {
                let tex = ctx.load_texture(
                    &format!("pdf_page_{}", idx),
                    img,
                    TextureOptions::LINEAR,
                );
                self.textures.insert(idx, tex);
                self.pending_render = None;
                ctx.request_repaint();
            }
        }
    }

    /// 翻页
    pub fn go_to_page(&mut self, page: usize, engine: &Arc<PdfEngine>, ctx: &Context) {
        if page >= self.page_count || page == self.current_page {
            return;
        }
        self.is_animating_out = true;
        self.page_exit_anim = SpringAnim::with_target(0.2, 0.8, 0.0, 0.0, 1.0);
        self.current_page = page;
        self.page_enter_anim = SpringAnim::with_target(0.3, 0.8, 0.0, 0.0, 1.0);

        if !self.textures.contains_key(&page) {
            self.request_render(engine, page, ctx);
        }
        ctx.request_repaint();
    }

    pub fn next_page(&mut self, engine: &Arc<PdfEngine>, ctx: &Context) {
        if self.current_page + 1 < self.page_count {
            self.go_to_page(self.current_page + 1, engine, ctx);
        }
    }

    pub fn prev_page(&mut self, engine: &Arc<PdfEngine>, ctx: &Context) {
        if self.current_page > 0 {
            self.go_to_page(self.current_page - 1, engine, ctx);
        }
    }

    /// 缩放
    pub fn zoom_in(&mut self, engine: &Arc<PdfEngine>, ctx: &Context) {
        self.scale = (self.scale + ZOOM_STEP).min(MAX_SCALE);
        self.textures.clear();
        self.request_render(engine, self.current_page, ctx);
    }

    pub fn zoom_out(&mut self, engine: &Arc<PdfEngine>, ctx: &Context) {
        self.scale = (self.scale - ZOOM_STEP).max(MIN_SCALE);
        self.textures.clear();
        self.request_render(engine, self.current_page, ctx);
    }

    pub fn zoom_to_fit(&mut self, available_width: f32, engine: &Arc<PdfEngine>, ctx: &Context) {
        if self.page_width > 0.0 {
            let new_scale = (available_width / self.page_width).max(0.5).min(2.0);
            if (new_scale - self.scale).abs() > 0.01 {
                self.scale = new_scale;
                self.textures.clear();
                self.request_render(engine, self.current_page, ctx);
            }
        }
    }

    pub fn current_texture(&self) -> Option<&TextureHandle> {
        self.textures.get(&self.current_page)
    }

    pub fn is_loading(&self) -> bool {
        self.pending_render.is_some()
    }

    pub fn current_page_size(&self) -> Vec2 {
        Vec2::new(self.page_width * self.scale, self.page_height * self.scale)
    }

    pub fn update_animations(&mut self, dt: f64) {
        self.page_enter_anim.update(dt);
        self.page_exit_anim.update(dt);
    }

    pub fn is_animating(&self) -> bool {
        !self.page_enter_anim.is_steady() || !self.page_exit_anim.is_steady()
    }

    pub fn page_alpha(&self) -> f32 {
        if self.is_animating_out {
            map_range(self.page_exit_anim.value(), 1.0, 0.0) as f32
        } else {
            map_range(self.page_enter_anim.value(), 0.0, 1.0) as f32
        }
    }

    pub fn page_offset_y(&self) -> f32 {
        if self.is_animating_out {
            map_range(self.page_exit_anim.value(), 0.0, -20.0) as f32
        } else {
            map_range(self.page_enter_anim.value(), 20.0, 0.0) as f32
        }
    }

    pub fn page_count(&self) -> usize {
        self.page_count
    }
}

// ── 渲染辅助函数 ──────────────────────────────────────────────────────────

/// 渲染 PDF 未就绪时的占位 UI
pub fn render_pdf_unavailable(ui: &mut egui::Ui, error: Option<&str>) {
    ui.vertical_centered(|ui| {
        ui.add_space(120.0);
        ui.label(
            egui::RichText::new("📄")
                .font(egui::FontId::new(48.0, egui::FontFamily::Proportional)),
        );
        ui.add_space(12.0);
        if let Some(msg) = error {
            ui.label(
                egui::RichText::new(msg)
                    .font(egui::FontId::new(13.0, egui::FontFamily::Proportional))
                    .color(egui::Color32::from_rgb(200, 80, 80)),
            );
        } else {
            ui.label(
                egui::RichText::new("PDF 预览")
                    .font(egui::FontId::new(15.0, egui::FontFamily::Proportional))
                    .color(egui::Color32::from_gray(140)),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("下载后可用系统阅读器打开")
                    .font(egui::FontId::new(12.0, egui::FontFamily::Proportional))
                    .color(egui::Color32::from_gray(140)),
            );
        }
    });
}

/// 带翻页控件和缩放控件的 PDF 查看器渲染
pub fn render_pdf_viewer(
    ui: &mut egui::Ui,
    viewer: &mut PdfViewer,
    engine: &Arc<PdfEngine>,
) {
    if !engine.is_available() {
        render_pdf_unavailable(ui, engine.error());
        return;
    }

    if !viewer.loaded {
        render_pdf_unavailable(ui, viewer.error.as_deref());
        return;
    }

    let page_count = viewer.page_count();
    let page_idx = viewer.current_page;

    // ── 顶部工具栏：翻页 + 缩放 ────────────────────────────
    ui.horizontal(|ui| {
        ui.add_space(8.0);

        let prev_enabled = page_idx > 0;
        let prev = egui::Button::new("◀")
            .min_size(Vec2::new(28.0, 28.0));
        if ui.add_enabled(prev_enabled, prev).clicked() {
            viewer.prev_page(engine, ui.ctx());
        }

        ui.add_space(4.0);
        if ui.button("▶").clicked() {
            viewer.next_page(engine, ui.ctx());
        }

        ui.add_space(8.0);
        ui.label(format!("第 {} / {} 页", page_idx + 1, page_count));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            if ui.button("➕").clicked() {
                viewer.zoom_in(engine, ui.ctx());
            }
            ui.add_space(2.0);
            if ui.button("➖").clicked() {
                viewer.zoom_out(engine, ui.ctx());
            }
            ui.add_space(2.0);
            if ui.button("适").clicked() {
                viewer.zoom_to_fit(ui.available_width() - 32.0, engine, ui.ctx());
            }
            ui.add_space(4.0);
            ui.label(format!("{:.0}%", viewer.scale * 100.0));
        });
    });

    ui.add_space(4.0);
    ui.separator();
    ui.add_space(4.0);

    // ── 页面渲染区 ──────────────────────────────────────────
    let _alpha = viewer.page_alpha();
    let y_offset = viewer.page_offset_y();

    egui::ScrollArea::vertical()
        .id_salt("pdf_scroll")
        .show(ui, |ui| {
            if y_offset > 0.5 {
                ui.add_space(y_offset);
            }

            if let Some(tex) = viewer.current_texture() {
                let size = viewer.current_page_size();
                let img_size = egui::vec2(size.x, size.y);

                ui.vertical_centered(|ui| {
                    egui::Frame::new()
                        .fill(egui::Color32::from_gray(245))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(200)))
                        .show(ui, |ui| {
                            ui.set_min_size(img_size);
                            ui.set_max_size(img_size);
                            ui.image((tex.id(), img_size));
                        });
                });
            } else if viewer.is_loading() {
                ui.vertical_centered(|ui| {
                    ui.add_space(160.0);
                    ui.label("渲染中...");
                });
                ui.ctx().request_repaint();
            } else {
                // 首次请求渲染
                viewer.request_render(engine, page_idx, ui.ctx());
            }
        });
}