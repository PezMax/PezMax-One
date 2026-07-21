//! PDF 渲染引擎：基于 pdfium-render（Chrome 同款引擎）
//!
//! 架构：
//! - `PdfEngine`：全局单例，持有 Arc<Pdfium>（Send + Sync）
//! - `PdfViewer`：每个 PDF 文档的查看状态（页码、缩放、纹理缓存、视图模式）
//! - 渲染在后台线程完成：传入 PDF 字节 + 页码，加载文档→渲染页面→返回纹理
//!
//! 视图模式：Grid（缩略图网格预览）/ Line（连续纵向滚动 + 左侧总览面板）

use crate::sokuou::SpringAnim;
use egui::Context;
use egui::{ColorImage, TextureHandle, TextureOptions, Vec2};
use pdfium_render::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::oneshot;

// ── 常量 ──────────────────────────────────────────────────────────────────

/// 渲染 DPI 倍率（pdfium 默认 72 DPI，我们提高到 200 DPI）
const RENDER_SCALE: f32 = 200.0 / 72.0;

/// 缩放步进
const ZOOM_STEP: f32 = 0.25;
const MIN_SCALE: f32 = 0.5;
const MAX_SCALE: f32 = 4.0;

/// 缩略图宽度（像素）
const THUMB_WIDTH: f32 = 150.0;

/// 总览面板缩略图宽度
const OVERVIEW_THUMB_WIDTH: f32 = 120.0;

/// 总览面板宽度
const OVERVIEW_PANEL_WIDTH: f32 = 150.0;

/// 最大并发渲染数（避免 pdfium 同时加载过多文档导致崩溃）
const MAX_CONCURRENT_RENDERS: usize = 3;

/// 视图模式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    /// 缩略图网格预览
    Grid,
    /// 连续纵向滚动（左侧可收起总览面板）
    Line,
}

impl ViewMode {
    pub fn label(self) -> &'static str {
        match self {
            ViewMode::Grid => "平摊",
            ViewMode::Line => "滚动",
        }
    }

    pub fn all() -> &'static [ViewMode] {
        &[ViewMode::Grid, ViewMode::Line]
    }
}

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
            Err(e1) => match Pdfium::bind_to_system_library() {
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
                    Self {
                        inner: None,
                        error: Some(msg),
                    }
                }
            },
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
        let page = pages
            .get(page_idx as u16)
            .map_err(|e| format!("获取第 {} 页失败: {}", page_idx, e))?;

        let w = (page.width().value * scale) as usize;
        let h = (page.height().value * scale) as usize;

        let bitmap = page
            .render_with_config(
                &PdfRenderConfig::new()
                    .set_target_width(w as i32)
                    .set_target_height(h as i32),
            )
            .map_err(|e| format!("渲染第 {} 页失败: {}", page_idx, e))?;

        let pixels = bitmap.as_raw_bytes().to_vec();
        Ok(ColorImage::from_rgba_unmultiplied([w, h], &pixels))
    }

    /// 渲染缩略图（低分辨率，快速）
    pub fn render_thumbnail(
        self: &Arc<Self>,
        pdf_bytes: Vec<u8>,
        page_idx: usize,
        thumb_width: f32,
        page_width: f32,
    ) -> Result<ColorImage, String> {
        let thumb_scale = thumb_width / page_width;
        self.render_page(pdf_bytes, page_idx, thumb_scale)
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
    pub fn get_first_page_size(
        self: &Arc<Self>,
        pdf_bytes: Vec<u8>,
    ) -> Result<(f32, f32), String> {
        let pdfium = self.inner.as_ref().ok_or("PDF 引擎未初始化")?;
        let doc = pdfium
            .load_pdf_from_byte_vec(pdf_bytes, None)
            .map_err(|e| format!("加载 PDF 失败: {}", e))?;
        let page = doc
            .pages()
            .get(0)
            .map_err(|e| format!("获取首页失败: {}", e))?;
        Ok((page.width().value, page.height().value))
    }
}

// ── PdfViewer ─────────────────────────────────────────────────────────────

/// 异步渲染任务（全分辨率）
struct RenderTask {
    rx: oneshot::Receiver<(usize, ColorImage)>,
    page_idx: usize,
}

/// 异步缩略图渲染任务
struct ThumbnailTask {
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

    /// 上次渲染时使用的缩放值（用于检测是否需要重新渲染）
    rendered_scale: f32,

    // 缩放过渡动画
    pub display_scale_anim: SpringAnim,

    // 全分辨率纹理缓存（所有页面）
    textures: HashMap<usize, TextureHandle>,

    // 缩略图纹理缓存
    thumbnails: HashMap<usize, TextureHandle>,

    // 缩略图渲染缩放倍率
    thumbnail_scale: f32,

    // 异步渲染队列
    pending_renders: Vec<RenderTask>,
    pending_thumbnails: Vec<ThumbnailTask>,

    // 文档是否已加载
    pub loaded: bool,
    pub error: Option<String>,

    // 视图模式
    pub view_mode: ViewMode,

    // 跳转页面信号
    pub scroll_to_page: Option<usize>,

    // 总览面板折叠状态
    pub overview_open: bool,

    // 所有页面是否已渲染完成（未完成时显示加载进度）
    all_rendered: bool,

    /// 缩放后是否正在重新渲染（保留旧纹理直到新纹理就绪）
    re_render_in_progress: bool,

    /// 下一个要开始渲染的页面索引（用于并发控制）
    next_render_idx: usize,

    /// 平摊模式当前列数（缩放唯一依据）
    pub grid_cols: usize,

    /// 平摊模式页面宽度动画（SpringAnim 平滑过渡）
    pub grid_size_anim: SpringAnim,
}

impl PdfViewer {
    pub fn new() -> Self {
        let mut display_scale_anim = SpringAnim::new(0.4, 0.8, 1.0);
        let _ = display_scale_anim.update(100.0);
        Self {
            pdf_bytes: None,
            page_count: 0,
            page_width: 595.0,
            page_height: 842.0,
            current_page: 0,
            scale: 1.0,
            rendered_scale: 1.0,
            display_scale_anim,
            textures: HashMap::new(),
            thumbnails: HashMap::new(),
            thumbnail_scale: 0.25,
            pending_renders: Vec::new(),
            pending_thumbnails: Vec::new(),
            loaded: false,
            error: None,
            view_mode: ViewMode::Line,
            scroll_to_page: None,
            overview_open: true,
            all_rendered: false,
            re_render_in_progress: false,
            next_render_idx: 0,
            grid_cols: 2,
            grid_size_anim: { let mut a = SpringAnim::new(0.4, 0.8, 0.0); let _ = a.update(100.0); a },
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
        self.thumbnails.clear();
        self.pending_renders.clear();
        self.pending_thumbnails.clear();
        self.current_page = 0;
        self.scale = 1.0;
        self.rendered_scale = 1.0;
        self.scroll_to_page = None;
        self.all_rendered = false;
        self.re_render_in_progress = false;
        self.next_render_idx = 0;
        self.grid_cols = 2.min(self.page_count.max(1));
        self.grid_size_anim = { let mut a = SpringAnim::new(0.4, 0.8, 0.0); let _ = a.update(100.0); a };
        self.display_scale_anim = SpringAnim::new(0.4, 0.8, 1.0);
        let _ = self.display_scale_anim.update(100.0);

        // 同步获取文档元数据
        let bytes = self.pdf_bytes.as_ref().unwrap().clone();
        match engine.get_page_count(bytes.clone()) {
            Ok(count) => {
                self.page_count = count;
                if let Ok((w, h)) = engine.get_first_page_size(bytes) {
                    self.page_width = w;
                    self.page_height = h;
                }
                self.thumbnail_scale = THUMB_WIDTH / self.page_width;
                self.loaded = true;
            }
            Err(e) => {
                self.error = Some(e);
                self.loaded = true;
                return;
            }
        }

        // 渲染所有页面（全分辨率 + 缩略图）
        self.request_all_renders(engine, ctx);
        self.request_all_thumbnails(engine, ctx);
    }

    /// 请求渲染所有页面的全分辨率纹理（限制并发数）
    fn request_all_renders(&mut self, engine: &Arc<PdfEngine>, ctx: &Context) {
        self.next_render_idx = 0;
        for _ in 0..MAX_CONCURRENT_RENDERS.min(self.page_count) {
            self.request_next_render(engine, ctx);
        }
    }

    /// 启动下一个待渲染页面（如果还有未开始的页）
    fn request_next_render(&mut self, engine: &Arc<PdfEngine>, ctx: &Context) {
        while self.next_render_idx < self.page_count {
            let idx = self.next_render_idx;
            self.next_render_idx += 1;
            // 跳过已缓存的页面
            if self.textures.contains_key(&idx) {
                continue;
            }
            self.request_render(engine, idx, ctx);
            return;
        }
    }

    /// 请求渲染所有页面的缩略图（限制并发数）
    fn request_all_thumbnails(&mut self, engine: &Arc<PdfEngine>, _ctx: &Context) {
        let bytes = match &self.pdf_bytes {
            Some(b) => b.clone(),
            None => return,
        };
        let page_width = self.page_width;

        for i in 0..self.page_count {
            if self.thumbnails.contains_key(&i) {
                continue;
            }
            if self.pending_thumbnails.iter().any(|t| t.page_idx == i) {
                continue;
            }

            let engine = engine.clone();
            let bytes = bytes.clone();
            let (tx, rx) = oneshot::channel();

            tokio::spawn(async move {
                let result = engine.render_thumbnail(
                    bytes, i, THUMB_WIDTH, page_width,
                );
                match result {
                    Ok(img) => {
                        tx.send((i, img)).ok();
                    }
                    Err(e) => {
                        log::error!("PDF thumbnail page {}: {}", i, e);
                        // 发送占位图，避免 oneshot 永远不会被 resolve
                        let placeholder = ColorImage::new([2, 2], egui::Color32::from_gray(200));
                        tx.send((i, placeholder)).ok();
                    }
                }
            });

            self.pending_thumbnails.push(ThumbnailTask { rx, page_idx: i });
        }
    }

    /// 请求渲染第 `page_idx` 页（后台线程）
    fn request_render(&mut self, engine: &Arc<PdfEngine>, page_idx: usize, _ctx: &Context) {
        if page_idx >= self.page_count {
            return;
        }
        // 已缓存同分辨率纹理 → 跳过
        if self.textures.contains_key(&page_idx)
            && (self.rendered_scale - self.scale).abs() < 0.01
        {
            return;
        }
        // 忽略重复请求
        if self
            .pending_renders
            .iter()
            .any(|t| t.page_idx == page_idx)
        {
            return;
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
                Ok(img) => {
                    tx.send((page_idx, img)).ok();
                }
                Err(e) => {
                    log::error!("PDF render page {}: {}", page_idx, e);
                    // 发送灰色占位图，避免红色闪烁
                    let placeholder = ColorImage::new([2, 2], egui::Color32::from_gray(220));
                    tx.send((page_idx, placeholder)).ok();
                }
            }
        });

        self.pending_renders.push(RenderTask {
            rx,
            page_idx,
        });
    }

    /// 每帧轮询渲染和缩略图结果
    pub fn poll_render(&mut self, engine: &Arc<PdfEngine>, ctx: &Context) {
        let page_count = self.page_count;

        // 轮询全分辨率渲染结果
        let mut completed: Vec<usize> = Vec::new();
        for task in &mut self.pending_renders {
            match task.rx.try_recv() {
                Ok((idx, img)) => {
                    let tex = ctx.load_texture(
                        &format!("pdf_page_{}", idx),
                        img,
                        TextureOptions::LINEAR,
                    );
                    self.textures.insert(idx, tex);
                    self.rendered_scale = self.scale;
                    completed.push(idx);
                    ctx.request_repaint();
                }
                Err(oneshot::error::TryRecvError::Closed) => {
                    completed.push(task.page_idx);
                }
                _ => {}
            }
        }
        self.pending_renders
            .retain(|t| !completed.contains(&t.page_idx));

        // 有渲染完成 → 启动下一个待渲染页
        if !completed.is_empty() {
            self.request_next_render(engine, ctx);
        }

        // 轮询缩略图渲染结果
        let mut thumb_completed: Vec<usize> = Vec::new();
        for task in &mut self.pending_thumbnails {
            match task.rx.try_recv() {
                Ok((idx, img)) => {
                    let tex = ctx.load_texture(
                        &format!("pdf_thumb_{}", idx),
                        img,
                        TextureOptions::LINEAR,
                    );
                    self.thumbnails.insert(idx, tex);
                    thumb_completed.push(idx);
                    ctx.request_repaint();
                }
                Err(oneshot::error::TryRecvError::Closed) => {
                    thumb_completed.push(task.page_idx);
                }
                _ => {}
            }
        }
        self.pending_thumbnails
            .retain(|t| !thumb_completed.contains(&t.page_idx));

        // 检查是否所有页面已渲染完成
        if !self.all_rendered && self.textures.len() >= page_count && self.pending_renders.is_empty() {
            self.all_rendered = true;
            ctx.request_repaint();
        }
        // 重新渲染（缩放后）完成
        if self.re_render_in_progress
            && self.textures.len() >= page_count
            && self.pending_renders.is_empty()
        {
            self.re_render_in_progress = false;
            ctx.request_repaint();
        }
    }

    /// 切换视图模式
    pub fn set_view_mode(&mut self, mode: ViewMode, _engine: &Arc<PdfEngine>, ctx: &Context) {
        if self.view_mode == mode {
            return;
        }
        self.view_mode = mode;
        ctx.request_repaint();
    }

    /// 缩放（仅调整动画目标值，不重新渲染，保持即时响应）
    pub fn zoom_in(&mut self, _engine: &Arc<PdfEngine>, ctx: &Context) {
        let new_scale = (self.scale + ZOOM_STEP).min(MAX_SCALE);
        if (new_scale - self.scale).abs() < 0.01 {
            return;
        }
        self.scale = new_scale;
        self.display_scale_anim.set_target(new_scale as f64);
        ctx.request_repaint();
    }

    pub fn zoom_out(&mut self, _engine: &Arc<PdfEngine>, ctx: &Context) {
        let new_scale = (self.scale - ZOOM_STEP).max(MIN_SCALE);
        if (new_scale - self.scale).abs() < 0.01 {
            return;
        }
        self.scale = new_scale;
        self.display_scale_anim.set_target(new_scale as f64);
        ctx.request_repaint();
    }

    pub fn zoom_to_fit(&mut self, available_width: f32, _engine: &Arc<PdfEngine>, ctx: &Context) {
        if self.page_width > 0.0 {
            let new_scale = (available_width / self.page_width).max(0.5).min(2.0);
            if (new_scale - self.scale).abs() > 0.01 {
                self.scale = new_scale;
                self.display_scale_anim.set_target(new_scale as f64);
                ctx.request_repaint();
            }
        }
    }

    // ── 平摊模式专用缩放（列数驱动） ────────────────────────

    /// 平摊模式放大：列数减一，页面动画变大，最少一列
    pub fn zoom_grid_in(&mut self, avail_width: f32, _engine: &Arc<PdfEngine>, ctx: &Context) {
        if self.grid_cols > 1 {
            self.grid_cols -= 1;
            let gap = 16.0;
            let target_w = (avail_width - (self.grid_cols as f32 - 1.0) * gap) / self.grid_cols as f32;
            self.grid_size_anim.set_target(target_w as f64);
            ctx.request_repaint();
        }
    }

    /// 平摊模式缩小：列数加一，页面动画变小，最多全部页排成一行
    pub fn zoom_grid_out(&mut self, avail_width: f32, _engine: &Arc<PdfEngine>, ctx: &Context) {
        if self.grid_cols < self.page_count {
            self.grid_cols += 1;
            let gap = 16.0;
            let target_w = (avail_width - (self.grid_cols as f32 - 1.0) * gap) / self.grid_cols as f32;
            self.grid_size_anim.set_target(target_w as f64);
            ctx.request_repaint();
        }
    }

    /// 指定页的全分辨率纹理
    pub fn page_texture(&self, page_idx: usize) -> Option<&TextureHandle> {
        self.textures.get(&page_idx)
    }

    /// 指定页的缩略图纹理
    pub fn thumbnail_texture(&self, page_idx: usize) -> Option<&TextureHandle> {
        self.thumbnails.get(&page_idx)
    }

    pub fn is_loading(&self) -> bool {
        !self.pending_renders.is_empty() || !self.pending_thumbnails.is_empty()
    }

    /// 显示尺寸
    pub fn display_size(&self) -> Vec2 {
        let s = self.display_scale_anim.value() as f32;
        Vec2::new(self.page_width * s, self.page_height * s)
    }

    /// 全分辨率渲染进度
    fn render_progress(&self) -> (usize, usize) {
        (self.textures.len(), self.page_count)
    }

    /// 显示缩放比例
    pub fn display_scale(&self) -> f32 {
        self.display_scale_anim.value() as f32
    }

    pub fn update_animations(&mut self, dt: f64) {
        self.display_scale_anim.update(dt);
        self.grid_size_anim.update(dt);
    }

    pub fn is_animating(&self) -> bool {
        !self.display_scale_anim.is_steady() || !self.grid_size_anim.is_steady()
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

/// 渲染加载进度
fn render_loading_progress(ui: &mut egui::Ui, rendered: usize, total: usize) {
    let avail_h = ui.available_height().max(100.0).min(2000.0);
    ui.vertical_centered(|ui| {
        ui.add_space(avail_h * 0.35);
        ui.label(
            egui::RichText::new("正在渲染试卷...")
                .font(egui::FontId::new(18.0, egui::FontFamily::Proportional))
                .color(crate::theme::colors::text_primary()),
        );
        ui.add_space(12.0);

        // 进度条
        let progress = if total > 0 {
            rendered as f32 / total as f32
        } else {
            0.0
        };
        let bar_width = 300.0;
        let bar_height = 6.0;
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(bar_width, bar_height),
            egui::Sense::hover(),
        );
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(3),
            egui::Color32::from_gray(200),
        );
        if progress > 0.0 {
            let filled_width = bar_width * progress;
            let filled_rect = egui::Rect::from_min_size(
                rect.min,
                egui::vec2(filled_width, bar_height),
            );
            ui.painter().rect_filled(
                filled_rect,
                egui::CornerRadius::same(3),
                crate::theme::colors::primary(),
            );
        }
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(format!("{}/{} 页", rendered, total))
                .font(egui::FontId::new(13.0, egui::FontFamily::Proportional))
                .color(crate::theme::colors::text_secondary()),
        );
    });
    ui.ctx().request_repaint();
}

/// 带视图模式切换和缩放控件的 PDF 查看器渲染
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

    // 安全保护：page_count 为 0 时显示空状态
    if viewer.page_count == 0 {
        ui.vertical_centered(|ui| {
            ui.add_space(80.0);
            ui.label(
                egui::RichText::new("文档为空")
                    .font(egui::FontId::new(15.0, egui::FontFamily::Proportional))
                    .color(crate::theme::colors::text_secondary()),
            );
        });
        return;
    }

    // 渲染中：显示进度（包括缩放后重新渲染）
    let (rendered, total) = viewer.render_progress();
    if (!viewer.all_rendered || viewer.re_render_in_progress) && total > 0 {
        render_loading_progress(ui, rendered, total);
        return;
    }

    // ── 顶部工具栏 ────────────────────────────────────────────
    let content_width = ui.available_width();
    render_toolbar(ui, viewer, engine, content_width);

    // ── 内容区 ────────────────────────────────────────────────
    match viewer.view_mode {
        ViewMode::Grid => render_grid_mode(ui, viewer, engine),
        ViewMode::Line => render_line_mode(ui, viewer, engine),
    }
}

/// 工具栏：视图模式切换 + 页码 + 缩放控制 + 总览折叠（仅 Line 模式）
fn render_toolbar(ui: &mut egui::Ui, viewer: &mut PdfViewer, engine: &Arc<PdfEngine>, content_width: f32) {
    use crate::theme::colors;
    use egui::FontId;

    ui.horizontal(|ui| {
        ui.add_space(8.0);

        // ── 视图模式切换按钮 ──────────────────────────────
        for &mode in ViewMode::all() {
            let is_active = viewer.view_mode == mode;
            let text_color = if is_active {
                egui::Color32::WHITE
            } else {
                colors::text_secondary()
            };
            let bg = if is_active {
                colors::primary()
            } else {
                egui::Color32::TRANSPARENT
            };
            let btn = egui::Button::new(
                egui::RichText::new(mode.label())
                    .font(FontId::new(13.0, egui::FontFamily::Proportional))
                    .color(text_color),
            )
            .min_size(egui::vec2(40.0, 26.0))
            .fill(bg)
            .stroke(egui::Stroke::new(1.0, colors::border()));
            if ui.add(btn).clicked() && !is_active {
                viewer.set_view_mode(mode, engine, ui.ctx());
            }
        }

        ui.add_space(12.0);

        // ── 页码 / 进度 ────────────────────────────────────
        match viewer.view_mode {
            ViewMode::Line => {
                ui.label(
                    egui::RichText::new(format!(
                        "第 {} / {} 页",
                        viewer.current_page + 1,
                        viewer.page_count
                    ))
                    .font(FontId::new(13.0, egui::FontFamily::Proportional))
                    .color(colors::text_secondary()),
                );
            }
            _ => {
                ui.label(
                    egui::RichText::new(format!("共 {} 页", viewer.page_count))
                        .font(FontId::new(13.0, egui::FontFamily::Proportional))
                        .color(colors::text_secondary()),
                );
            }
        }

        // ── 右侧控制 ────────────────────────────────────────
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);

            // 缩放控件（两种模式都可用）
            if ui
                .button(egui::RichText::new("➕").font(FontId::new(14.0, egui::FontFamily::Proportional)))
                .clicked()
            {
                match viewer.view_mode {
                    ViewMode::Grid => viewer.zoom_grid_in(content_width, engine, ui.ctx()),
                    ViewMode::Line => viewer.zoom_in(engine, ui.ctx()),
                }
            }
            ui.add_space(2.0);
            if ui
                .button(egui::RichText::new("➖").font(FontId::new(14.0, egui::FontFamily::Proportional)))
                .clicked()
            {
                match viewer.view_mode {
                    ViewMode::Grid => viewer.zoom_grid_out(content_width, engine, ui.ctx()),
                    ViewMode::Line => viewer.zoom_out(engine, ui.ctx()),
                }
            }
            ui.add_space(2.0);
            if ui
                .button(egui::RichText::new("适").font(FontId::new(13.0, egui::FontFamily::Proportional)))
                .clicked()
            {
                viewer.zoom_to_fit(content_width, engine, ui.ctx());
            }
            ui.add_space(4.0);
            let zoom_text = match viewer.view_mode {
                ViewMode::Grid => format!("{} 列", viewer.grid_cols),
                ViewMode::Line => format!("{:.0}%", viewer.display_scale() * 100.0),
            };
            ui.label(
                egui::RichText::new(zoom_text)
                    .font(FontId::new(12.0, egui::FontFamily::Proportional))
                    .color(colors::text_secondary()),
            );
            ui.add_space(8.0);

            // 总览面板折叠切换（仅 Line 模式）
            if viewer.view_mode == ViewMode::Line {
                let overview_label = if viewer.overview_open { "◀ 总览" } else { "▶ 总览" };
                if ui
                    .button(
                        egui::RichText::new(overview_label)
                            .font(FontId::new(13.0, egui::FontFamily::Proportional))
                            .color(colors::text_secondary()),
                    )
                    .clicked()
                {
                    viewer.overview_open = !viewer.overview_open;
                }
            }
        });
    });

    ui.add_space(4.0);
    ui.separator();
    ui.add_space(4.0);
}

// ── 平摊模式（纵向栅格）───────────────────────────────────────────────────

/// 平摊模式：grid_cols 列，页面宽度由 grid_size_anim 平滑过渡，Grid 处理布局和滚动
fn render_grid_mode(ui: &mut egui::Ui, viewer: &mut PdfViewer, engine: &Arc<PdfEngine>) {
    let gap = 16.0;
    let cols = viewer.grid_cols.max(1);

    // 动画未完成时持续请求重绘
    if !viewer.grid_size_anim.is_steady() {
        ui.ctx().request_repaint();
    }

    egui::ScrollArea::vertical()
        .id_salt("pdf_grid_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // 在 ScrollArea 内部获取可用宽度，考虑滚动条占位
            let avail_width = ui.available_width().max(100.0);

            // 校准动画：稳态时确保值匹配当前列数的目标宽度
            let target_w = (avail_width - (cols as f32 - 1.0) * gap) / cols as f32;
            if viewer.grid_size_anim.is_steady() {
                if (viewer.grid_size_anim.value() - target_w as f64).abs() > 0.5 {
                    viewer.grid_size_anim = {
                        let mut a = SpringAnim::new(0.4, 0.8, target_w as f64);
                        let _ = a.update(100.0);
                        a
                    };
                }
            }

            // 页面宽度：动画值，初值匹配当前列数，缩放时平滑过渡
            let page_w = viewer.grid_size_anim.value().max(1.0) as f32;
            let page_h = page_w * (viewer.page_height / viewer.page_width.max(1.0));
            let page_size = egui::vec2(page_w, page_h);

            // 使用 Grid 布局：正确注册分配空间，ScrollArea 感知内容高度，支持滚动
            egui::Grid::new("pdf_grid_pages")
                .min_col_width(page_size.x)
                .max_col_width(page_size.x)
                .spacing(egui::vec2(gap, gap))
                .show(ui, |ui| {
                    for i in 0..viewer.page_count {
                        let (rect, response) = ui.allocate_exact_size(page_size, egui::Sense::click());

                        if let Some(tex) = viewer.page_texture(i) {
                            ui.painter().rect_stroke(
                                rect,
                                egui::CornerRadius::ZERO,
                                egui::Stroke::new(1.0, egui::Color32::from_gray(200)),
                                egui::StrokeKind::Inside,
                            );
                            ui.painter().image(
                                tex.id(),
                                rect,
                                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                                egui::Color32::WHITE,
                            );
                        }
                        // 页码
                        ui.painter().text(
                            egui::pos2(rect.right() - 4.0, rect.bottom() - 4.0),
                            egui::Align2::RIGHT_BOTTOM,
                            &format!("{}", i + 1),
                            egui::FontId::new(11.0, egui::FontFamily::Proportional),
                            egui::Color32::from_gray(60),
                        );

                        // 点击 → 切换到滚动模式并跳转
                        if response.clicked() {
                            viewer.scroll_to_page = Some(i);
                            viewer.set_view_mode(ViewMode::Line, engine, ui.ctx());
                        }

                        // 换行
                        if (i + 1) % cols == 0 && i + 1 < viewer.page_count {
                            ui.end_row();
                        }
                    }
                });
        });
}

// ── 阅读模式（连续）────────────────────────────────────────────────────────

/// 阅读模式：左侧总览面板 + 右侧连续纵向滚动
fn render_line_mode(ui: &mut egui::Ui, viewer: &mut PdfViewer, engine: &Arc<PdfEngine>) {
    use crate::theme::colors;

    // 占满可用区域，确保父布局感知高度
    let avail = egui::vec2(ui.available_width(), ui.available_height());
    if avail.x <= 0.0 || avail.y <= 0.0 {
        return;
    }

    let (outer_rect, _) = ui.allocate_exact_size(avail, egui::Sense::hover());
    let mut outer = ui.child_ui(outer_rect, egui::Layout::left_to_right(egui::Align::TOP), None);

    // ── 左侧总览面板（固定宽度） ────────────────────────────
    if viewer.overview_open {
        let (o_rect, _) = outer.allocate_exact_size(
            egui::vec2(OVERVIEW_PANEL_WIDTH, outer_rect.height()),
            egui::Sense::hover(),
        );
        let mut o_ui = outer.child_ui(o_rect, egui::Layout::top_down(egui::Align::LEFT), None);
        o_ui.painter().rect_filled(o_rect, egui::CornerRadius::ZERO, colors::bg_card());
        let sep = o_rect.right() + 2.0;
        o_ui.painter().line_segment(
            [egui::pos2(sep, o_rect.top()), egui::pos2(sep, o_rect.bottom())],
            egui::Stroke::new(1.0, colors::border()),
        );
        render_overview_content(&mut o_ui, viewer, engine);
    }

    // ── 右侧主内容区（填满剩余宽度） ────────────────────────
    let c_w = (outer_rect.width() - if viewer.overview_open { OVERVIEW_PANEL_WIDTH } else { 0.0 }).max(200.0);
    let (c_rect, _) = outer.allocate_exact_size(
        egui::vec2(c_w, outer_rect.height()),
        egui::Sense::hover(),
    );
    let mut c_ui = outer.child_ui(c_rect, egui::Layout::top_down(egui::Align::LEFT), None);
    render_line_content(&mut c_ui, viewer, engine);
}

/// 总览面板内容：缩略图列表，当前页高亮，点击跳转
fn render_overview_content(ui: &mut egui::Ui, viewer: &mut PdfViewer, _engine: &Arc<PdfEngine>) {
    use crate::theme::colors;

    let panel_width = OVERVIEW_PANEL_WIDTH;
    let thumb_w = OVERVIEW_THUMB_WIDTH;
    let page_width = viewer.page_width.max(1.0);
    let page_height = viewer.page_height.max(1.0);
    let thumb_h = (page_height * (OVERVIEW_THUMB_WIDTH / page_width)).max(20.0);
    let current_page = viewer.current_page;

    egui::ScrollArea::vertical()
        .id_salt("pdf_overview_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(4.0);
            for i in 0..viewer.page_count {
                let is_current = i == current_page;
                let item_h = thumb_h + 8.0 + 14.0; // 缩略图 + 内边距 + 页码

                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(panel_width, item_h),
                    egui::Sense::click(),
                );

                // 当前页高亮背景
                if is_current {
                    ui.painter().rect_filled(
                        rect,
                        egui::CornerRadius::same(3),
                        colors::primary().linear_multiply(0.15),
                    );
                    // 左侧强调色条
                    ui.painter().rect_filled(
                        egui::Rect::from_min_size(
                            egui::pos2(rect.left(), rect.top()),
                            egui::vec2(3.0, rect.height()),
                        ),
                        egui::CornerRadius::ZERO,
                        colors::primary(),
                    );
                }

                // 缩略图
                let thumb_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.left() + (panel_width - thumb_w) / 2.0, rect.top() + 4.0),
                    egui::vec2(thumb_w, thumb_h),
                );
                if let Some(tex) = viewer.thumbnail_texture(i) {
                    ui.painter().image(
                        tex.id(),
                        thumb_rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                } else {
                    ui.painter().rect_filled(
                        thumb_rect,
                        egui::CornerRadius::same(2),
                        egui::Color32::from_gray(220),
                    );
                }

                // 页码标签
                let page_color = if is_current { colors::primary() } else { colors::text_secondary() };
                ui.painter().text(
                    egui::pos2(rect.center().x, rect.bottom() - 2.0),
                    egui::Align2::CENTER_BOTTOM,
                    &format!("{}", i + 1),
                    egui::FontId::new(10.0, egui::FontFamily::Proportional),
                    page_color,
                );

                // 点击跳转
                if response.clicked() && i != current_page {
                    viewer.scroll_to_page = Some(i);
                    viewer.current_page = i;
                    ui.ctx().request_repaint();
                }
            }
        });
}

/// 阅读模式主内容区：连续纵向滚动，页面居中，ScrollArea 占满全宽
fn render_line_content(ui: &mut egui::Ui, viewer: &mut PdfViewer, _engine: &Arc<PdfEngine>) {
    let page_size = viewer.display_size();
    // 安全保护：防止 page_size 为零或 NaN 导致崩溃
    if page_size.x <= 0.0 || page_size.y <= 0.0 || !page_size.x.is_finite() || !page_size.y.is_finite() {
        return;
    }
    let scroll_to = viewer.scroll_to_page.take();
    let full_width = ui.available_width().max(100.0);
    let x_offset = (full_width - page_size.x).max(0.0) / 2.0;

    let output = egui::ScrollArea::vertical()
        .id_salt("pdf_scroll_line")
        .show(ui, |ui| {
            // 为每个页面分配空间（占满全宽）
            for i in 0..viewer.page_count {
                let y_start = ui.cursor().top();

                // 分配全宽空间，让 ScrollArea 感知完整宽度，滚动条贴右
                let (rect, _response) = ui.allocate_exact_size(
                    egui::vec2(full_width, page_size.y),
                    egui::Sense::click(),
                );

                let page_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.left() + x_offset, y_start),
                    page_size,
                );

                // 渲染页面纹理
                if let Some(tex) = viewer.page_texture(i) {
                    // 页面边框
                    ui.painter().rect_stroke(
                        page_rect,
                        egui::CornerRadius::ZERO,
                        egui::Stroke::new(1.0, egui::Color32::from_gray(200)),
                        egui::StrokeKind::Inside,
                    );
                    ui.painter().image(
                        tex.id(),
                        page_rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                }

                // 跳转到指定页
                if scroll_to == Some(i) {
                    ui.scroll_to_rect(page_rect, Some(egui::Align::TOP));
                }
            }
        });

    // 更新当前页码（scroll_to_rect 在当前帧通过 layout 生效，offset 已更新）
    // 但当 scroll_to 刚被触发时，current_page 已在点击处理中设置，无需再计算
    if scroll_to.is_none() {
        let scroll_y = output.state.offset.y;
        let page_step = page_size.y;
        if page_step > 0.0 && page_step.is_finite() {
            let new_page = (scroll_y / page_step)
                .clamp(0.0, (viewer.page_count.max(1) - 1) as f32)
                .floor() as usize;
            if new_page != viewer.current_page {
                viewer.current_page = new_page;
                ui.ctx().request_repaint();
            }
        }
    }
}