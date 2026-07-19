use crate::api::client::ApiClient;
use crate::api::models::*;
use crate::sokuou::{map_range, Easing, Progress, SpringAnim};
use crate::theme;
use egui::Context;

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
            Section::Profile => "⚙️",
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
    pub year: Option<i32>,
}

/// 应用主状态
pub struct PezMaxApp {
    pub api: ApiClient,

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

    // Toast 通知
    pub toasts: Vec<AnimatedToast>,
    pub unread_notifications: i32,

    // 书签创建表单
    pub bookmark_form_name: String,
    pub bookmark_form_url: String,

    // 贡献文件元数据表单
    pub contribute_subject: String,
    pub contribute_school: String,
    pub contribute_year: String,

    // 设置开关
    pub setting_auto_launch: bool,
    pub setting_silent_download: bool,
}

impl PezMaxApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::setup_fonts(&cc.egui_ctx);
        theme::apply_metro_theme(&cc.egui_ctx);

        Self {
            api: ApiClient::new(None),
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
            search_query: String::new(),
            filters: FilterState::default(),
            file_list: vec![],
            file_total: 0,
            file_page: PageParams::default(),
            is_loading: false,
            selected_file: None,
            preview_visible: false,
            preview_anim: SpringAnim::new(0.4, 0.8, 0.0),
            browse_selected_idx: None,
            page_enter_anim: SpringAnim::new(0.4, 0.8, 1.0), // 初始稳态
            toasts: vec![],
            unread_notifications: 0,
            bookmark_form_name: String::new(),
            bookmark_form_url: String::new(),
            contribute_subject: String::new(),
            contribute_school: String::new(),
            contribute_year: String::new(),
            setting_auto_launch: false,
            setting_silent_download: false,
        }
    }

    /// 登录成功后调用：进入首页，触发入场动画
    pub fn login_success(&mut self) {
        self.is_logged_in = true;
        self.current_section = Section::Home;
        self.current_subsection = Subsection::None;
        self.page_enter_anim = SpringAnim::with_target(0.4, 0.8, 0.0, 0.0, 1.0);
        self.sidebar_indicator_anim.set_target(0.0); // Home
    }

    /// 切换顶级 Section（默认跳到该 Section 的第一个子标签）
    pub fn navigate_section(&mut self, section: Section) {
        if self.current_section != section {
            self.page_enter_anim = SpringAnim::with_target(0.4, 0.8, 0.0, 0.0, 1.0);
            self.sidebar_indicator_anim.set_target(section.index() as f64);
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
        }
        self.current_section = section;
        self.current_subsection = sub;
    }

    /// 切换认证子页面
    pub fn set_auth_page(&mut self, page: AuthPage) {
        self.auth_page = page;
    }

    /// 添加 Toast 通知（最多同时显示 3 条）
    pub fn add_toast(&mut self, message: impl Into<String>, level: ToastLevel) {
        self.toasts.push(AnimatedToast::new(message, level));
        if self.toasts.len() > 3 {
            self.toasts.remove(0);
        }
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
        let dt = ctx.input(|i| i.stable_dt) as f64;

        // 每帧推进所有动画状态
        self.sidebar_anim.update(dt);
        self.sidebar_indicator_anim.update(dt);
        self.preview_anim.update(dt);
        self.page_enter_anim.update(dt);
        for toast in &mut self.toasts {
            toast.enter.update(dt);
            toast.exit.update(dt);
        }

        // 有动画进行时持续请求重绘
        if !self.sidebar_anim.is_steady()
            || !self.sidebar_indicator_anim.is_steady()
            || !self.preview_anim.is_steady()
            || !self.page_enter_anim.is_steady()
            || self.toasts.iter().any(|t| !t.enter.is_steady() || !t.exit.is_steady())
        {
            ctx.request_repaint();
        }

        self.cleanup_toasts();

        // 未登录：全屏认证页面
        if !self.is_logged_in {
            match self.auth_page {
                AuthPage::Login => crate::pages::login::render(self, ctx),
                AuthPage::Register => crate::pages::register::render(self, ctx),
                AuthPage::ForgetPassword => crate::pages::forget_password::render(self, ctx),
            }
            return;
        }

        // 已登录：侧边栏 → Topbar → 子标签栏 → 内容区
        crate::components::sidebar::render(self, ctx);
        crate::components::topbar::render(self, ctx);

        // 子标签栏仅 Browse / Community / Profile 显示
        let section = self.current_section;
        let subsections = section.subsections();
        if !subsections.is_empty() {
            render_subtab_bar(self, ctx, &subsections);
        }

        // 内容区（页面切换时有轻微入场偏移动画）
        let enter_v = self.page_enter_anim.value();
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::colors::BG_WHITE))
            .show(ctx, |ui| {
                if !self.page_enter_anim.is_steady() {
                    let offset = map_range(enter_v, 20.0, 0.0) as f32;
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
        .frame(
            egui::Frame::new()
                .fill(colors::BG_CARD)
                .stroke(egui::Stroke::new(1.0, colors::BORDER)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                for &(sub, label) in subsections {
                    let is_active = app.current_subsection == sub;
                    let text_color = if is_active {
                        colors::PRIMARY
                    } else {
                        colors::TEXT_SECONDARY
                    };

                    let btn = egui::Button::new(
                        egui::RichText::new(label)
                            .font(egui::FontId::new(14.0, egui::FontFamily::Proportional))
                            .color(text_color),
                    )
                    .fill(egui::Color32::TRANSPARENT)
                    .corner_radius(egui::CornerRadius::same(0));

                    let resp = ui.add(btn);

                    if resp.clicked() && !is_active {
                        app.current_subsection = sub;
                    }

                    // 选中：底部蓝色下划线
                    if is_active {
                        let r = resp.rect;
                        ui.painter().line_segment(
                            [
                                egui::pos2(r.left() + 4.0, r.bottom() - 2.0),
                                egui::pos2(r.right() - 4.0, r.bottom() - 2.0),
                            ],
                            egui::Stroke::new(2.0, colors::PRIMARY),
                        );
                    }

                    ui.add_space(8.0);
                }
            });
        });
}

