// Linux 菜单树定义
//
// 稳定 ID 分配：ID = 段号 × 100 + 项索引。段号大致对应 File=1 / View=3 / Go=4 / Help=5。
// 稳定 ID 让 backend.set_theme_mode / set_accent 能通过 O(1) 查找定位到具体项。
//
// 说明：不包含 Edit 菜单——egui 有自己的编辑处理，DBusMenu 没有能可靠映射到 egui
// 焦点组件的 undo/redo/cut/copy/paste 通路（macOS 上会在 muda 后端里用系统预定义项）。

use super::proto::MenuItem;
use crate::app::Section;
use crate::app_menu::MenuCommand;
use crate::theme::{self, ThemeMode};

/// 菜单根 ID（DBusMenu 规范：0 = 顶层）
pub const ROOT_ID: i32 = 0;

// ── File ────────────────────────────────────────────────
pub const ID_OPEN_DL_DIR: i32 = 101;
pub const ID_CLEAR_CACHE: i32 = 102;
pub const ID_FILE_SEP_1: i32 = 103;
pub const ID_QUIT: i32 = 104;

// ── View ────────────────────────────────────────────────
pub const ID_TOGGLE_SIDEBAR: i32 = 301;
pub const ID_VIEW_SEP_1: i32 = 302;
pub const ID_THEME_SUBMENU: i32 = 310;
pub const ID_THEME_SYSTEM: i32 = 311;
pub const ID_THEME_LIGHT: i32 = 312;
pub const ID_THEME_DARK: i32 = 313;
pub const ID_ACCENT_SUBMENU: i32 = 320;
pub const ID_ACCENT_BASE: i32 = 321; // 321..(321+n)

// ── Go ──────────────────────────────────────────────────
pub const ID_GO_HOME: i32 = 401;
pub const ID_GO_BROWSE: i32 = 402;
pub const ID_GO_COMMUNITY: i32 = 403;
pub const ID_GO_PROFILE: i32 = 404;

// ── Help ────────────────────────────────────────────────
pub const ID_ABOUT: i32 = 501;
pub const ID_HOMEPAGE: i32 = 502;
pub const ID_LOG_DIR: i32 = 503;

/// 顶层菜单 ID（用于父子结构查询）
pub const ID_FILE: i32 = 100;
pub const ID_VIEW: i32 = 300;
pub const ID_GO: i32 = 400;
pub const ID_HELP: i32 = 500;

/// 构造完整菜单树。初始勾选状态由 `theme_mode` / `accent_idx` 决定。
pub fn build(theme_mode: ThemeMode, accent_idx: usize) -> MenuItem {
    let mut root = MenuItem::submenu(ROOT_ID, "");

    root.push_child(build_file());
    root.push_child(build_view(theme_mode, accent_idx));
    root.push_child(build_go());
    root.push_child(build_help());

    root
}

fn build_file() -> MenuItem {
    let mut m = MenuItem::submenu(ID_FILE, "文件");
    m.push_child(MenuItem::standard(ID_OPEN_DL_DIR, "打开下载目录…"));
    m.push_child(MenuItem::standard(ID_CLEAR_CACHE, "清理缓存"));
    m.push_child(MenuItem::separator(ID_FILE_SEP_1));
    m.push_child(MenuItem::standard(ID_QUIT, "退出"));
    m
}

fn build_view(theme_mode: ThemeMode, accent_idx: usize) -> MenuItem {
    let mut m = MenuItem::submenu(ID_VIEW, "视图");
    m.push_child(MenuItem::standard(ID_TOGGLE_SIDEBAR, "折叠/展开侧栏"));
    m.push_child(MenuItem::separator(ID_VIEW_SEP_1));

    // ── 主题子菜单 ─────────────────────────────────
    let mut theme_sub = MenuItem::submenu(ID_THEME_SUBMENU, "主题");
    theme_sub.push_child(MenuItem::radio(
        ID_THEME_SYSTEM,
        "跟随系统",
        theme_mode == ThemeMode::System,
    ));
    theme_sub.push_child(MenuItem::radio(
        ID_THEME_LIGHT,
        "浅色",
        theme_mode == ThemeMode::Light,
    ));
    theme_sub.push_child(MenuItem::radio(
        ID_THEME_DARK,
        "深色",
        theme_mode == ThemeMode::Dark,
    ));
    m.push_child(theme_sub);

    // ── 强调色子菜单 ───────────────────────────────
    let mut accent_sub = MenuItem::submenu(ID_ACCENT_SUBMENU, "强调色");
    for (i, preset) in theme::ACCENT_PRESETS.iter().enumerate() {
        accent_sub.push_child(MenuItem::radio(
            ID_ACCENT_BASE + i as i32,
            preset.name,
            i == accent_idx,
        ));
    }
    m.push_child(accent_sub);

    m
}

fn build_go() -> MenuItem {
    let mut m = MenuItem::submenu(ID_GO, "转到");
    m.push_child(MenuItem::standard(ID_GO_HOME, "首页"));
    m.push_child(MenuItem::standard(ID_GO_BROWSE, "浏览"));
    m.push_child(MenuItem::standard(ID_GO_COMMUNITY, "社区"));
    m.push_child(MenuItem::standard(ID_GO_PROFILE, "个人"));
    m
}

fn build_help() -> MenuItem {
    let mut m = MenuItem::submenu(ID_HELP, "帮助");
    m.push_child(MenuItem::standard(ID_ABOUT, "关于 PezMax One"));
    m.push_child(MenuItem::standard(ID_HOMEPAGE, "项目主页"));
    m.push_child(MenuItem::standard(ID_LOG_DIR, "打开日志目录"));
    m
}

/// 菜单项点击 ID → MenuCommand 映射。
/// 未知 ID 返回 None（分隔符 / 子菜单本身 / 陌生 ID）。
pub fn id_to_command(id: i32) -> Option<MenuCommand> {
    use MenuCommand as C;
    Some(match id {
        ID_OPEN_DL_DIR => C::OpenDownloadDir,
        ID_CLEAR_CACHE => C::ClearCache,
        ID_QUIT => C::Quit,
        ID_TOGGLE_SIDEBAR => C::ToggleSidebar,
        ID_THEME_SYSTEM => C::SetThemeMode(ThemeMode::System),
        ID_THEME_LIGHT => C::SetThemeMode(ThemeMode::Light),
        ID_THEME_DARK => C::SetThemeMode(ThemeMode::Dark),
        ID_GO_HOME => C::NavigateTo(Section::Home),
        ID_GO_BROWSE => C::NavigateTo(Section::Browse),
        ID_GO_COMMUNITY => C::NavigateTo(Section::Community),
        ID_GO_PROFILE => C::NavigateTo(Section::Profile),
        ID_ABOUT => C::About,
        ID_HOMEPAGE => C::OpenHomepage,
        ID_LOG_DIR => C::OpenLogDir,
        id if (ID_ACCENT_BASE..ID_ACCENT_BASE + theme::ACCENT_PRESETS.len() as i32).contains(&id) => {
            C::SetAccent((id - ID_ACCENT_BASE) as usize)
        }
        _ => return None,
    })
}

/// 主题模式对应的菜单项 ID
pub fn theme_mode_id(mode: ThemeMode) -> i32 {
    match mode {
        ThemeMode::System => ID_THEME_SYSTEM,
        ThemeMode::Light => ID_THEME_LIGHT,
        ThemeMode::Dark => ID_THEME_DARK,
    }
}

/// 全部主题模式 ID（用于刷新一组勾选状态时枚举）
pub const THEME_ALL_IDS: &[i32] = &[ID_THEME_SYSTEM, ID_THEME_LIGHT, ID_THEME_DARK];
