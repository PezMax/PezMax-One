use crate::api::client::ApiClient;
use crate::api::models::*;
use crate::components::action_bar;
use crate::pdf::{PdfEngine, PdfViewer};
use crate::sokuou::{map_range, Easing, Progress, SpringAnim};
use crate::theme;
use anyhow;
use base64::Engine;
use egui::Context;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

/// 保存到磁盘的凭证
#[derive(Debug, Serialize, Deserialize)]
struct SavedCredentials {
    token: String,
    username: String,
    remember_me: bool,
}

fn get_data_dir() -> std::path::PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        let dir = std::path::PathBuf::from(appdata).join("PezMax");
        let _ = std::fs::create_dir_all(&dir);
        dir
    } else {
        std::path::PathBuf::from(".")
    }
}

fn avatar_cache_dir() -> std::path::PathBuf {
    let dir = get_data_dir().join("avatar_cache");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn bookmark_cover_cache_dir() -> std::path::PathBuf {
    let dir = get_data_dir().join("bookmark_cover_cache");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn credentials_path() -> std::path::PathBuf {
    get_data_dir().join("credentials.json")
}

fn save_credentials(token: &str, username: &str, remember_me: bool) {
    let creds = SavedCredentials {
        token: token.to_string(),
        username: username.to_string(),
        remember_me,
    };
    if let Ok(json) = serde_json::to_string(&creds) {
        let _ = std::fs::write(credentials_path(), json);
    }
}

fn load_credentials() -> Option<SavedCredentials> {
    let path = credentials_path();
    if !path.exists() {
        return None;
    }
    let json = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&json).ok()
}

fn clear_credentials() {
    let path = credentials_path();
    let _ = std::fs::remove_file(path);
}

/// 将 base64 图片（JPEG 格式）解码为 egui 纹理
fn decode_base64_image(b64: &str, ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .ok()?;
    let img = image::load_from_memory(&bytes).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let pixels = rgba.into_raw();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(
        [w as usize, h as usize],
        &pixels,
    );
    Some(ctx.load_texture("captcha", color_image, egui::TextureOptions::LINEAR))
}

/// 认证阶段的子页面（is_logged_in == false 时使用）
#[derive(Debug, Clone, PartialEq)]
pub enum AuthPage {
    Login,
    Register,
    ForgetPassword,
}

/// 顶级功能区（侧边栏 4 个入口）
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Section {
    Home,
    Browse,
    Community,
    Profile,
}

impl Section {
    pub fn index(self) -> usize {
        match self {
            Section::Home => 0,
            Section::Browse => 1,
            Section::Community => 2,
            Section::Profile => 3,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Section::Home => "首页",
            Section::Browse => "浏览",
            Section::Community => "社区",
            Section::Profile => "个人",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Section::Home => "🏠",
            Section::Browse => "📂",
            Section::Community => "👥",
            Section::Profile => "👤",
        }
    }

    pub fn default_subsection(self) -> Subsection {
        match self {
            Section::Home => Subsection::None,
            Section::Browse => Subsection::ResourceManager,
            Section::Community => Subsection::UserRanking,
            Section::Profile => Subsection::PersonalCenter,
        }
    }

    /// 该 Section 下的子标签列表，Home 返回空
    pub fn subsections(self) -> Vec<(Subsection, &'static str)> {
        match self {
            Section::Home => vec![],
            Section::Browse => vec![
                (Subsection::ResourceManager, "资源管理"),
                (Subsection::ExternalBookmarks, "外部书签"),
                (Subsection::MyFavorites, "我的收藏"),
            ],
            Section::Community => vec![
                (Subsection::UserRanking, "用户排行"),
                (Subsection::ContributeFile, "贡献文件"),
                (Subsection::ReportRecord, "举报记录"),
            ],
            Section::Profile => vec![
                (Subsection::PersonalCenter, "个人中心"),
                (Subsection::Notifications, "通知"),
                (Subsection::DownloadHistory, "下载记录"),
                (Subsection::AppSettings, "设置"),
            ],
        }
    }
}

/// 各功能区内的子标签
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Subsection {
    None,
    // Browse
    ResourceManager,
    ExternalBookmarks,
    MyFavorites,
    // Community
    UserRanking,
    ContributeFile,
    ReportRecord,
    // Profile
    PersonalCenter,
    Notifications,
    DownloadHistory,
    AppSettings,
}

/// 账号设置当前编辑中的子区域
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccountEditSection {
    None,
    Avatar,
    Username,
    Security,
    Password,
}

/// Toast 通知级别
#[derive(Debug, Clone, PartialEq)]
pub enum ToastLevel {
    Info,
    Success,
    Warning,
    Error,
}

/// 带入场/离场动画的 Toast
pub struct AnimatedToast {
    pub message: String,
    pub level: ToastLevel,
    pub enter: Progress,
    pub exit: Progress,
    pub exit_triggered: bool,
    pub created_at: std::time::Instant,
}

impl AnimatedToast {
    pub fn new(message: impl Into<String>, level: ToastLevel) -> Self {
        let mut enter = Progress::with_easing(0.25, Easing::EaseOutCubic);
        enter.set_target(1.0);
        Self {
            message: message.into(),
            level,
            enter,
            exit: Progress::with_easing(0.25, Easing::EaseInCubic),
            exit_triggered: false,
            created_at: std::time::Instant::now(),
        }
    }
}

/// 浏览页筛选状态
#[derive(Default)]
pub struct FilterState {
    pub subject: Option<String>,
    pub school: Option<String>,
}

/// 登录异步结果
pub struct LoginResult {
    pub token: String,
    pub user: UserInfo,
}

/// 通用异步数据加载器
pub struct AsyncData<T> {
    rx: Option<oneshot::Receiver<anyhow::Result<T>>>,
    pub data: Option<T>,
    pub error: Option<String>,
    pub loading: bool,
    loaded: bool,
}

impl<T: Send + 'static> AsyncData<T> {
    pub fn new() -> Self {
        Self {
            rx: None,
            data: None,
            error: None,
            loading: false,
            loaded: false,
        }
    }

    /// 启动异步加载（重复调用不会重复启动）
    pub fn load<F, Fut>(&mut self, f: F)
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = anyhow::Result<T>> + Send,
    {
        if self.loading || self.loaded {
            return;
        }
        self.loading = true;
        let (tx, rx) = oneshot::channel();
        self.rx = Some(rx);
        tokio::spawn(async move {
            let result = f().await;
            tx.send(result).ok();
        });
    }

    /// 每帧轮询结果
    pub fn poll(&mut self) {
        if let Some(rx) = &mut self.rx {
            if let Ok(result) = rx.try_recv() {
                self.rx = None;
                self.loading = false;
                match result {
                    Ok(data) => {
                        self.data = Some(data);
                        self.loaded = true;
                    }
                    Err(e) => {
                        self.error = Some(e.to_string());
                        self.loaded = true;
                    }
                }
            }
        }
    }

    pub fn is_loaded(&self) -> bool { self.loaded }
    pub fn is_loading(&self) -> bool { self.loading }
    pub fn reset(&mut self) {
        self.rx = None;
        self.data = None;
        self.error = None;
        self.loading = false;
        self.loaded = false;
    }
}

/// 应用主状态
pub struct PezMaxApp {
    pub api: ApiClient,

    // 登录表单状态
    pub login_username: String,
    pub login_password: String,
    pub login_captcha: String,
    pub login_captcha_uuid: String,
    pub login_captcha_img: String,
    pub login_captcha_texture: Option<egui::TextureHandle>,
    pub login_captcha_enabled: bool,
    pub login_loading: bool,
    pub login_error: String,
    pub login_remember: bool,
    pub captcha_loaded: bool,

    // 异步结果接收器
    pub captcha_rx: Option<oneshot::Receiver<anyhow::Result<CaptchaResponse>>>,
    pub login_rx: Option<oneshot::Receiver<anyhow::Result<LoginResult>>>,
    pub auto_login_rx: Option<oneshot::Receiver<anyhow::Result<(UserInfo, String)>>>,

    // 异步数据加载器
    pub notifications: AsyncData<Vec<Notification>>,
    pub download_records: AsyncData<Vec<DownloadRecord>>,
    pub recent_files: AsyncData<Vec<PaperFile>>,
    pub user_stats_data: AsyncData<UserStats>,
    // Browse 页面
    pub file_list_data: AsyncData<Vec<PaperFile>>,
    pub subjects_data: AsyncData<Vec<String>>,
    pub schools_data: AsyncData<Vec<String>>,
    pub bookmarks_data: AsyncData<Vec<Bookmark>>,
    pub favorites_data: AsyncData<Vec<FavoriteRecord>>,
    // Community 页面
    pub user_rank_data: AsyncData<Vec<UserRankItem>>,
    pub my_reports_data: AsyncData<Vec<Report>>,
    // 排行头像缓存（支持 GIF 动图多帧）
    pub rank_avatar_textures: HashMap<i64, Vec<egui::TextureHandle>>,
    pub rank_avatar_delays: HashMap<i64, Vec<f32>>,       // 每帧延迟（秒）
    pub rank_avatar_timer: HashMap<i64, f32>,              // 当前动画计时
    pub rank_avatar_frame_idx: HashMap<i64, usize>,        // 当前帧索引
    pub rank_avatar_rx: Option<mpsc::UnboundedReceiver<(i64, anyhow::Result<Vec<u8>>)>>,
    pub rank_avatar_tx: Option<mpsc::UnboundedSender<(i64, anyhow::Result<Vec<u8>>)>>,
    pub rank_avatar_failed: HashSet<i64>,
    pub rank_avatar_pending: HashSet<i64>,

    // 认证
    pub is_logged_in: bool,
    pub auth_page: AuthPage,
    pub token: Option<String>,
    pub current_user: Option<UserInfo>,
    pub user_stats: Option<UserStats>,

    // 顶级导航
    pub current_section: Section,
    pub current_subsection: Subsection,

    // 侧边栏（可折叠汉堡菜单）
    // sidebar_anim: 0.0 = 折叠(48px) / 1.0 = 展开(200px)
    // sidebar_indicator_anim: 值为当前高亮 Section 的索引（0-3），弹簧插值
    pub sidebar_open: bool,
    pub sidebar_anim: SpringAnim,
    pub sidebar_indicator_anim: SpringAnim,
    // 子标签下划线 X 位置（值 = 当前 subsection 在列表中的浮点索引）
    pub subtab_indicator_anim: SpringAnim,

    // 浏览状态
    pub search_query: String,
    pub filters: FilterState,
    pub file_list: Vec<PaperFile>,
    pub file_total: i64,
    pub file_page: PageParams,
    pub is_loading: bool,
    pub selected_file: Option<PaperFile>,
    pub preview_visible: bool,
    pub preview_anim: SpringAnim,
    pub browse_selected_idx: Option<usize>, // MOCK_FILES 索引，None = 列表视图

    // 页面切换入场动画
    pub page_enter_anim: SpringAnim,
    // 认证页切换淡入（0→1）
    pub auth_anim: Progress,

    // Toast 通知
    pub toasts: Vec<AnimatedToast>,
    pub unread_notifications: i32,

    // 书签
    pub selected_bookmark: Option<Bookmark>,
    pub bookmark_detail_anim: SpringAnim,
    pub bookmark_form_name: String,
    pub bookmark_form_url: String,
    pub bookmark_form_resource_type: String,
    pub bookmark_form_collection: String,
    pub bookmark_form_subject: String,
    pub bookmark_form_description: String,
    pub bookmark_edit_target: Option<Bookmark>, // Some=编辑模式, None=新建模式
    pub show_bookmark_form: bool,
    pub favorite_bookmark_ids: HashSet<i64>,       // 已收藏的书签 ID
    pub bookmark_fav_data: Vec<(i64, bool)>,        // (bookmark_id, is_add) 待处理的收藏操作
    pub bookmark_favorite_ids_rx: Option<oneshot::Receiver<anyhow::Result<HashSet<i64>>>>,
    pub bookmark_cover_textures: HashMap<i64, egui::TextureHandle>,  // 书签封面纹理缓存
    pub bookmark_cover_requested: HashSet<i64>,                 // 已请求封面的书签
    pub bookmark_cover_rx: Option<tokio::sync::oneshot::Receiver<anyhow::Result<Vec<u8>>>>,
    pub bookmark_cover_pending_id: Option<i64>,                 // 当前等待中的封面请求 ID
    pub bookmark_cover_bulk_rx: Option<tokio::sync::mpsc::UnboundedReceiver<(i64, anyhow::Result<Vec<u8>>)>>,
    pub bookmark_covers_triggered: bool,  // 防止每帧重复创建封面加载通道

    // 贡献文件元数据表单
    pub contribute_subject: String,
    pub contribute_school: String,
    pub contribute_year: String,
    pub contribute_file_path: Option<String>,

    // 举报表单
    pub report_content: String,
    pub report_type: String,
    pub show_report_dialog: bool,

    // 设置开关
    pub setting_auto_launch: bool,
    pub setting_silent_download: bool,

    // 外观：外观模式 + 强调色索引（对应 theme::ACCENT_PRESETS）
    pub theme_mode: theme::ThemeMode,
    pub accent_idx: usize,

    // 搜索框提示文字动画（SpringAnim: 0.0=隐藏, 1.0=完全显示）
    pub search_hint_anim: SpringAnim,
    pub search_was_focused: bool,

    // 试卷详情面板：是否显示文件信息弹窗
    pub show_info_dialog: bool,
    // 已收藏的文件 ID 集合（用于工具栏收藏按钮状态）
    pub favorite_file_ids: std::collections::HashSet<i64>,
    pub favorite_ids_loaded: bool,
    // 预览模式下底部操作栏的待处理动作（每帧渲染后重置）
    pub preview_bar_action: action_bar::Action,
    // 预览模式，用于 app.rs 中控制边距/面板渲染
    pub preview_mode: bool,

    // 已收藏文件 ID 加载
    pub favorite_ids_rx: Option<oneshot::Receiver<anyhow::Result<std::collections::HashSet<i64>>>>,

    // 头像加载
    pub avatar_texture: Option<egui::TextureHandle>,
    pub avatar_image_size: Option<(usize, usize)>,
    pub avatar_load_rx: Option<oneshot::Receiver<anyhow::Result<Vec<u8>>>>,

    // 账号设置状态
    pub account_edit_section: AccountEditSection,
    pub account_edit_username: String,
    pub account_edit_nickname: String,
    pub account_edit_old_password: String,
    pub account_edit_new_password: String,
    pub account_edit_confirm_password: String,
    pub account_edit_security_questions: Vec<crate::api::models::SecurityQuestion>,
    pub account_edit_loading: bool,
    pub account_edit_error: String,
    pub account_edit_success: String,
    pub account_edit_message_timer: f32,

    // PDF 引擎（全局单例，Arc<Sync>）
    pub pdf_engine: Arc<PdfEngine>,
    // PDF 查看器（当前打开的 PDF 文档状态）
    pub pdf_viewer: PdfViewer,
    // PDF 字节加载
    pub pdf_loading: bool,
    pub pdf_bytes_rx: Option<oneshot::Receiver<anyhow::Result<Vec<u8>>>>,
}

impl PezMaxApp {
    pub fn new(cc: &eframe::CreationContext<'_>, pdf_engine: Arc<PdfEngine>) -> Self {
        theme::setup_fonts(&cc.egui_ctx);
        theme::apply_metro_theme(&cc.egui_ctx);

        let mut app = Self {
            api: ApiClient::new(None),

            // 登录表单
            login_username: String::new(),
            login_password: String::new(),
            login_captcha: String::new(),
            login_captcha_uuid: String::new(),
            login_captcha_img: String::new(),
            login_captcha_texture: None,
            login_captcha_enabled: true,
            login_loading: false,
            login_error: String::new(),
            login_remember: false,
            captcha_loaded: false,
            captcha_rx: None,
            login_rx: None,
            auto_login_rx: None,

            notifications: AsyncData::new(),
            download_records: AsyncData::new(),
            recent_files: AsyncData::new(),
            user_stats_data: AsyncData::new(),
            file_list_data: AsyncData::new(),
            subjects_data: AsyncData::new(),
            schools_data: AsyncData::new(),
            bookmarks_data: AsyncData::new(),
            favorites_data: AsyncData::new(),
            user_rank_data: AsyncData::new(),
            my_reports_data: AsyncData::new(),
            rank_avatar_textures: HashMap::new(),
            rank_avatar_delays: HashMap::new(),
            rank_avatar_timer: HashMap::new(),
            rank_avatar_frame_idx: HashMap::new(),
            rank_avatar_rx: None,
            rank_avatar_tx: None,
            rank_avatar_failed: HashSet::new(),
            rank_avatar_pending: HashSet::new(),

            is_logged_in: false,
            auth_page: AuthPage::Login,
            token: None,
            current_user: None,
            user_stats: None,
            current_section: Section::Home,
            current_subsection: Subsection::None,
            sidebar_open: true,
            sidebar_anim: SpringAnim::new(0.5, 0.825, 1.0),
            sidebar_indicator_anim: SpringAnim::new(0.3, 0.8, 0.0), // 初始指向 Home(0)
            subtab_indicator_anim: SpringAnim::new(0.25, 0.85, 0.0),
            search_query: String::new(),
            filters: FilterState::default(),
            file_list: vec![],
            file_total: 0,
            file_page: PageParams::default(),
            is_loading: false,
            selected_file: None,
            preview_visible: false,
            preview_anim: SpringAnim::new(0.4, 0.8, 0.0),
            selected_bookmark: None,
            bookmark_detail_anim: SpringAnim::new(0.4, 0.8, 0.0),
            browse_selected_idx: None,
            page_enter_anim: SpringAnim::new(0.4, 0.8, 1.0), // 初始稳态
            auth_anim: {
                let mut p = Progress::with_easing(0.2, Easing::EaseOutCubic);
                p.set_target(1.0);
                p
            },
            toasts: vec![],
            unread_notifications: 0,
            bookmark_form_name: String::new(),
            bookmark_form_url: String::new(),
            bookmark_form_resource_type: String::new(),
            bookmark_form_collection: String::new(),
            bookmark_form_subject: String::new(),
            bookmark_form_description: String::new(),
            bookmark_edit_target: None,
            show_bookmark_form: false,
            favorite_bookmark_ids: HashSet::new(),
            bookmark_fav_data: Vec::new(),
            bookmark_favorite_ids_rx: None,
            bookmark_cover_textures: HashMap::new(),
            bookmark_cover_requested: HashSet::new(),
            bookmark_cover_rx: None,
            bookmark_cover_pending_id: None,
            bookmark_cover_bulk_rx: None,
            bookmark_covers_triggered: false,
            contribute_subject: String::new(),
            contribute_school: String::new(),
            contribute_year: String::new(),
            contribute_file_path: None,
            report_content: String::new(),
            report_type: String::new(),
            show_report_dialog: false,
            setting_auto_launch: false,
            setting_silent_download: false,
            theme_mode: theme::ThemeMode::System,
            accent_idx: 0,

            search_hint_anim: SpringAnim::new(0.25, 0.7, 0.0),
            search_was_focused: false,

            show_info_dialog: false,
            favorite_file_ids: std::collections::HashSet::new(),
            favorite_ids_loaded: false,
            favorite_ids_rx: None,
            preview_bar_action: action_bar::Action::None,
            preview_mode: false,

            avatar_texture: None,
            avatar_image_size: None,
            avatar_load_rx: None,

            account_edit_section: AccountEditSection::None,
            account_edit_username: String::new(),
            account_edit_nickname: String::new(),
            account_edit_old_password: String::new(),
            account_edit_new_password: String::new(),
            account_edit_confirm_password: String::new(),
            account_edit_security_questions: vec![],
            account_edit_loading: false,
            account_edit_error: String::new(),
            account_edit_success: String::new(),
            account_edit_message_timer: 0.0,

            pdf_engine,
            pdf_viewer: PdfViewer::new(),
            pdf_loading: false,
            pdf_bytes_rx: None,
        };

        // 尝试从本地加载凭证并自动登录
        app.try_auto_login();

        app
    }

    /// 尝试从本地加载凭证并自动登录
    pub fn try_auto_login(&mut self) {
        if let Some(creds) = load_credentials() {
            self.login_username = creds.username;
            self.login_remember = creds.remember_me;
            // 设置 token 并异步验证
            let api = self.api.clone();
            let saved_token = creds.token.clone();
            self.is_logged_in = true;
            let (tx, rx) = oneshot::channel();
            self.auto_login_rx = Some(rx);
            tokio::spawn(async move {
                api.set_token(saved_token.clone()).await;
                let result = api.get_user_info().await;
                let result = match result {
                    Ok(resp) => {
                        match resp.data {
                            Some(data) => Ok((data.user, saved_token)),
                            None => Err(anyhow::anyhow!("获取用户信息失败: {}", resp.msg)),
                        }
                    }
                    Err(e) => Err(e),
                };
                tx.send(result).ok();
            });
        }
    }

    /// 登录成功后调用：进入首页，触发入场动画，加载统计数据
    pub fn login_success(&mut self) {
        self.is_logged_in = true;
        self.current_section = Section::Home;
        self.current_subsection = Subsection::None;
        self.page_enter_anim = SpringAnim::with_target(0.4, 0.8, 0.0, 0.0, 1.0);
        self.sidebar_indicator_anim.set_target(0.0); // Home

        // 保存凭证（如果勾选了"记住我"）
        if self.login_remember {
            if let Some(ref token) = self.token {
                save_credentials(token, &self.login_username, true);
            }
        } else {
            clear_credentials();
        }

        // 清空登录表单
        self.login_username.clear();
        self.login_password.clear();
        self.login_captcha.clear();
        self.login_captcha_uuid.clear();
        self.login_captcha_img.clear();
        self.login_captcha_texture = None;
        self.login_error.clear();
        self.captcha_loaded = false;
        // 自动加载首页数据
        self.trigger_load_user_stats();
        self.trigger_load_recent_files();
        self.trigger_load_favorite_ids();
        self.trigger_load_bookmark_favorite_ids();
        // 加载头像
        self.trigger_load_avatar();
    }

    /// 异步加载验证码
    pub fn trigger_captcha_load(&mut self) {
        if self.captcha_rx.is_some() {
            return; // 已有请求进行中
        }
        let api = self.api.clone();
        let (tx, rx) = oneshot::channel();
        self.captcha_rx = Some(rx);
        tokio::spawn(async move {
            let result = api.get_captcha().await;
            let result = match result {
                Ok(api_resp) => {
                    if let Some(data) = api_resp.data {
                        Ok(data)
                    } else {
                        Err(anyhow::anyhow!("验证码响应为空: {} {}", api_resp.code, api_resp.msg))
                    }
                }
                Err(e) => Err(e),
            };
            tx.send(result).ok();
        });
    }

    /// 异步执行登录
    pub fn trigger_login(&mut self) {
        if self.login_loading || self.login_rx.is_some() {
            return;
        }
        self.login_loading = true;
        self.login_error.clear();

        let api = self.api.clone();
        let username = self.login_username.clone();
        let password = self.login_password.clone();
        let code = if self.login_captcha_enabled {
            Some(self.login_captcha.clone())
        } else {
            None
        };
        let uuid = if self.login_captcha_enabled {
            Some(self.login_captcha_uuid.clone())
        } else {
            None
        };

        let (tx, rx) = oneshot::channel();
        self.login_rx = Some(rx);

        tokio::spawn(async move {
            let result = async {
                // 1. 登录获取 token
                let login_resp = api.desktop_login(&username, &password, code, uuid).await?;
                let token = login_resp.data.as_ref()
                    .map(|d| d.token.clone())
                    .unwrap_or_default();
                if token.is_empty() {
                    anyhow::bail!("登录响应缺少 token");
                }
                api.set_token(token.clone()).await;

                // 2. 获取用户信息（含封禁检查）
                let info_resp = api.get_user_info().await?;
                let info_data = info_resp.data.ok_or_else(|| anyhow::anyhow!("获取用户信息失败"))?;

                // 检查账号状态
                if info_data.user.status == "0" {
                    api.clear_token().await;
                    anyhow::bail!("账号已被封禁，无法登录");
                }

                Ok(LoginResult {
                    token,
                    user: info_data.user,
                })
            }.await;

            tx.send(result).ok();
        });
    }

    /// 异步加载通知列表
    pub fn trigger_load_notifications(&mut self) {
        let api = self.api.clone();
        let user_id = self.current_user.as_ref().map(|u| u.user_id).unwrap_or(0);
        self.notifications.load(move || async move {
            let resp = api.get_popup_notifications(user_id).await?;
            resp.data.ok_or_else(|| anyhow::anyhow!("通知数据为空"))
        });
    }

    /// 异步加载下载记录
    pub fn trigger_load_download_records(&mut self) {
        let api = self.api.clone();
        let user_id = self.current_user.as_ref().map(|u| u.user_id).unwrap_or(0);
        self.download_records.load(move || async move {
            let params = PageParams { page_num: 1, page_size: 20, ..Default::default() };
            let resp = api.get_download_list(user_id, &params).await?;
            Ok(resp.rows)
        });
    }

    /// 异步加载最近文件（首页）
    pub fn trigger_load_recent_files(&mut self) {
        let api = self.api.clone();
        self.recent_files.load(move || async move {
            let params = PageParams { page_num: 1, page_size: 10, ..Default::default() };
            let resp = api.get_file_list(&params).await?;
            Ok(resp.rows)
        });
    }

    /// 异步加载用户统计（客户端聚合模式，参考 PezMax-Desktop）：
    /// - downloadCount: 下载记录列表 total
    /// - favoriteCount: 文件收藏列表 total + 书签收藏列表 total
    /// - uploadCount: getInfo 返回的 uploadCount
    pub fn trigger_load_user_stats(&mut self) {
        let api = self.api.clone();
        let user_id = self.current_user.as_ref().map(|u| u.user_id).unwrap_or(0);
        if user_id == 0 {
            return;
        }
        self.user_stats_data.load(move || {
            let api = api.clone();
            async move {
                let page_params = crate::api::models::PageParams {
                    page_size: 1,
                    ..Default::default()
                };
                // 并行：文件收藏 + 书签收藏 + 下载列表（仅取 total）
                let (fav_res, bm_fav_res, dl_res, info_res) = tokio::join!(
                    api.get_favorite_list(user_id, &page_params),
                    api.get_bookmark_favorite_list(user_id, &page_params),
                    api.get_download_list(user_id, &page_params),
                    api.get_desktop_user_info(),
                );
                let favorite_count = fav_res.as_ref().map(|r| r.total).unwrap_or(0)
                    + bm_fav_res.as_ref().map(|r| r.total).unwrap_or(0);
                let download_count = dl_res.as_ref().map(|r| r.total).unwrap_or(0);
                let upload_count = info_res
                    .as_ref()
                    .ok()
                    .and_then(|r| r.data.as_ref())
                    .map(|u| u.upload_count)
                    .unwrap_or(0);
                Ok(crate::api::models::UserStats {
                    favorite_count,
                    download_count,
                    upload_count,
                })
            }
        });
    }

    /// 异步加载用户头像
    pub fn trigger_load_avatar(&mut self) {
        let avatar_url = self.current_user.as_ref().map(|u| u.avatar.clone()).unwrap_or_default();
        if avatar_url.is_empty() || self.avatar_load_rx.is_some() {
            return;
        }
        let api = self.api.clone();
        let (tx, rx) = oneshot::channel();
        self.avatar_load_rx = Some(rx);
        tokio::spawn(async move {
            let result = api.download_raw_url(&avatar_url).await;
            tx.send(result).ok();
        });
    }

    /// 异步加载密保问题（账号设置用）
    pub fn trigger_load_security_questions(&mut self) {
        let api = self.api.clone();
        tokio::spawn(async move {
            let _ = api.get_security().await;
        });
    }

    /// 异步加载文件列表（浏览页）——分页拉取全量数据
    pub fn trigger_load_file_list(&mut self) {
        let api = self.api.clone();
        self.file_list_data.load(move || async move {
            const PAGE_SIZE: i32 = 100;
            let mut all = Vec::new();
            let mut page_num = 1i32;
            loop {
                let params = PageParams { page_num, page_size: PAGE_SIZE, ..Default::default() };
                let resp = api.get_file_list(&params).await?;
                if resp.code != 200 {
                    return Err(anyhow::anyhow!("服务器错误 {}: {}", resp.code, resp.msg));
                }
                let fetched = resp.rows.len() as i64;
                all.extend(resp.rows);
                // 已取完：本页不足 PAGE_SIZE，或已达到 total
                if fetched < PAGE_SIZE as i64 || all.len() as i64 >= resp.total {
                    break;
                }
                page_num += 1;
            }
            Ok(all)
        });
    }

    /// 异步加载学科列表
    pub fn trigger_load_subjects(&mut self) {
        let api = self.api.clone();
        self.subjects_data.load(move || async move {
            let resp = api.get_subjects(None).await?;
            if resp.code != 200 {
                return Err(anyhow::anyhow!("学科列表错误 {}: {}", resp.code, resp.msg));
            }
            resp.data.ok_or_else(|| anyhow::anyhow!("学科列表为空"))
        });
    }

    /// 异步加载学校列表
    pub fn trigger_load_schools(&mut self) {
        let api = self.api.clone();
        self.schools_data.load(move || async move {
            let resp = api.get_schools(None).await?;
            if resp.code != 200 {
                return Err(anyhow::anyhow!("学校列表错误 {}: {}", resp.code, resp.msg));
            }
            resp.data.ok_or_else(|| anyhow::anyhow!("学校列表为空"))
        });
    }

    /// 异步加载书签列表
    pub fn trigger_load_bookmarks(&mut self) {
        let api = self.api.clone();
        self.bookmarks_data.load(move || async move {
            let params = PageParams { page_num: 1, page_size: 50, ..Default::default() };
            let resp = api.get_bookmark_list(&params).await?;
            if resp.code != 200 {
                return Err(anyhow::anyhow!("书签列表错误 {}: {}", resp.code, resp.msg));
            }
            Ok(resp.rows)
        });
    }

    /// 异步加载收藏列表
    pub fn trigger_load_favorites(&mut self) {
        let api = self.api.clone();
        let user_id = self.current_user.as_ref().map(|u| u.user_id).unwrap_or(0);
        self.favorites_data.load(move || async move {
            let params = PageParams { page_num: 1, page_size: 200, ..Default::default() };
            let resp = api.get_favorite_list(user_id, &params).await?;
            if resp.code != 200 {
                return Err(anyhow::anyhow!("收藏列表错误 {}: {}", resp.code, resp.msg));
            }
            Ok(resp.rows)
        });
    }

    /// 异步加载收藏 ID 集合（用于工具栏按钮状态，轻量级：pageSize=200 取全量 ID）
    pub fn trigger_load_favorite_ids(&mut self) {
        let api = self.api.clone();
        let user_id = self.current_user.as_ref().map(|u| u.user_id).unwrap_or(0);
        if user_id == 0 {
            return;
        }
        let (tx, rx) = oneshot::channel();
        tokio::spawn(async move {
            let params = PageParams { page_num: 1, page_size: 200, ..Default::default() };
            let result = match api.get_favorite_list(user_id, &params).await {
                Ok(resp) => Ok(resp.rows.into_iter().map(|r| r.file_id).collect::<std::collections::HashSet<i64>>()),
                Err(e) => Err(e),
            };
            tx.send(result).ok();
        });
        self.favorite_ids_rx = Some(rx);
    }

    /// 异步加载书签收藏 ID 集合（用于列表星标状态）
    pub fn trigger_load_bookmark_favorite_ids(&mut self) {
        let api = self.api.clone();
        let user_id = self.current_user.as_ref().map(|u| u.user_id).unwrap_or(0);
        if user_id == 0 {
            return;
        }
        let (tx, rx) = oneshot::channel();
        tokio::spawn(async move {
            let params = PageParams { page_num: 1, page_size: 200, ..Default::default() };
            let result = match api.get_bookmark_favorite_list(user_id, &params).await {
                Ok(resp) => Ok(resp.rows.into_iter().map(|r| r.bookmark_id).collect::<std::collections::HashSet<i64>>()),
                Err(e) => Err(e),
            };
            tx.send(result).ok();
        });
        self.bookmark_favorite_ids_rx = Some(rx);
    }

    /// 异步加载上传排行榜
    pub fn trigger_load_user_rank(&mut self) {
        let api = self.api.clone();
        self.user_rank_data.load(move || async move {
            let resp = api.get_user_rank().await?;
            resp.data.ok_or_else(|| anyhow::anyhow!("排行榜数据为空"))
        });
    }

    /// 异步加载排行用户头像（支持 GIF 动图，带磁盘缓存）
    /// 每帧只处理一次真正的加载，避免重复触发
    pub fn trigger_load_rank_avatars(&mut self, items: &[UserRankItem]) {
        // 创建通道（如果尚未创建）
        if self.rank_avatar_tx.is_none() || self.rank_avatar_rx.is_none() {
            let (tx, rx) = mpsc::unbounded_channel();
            self.rank_avatar_tx = Some(tx);
            self.rank_avatar_rx = Some(rx);
        }
        let tx = self.rank_avatar_tx.clone().unwrap();
        let cache_dir = avatar_cache_dir();

        for item in items {
            if item.avatar.is_empty() {
                continue;
            }
            let user_id = item.user_id;
            // 跳过已加载、已失败、加载中的
            if self.rank_avatar_textures.contains_key(&user_id)
                || self.rank_avatar_failed.contains(&user_id)
                || self.rank_avatar_pending.contains(&user_id)
            {
                continue;
            }
            // 标记为加载中
            self.rank_avatar_pending.insert(user_id);

            // 尝试从磁盘缓存加载
            let cache_path = cache_dir.join(format!("{}.cache", user_id));
            if cache_path.exists() {
                if let Ok(bytes) = std::fs::read(&cache_path) {
                    if !bytes.is_empty() {
                        let tx = tx.clone();
                        tokio::spawn(async move {
                            tx.send((user_id, Ok(bytes))).ok();
                        });
                        continue;
                    }
                }
            }
            // 下载
            let avatar_url = item.avatar.clone();
            let tx = tx.clone();
            let api = self.api.clone();
            tokio::spawn(async move {
                let result = api.download_raw_url(&avatar_url).await;
                // 下载成功时，保存到磁盘缓存
                if let Ok(ref bytes) = result {
                    if !bytes.is_empty() {
                        let _ = std::fs::write(&cache_path, bytes);
                    }
                }
                tx.send((user_id, result)).ok();
            });
        }
    }

    /// 处理单个排行头像的下载结果，支持 GIF 动图解码
    /// 返回值表示是否成功处理
    fn process_rank_avatar_result(&mut self, ctx: &egui::Context, user_id: i64, bytes: Vec<u8>) -> bool {
        // 从 pending 中移除
        self.rank_avatar_pending.remove(&user_id);

        // 检测是否为 GIF（魔术字节 47 49 46 = "GIF"）
        let is_gif = bytes.len() > 6 && bytes[0] == 0x47 && bytes[1] == 0x49 && bytes[2] == 0x46;

        if is_gif {
            // ── 尝试 GIF 解码 ──────────────────────────────────────
            use image::codecs::gif::GifDecoder;
            use image::AnimationDecoder;
            use std::io::Cursor;

            match GifDecoder::new(Cursor::new(&bytes)) {
                Ok(decoder) => {
                    match decoder.into_frames().collect_frames() {
                        Ok(frames) if !frames.is_empty() => {
                            let mut textures = Vec::with_capacity(frames.len());
                            let mut delays = Vec::with_capacity(frames.len());
                            for frame in &frames {
                                let rgba = frame.buffer();
                                let (w, h) = rgba.dimensions();
                                if w == 0 || h == 0 { continue; }
                                let pixels = rgba.clone().into_raw();
                                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                    [w as usize, h as usize], &pixels,
                                );
                                let tex_name = format!("rank_avatar_{}_f{}", user_id, textures.len());
                                textures.push(ctx.load_texture(
                                    &tex_name, color_image, egui::TextureOptions::LINEAR,
                                ));
                                let delay: std::time::Duration = frame.delay().into();
                                delays.push(delay.as_secs_f32().max(0.05));
                            }
                            if !textures.is_empty() {
                                self.rank_avatar_textures.insert(user_id, textures);
                                self.rank_avatar_delays.insert(user_id, delays);
                                self.rank_avatar_frame_idx.insert(user_id, 0);
                                self.rank_avatar_timer.insert(user_id, 0.0);
                                return true;
                            }
                        }
                        Ok(_) => {} // 空帧 → fallthrough 到静态解码
                        Err(e) => {
                            log::info!("GIF 帧解码失败 (user={}): {}，尝试静态解码", user_id, e);
                        }
                    }
                }
                Err(e) => {
                    log::info!("GIF 解码器创建失败 (user={}): {}，尝试静态解码", user_id, e);
                }
            }
            // GIF 解码失败 → 降级为静态图片解码
        }

        // ── 静态图片解码（GIF 降级也走这里） ──────────────────────
        match image::load_from_memory(&bytes) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                if w > 0 && h > 0 {
                    let pixels = rgba.into_raw();
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                        [w as usize, h as usize], &pixels,
                    );
                    let tex_name = format!("rank_avatar_{}", user_id);
                    let tex = ctx.load_texture(&tex_name, color_image, egui::TextureOptions::LINEAR);
                    self.rank_avatar_textures.insert(user_id, vec![tex]);
                    return true;
                }
            }
            Err(e) => {
                log::info!("静态头像解码失败 (user={}): {}", user_id, e);
            }
        }

        // 所有解码方式都失败
        self.rank_avatar_failed.insert(user_id);
        false
    }

    /// 处理书签封面下载结果
    fn process_bookmark_cover_result(&mut self, ctx: &egui::Context, bookmark_id: i64, bytes: &[u8]) {
        log::info!("处理书签封面 {}, {} bytes", bookmark_id, bytes.len());
        match image::load_from_memory(bytes) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                if w > 0 && h > 0 {
                    let pixels = rgba.into_raw();
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                        [w as usize, h as usize], &pixels,
                    );
                    let tex_name = format!("bookmark_cover_{}", bookmark_id);
                    let tex = ctx.load_texture(&tex_name, color_image, egui::TextureOptions::LINEAR);
                    self.bookmark_cover_textures.insert(bookmark_id, tex);
                }
            }
            Err(e) => {
                log::info!("书签封面解码失败 (bookmark={}): {}", bookmark_id, e);
            }
        }
    }

    /// 异步加载我的举报列表
    pub fn trigger_load_my_reports(&mut self) {
        let api = self.api.clone();
        self.my_reports_data.load(move || async move {
            let params = PageParams { page_num: 1, page_size: 20, ..Default::default() };
            let resp = api.get_report_list(&params).await?;
            Ok(resp.rows)
        });
    }

    /// 切换顶级 Section（默认跳到该 Section 的第一个子标签）
    pub fn navigate_section(&mut self, section: Section) {
        if self.current_section != section {
            self.page_enter_anim = SpringAnim::with_target(0.4, 0.8, 0.0, 0.0, 1.0);
            self.sidebar_indicator_anim.set_target(section.index() as f64);
            self.subtab_indicator_anim.set_target(0.0);
            self.browse_selected_idx = None;
        }
        self.current_section = section;
        self.current_subsection = section.default_subsection();
    }

    /// 直接跳转到指定 Section + Subsection
    pub fn navigate_to(&mut self, section: Section, sub: Subsection) {
        if self.current_section != section {
            self.page_enter_anim = SpringAnim::with_target(0.4, 0.8, 0.0, 0.0, 1.0);
            self.sidebar_indicator_anim.set_target(section.index() as f64);
            self.subtab_indicator_anim.set_target(0.0);
        }
        let sub_idx = section.subsections()
            .iter()
            .position(|&(s, _)| s == sub)
            .unwrap_or(0) as f64;
        self.subtab_indicator_anim.set_target(sub_idx);
        self.current_section = section;
        self.current_subsection = sub;
    }

    /// 切换当前 Section 内的子标签（带弹簧动画）
    pub fn navigate_subsection(&mut self, sub: Subsection) {
        let sub_idx = self.current_section.subsections()
            .iter()
            .position(|&(s, _)| s == sub)
            .unwrap_or(0) as f64;
        self.subtab_indicator_anim.set_target(sub_idx);
        self.current_subsection = sub;
    }

    /// 切换认证子页面（触发淡入动画）
    pub fn set_auth_page(&mut self, page: AuthPage) {
        self.auth_anim = Progress::with_easing(0.2, Easing::EaseOutCubic);
        self.auth_anim.set_target(1.0);
        self.auth_page = page;
    }

    /// 添加 Toast 通知（最多同时显示 3 条）
    pub fn add_toast(&mut self, message: impl Into<String>, level: ToastLevel) {
        self.toasts.push(AnimatedToast::new(message, level));
        if self.toasts.len() > 3 {
            self.toasts.remove(0);
        }
    }

    /// 登出：清除凭证和 token
    pub fn logout(&mut self) {
        self.is_logged_in = false;
        self.auth_page = AuthPage::Login;
        self.token = None;
        self.current_user = None;
        self.user_stats = None;
        self.favorite_file_ids.clear();
        self.favorite_ids_loaded = false;
        self.favorite_ids_rx = None;
        self.avatar_texture = None;
        self.avatar_load_rx = None;
        self.rank_avatar_textures.clear();
        self.rank_avatar_delays.clear();
        self.rank_avatar_timer.clear();
        self.rank_avatar_frame_idx.clear();
        self.rank_avatar_rx = None;
        self.rank_avatar_tx = None;
        self.rank_avatar_failed.clear();
        self.rank_avatar_pending.clear();
        clear_credentials();
        // 清除 API token
        let api = self.api.clone();
        tokio::spawn(async move {
            api.clear_token().await;
        });
        // 重置异步数据
        self.notifications = AsyncData::new();
        self.download_records = AsyncData::new();
        self.recent_files = AsyncData::new();
        self.user_stats_data = AsyncData::new();
        self.user_rank_data = AsyncData::new();
        self.my_reports_data = AsyncData::new();
    }

    /// 异步加载 PDF 文件字节（用于预览）
    pub fn trigger_load_pdf_bytes(&mut self, file_id: i64) {
        if self.pdf_loading || self.pdf_bytes_rx.is_some() {
            return;
        }
        self.pdf_loading = true;
        let api = self.api.clone();
        let (tx, rx) = oneshot::channel();
        self.pdf_bytes_rx = Some(rx);
        tokio::spawn(async move {
            let result = api.download_paper(file_id).await;
            tx.send(result.map_err(|e| anyhow::anyhow!("下载 PDF 失败: {}", e))).ok();
        });
    }

    /// 4s 后触发离场动画，4.7s 后移除
    pub fn cleanup_toasts(&mut self) {
        let now = std::time::Instant::now();
        for t in &mut self.toasts {
            if !t.exit_triggered
                && now.duration_since(t.created_at).as_secs_f32() > 4.0
            {
                t.exit_triggered = true;
                t.exit.set_target(1.0);
            }
        }
        self.toasts
            .retain(|t| now.duration_since(t.created_at).as_secs_f32() < 4.7);
    }
}

impl eframe::App for PezMaxApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // 首次帧：记录 DPI/缩放信息以帮助诊断跨平台文字清晰度
        use std::sync::atomic::{AtomicBool, Ordering};
        static LOGGED_DPI: AtomicBool = AtomicBool::new(false);
        if !LOGGED_DPI.swap(true, Ordering::Relaxed) {
            let pp = ctx.pixels_per_point();
            let native_pp = ctx.input(|i| i.viewport().native_pixels_per_point);
            log::info!(
                "DPI info: pixels_per_point={}, native_pixels_per_point={:?}",
                pp, native_pp
            );
        }

        // 外观同步：每帧解析有效深色模式 + 强调色，变化时触发过渡动画
        let effective_dark = match self.theme_mode {
            theme::ThemeMode::Light  => false,
            theme::ThemeMode::Dark   => true,
            theme::ThemeMode::System => theme::effective_dark(ctx),
        };

        // 强调色变化 → 启动平滑过渡（MetroAnim 驱动 0.3s）
        if theme::accent_idx() != self.accent_idx {
            theme::start_accent_transition(self.accent_idx);
            theme::set_accent(self.accent_idx);
        }
        // 深色模式变化 → 启动平滑过渡（MetroAnim 驱动 0.3s）
        if theme::is_dark() != effective_dark {
            theme::start_dark_transition(effective_dark);
            theme::set_dark(effective_dark);
        }
        // 任意过渡（强调色/深色）进行中 → 每帧刷新主题（颜色插值）并保持重绘
        if theme::is_transitioning() || theme::is_dark_transitioning() {
            theme::apply_metro_theme(ctx);
            ctx.request_repaint();
        }

        let dt = ctx.input(|i| i.stable_dt) as f64;

        // 每帧推进所有动画状态
        self.sidebar_anim.update(dt);
        self.sidebar_indicator_anim.update(dt);
        self.subtab_indicator_anim.update(dt);
        self.preview_anim.update(dt);
        self.bookmark_detail_anim.update(dt);
        self.page_enter_anim.update(dt);
        self.auth_anim.update(dt);
        self.search_hint_anim.update(dt);
        self.pdf_viewer.update_animations(dt);
        theme::update_accent_transition(dt);
        theme::update_dark_transition(dt);
        for toast in &mut self.toasts {
            toast.enter.update(dt);
            toast.exit.update(dt);
        }

        // 轮询 PDF 渲染结果
        self.pdf_viewer.poll_render(&self.pdf_engine, ctx);

        // 轮询头像下载结果
        if let Some(rx) = &mut self.avatar_load_rx {
            if let Ok(result) = rx.try_recv() {
                self.avatar_load_rx = None;
                match result {
                    Ok(bytes) => {
                        if let Ok(img) = image::load_from_memory(&bytes) {
                            let rgba = img.to_rgba8();
                            let (w, h) = rgba.dimensions();
                            if w > 0 && h > 0 {
                                let pixels = rgba.into_raw();
                                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                    [w as usize, h as usize],
                                    &pixels,
                                );
                                self.avatar_image_size = Some((w as usize, h as usize));
                                self.avatar_texture = Some(ctx.load_texture(
                                    "user_avatar",
                                    color_image,
                                    egui::TextureOptions::LINEAR,
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        log::info!("头像加载失败: {}", e);
                    }
                }
            }
        }

        // 轮询收藏 ID 列表加载结果
        if let Some(rx) = &mut self.favorite_ids_rx {
            if let Ok(result) = rx.try_recv() {
                self.favorite_ids_rx = None;
                match result {
                    Ok(ids) => {
                        self.favorite_file_ids = ids;
                        self.favorite_ids_loaded = true;
                    }
                    Err(e) => {
                        log::info!("收藏 ID 列表加载失败: {}", e);
                    }
                }
            }
        }

        // 轮询书签收藏 ID 列表加载结果
        if let Some(rx) = &mut self.bookmark_favorite_ids_rx {
            if let Ok(result) = rx.try_recv() {
                self.bookmark_favorite_ids_rx = None;
                match result {
                    Ok(ids) => {
                        self.favorite_bookmark_ids = ids;
                    }
                    Err(e) => {
                        log::info!("书签收藏 ID 列表加载失败: {}", e);
                    }
                }
            }
        }

        // 轮询排行头像下载结果（先收集再处理，避免双重 mutable borrow）
        let mut results: Vec<(i64, Vec<u8>)> = Vec::new();
        if let Some(rx) = &mut self.rank_avatar_rx {
            while let Ok((user_id, result)) = rx.try_recv() {
                match result {
                    Ok(bytes) => results.push((user_id, bytes)),
                    Err(e) => log::info!("排行头像加载失败 (user={}): {}", user_id, e),
                }
            }
        }
        for (user_id, bytes) in results {
            self.process_rank_avatar_result(ctx, user_id, bytes);
        }

        // 轮询书签封面加载结果（详情页）
        if let Some(rx) = &mut self.bookmark_cover_rx {
            if let Ok(result) = rx.try_recv() {
                let pending_id = self.bookmark_cover_pending_id.take();
                self.bookmark_cover_rx = None;
                match result {
                    Ok(bytes) => {
                        if let Some(id) = pending_id {
                            self.process_bookmark_cover_result(ctx, id, &bytes);
                        }
                    }
                    Err(e) => {
                        log::info!("书签封面加载失败: {}", e);
                    }
                }
            }
        }

        // 轮询书签封面批量加载结果（列表页，mpsc channel）
        // 先收集再处理，避免双重可变借用
        {
            let mut cover_results: Vec<(i64, Vec<u8>)> = Vec::new();
            if let Some(rx) = &mut self.bookmark_cover_bulk_rx {
                while let Ok((id, result)) = rx.try_recv() {
                    match result {
                        Ok(bytes) => cover_results.push((id, bytes)),
                        Err(e) => {
                            log::info!("书签封面加载失败 (bookmark={}): {}", id, e);
                        }
                    }
                }
            }
            for (id, bytes) in cover_results {
                self.process_bookmark_cover_result(ctx, id, &bytes);
            }
        }

        // 更新 GIF 动图帧
        if !self.rank_avatar_delays.is_empty() {
            for (&user_id, delays) in &self.rank_avatar_delays.clone() {
                if delays.len() <= 1 {
                    continue; // 单帧不需要动画
                }
                let timer = self.rank_avatar_timer.entry(user_id).or_insert(0.0);
                *timer += dt as f32;
                let current_delay = delays[*self.rank_avatar_frame_idx.get(&user_id).unwrap_or(&0)];
                if *timer >= current_delay {
                    *timer = 0.0;
                    let idx = self.rank_avatar_frame_idx.entry(user_id).or_insert(0);
                    *idx = (*idx + 1) % delays.len();
                }
            }
            ctx.request_repaint();
        }

        // 轮询 PDF 字节下载结果
        if let Some(rx) = &mut self.pdf_bytes_rx {
            if let Ok(result) = rx.try_recv() {
                self.pdf_loading = false;
                self.pdf_bytes_rx = None;
                match result {
                    Ok(bytes) => {
                        self.pdf_viewer.load_document(&self.pdf_engine, bytes, ctx);
                    }
                    Err(e) => {
                        log::error!("PDF 下载失败: {}", e);
                        self.pdf_viewer.error = Some(e.to_string());
                        self.pdf_viewer.loaded = true;
                    }
                }
            }
        }

        // 有动画进行时持续请求重绘
        if !self.sidebar_anim.is_steady()
            || !self.sidebar_indicator_anim.is_steady()
            || !self.subtab_indicator_anim.is_steady()
            || !self.preview_anim.is_steady()
            || !self.bookmark_detail_anim.is_steady()
            || !self.page_enter_anim.is_steady()
            || !self.auth_anim.is_steady()
            || !self.search_hint_anim.is_steady()
            || theme::is_transitioning()
            || self.pdf_viewer.is_animating()
            || self.pdf_viewer.is_loading()
            || self.toasts.iter().any(|t| !t.enter.is_steady() || !t.exit.is_steady())
            || self.avatar_load_rx.is_some()
            || self.rank_avatar_rx.is_some()
            || self.bookmark_cover_rx.is_some()
            || self.bookmark_cover_bulk_rx.is_some()
            || self.rank_avatar_delays.values().any(|d| d.len() > 1)
        {
            ctx.request_repaint();
        }

        self.cleanup_toasts();

        // 账号设置消息 3 秒后自动消失
        if self.account_edit_message_timer > 0.0 {
            self.account_edit_message_timer -= dt as f32;
            if self.account_edit_message_timer <= 0.0 {
                self.account_edit_error.clear();
                self.account_edit_success.clear();
                self.account_edit_message_timer = 0.0;
            }
        }

        // 轮询异步结果

        // 验证码加载结果
        if let Some(rx) = &mut self.captcha_rx {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(captcha) => {
                        self.login_captcha_enabled = captcha.captcha_enabled;
                        self.login_captcha_uuid = captcha.uuid;
                        self.login_captcha_img = captcha.img;
                        self.captcha_loaded = true;
                        // 解码验证码图片
                        if !self.login_captcha_img.is_empty() {
                            if let Some(texture) = decode_base64_image(&self.login_captcha_img, ctx) {
                                self.login_captcha_texture = Some(texture);
                            }
                        }
                    }
                    Err(e) => {
                        self.login_error = format!("验证码加载失败: {}", e);
                        self.captcha_loaded = true;
                    }
                }
                self.captcha_rx = None;
            }
        }

        // 登录结果
        if let Some(rx) = &mut self.login_rx {
            if let Ok(result) = rx.try_recv() {
                self.login_loading = false;
                match result {
                    Ok(login_result) => {
                        self.token = Some(login_result.token.clone());
                        self.current_user = Some(login_result.user);
                        self.login_success();
                        self.add_toast("登录成功，欢迎回来！", crate::app::ToastLevel::Success);
                    }
                    Err(e) => {
                        self.login_error = e.to_string();
                        // 刷新验证码
                        self.captcha_loaded = false;
                        ctx.request_repaint();
                    }
                }
                self.login_rx = None;
            }
        }

        // 轮询异步数据加载器（仅登录后）
        if self.is_logged_in {
            // 自动登录结果轮询
            if let Some(rx) = &mut self.auto_login_rx {
                if let Ok(result) = rx.try_recv() {
                    self.auto_login_rx = None;
                    match result {
                        Ok((user, token)) => {
                            self.current_user = Some(user);
                            self.token = Some(token);
                            self.login_success();
                        }
                        Err(_) => {
                            // 自动登录失败，清除凭证并退回登录页
                            clear_credentials();
                            self.token = None;
                            self.current_user = None;
                            self.is_logged_in = false;
                            self.auth_page = AuthPage::Login;
                        }
                    }
                }
            }

            self.notifications.poll();
            self.download_records.poll();
            self.recent_files.poll();
            self.user_stats_data.poll();
            self.file_list_data.poll();
            self.subjects_data.poll();
            self.schools_data.poll();
            self.bookmarks_data.poll();
            // 如果书签数据被重置，也重置封面加载标记
            // 如果书签数据被重置，也重置封面加载标记
            if !self.bookmarks_data.is_loading() && !self.bookmarks_data.is_loaded() {
                self.bookmark_covers_triggered = false;
            }
            // 书签数据加载完成后，触发封面加载（仅一次）
            if self.bookmarks_data.is_loaded() && !self.bookmark_covers_triggered {
                self.bookmark_covers_triggered = true;
                let bookmarks = self.bookmarks_data.data.clone();
                if let Some(ref list) = bookmarks {
                    log::info!("触发书签封面加载: {} 条书签", list.len());
                    let cache_dir = bookmark_cover_cache_dir();
                    let (tx, rx) = mpsc::unbounded_channel();
                    for bm in list {
                        if !bm.cover_url.is_empty()
                            && !self.bookmark_cover_requested.contains(&bm.id)
                        {
                            self.bookmark_cover_requested.insert(bm.id);
                            let api = self.api.clone();
                            let id = bm.id;
                            let url = bm.cover_url.clone();
                            let txc = tx.clone();
                            let cache_path = cache_dir.join(format!("bm_cover_{}.cache", id));
                            log::info!("书签 {} 封面URL: {}", id, url);
                            tokio::spawn(async move {
                                // 优先读磁盘缓存
                                if cache_path.exists() {
                                    if let Ok(cached) = std::fs::read(&cache_path) {
                                        if !cached.is_empty() {
                                            log::info!("书签 {} 封面命中磁盘缓存 ({} bytes)", id, cached.len());
                                            let _ = txc.send((id, Ok(cached)));
                                            return;
                                        }
                                    }
                                }
                                let result = api.download_bytes(&url).await;
                                // 成功后写磁盘缓存
                                if let Ok(ref bytes) = result {
                                    if !bytes.is_empty() {
                                        let _ = std::fs::write(&cache_path, bytes);
                                        log::info!("书签 {} 封面下载成功 {} bytes, 已缓存", id, bytes.len());
                                    }
                                } else if let Err(ref e) = result {
                                    log::info!("书签 {} 封面下载失败: {}", id, e);
                                }
                                let _ = txc.send((id, result));
                            });
                        }
                    }
                    drop(tx);
                    self.bookmark_cover_bulk_rx = Some(rx);
                }
            }
            // 处理书签收藏队列
            {
                let user_id = self.current_user.as_ref().map(|u| u.user_id).unwrap_or(0);
                for (bookmark_id, is_add) in std::mem::take(&mut self.bookmark_fav_data) {
                    let api = self.api.clone();
                    tokio::spawn(async move {
                        if is_add {
                            let _ = api.add_bookmark_favorite(user_id, bookmark_id).await;
                        } else {
                            let _ = api.remove_bookmark_favorite(user_id, bookmark_id).await;
                        }
                    });
                }
            }
            self.favorites_data.poll();
            self.user_rank_data.poll();
            // 排行榜数据加载完成后，触发头像加载
            if self.user_rank_data.is_loaded() {
                let items = self.user_rank_data.data.clone();
                if let Some(ref items) = items {
                    self.trigger_load_rank_avatars(items);
                }
            }
            self.my_reports_data.poll();
            // 同步 user_stats_data → user_stats（兼容旧代码）
            if let Some(ref stats) = self.user_stats_data.data {
                self.user_stats = Some(stats.clone());
            }
        }

        // 未登录：全屏认证页面
        if !self.is_logged_in {
            match self.auth_page {
                AuthPage::Login => crate::pages::login::render(self, ctx),
                AuthPage::Register => crate::pages::register::render(self, ctx),
                AuthPage::ForgetPassword => crate::pages::forget_password::render(self, ctx),
            }
            // 认证页切换时叠加白色蒙版淡入
            if !self.auth_anim.is_steady() {
                let overlay_alpha = ((1.0 - self.auth_anim.value() as f32) * 255.0) as u8;
                egui::Area::new(egui::Id::new("auth_fade_overlay"))
                    .order(egui::Order::Foreground)
                    .fixed_pos(egui::pos2(0.0, 0.0))
                    .show(ctx, |ui| {
                        ui.painter().rect_filled(
                            ctx.screen_rect(),
                            egui::CornerRadius::ZERO,
                            egui::Color32::from_rgba_unmultiplied(255, 255, 255, overlay_alpha),
                        );
                    });
            }
            return;
        }

        // 自动登录验证中：显示加载提示
        if self.auto_login_rx.is_some() {
            egui::CentralPanel::default()
                .frame(egui::Frame::new().fill(crate::theme::colors::bg_white()))
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(ui.available_height() * 0.4);
                        ui.label(
                            egui::RichText::new("验证登录状态...")
                                .font(egui::FontId::new(18.0, egui::FontFamily::Proportional))
                                .color(crate::theme::colors::text_secondary()),
                        );
                    });
                });
            ctx.request_repaint();
            return;
        }

        // 已登录：侧边栏 → Topbar → 子标签栏 → 内容区
        crate::components::sidebar::render(self, ctx);
        crate::components::topbar::render(self, ctx);

        // 子标签栏仅 Browse / Community / Profile 显示
        // 在渲染子标签栏之前，处理上帧的"返回阅读"请求
        if self.preview_bar_action == action_bar::Action::Back
            && self.selected_file.is_some()
            && self.current_subsection != Subsection::ResourceManager
        {
            self.navigate_subsection(Subsection::ResourceManager);
            self.preview_bar_action = action_bar::Action::None;
        }
        let section = self.current_section;
        let subsections = section.subsections();
        if !subsections.is_empty() {
            render_subtab_bar(self, ctx, &subsections);
        }

        // 预览模式：底部操作栏（TopBottomPanel 必须在 CentralPanel 之前渲染）
        let preview_mode = self.current_section == Section::Browse && self.selected_file.is_some();
        self.preview_mode = preview_mode;
        self.preview_bar_action = action_bar::Action::None;
        if preview_mode {
            let file_name = self.selected_file.as_ref().map(|f| f.file_name.as_str()).unwrap_or("");
            let file_id = self.selected_file.as_ref().map(|f| f.file_id).unwrap_or(0);
            let is_favorited = self.favorite_file_ids.contains(&file_id);
            let bar_mode = if self.current_subsection == Subsection::ResourceManager {
                action_bar::PreviewMode::Reading
            } else {
                action_bar::PreviewMode::Away
            };
            // 保存当前帧操作栏按钮动作，供下一帧在子标签栏之前处理
            let bar_action = action_bar::render_bar(ctx, file_name, bar_mode, is_favorited);
            // Back + Away：先导航回 ResourceManager，下一帧再处理 Back
            // 其他动作：直接传递给 browse.rs 处理
            if bar_action == action_bar::Action::Back && bar_mode == action_bar::PreviewMode::Away {
                // 标记"返回阅读"，下一帧在子标签渲染前切换回 ResourceManager
                self.preview_bar_action = bar_action;
            } else if bar_action != action_bar::Action::None {
                self.preview_bar_action = bar_action;
            }
        }

        // 内容区
        let enter_v = self.page_enter_anim.value();
        egui::CentralPanel::default()
            .frame(egui::Frame::new()
                .fill(theme::colors::bg_white())
                .inner_margin(if preview_mode { egui::Margin::ZERO } else { egui::Margin::symmetric(20, 0) })
                .stroke(egui::Stroke::NONE),
            )
            .show(ctx, |ui| {
                if !self.page_enter_anim.is_steady() {
                    let offset = map_range(enter_v, 20.0, 0.0) as f32;
                    let alpha = map_range(enter_v, 0.4, 1.0) as f32;
                    ui.set_opacity(alpha.clamp(0.0, 1.0));
                    if offset > 0.1 {
                        ui.add_space(offset);
                    }
                }

                match self.current_section {
                    Section::Home => crate::pages::home::render(self, ui),
                    Section::Browse => match self.current_subsection {
                        Subsection::ExternalBookmarks => {
                            crate::pages::browse::render_bookmarks(self, ui)
                        }
                        Subsection::MyFavorites => {
                            crate::pages::browse::render_favorites(self, ui)
                        }
                        _ => crate::pages::browse::render_resource_manager(self, ui),
                    },
                    Section::Community => match self.current_subsection {
                        Subsection::ContributeFile => {
                            crate::pages::community::render_contribute_file(self, ui)
                        }
                        Subsection::ReportRecord => {
                            crate::pages::community::render_report_record(self, ui)
                        }
                        _ => crate::pages::community::render_user_ranking(self, ui),
                    },
                    Section::Profile => match self.current_subsection {
                        Subsection::Notifications => {
                            crate::pages::profile::render_notifications(self, ui)
                        }
                        Subsection::DownloadHistory => {
                            crate::pages::profile::render_download_history(self, ui)
                        }
                        Subsection::AppSettings => {
                            crate::pages::profile::render_app_settings(self, ui)
                        }
                        _ => crate::pages::profile::render_personal_center(self, ui),
                    },
                }
            });

        crate::components::toast::render(self, ctx);
    }
}

/// 子标签栏（Browse / Community / Profile 功能区内的水平标签）
fn render_subtab_bar(
    app: &mut PezMaxApp,
    ctx: &egui::Context,
    subsections: &[(Subsection, &'static str)],
) {
    use theme::colors;

    egui::TopBottomPanel::top("subtab_bar")
        .min_height(44.0)
        .max_height(44.0)
        .show_separator_line(false)
        .frame(
            egui::Frame::new()
                .fill(colors::bg_white())
                .inner_margin(egui::Margin::ZERO)
                .stroke(egui::Stroke::NONE),
        )
        .show(ctx, |ui| {
            // 收集各 tab 的 rect，之后用于插值下划线位置
            let mut tab_rects: Vec<egui::Rect> = Vec::with_capacity(subsections.len());

            ui.horizontal(|ui| {
                ui.add_space(16.0);
                for (i, &(sub, label)) in subsections.iter().enumerate() {
                    let is_active = app.current_subsection == sub;
                    let text_color = if is_active {
                        colors::primary()
                    } else {
                        colors::text_secondary()
                    };

                    // 用 allocate_ui 隔离样式，彻底清除任何背景框
                    let resp = ui.allocate_ui(egui::vec2(80.0, 36.0), |ui| {
                        ui.style_mut().visuals.widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
                        ui.style_mut().visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
                        ui.style_mut().visuals.widgets.hovered = ui.style().visuals.widgets.inactive.clone();
                        ui.style_mut().visuals.widgets.active = ui.style().visuals.widgets.inactive.clone();
                        ui.add(
                            egui::Button::new(
                                egui::RichText::new(label)
                                    .font(egui::FontId::new(14.0, egui::FontFamily::Proportional))
                                    .color(text_color),
                            )
                            .min_size(egui::vec2(80.0, 36.0)),
                        )
                    });
                    let resp = resp.inner;
                    tab_rects.push(resp.rect);

                    if resp.clicked() && !is_active {
                        app.navigate_subsection(sub);
                    }
                }
            });

            // 弹簧插值下划线：在两个相邻 tab rect 之间平滑滑动
            if tab_rects.len() >= 2 {
                let idx_f = app.subtab_indicator_anim.value();
                let lo = (idx_f.floor() as usize).min(tab_rects.len() - 1);
                let hi = (idx_f.ceil()  as usize).min(tab_rects.len() - 1);
                let t  = idx_f.fract() as f32;

                let r_lo = tab_rects[lo];
                let r_hi = tab_rects[hi];
                let x0 = egui::lerp(r_lo.left()  + 4.0..=r_hi.left()  + 4.0, t);
                let x1 = egui::lerp(r_lo.right() - 4.0..=r_hi.right() - 4.0, t);
                let y  = r_lo.bottom() - 2.0;
                ui.painter().line_segment(
                    [egui::pos2(x0, y), egui::pos2(x1, y)],
                    egui::Stroke::new(2.0, colors::primary()),
                );
            } else if let Some(&r) = tab_rects.first() {
                // 只有一个 tab：直接画
                ui.painter().line_segment(
                    [egui::pos2(r.left() + 4.0, r.bottom() - 2.0), egui::pos2(r.right() - 4.0, r.bottom() - 2.0)],
                    egui::Stroke::new(2.0, colors::primary()),
                );
            }
        });
}

