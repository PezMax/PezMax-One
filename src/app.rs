use crate::api::client::ApiClient;
use crate::api::models::*;
use crate::cache::CacheManager;
use crate::components::action_bar;
use crate::components::animated_counter::AnimatedCounter;
use crate::pdf::{PdfEngine, PdfViewer};
use crate::settings::AppSettings;
use crate::sokuou::{map_range, EasingMode, Easing, MetroAnim, Progress, SpringAnim, UwpEasing};
use crate::theme;
use anyhow;
use base64::Engine;
use egui::Context;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

/// 平台默认下载目录 `~/Downloads/PezMax`。
pub fn default_download_dir() -> String {
    if let Some(home) = dirs::home_dir() {
        home.join("Downloads").join("PezMax").to_string_lossy().to_string()
    } else {
        "~/Downloads/PezMax".to_string()
    }
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

    /// 重新加载（重置状态后加载，用于刷新已加载的数据）
    pub fn reload<F, Fut>(&mut self, f: F)
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = anyhow::Result<T>> + Send,
    {
        self.reset();
        self.load(f);
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

    // 注册流程（4 步：账号密码 → 3 个密保 → 昵称 → 验证码 & 免责声明）
    pub register_step: i32,
    pub register_username: String,
    pub register_password: String,
    pub register_confirm_password: String,
    pub register_nickname: String,
    pub register_captcha: String,
    pub register_captcha_uuid: String,
    pub register_captcha_img: String,
    pub register_captcha_texture: Option<egui::TextureHandle>,
    pub register_captcha_enabled: bool,
    pub register_security_questions: Vec<SecurityQuestion>,
    pub register_loading: bool,
    pub register_error: String,
    pub register_disclaimer_open: bool,
    pub register_disclaimer_countdown: crate::sokuou::Progress,
    pub register_captcha_rx: Option<oneshot::Receiver<anyhow::Result<CaptchaResponse>>>,
    pub register_rx: Option<oneshot::Receiver<anyhow::Result<()>>>,

    // 找回密码流程（3 步：用户名+验证码 → 三个密保答题 → 新密码）
    pub forget_step: i32,
    pub forget_username: String,
    pub forget_captcha: String,
    pub forget_captcha_uuid: String,
    pub forget_captcha_img: String,
    pub forget_captcha_texture: Option<egui::TextureHandle>,
    pub forget_captcha_enabled: bool,
    pub forget_questions: Vec<SecurityQuestion>, // question 从后端拿，answer 用户填
    pub forget_new_password: String,
    pub forget_confirm_password: String,
    pub forget_loading: bool,
    pub forget_error: String,
    pub forget_captcha_rx: Option<oneshot::Receiver<anyhow::Result<CaptchaResponse>>>,
    pub forget_questions_rx: Option<oneshot::Receiver<anyhow::Result<Vec<SecurityQuestion>>>>,
    pub forget_reset_rx: Option<oneshot::Receiver<anyhow::Result<()>>>,

    // Auth 步骤指示器动画（0..=3）
    pub auth_step_anim: SpringAnim,

    // 异步数据加载器
    pub notifications: AsyncData<Vec<Notification>>,
    pub download_records: AsyncData<Vec<DownloadRecord>>,
    pub recent_files: AsyncData<Vec<PaperFile>>,
    pub user_stats_data: AsyncData<UserStats>,
    // 动画计数器（电表翻动效果）
    pub fav_anim: AnimatedCounter,
    pub dl_anim: AnimatedCounter,
    pub ul_anim: AnimatedCounter,
    // Browse 页面
    pub file_list_data: AsyncData<Vec<PaperFile>>,
    pub subjects_data: AsyncData<Vec<String>>,
    pub schools_data: AsyncData<Vec<String>>,
    pub bookmarks_data: AsyncData<Vec<Bookmark>>,
    pub favorites_data: AsyncData<Vec<FavoriteRecord>>,
    pub bookmark_favorites_data: AsyncData<Vec<BookmarkFavorite>>,
    pub favorites_tab_idx: usize, // 0=试卷收藏, 1=书签收藏
    pub favorites_tab_anim: SpringAnim,
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
    // 子分页切换过渡（内容区左右滑入 + 淡入）
    // value: 0.0 = 起始（新页在偏移位置、半透明）；1.0 = 稳态（无偏移、完全不透明）
    // dir: +1.0 = 从右侧滑入（切到更右侧的分页）；-1.0 = 从左侧滑入
    pub subsection_transition_anim: MetroAnim,
    pub subsection_transition_dir: f32,

    // 浏览状态
    pub search_query: String,
    /// 搜索防抖后的字符串——UI 上 search_query 实时更新，页面过滤/服务器查询
    /// 需读取此字段。停止输入 300ms 后由 update() 同步。
    pub search_query_debounced: String,
    /// search_query 最近一次变化的时刻（秒，来自 ctx.input.time）
    pub search_query_changed_at: Option<f64>,
    // 各 subsection 局部搜索（不走顶栏全局搜索）
    pub download_history_search: String,
    pub paper_favorites_search: String,
    pub bookmark_favorites_search: String,
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
    pub bookmark_detail_rx: Option<oneshot::Receiver<anyhow::Result<Bookmark>>>, // 书签详情异步加载
    pub bookmark_favorite_ids_rx: Option<oneshot::Receiver<anyhow::Result<HashSet<i64>>>>,
    pub bookmark_cover_textures: HashMap<i64, egui::TextureHandle>,  // 书签封面纹理缓存
    pub bookmark_cover_requested: HashSet<i64>,                 // 已请求封面的书签
    pub bookmark_cover_rx: Option<tokio::sync::oneshot::Receiver<anyhow::Result<Vec<u8>>>>,
    pub bookmark_cover_pending_id: Option<i64>,                 // 当前等待中的封面请求 ID
    pub bookmark_cover_bulk_rx: Option<tokio::sync::mpsc::UnboundedReceiver<(i64, anyhow::Result<Vec<u8>>)>>,
    pub bookmark_covers_triggered: bool,  // 防止每帧重复创建封面加载通道
    pub bookmark_title_cache: HashMap<i64, String>,  // 书签标题缓存（bookmark_id → title）
    pub bookmark_title_rx: Option<tokio::sync::mpsc::UnboundedReceiver<(i64, String)>>,
    pub bookmark_title_tx: Option<tokio::sync::mpsc::UnboundedSender<(i64, String)>>,

    // 贡献文件元数据表单
    pub contribute_subject: String,
    pub contribute_school: String,
    pub contribute_year: String,
    pub contribute_file_path: Option<String>,
    // 从选中文件解析出的元数据
    pub contribute_file_name: Option<String>,
    pub contribute_file_format: Option<String>,
    pub contribute_file_size: Option<u64>,
    // 上传流程 rx & 进度动画
    pub contribute_upload_rx: Option<oneshot::Receiver<anyhow::Result<PaperFile>>>,
    pub contribute_uploading: bool,

    // 举报对话框（新 · 覆盖旧的 report_content/report_type）
    pub report_content: String,           // 保留：旧 UI 兼容，后续移除
    pub report_type: String,              // 保留：旧 UI 兼容，后续移除
    pub show_report_dialog: bool,
    pub report_target_file_id: Option<i64>,
    pub report_target_user_id: Option<i64>,
    pub report_target_file_name: String,
    pub report_reason: String,
    pub report_remark: String,
    pub report_submit_rx: Option<oneshot::Receiver<anyhow::Result<()>>>,

    // 举报记录：分页 + 状态筛选 + 时间线弹窗
    pub report_status_filter: Option<i64>, // 0/1/2/3/None
    pub report_page_num: i64,
    pub report_has_more: bool,
    pub selected_report_id: Option<i64>,
    pub show_report_timeline: bool,
    pub report_timeline_data: Option<serde_json::Value>,
    pub report_timeline_rx: Option<oneshot::Receiver<anyhow::Result<serde_json::Value>>>,
    pub report_timeline_anim: SpringAnim,

    // 设置开关
    pub setting_auto_launch: bool,
    pub setting_silent_download: bool,

    // 默认下载路径
    pub setting_download_dir: String,

    // PDF 设置
    pub setting_pdf_view_mode: crate::pdf::ViewMode,
    pub setting_pdf_scale: f32,

    // 关于弹窗
    pub show_about_dialog: bool,

    // 外观：外观模式 + 强调色索引（对应 theme::ACCENT_PRESETS）
    pub theme_mode: theme::ThemeMode,
    pub accent_idx: usize,

    // 缓存管理器 + 设置 + PDF 文件 ID 跟踪
    pub cache_manager: CacheManager,
    pub settings: AppSettings,
    pub pdf_file_id: Option<i64>,

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
    // 头像上传（Ok(None) 表示用户取消选择文件）
    pub avatar_upload_rx: Option<oneshot::Receiver<anyhow::Result<Option<UserInfo>>>>,

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

    // 密码 / 密保编辑的 2 步验证网关
    pub password_verify_step: u8, // 0=verify old, 1=set new
    pub password_verify_rx: Option<oneshot::Receiver<anyhow::Result<bool>>>,
    pub security_verify_step: u8, // 0=verify password → 1=edit
    pub security_preload_rx: Option<oneshot::Receiver<anyhow::Result<Vec<SecurityQuestion>>>>,

    // 通知已读集合（客户端持久化）
    pub read_notification_ids: HashSet<i64>,

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

        // 初始化 CacheManager（创建目录结构，迁移旧缓存）
        let cache_manager = CacheManager::new();
        // 加载本地设置
        let settings = AppSettings::load(&cache_manager);
        // 应用设置到主题全局变量
        theme::set_accent(settings.accent_idx);
        theme::set_dark(matches!(settings.theme_mode, theme::ThemeMode::Dark));

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

            // 注册流程默认
            register_step: 1,
            register_username: String::new(),
            register_password: String::new(),
            register_confirm_password: String::new(),
            register_nickname: String::new(),
            register_captcha: String::new(),
            register_captcha_uuid: String::new(),
            register_captcha_img: String::new(),
            register_captcha_texture: None,
            register_captcha_enabled: true,
            register_security_questions: vec![
                SecurityQuestion { question: String::new(), answer: String::new() },
                SecurityQuestion { question: String::new(), answer: String::new() },
                SecurityQuestion { question: String::new(), answer: String::new() },
            ],
            register_loading: false,
            register_error: String::new(),
            register_disclaimer_open: false,
            register_disclaimer_countdown: Progress::with_easing(1.0, Easing::Linear),
            register_captcha_rx: None,
            register_rx: None,

            // 找回密码流程默认
            forget_step: 1,
            forget_username: String::new(),
            forget_captcha: String::new(),
            forget_captcha_uuid: String::new(),
            forget_captcha_img: String::new(),
            forget_captcha_texture: None,
            forget_captcha_enabled: true,
            forget_questions: vec![],
            forget_new_password: String::new(),
            forget_confirm_password: String::new(),
            forget_loading: false,
            forget_error: String::new(),
            forget_captcha_rx: None,
            forget_questions_rx: None,
            forget_reset_rx: None,

            auth_step_anim: SpringAnim::new(0.3, 0.85, 0.0),

            notifications: AsyncData::new(),
            download_records: AsyncData::new(),
            recent_files: AsyncData::new(),
            user_stats_data: AsyncData::new(),
            fav_anim: AnimatedCounter::new(),
            dl_anim: AnimatedCounter::new(),
            ul_anim: AnimatedCounter::new(),
            file_list_data: AsyncData::new(),
            subjects_data: AsyncData::new(),
            schools_data: AsyncData::new(),
            bookmarks_data: AsyncData::new(),
            favorites_data: AsyncData::new(),
            bookmark_favorites_data: AsyncData::new(),
            favorites_tab_idx: 0,
            favorites_tab_anim: SpringAnim::new(0.3, 0.8, 0.0),
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
            subsection_transition_anim: {
                let mut m = MetroAnim::new(0.28, UwpEasing::Quadratic, EasingMode::EaseOut);
                m.jump_to(1.0);
                m
            },
            subsection_transition_dir: 0.0,
            search_query: String::new(),
            search_query_debounced: String::new(),
            search_query_changed_at: None,
            download_history_search: String::new(),
            paper_favorites_search: String::new(),
            bookmark_favorites_search: String::new(),
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
            bookmark_detail_rx: None,
            bookmark_favorite_ids_rx: None,
            bookmark_cover_textures: HashMap::new(),
            bookmark_cover_requested: HashSet::new(),
            bookmark_cover_rx: None,
            bookmark_cover_pending_id: None,
            bookmark_cover_bulk_rx: None,
            bookmark_covers_triggered: false,
            bookmark_title_cache: HashMap::new(),
            bookmark_title_rx: None,
            bookmark_title_tx: None,
            contribute_subject: String::new(),
            contribute_school: String::new(),
            contribute_year: String::new(),
            contribute_file_path: None,
            contribute_file_name: None,
            contribute_file_format: None,
            contribute_file_size: None,
            contribute_upload_rx: None,
            contribute_uploading: false,
            report_content: String::new(),
            report_type: String::new(),
            show_report_dialog: false,
            report_target_file_id: None,
            report_target_user_id: None,
            report_target_file_name: String::new(),
            report_reason: String::new(),
            report_remark: String::new(),
            report_submit_rx: None,
            report_status_filter: None,
            report_page_num: 1,
            report_has_more: true,
            selected_report_id: None,
            show_report_timeline: false,
            report_timeline_data: None,
            report_timeline_rx: None,
            report_timeline_anim: SpringAnim::new(0.4, 0.8, 0.0),
            show_about_dialog: false,
            setting_auto_launch: settings.setting_auto_launch,
            setting_silent_download: settings.setting_silent_download,
            setting_download_dir: settings
                .download_dir
                .clone()
                .unwrap_or_else(default_download_dir),
            setting_pdf_view_mode: settings.pdf_view_mode,
            setting_pdf_scale: settings.pdf_scale,
            theme_mode: settings.theme_mode,
            accent_idx: settings.accent_idx,

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
            avatar_upload_rx: None,

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

            password_verify_step: 0,
            password_verify_rx: None,
            security_verify_step: 0,
            security_preload_rx: None,

            read_notification_ids: cache_manager.load_read_notifications(),

            pdf_engine,
            pdf_viewer: PdfViewer::new(),
            pdf_loading: false,
            pdf_bytes_rx: None,

            // 新字段
            cache_manager,
            settings,
            pdf_file_id: None,
        };

        // 从缓存加载用户统计（如果有），让首页个人页能立即显示
        app.load_user_stats_cache();

        // 尝试从本地加载凭证并自动登录
        app.try_auto_login();

        app
    }

    /// 尝试从本地加载凭证并自动登录
    pub fn try_auto_login(&mut self) {
        if let Some(creds) = self.cache_manager.load_credentials() {
            self.login_username = creds.username;
            self.login_remember = creds.remember_me;
            // 记住我时把混淆密码解密回填，用户回到登录页可看到已填
            if let Some(cipher) = &creds.password_encrypted {
                if let Some(plain) = crate::api::crypto::deobfuscate(cipher) {
                    self.login_password = plain;
                }
            }
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

        // 保存凭证（如果勾选了"记住我"，同时混淆保存密码用于下次自动填充）
        if self.login_remember {
            if let Some(ref token) = self.token {
                self.cache_manager.save_credentials(
                    token,
                    &self.login_username,
                    true,
                    Some(&self.login_password),
                );
            }
        } else {
            self.cache_manager.clear_credentials();
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

    // ── 注册流程 triggers ────────────────────────────────────────

    /// 独立于 login 的注册页验证码加载
    pub fn trigger_register_captcha(&mut self) {
        if self.register_captcha_rx.is_some() { return; }
        let api = self.api.clone();
        let (tx, rx) = oneshot::channel();
        self.register_captcha_rx = Some(rx);
        tokio::spawn(async move {
            let result = api.get_captcha().await.and_then(|resp| {
                resp.data.ok_or_else(|| anyhow::anyhow!("验证码为空"))
            });
            tx.send(result).ok();
        });
    }

    /// 提交注册
    pub fn trigger_register(&mut self) {
        if self.register_loading || self.register_rx.is_some() { return; }
        self.register_loading = true;
        self.register_error.clear();

        let api = self.api.clone();
        let qs = &self.register_security_questions;
        let req = RegisterRequest {
            username: self.register_username.clone(),
            password: self.register_password.clone(),
            confirm_password: self.register_confirm_password.clone(),
            nickname: if self.register_nickname.is_empty() {
                self.register_username.clone()
            } else {
                self.register_nickname.clone()
            },
            code: if self.register_captcha_enabled { Some(self.register_captcha.clone()) } else { None },
            uuid: if self.register_captcha_enabled { Some(self.register_captcha_uuid.clone()) } else { None },
            security_question_one:   qs[0].question.trim().to_string(),
            security_answer_one:     qs[0].answer.trim().to_string(),
            security_question_two:   qs[1].question.trim().to_string(),
            security_answer_two:     qs[1].answer.trim().to_string(),
            security_question_three: qs[2].question.trim().to_string(),
            security_answer_three:   qs[2].answer.trim().to_string(),
        };

        let (tx, rx) = oneshot::channel();
        self.register_rx = Some(rx);
        tokio::spawn(async move {
            let result = api.desktop_register(&req).await.and_then(|resp| {
                if resp.code == 200 { Ok(()) } else { Err(anyhow::anyhow!("{}", resp.msg)) }
            });
            tx.send(result).ok();
        });
    }

    // ── 找回密码 triggers ────────────────────────────────────────

    pub fn trigger_forget_captcha(&mut self) {
        if self.forget_captcha_rx.is_some() { return; }
        let api = self.api.clone();
        let (tx, rx) = oneshot::channel();
        self.forget_captcha_rx = Some(rx);
        tokio::spawn(async move {
            let result = api.get_captcha().await.and_then(|resp| {
                resp.data.ok_or_else(|| anyhow::anyhow!("验证码为空"))
            });
            tx.send(result).ok();
        });
    }

    /// 拉取该用户名的 3 个密保问题
    pub fn trigger_forget_load_questions(&mut self) {
        if self.forget_questions_rx.is_some() || self.forget_loading { return; }
        self.forget_loading = true;
        self.forget_error.clear();
        let api = self.api.clone();
        let username = self.forget_username.clone();
        let (tx, rx) = oneshot::channel();
        self.forget_questions_rx = Some(rx);
        tokio::spawn(async move {
            let result = api.get_security_questions(&username).await.and_then(|resp| {
                resp.data.ok_or_else(|| anyhow::anyhow!("{}", resp.msg))
            });
            tx.send(result).ok();
        });
    }

    /// 提交密保重置密码
    pub fn trigger_forget_reset(&mut self) {
        if self.forget_reset_rx.is_some() || self.forget_loading { return; }
        self.forget_loading = true;
        self.forget_error.clear();

        // 组装 payload：参考 ref 的 resetPasswordBySecurity({ userName, code, uuid, securityAnswerOne, Two, Three, newPassword })
        let new_pwd = self.forget_new_password.trim();
        let mut payload = serde_json::json!({
            "userName": self.forget_username.trim(),
            "code": self.forget_captcha.trim(),
            "uuid": self.forget_captcha_uuid,
            "newPassword": new_pwd,
            "confirmPassword": new_pwd,
        });
        if let Some(m) = payload.as_object_mut() {
            for (i, q) in self.forget_questions.iter().enumerate() {
                let key = match i {
                    0 => "securityAnswerOne",
                    1 => "securityAnswerTwo",
                    _ => "securityAnswerThree",
                };
                m.insert(key.to_string(), serde_json::Value::String(q.answer.clone()));
            }
        }

        let api = self.api.clone();
        let (tx, rx) = oneshot::channel();
        self.forget_reset_rx = Some(rx);
        tokio::spawn(async move {
            let result = api.reset_password_by_security(&payload).await.and_then(|resp| {
                if resp.code == 200 { Ok(()) } else { Err(anyhow::anyhow!("{}", resp.msg)) }
            });
            tx.send(result).ok();
        });
    }

    /// 清空注册流程（切页/成功后）
    pub fn reset_register_flow(&mut self) {
        self.register_step = 1;
        self.register_username.clear();
        self.register_password.clear();
        self.register_confirm_password.clear();
        self.register_nickname.clear();
        self.register_captcha.clear();
        self.register_captcha_uuid.clear();
        self.register_captcha_texture = None;
        for q in self.register_security_questions.iter_mut() {
            q.question.clear();
            q.answer.clear();
        }
        self.register_error.clear();
        self.register_disclaimer_open = false;
        self.register_disclaimer_countdown.jump_to(0.0);
        self.auth_step_anim.set_target(0.0);
    }

    /// 验证旧密码（密码修改前）
    pub fn trigger_verify_password(&mut self, password: String) {
        if self.password_verify_rx.is_some() { return; }
        let api = self.api.clone();
        let (tx, rx) = oneshot::channel();
        self.password_verify_rx = Some(rx);
        tokio::spawn(async move {
            let result = api.verify_password(&password).await.map(|resp| resp.code == 200);
            tx.send(result).ok();
        });
    }

    /// 预加载现有密保问题
    pub fn trigger_preload_security(&mut self) {
        if self.security_preload_rx.is_some() { return; }
        let api = self.api.clone();
        let (tx, rx) = oneshot::channel();
        self.security_preload_rx = Some(rx);
        tokio::spawn(async move {
            let result = api.get_security().await.and_then(|resp| {
                let data = resp.data.unwrap_or(serde_json::Value::Null);
                // 后端可能返回 { securityQuestionOne, ...Two, ...Three } 或 [] 或 [{question, answer}, ...]
                let mut out: Vec<SecurityQuestion> = Vec::with_capacity(3);
                if let Some(obj) = data.as_object() {
                    for (qk, ak) in [
                        ("securityQuestionOne", "securityAnswerOne"),
                        ("securityQuestionTwo", "securityAnswerTwo"),
                        ("securityQuestionThree", "securityAnswerThree"),
                    ] {
                        let q = obj.get(qk).and_then(|v| v.as_str()).unwrap_or_default().to_string();
                        let a = obj.get(ak).and_then(|v| v.as_str()).unwrap_or_default().to_string();
                        out.push(SecurityQuestion { question: q, answer: a });
                    }
                } else if let Some(arr) = data.as_array() {
                    for item in arr.iter().take(3) {
                        let q = item.get("question").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                        let a = item.get("answer").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                        out.push(SecurityQuestion { question: q, answer: a });
                    }
                }
                while out.len() < 3 {
                    out.push(SecurityQuestion { question: String::new(), answer: String::new() });
                }
                Ok(out)
            });
            tx.send(result).ok();
        });
    }

    /// 打开举报对话框，预填目标文件信息
    pub fn open_report_dialog_for_file(&mut self, file: &PaperFile) {
        self.report_target_file_id = Some(file.file_id);
        self.report_target_file_name = if file.file_name.is_empty() {
            format!("#{}", file.file_id)
        } else {
            file.file_name.clone()
        };
        self.report_target_user_id = None; // 未来可从 file.upload_user_id 拿
        self.report_reason.clear();
        self.report_remark.clear();
        self.show_report_dialog = true;
    }

    /// 提交举报
    pub fn trigger_submit_report(&mut self) {
        if self.report_submit_rx.is_some() { return; }
        let report = Report {
            report_id: 0,
            report_type: "file".to_string(),
            content: self.report_reason.clone(),
            status: String::new(),
            create_time: String::new(),
            file_id: self.report_target_file_id.unwrap_or(0),
            user_id: self.report_target_user_id.unwrap_or(0),
            remark: self.report_remark.clone(),
            file_name: self.report_target_file_name.clone(),
            result: None,
        };
        let api = self.api.clone();
        let (tx, rx) = oneshot::channel();
        self.report_submit_rx = Some(rx);
        tokio::spawn(async move {
            let result = api.create_report(&report).await.and_then(|r| {
                if r.code == 200 { Ok(()) } else { Err(anyhow::anyhow!("{}", r.msg)) }
            });
            tx.send(result).ok();
        });
    }

    /// 拉举报时间线，成功后打开弹窗
    pub fn trigger_load_report_timeline(&mut self, report_id: i64) {
        if self.report_timeline_rx.is_some() { return; }
        self.selected_report_id = Some(report_id);
        let api = self.api.clone();
        let (tx, rx) = oneshot::channel();
        self.report_timeline_rx = Some(rx);
        tokio::spawn(async move {
            let result = api.get_report_timeline(report_id).await.and_then(|r| {
                r.data.ok_or_else(|| anyhow::anyhow!("{}", r.msg))
            });
            tx.send(result).ok();
        });
    }

    /// 贡献文件两阶段上传：先 /datum/file/upload 拿 fileUrl，再 POST /datum/file 建记录。
    pub fn trigger_contribute_upload(&mut self) {
        if self.contribute_uploading || self.contribute_upload_rx.is_some() {
            return;
        }
        let path = match &self.contribute_file_path {
            Some(p) if !p.is_empty() => p.clone(),
            _ => {
                self.add_toast("请先选择文件", crate::app::ToastLevel::Error);
                return;
            }
        };
        let file_name = self.contribute_file_name.clone().unwrap_or_default();
        let file_format = self.contribute_file_format.clone().unwrap_or_else(|| "pdf".to_string());
        let file_size = self.contribute_file_size.unwrap_or(0) as i64;
        let subject = self.contribute_subject.clone();
        let school = self.contribute_school.clone();
        let year: i64 = self.contribute_year.trim().parse().unwrap_or(0);
        let creator = self.current_user.as_ref().map(|u| u.user_name.clone()).unwrap_or_default();

        self.contribute_uploading = true;
        let api = self.api.clone();
        let (tx, rx) = oneshot::channel();
        self.contribute_upload_rx = Some(rx);

        tokio::spawn(async move {
            let result = async {
                // 阶段 1：上传字节
                let file_url = api.upload_paper_bytes(&path).await?;
                // 阶段 2：建元数据记录
                let mut file = PaperFile {
                    file_name,
                    file_format,
                    file_size,
                    file_url,
                    file_subject: subject,
                    school_name: school,
                    file_year: year,
                    create_by: creator,
                    ..Default::default()
                };
                let resp = api.create_file(&file).await?;
                if resp.code != 200 {
                    anyhow::bail!("{}", resp.msg);
                }
                // create_file 若返回 fileId 就填回来（少数后端会返回）
                if let Some(v) = resp.data {
                    if let Some(id) = v.get("fileId").and_then(|x| x.as_i64()) {
                        file.file_id = id;
                    }
                }
                Ok(file)
            }
            .await;
            tx.send(result).ok();
        });
    }

    /// 清空找回密码流程
    pub fn reset_forget_flow(&mut self) {
        self.forget_step = 1;
        self.forget_username.clear();
        self.forget_captcha.clear();
        self.forget_captcha_uuid.clear();
        self.forget_captcha_texture = None;
        self.forget_questions.clear();
        self.forget_new_password.clear();
        self.forget_confirm_password.clear();
        self.forget_error.clear();
        self.auth_step_anim.set_target(0.0);
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
        self.user_stats_data.reload(move || {
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

    /// 从本地缓存加载用户统计（启动时调用，让首页个人页能立即显示）
    fn load_user_stats_cache(&mut self) {
        if let Some(stats) = self.cache_manager.load_user_stats::<UserStats>() {
            self.user_stats = Some(stats.clone());
            self.fav_anim.jump_to(stats.favorite_count);
            self.dl_anim.jump_to(stats.download_count);
            self.ul_anim.jump_to(stats.upload_count);
        }
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

    /// 打开文件选择器上传头像，成功后拉取最新 profile 返回
    pub fn trigger_upload_avatar(&mut self) {
        if self.avatar_upload_rx.is_some() {
            return;
        }
        let api = self.api.clone();
        let (tx, rx) = oneshot::channel();
        self.avatar_upload_rx = Some(rx);
        tokio::spawn(async move {
            let file = rfd::AsyncFileDialog::new()
                .add_filter("图片", &["jpg", "jpeg", "png", "gif"])
                .pick_file()
                .await;
            let Some(file) = file else {
                tx.send(Ok(None)).ok();
                return;
            };
            let path = file.path().to_string_lossy().to_string();
            let result: anyhow::Result<Option<UserInfo>> = async {
                // 头像大小限制 10MB
                const MAX_AVATAR_BYTES: u64 = 10 * 1024 * 1024;
                let meta = tokio::fs::metadata(&path).await?;
                if meta.len() > MAX_AVATAR_BYTES {
                    return Err(anyhow::anyhow!(
                        "文件过大：{:.1}MB，超过 10MB 上限",
                        meta.len() as f64 / (1024.0 * 1024.0),
                    ));
                }
                let up = api.upload_avatar(&path).await?;
                if up.code != 200 {
                    return Err(anyhow::anyhow!("{}", up.msg));
                }
                let profile = api.get_profile().await?;
                if profile.code != 200 {
                    return Err(anyhow::anyhow!("{}", profile.msg));
                }
                Ok(profile.data)
            }
            .await;
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

    /// 异步加载书签收藏列表
    pub fn trigger_load_bookmark_favorites(&mut self) {
        let api = self.api.clone();
        let user_id = self.current_user.as_ref().map(|u| u.user_id).unwrap_or(0);
        self.bookmark_favorites_data.load(move || async move {
            let params = PageParams { page_num: 1, page_size: 200, ..Default::default() };
            let resp = api.get_bookmark_favorite_list(user_id, &params).await?;
            if resp.code != 200 {
                return Err(anyhow::anyhow!("书签收藏列表错误 {}: {}", resp.code, resp.msg));
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
            if let Some(cached) = self.cache_manager.read_avatar_cache(user_id) {
                let tx = tx.clone();
                tokio::spawn(async move {
                    tx.send((user_id, Ok(cached))).ok();
                });
                continue;
            }
            // 下载
            let avatar_url = item.avatar.clone();
            let tx = tx.clone();
            let api = self.api.clone();
            let cm = self.cache_manager.clone();
            tokio::spawn(async move {
                let result = api.download_raw_url(&avatar_url).await;
                // 下载成功时，保存到磁盘缓存
                if let Ok(ref bytes) = result {
                    cm.write_avatar_cache(user_id, bytes);
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

    /// 处理书签封面下载结果（中心裁剪至 16:9 宽高比，避免拉伸）
    fn process_bookmark_cover_result(&mut self, ctx: &egui::Context, bookmark_id: i64, bytes: &[u8]) {
        log::info!("处理书签封面 {}, {} bytes", bookmark_id, bytes.len());
        match image::load_from_memory(bytes) {
            Ok(mut img) => {
                let (w, h) = (img.width(), img.height());
                if w > 0 && h > 0 {
                    // 中心裁剪至 16:9 宽高比
                    let target_ratio = 16.0 / 9.0;
                    let img_ratio = w as f64 / h as f64;
                    let cropped = if img_ratio > target_ratio {
                        // 图片太宽，裁剪左右
                        let new_w = (h as f64 * target_ratio) as u32;
                        let offset = (w - new_w) / 2;
                        image::imageops::crop(&mut img, offset, 0, new_w, h).to_image()
                    } else if img_ratio < target_ratio {
                        // 图片太高，裁剪上下
                        let new_h = (w as f64 / target_ratio) as u32;
                        let offset = (h - new_h) / 2;
                        image::imageops::crop(&mut img, 0, offset, w, new_h).to_image()
                    } else {
                        img.to_rgba8()
                    };
                    let (cw, ch) = cropped.dimensions();
                    let pixels = cropped.into_raw();
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                        [cw as usize, ch as usize], &pixels,
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

        // 导航到首页或个人页时刷新统计
        if section == Section::Home || section == Section::Profile {
            self.trigger_load_user_stats();
        }
    }

    /// 直接跳转到指定 Section + Subsection
    pub fn navigate_to(&mut self, section: Section, sub: Subsection) {
        let same_section = self.current_section == section;
        if !same_section {
            self.page_enter_anim = SpringAnim::with_target(0.4, 0.8, 0.0, 0.0, 1.0);
            self.sidebar_indicator_anim.set_target(section.index() as f64);
            self.subtab_indicator_anim.set_target(0.0);
        }
        let subs = section.subsections();
        let new_idx = subs.iter().position(|&(s, _)| s == sub).unwrap_or(0);
        // 同 Section 内跳转：走子分页过渡；跨 Section 跳转由 page_enter_anim 处理
        if same_section {
            if let Some(old) = subs.iter().position(|&(s, _)| s == self.current_subsection) {
                if old != new_idx {
                    self.subsection_transition_dir = if new_idx > old { 1.0 } else { -1.0 };
                    self.subsection_transition_anim.jump_to(0.0);
                    self.subsection_transition_anim.set_target(1.0);
                }
            }
        }
        self.subtab_indicator_anim.set_target(new_idx as f64);
        self.current_section = section;
        self.current_subsection = sub;

        // 导航到首页或个人页时刷新统计
        if section == Section::Home || section == Section::Profile {
            self.trigger_load_user_stats();
        }
    }

    /// 切换当前 Section 内的子标签（带弹簧动画）
    pub fn navigate_subsection(&mut self, sub: Subsection) {
        let subs = self.current_section.subsections();
        let old_idx = subs.iter().position(|&(s, _)| s == self.current_subsection);
        let new_idx = subs.iter().position(|&(s, _)| s == sub).unwrap_or(0);
        // 触发内容区左右滑动过渡（仅当索引真正变化）
        if let Some(o) = old_idx {
            if o != new_idx {
                self.subsection_transition_dir = if new_idx > o { 1.0 } else { -1.0 };
                self.subsection_transition_anim.jump_to(0.0);
                self.subsection_transition_anim.set_target(1.0);
            }
        }
        self.subtab_indicator_anim.set_target(new_idx as f64);
        self.current_subsection = sub;

        // 导航到首页或个人页时刷新统计
        let section = self.current_section;
        if section == Section::Home || section == Section::Profile {
            self.trigger_load_user_stats();
        }
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
        self.fav_anim.jump_to(0);
        self.dl_anim.jump_to(0);
        self.ul_anim.jump_to(0);
        self.favorite_file_ids.clear();
        self.favorite_ids_loaded = false;
        self.favorite_ids_rx = None;
        self.avatar_texture = None;
        self.avatar_load_rx = None;
        self.avatar_upload_rx = None;
        self.rank_avatar_textures.clear();
        self.rank_avatar_delays.clear();
        self.rank_avatar_timer.clear();
        self.rank_avatar_frame_idx.clear();
        self.rank_avatar_rx = None;
        self.rank_avatar_tx = None;
        self.rank_avatar_failed.clear();
        self.rank_avatar_pending.clear();
        self.cache_manager.clear_credentials();
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
        self.favorites_data = AsyncData::new();
        self.bookmark_favorites_data = AsyncData::new();
    }

    /// 异步加载 PDF 文件字节（用于预览）
    pub fn trigger_load_pdf_bytes(&mut self, file_id: i64) {
        if self.pdf_loading || self.pdf_bytes_rx.is_some() {
            return;
        }
        self.pdf_file_id = Some(file_id);
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

    /// 清除所有缓存（磁盘 + 内存纹理）
    pub fn clear_cache(&mut self) {
        // 清空磁盘缓存
        let _ = self.cache_manager.clear_all_cache();
        // 清空内存纹理缓存
        self.rank_avatar_textures.clear();
        self.rank_avatar_delays.clear();
        self.rank_avatar_timer.clear();
        self.rank_avatar_frame_idx.clear();
        self.rank_avatar_failed.clear();
        self.rank_avatar_pending.clear();
        self.bookmark_cover_textures.clear();
        self.bookmark_cover_requested.clear();
        self.bookmark_title_cache.clear();
        self.avatar_texture = None;
        self.avatar_image_size = None;
        self.pdf_viewer.clear_textures();
        self.add_toast("缓存已清理", ToastLevel::Success);
    }

    /// 同步开机自启设置到系统注册表（Windows only）
    pub fn sync_auto_launch(&self) {
        #[cfg(target_os = "windows")]
        {
            let enable = self.setting_auto_launch;
            let current_exe = std::env::current_exe().ok();
            std::thread::spawn(move || {
                let app_name = "PezMax";
                if enable {
                    if let Some(exe) = current_exe {
                        let exe_str = exe.to_string_lossy().to_string();
                        let _ = std::process::Command::new("reg")
                            .args([
                                "add", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                                "/v", app_name,
                                "/t", "REG_SZ",
                                "/d", &exe_str,
                                "/f",
                            ])
                            .output();
                        log::info!("已设置开机自启: {}", exe_str);
                    }
                } else {
                    let _ = std::process::Command::new("reg")
                        .args([
                            "delete", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                            "/v", app_name,
                            "/f",
                        ])
                        .output();
                    log::info!("已移除开机自启");
                }
            });
        }
        #[cfg(not(target_os = "windows"))]
        {
            if self.setting_auto_launch {
                log::info!("开机自启仅在 Windows 平台支持");
            }
        }
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

        // 主题/强调色变化时保存设置
        let dir_changed = self.settings.download_dir.as_deref() != Some(self.setting_download_dir.as_str());
        let auto_launch_changed = self.settings.setting_auto_launch != self.setting_auto_launch;
        if self.settings.theme_mode != self.theme_mode
            || self.settings.accent_idx != self.accent_idx
            || auto_launch_changed
            || self.settings.setting_silent_download != self.setting_silent_download
            || self.settings.pdf_view_mode != self.setting_pdf_view_mode
            || self.settings.pdf_scale != self.setting_pdf_scale
            || dir_changed
        {
            self.settings.theme_mode = self.theme_mode;
            self.settings.accent_idx = self.accent_idx;
            self.settings.setting_auto_launch = self.setting_auto_launch;
            self.settings.setting_silent_download = self.setting_silent_download;
            self.settings.pdf_view_mode = self.setting_pdf_view_mode;
            self.settings.pdf_scale = self.setting_pdf_scale;
            self.settings.download_dir = Some(self.setting_download_dir.clone());
            self.settings.save(&self.cache_manager);

            // 仅在开机自启开关实际变化时同步注册表（注册表 I/O 在后台线程执行，避免阻塞 UI）
            if auto_launch_changed {
                self.sync_auto_launch();
            }
        }

        // 同步 PDF 设置到 PdfViewer（仅在登录后）
        if self.is_logged_in {
            if self.pdf_viewer.view_mode != self.setting_pdf_view_mode {
                self.pdf_viewer.set_view_mode(
                    self.setting_pdf_view_mode,
                    &self.pdf_engine,
                    ctx,
                );
            }
        }

        // 清理缓存后刷新大小显示（toast 已添加，不需要额外操作）

        let dt = ctx.input(|i| i.stable_dt) as f64;

        // 搜索防抖：停止输入 300ms 后再把 search_query 同步到 debounced
        if let Some(t) = self.search_query_changed_at {
            let now = ctx.input(|i| i.time);
            if now - t >= 0.3 {
                self.search_query_debounced = self.search_query.clone();
                self.search_query_changed_at = None;
            } else {
                let remaining_ms = ((0.3 - (now - t)) * 1000.0).ceil() as u64;
                ctx.request_repaint_after(std::time::Duration::from_millis(remaining_ms + 10));
            }
        } else if self.search_query != self.search_query_debounced {
            // 兜底：例如以编程方式改了 search_query 但未触发 TextEdit changed
            self.search_query_debounced = self.search_query.clone();
        }

        // 每帧推进所有动画状态
        self.sidebar_anim.update(dt);
        self.sidebar_indicator_anim.update(dt);
        self.subtab_indicator_anim.update(dt);
        self.subsection_transition_anim.update(dt);
        self.preview_anim.update(dt);
        self.bookmark_detail_anim.update(dt);
        self.favorites_tab_anim.update(dt);
        self.page_enter_anim.update(dt);
        self.auth_anim.update(dt);
        self.auth_step_anim.update(dt);
        self.register_disclaimer_countdown.update(dt);
        self.report_timeline_anim.update(dt);
        self.search_hint_anim.update(dt);
        self.pdf_viewer.update_animations(dt);
        self.fav_anim.update(dt);
        self.dl_anim.update(dt);
        self.ul_anim.update(dt);
        theme::update_accent_transition(dt);
        theme::update_dark_transition(dt);
        for toast in &mut self.toasts {
            toast.enter.update(dt);
            toast.exit.update(dt);
        }

        // 轮询 PDF 渲染结果
        self.pdf_viewer.poll_render(&self.pdf_engine, ctx, &self.cache_manager);

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

        // 轮询头像上传结果
        if let Some(rx) = &mut self.avatar_upload_rx {
            if let Ok(result) = rx.try_recv() {
                self.avatar_upload_rx = None;
                self.account_edit_loading = false;
                match result {
                    Ok(Some(user)) => {
                        self.current_user = Some(user);
                        self.avatar_texture = None;
                        self.avatar_image_size = None;
                        self.trigger_load_avatar();
                        self.account_edit_success = "头像上传成功".to_string();
                        self.account_edit_message_timer = 3.0;
                        self.account_edit_section = AccountEditSection::None;
                    }
                    Ok(None) => {
                        // 用户取消，静默处理
                    }
                    Err(e) => {
                        self.account_edit_error = format!("头像上传失败: {}", e);
                        self.account_edit_message_timer = 3.0;
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
                            ctx.request_repaint(); // 封面纹理就绪后触发重绘
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
            let need_repaint = !cover_results.is_empty();
            for (id, bytes) in cover_results {
                self.process_bookmark_cover_result(ctx, id, &bytes);
            }
            if need_repaint {
                ctx.request_repaint(); // 批量封面纹理就绪后触发重绘
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
                        let file_id = self.pdf_file_id;
                        self.pdf_viewer.load_document(
                            &self.pdf_engine,
                            bytes,
                            ctx,
                            file_id,
                            &self.cache_manager,
                        );
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
            || !self.subsection_transition_anim.is_steady()
            || !self.preview_anim.is_steady()
            || !self.bookmark_detail_anim.is_steady()
            || !self.favorites_tab_anim.is_steady()
            || !self.page_enter_anim.is_steady()
            || !self.auth_anim.is_steady()
            || !self.auth_step_anim.is_steady()
            || !self.register_disclaimer_countdown.is_steady()
            || !self.report_timeline_anim.is_steady()
            || !self.search_hint_anim.is_steady()
            || !self.fav_anim.is_steady()
            || !self.dl_anim.is_steady()
            || !self.ul_anim.is_steady()
            || theme::is_transitioning()
            || self.pdf_viewer.is_animating()
            || self.pdf_viewer.is_loading()
            || self.toasts.iter().any(|t| !t.enter.is_steady() || !t.exit.is_steady())
            || self.avatar_load_rx.is_some()
            || self.avatar_upload_rx.is_some()
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

        // 注册页验证码
        if let Some(rx) = &mut self.register_captcha_rx {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(captcha) => {
                        self.register_captcha_enabled = captcha.captcha_enabled;
                        self.register_captcha_uuid = captcha.uuid;
                        self.register_captcha_img = captcha.img.clone();
                        if !captcha.img.is_empty() {
                            if let Some(tex) = decode_base64_image(&captcha.img, ctx) {
                                self.register_captcha_texture = Some(tex);
                            }
                        }
                    }
                    Err(e) => self.register_error = format!("验证码加载失败: {}", e),
                }
                self.register_captcha_rx = None;
            }
        }
        // 注册提交结果
        if let Some(rx) = &mut self.register_rx {
            if let Ok(result) = rx.try_recv() {
                self.register_loading = false;
                match result {
                    Ok(_) => {
                        self.add_toast("注册成功，请登录", crate::app::ToastLevel::Success);
                        self.reset_register_flow();
                        self.auth_page = AuthPage::Login;
                    }
                    Err(e) => {
                        self.register_error = e.to_string();
                        // 刷验证码
                        self.register_captcha.clear();
                        self.register_captcha_texture = None;
                        self.register_captcha_uuid.clear();
                        self.trigger_register_captcha();
                    }
                }
                self.register_rx = None;
            }
        }
        // 找回密码页验证码
        if let Some(rx) = &mut self.forget_captcha_rx {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(captcha) => {
                        self.forget_captcha_enabled = captcha.captcha_enabled;
                        self.forget_captcha_uuid = captcha.uuid;
                        self.forget_captcha_img = captcha.img.clone();
                        if !captcha.img.is_empty() {
                            if let Some(tex) = decode_base64_image(&captcha.img, ctx) {
                                self.forget_captcha_texture = Some(tex);
                            }
                        }
                    }
                    Err(e) => self.forget_error = format!("验证码加载失败: {}", e),
                }
                self.forget_captcha_rx = None;
            }
        }
        // 找回密码：拉密保问题
        if let Some(rx) = &mut self.forget_questions_rx {
            if let Ok(result) = rx.try_recv() {
                self.forget_loading = false;
                match result {
                    Ok(mut questions) => {
                        // 后端返回的 answer 字段可能为空，清一下防串数据
                        for q in questions.iter_mut() { q.answer.clear(); }
                        self.forget_questions = questions;
                        self.forget_step = 2;
                        self.auth_step_anim.set_target(1.0);
                    }
                    Err(e) => {
                        self.forget_error = e.to_string();
                        self.forget_captcha.clear();
                        self.forget_captcha_texture = None;
                        self.forget_captcha_uuid.clear();
                        self.trigger_forget_captcha();
                    }
                }
                self.forget_questions_rx = None;
            }
        }
        // 密码验证结果（密码修改 / 密保编辑前的通用网关）
        if let Some(rx) = &mut self.password_verify_rx {
            if let Ok(result) = rx.try_recv() {
                self.account_edit_loading = false;
                match result {
                    Ok(true) => {
                        self.account_edit_error.clear();
                        match self.account_edit_section {
                            AccountEditSection::Password => {
                                self.password_verify_step = 1;
                            }
                            AccountEditSection::Security => {
                                // 通过后立即拉现有密保
                                self.trigger_preload_security();
                            }
                            _ => {
                                self.password_verify_step = 1;
                            }
                        }
                    }
                    Ok(false) => {
                        self.account_edit_error = "旧密码不正确".to_string();
                        self.account_edit_message_timer = 3.0;
                    }
                    Err(e) => {
                        self.account_edit_error = format!("验证失败: {}", e);
                        self.account_edit_message_timer = 3.0;
                    }
                }
                self.password_verify_rx = None;
            }
        }
        // 密保预加载结果
        if let Some(rx) = &mut self.security_preload_rx {
            if let Ok(result) = rx.try_recv() {
                self.account_edit_loading = false;
                match result {
                    Ok(qs) => {
                        self.account_edit_security_questions = qs;
                        self.security_verify_step = 1;
                    }
                    Err(e) => {
                        self.account_edit_error = format!("加载密保失败: {}", e);
                        self.account_edit_message_timer = 3.0;
                    }
                }
                self.security_preload_rx = None;
            }
        }

        // 举报提交
        if let Some(rx) = &mut self.report_submit_rx {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(_) => {
                        self.add_toast("举报已提交，请等待审核", crate::app::ToastLevel::Success);
                        self.show_report_dialog = false;
                        self.report_reason.clear();
                        self.report_remark.clear();
                        // 强制刷新我的举报列表
                        self.my_reports_data.reset();
                    }
                    Err(e) => {
                        self.add_toast(&format!("提交失败: {}", e), crate::app::ToastLevel::Error);
                    }
                }
                self.report_submit_rx = None;
            }
        }
        // 举报时间线
        if let Some(rx) = &mut self.report_timeline_rx {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(data) => {
                        self.report_timeline_data = Some(data);
                        self.show_report_timeline = true;
                        // 重置动画状态并播入场
                        self.report_timeline_anim = SpringAnim::with_target(0.4, 0.8, 0.0, 0.0, 1.0);
                    }
                    Err(e) => {
                        self.add_toast(&format!("加载时间线失败: {}", e), crate::app::ToastLevel::Error);
                    }
                }
                self.report_timeline_rx = None;
            }
        }

        // 贡献文件上传
        if let Some(rx) = &mut self.contribute_upload_rx {
            if let Ok(result) = rx.try_recv() {
                self.contribute_uploading = false;
                match result {
                    Ok(_file) => {
                        self.add_toast("上传成功，等待审核", crate::app::ToastLevel::Success);
                        self.contribute_subject.clear();
                        self.contribute_school.clear();
                        self.contribute_year.clear();
                        self.contribute_file_path = None;
                        self.contribute_file_name = None;
                        self.contribute_file_format = None;
                        self.contribute_file_size = None;
                        if let Some(ref mut stats) = self.user_stats {
                            stats.upload_count += 1;
                        }
                        self.trigger_load_user_stats();
                    }
                    Err(e) => {
                        self.add_toast(&format!("上传失败: {}", e), crate::app::ToastLevel::Error);
                    }
                }
                self.contribute_upload_rx = None;
            }
        }

        // 找回密码：重置密码结果
        if let Some(rx) = &mut self.forget_reset_rx {
            if let Ok(result) = rx.try_recv() {
                self.forget_loading = false;
                match result {
                    Ok(_) => {
                        self.add_toast("密码重置成功，请用新密码登录", crate::app::ToastLevel::Success);
                        self.reset_forget_flow();
                        self.auth_page = AuthPage::Login;
                    }
                    Err(e) => self.forget_error = e.to_string(),
                }
                self.forget_reset_rx = None;
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
                        // 刷新验证码：清旧输入、旧纹理、旧 uuid，触发下一帧重新拉
                        self.login_captcha.clear();
                        self.login_captcha_texture = None;
                        self.login_captcha_uuid.clear();
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
                            self.cache_manager.clear_credentials();
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
            // 如果书签数据被重置，同时重置封面加载标记和请求记录，
            // 以便刷新后重新加载（包括之前失败的封面）
            if !self.bookmarks_data.is_loading() && !self.bookmarks_data.is_loaded() {
                self.bookmark_covers_triggered = false;
                self.bookmark_cover_requested.clear();
            }
            // 书签数据加载完成后，触发封面加载（仅一次）
            if self.bookmarks_data.is_loaded() && !self.bookmark_covers_triggered {
                self.bookmark_covers_triggered = true;
                let bookmarks = self.bookmarks_data.data.clone();
                if let Some(ref list) = bookmarks {
                    log::info!("触发书签封面加载: {} 条书签", list.len());
                    let cm = self.cache_manager.clone();
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
                            let cm_clone = cm.clone();
                            log::info!("书签 {} 封面URL: {}", id, url);
                            tokio::spawn(async move {
                                // 优先读磁盘缓存
                                if let Some(cached) = cm_clone.read_bookmark_cover_cache(id) {
                                    log::info!("书签 {} 封面命中磁盘缓存 ({} bytes)", id, cached.len());
                                    let _ = txc.send((id, Ok(cached)));
                                    return;
                                }
                                let result = api.download_bytes(&url).await;
                                // 成功后写磁盘缓存
                                if let Ok(ref bytes) = result {
                                    if !bytes.is_empty() {
                                        cm_clone.write_bookmark_cover_cache(id, bytes);
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
            self.bookmark_favorites_data.poll();

            // 书签详情异步加载结果
            if let Some(rx) = &mut self.bookmark_detail_rx {
                if let Ok(result) = rx.try_recv() {
                    self.bookmark_detail_rx = None;
                    match result {
                        Ok(bookmark) => {
                            self.selected_bookmark = Some(bookmark);
                        }
                        Err(e) => {
                            log::error!("获取书签详情失败: {}", e);
                        }
                    }
                }
            }

            // 书签标题缓存更新
            if let Some(rx) = &mut self.bookmark_title_rx {
                while let Ok((id, title)) = rx.try_recv() {
                    self.bookmark_title_cache.insert(id, title);
                }
            }

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
                // 更新动画计数器（set_target 内部已处理"值未变"的情况）
                self.fav_anim.set_target(stats.favorite_count);
                self.dl_anim.set_target(stats.download_count);
                self.ul_anim.set_target(stats.upload_count);
                // 写入本地缓存
                self.cache_manager.save_user_stats(stats);
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
        // 子分页切换过渡：把内容整体沿 X 轴滑入，同时淡入
        // 通过左右 inner_margin 的对称增减实现横向位移（不改变可视宽度）
        let sub_t = self.subsection_transition_anim.value();
        let sub_active = !self.subsection_transition_anim.is_steady();
        let (baseline_h, baseline_v) = if preview_mode && self.current_subsection == Subsection::ResourceManager {
            (0.0f64, 0.0f64)
        } else {
            (20.0f64, 0.0f64)
        };
        let slide_dist: f64 = 48.0;
        let sub_offset = self.subsection_transition_dir as f64 * slide_dist * (1.0 - sub_t);
        let left = (baseline_h + sub_offset).clamp(i8::MIN as f64 + 1.0, i8::MAX as f64) as i8;
        let right = (baseline_h - sub_offset).clamp(i8::MIN as f64 + 1.0, i8::MAX as f64) as i8;
        let content_margin = egui::Margin {
            left,
            right,
            top: baseline_v as i8,
            bottom: baseline_v as i8,
        };
        egui::CentralPanel::default()
            .frame(egui::Frame::new()
                .fill(theme::colors::bg_white())
                .inner_margin(content_margin)
                .stroke(egui::Stroke::NONE),
            )
            .show(ctx, |ui| {
                let mut alpha_final: f32 = 1.0;
                if !self.page_enter_anim.is_steady() {
                    let offset = map_range(enter_v, 20.0, 0.0) as f32;
                    alpha_final = alpha_final.min(map_range(enter_v, 0.4, 1.0) as f32);
                    if offset > 0.1 {
                        ui.add_space(offset);
                    }
                }
                if sub_active {
                    alpha_final = alpha_final.min(map_range(sub_t, 0.35, 1.0) as f32);
                }
                if alpha_final < 0.999 {
                    ui.set_opacity(alpha_final.clamp(0.0, 1.0));
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

        // 上传进度 toast（右下角）
        if self.contribute_uploading {
            let name = self.contribute_file_name.clone().unwrap_or_else(|| "文件".to_string());
            crate::components::upload_progress_toast::render(ctx, &name);
        }

        // 举报对话框（全局，任何页面都能打开）
        let (submit_report, close_report) = crate::components::report_dialog::render(ctx, self);
        if submit_report { self.trigger_submit_report(); }
        if close_report { self.show_report_dialog = false; }

        // 举报时间线弹窗
        if crate::components::timeline_panel::render(ctx, self) {
            // 退出动画：反向 anim
            self.report_timeline_anim.set_target(0.0);
            self.show_report_timeline = false;
        }
    }

    fn on_exit(&mut self, ctx: std::option::Option<&eframe::glow::Context>) {
        // 保存窗口状态 — 注意 eframe 0.31 的 glow::Context 没有 viewport info，
        // 窗口大小/位置由 settings 保存但仅在 on_exit 时可行
        if let Some(_ctx) = ctx {
            // glow::Context 无法直接获取 viewport 信息，暂不保存窗口位置
        }
        // 保存所有设置
        self.settings.save(&self.cache_manager);
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

