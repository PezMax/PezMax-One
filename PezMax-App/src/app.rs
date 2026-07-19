// PezMax 应用状态
// 管理全局状态：导航、认证、页面切换、Toast 通知

use crate::api::client::ApiClient;
use crate::api::models::*;
use crate::theme;
use egui::Context;

/// 应用页面路由
#[derive(Debug, Clone, PartialEq)]
pub enum Page {
    Login,
    Register,
    ForgetPassword,
    Home,
    FileExplorer,
    FileDetail(i64),
    Bookmarks,
    Downloads,
    Favorites,
    Notifications,
    Profile,
    Security,
    Report,
    Settings,
}

/// Toast 通知类型
#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub level: ToastLevel,
    pub created_at: std::time::Instant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToastLevel {
    Info,
    Success,
    Warning,
    Error,
}

/// 应用主状态
pub struct PezMaxApp {
    pub api: ApiClient,
    pub current_page: Page,
    pub page_history: Vec<Page>,
    pub toasts: Vec<Toast>,

    // 认证状态
    pub is_logged_in: bool,
    pub token: Option<String>,
    pub current_user: Option<UserInfo>,
    pub user_stats: Option<UserStats>,

    // 搜索状态
    pub search_query: String,
    pub search_results: Vec<PaperFile>,

    // 分页状态
    pub file_page: PageParams,
    pub file_list: Vec<PaperFile>,
    pub file_total: i64,
    pub is_loading: bool,

    // 侧边栏
    pub sidebar_open: bool,

    // 通知角标
    pub unread_notifications: i32,
}

impl PezMaxApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply_metro_theme(&cc.egui_ctx);

        let api = ApiClient::new(None);

        Self {
            api,
            current_page: Page::Login,
            page_history: vec![],
            toasts: vec![],
            is_logged_in: false,
            token: None,
            current_user: None,
            user_stats: None,
            search_query: String::new(),
            search_results: vec![],
            file_page: PageParams::default(),
            file_list: vec![],
            file_total: 0,
            is_loading: false,
            sidebar_open: true,
            unread_notifications: 0,
        }
    }

    /// 导航到新页面
    pub fn navigate(&mut self, page: Page) {
        self.page_history.push(self.current_page.clone());
        self.current_page = page;
    }

    /// 返回上一页
    pub fn go_back(&mut self) {
        if let Some(prev) = self.page_history.pop() {
            self.current_page = prev;
        }
    }

    /// 添加 Toast 通知
    pub fn add_toast(&mut self, message: impl Into<String>, level: ToastLevel) {
        self.toasts.push(Toast {
            message: message.into(),
            level,
            created_at: std::time::Instant::now(),
        });
        // 保留最近 5 条
        if self.toasts.len() > 5 {
            self.toasts.remove(0);
        }
    }

    /// 清理过期 Toast（超过 5 秒）
    pub fn cleanup_toasts(&mut self) {
        let now = std::time::Instant::now();
        self.toasts.retain(|t| now.duration_since(t.created_at).as_secs() < 5);
    }

    /// 获取当前页面标题
    pub fn page_title(&self) -> &str {
        match self.current_page {
            Page::Login => "登录",
            Page::Register => "注册",
            Page::ForgetPassword => "找回密码",
            Page::Home => "首页",
            Page::FileExplorer => "试卷浏览",
            Page::FileDetail(_) => "文件详情",
            Page::Bookmarks => "书签管理",
            Page::Downloads => "下载记录",
            Page::Favorites => "我的收藏",
            Page::Notifications => "通知中心",
            Page::Profile => "个人中心",
            Page::Security => "安全设置",
            Page::Report => "举报中心",
            Page::Settings => "系统设置",
        }
    }
}

impl eframe::App for PezMaxApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.cleanup_toasts();

        // 登录态检查：未登录时只显示登录/注册/找回密码
        if !self.is_logged_in {
            match self.current_page {
                Page::Login | Page::Register | Page::ForgetPassword => {}
                _ => self.current_page = Page::Login,
            }
        }

        // 渲染当前页面
        match self.current_page {
            Page::Login => crate::pages::login::render(self, ctx),
            Page::Register => crate::pages::register::render(self, ctx),
            Page::ForgetPassword => crate::pages::forget_password::render(self, ctx),
            _ => {
                // 已登录状态：渲染侧边栏 + 内容区
                crate::components::sidebar::render(self, ctx);
                crate::components::topbar::render(self, ctx);

                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(theme::colors::BG_WHITE))
                    .show(ctx, |ui| {
                        match self.current_page {
                            Page::Home => crate::pages::home::render(self, ui),
                            Page::FileExplorer => crate::pages::file_explorer::render(self, ui),
                            Page::FileDetail(id) => crate::pages::file_detail::render(self, ui, id),
                            Page::Bookmarks => crate::pages::bookmarks::render(self, ui),
                            Page::Downloads => crate::pages::downloads::render(self, ui),
                            Page::Favorites => crate::pages::favorites::render(self, ui),
                            Page::Notifications => crate::pages::notifications::render(self, ui),
                            Page::Profile => crate::pages::profile::render(self, ui),
                            Page::Security => crate::pages::security::render(self, ui),
                            Page::Report => crate::pages::report::render(self, ui),
                            Page::Settings => crate::pages::settings::render(self, ui),
                            _ => {}
                        }
                    });

                // 全局 Toast 渲染
                crate::components::toast::render(self, ctx);
            }
        }
    }
}