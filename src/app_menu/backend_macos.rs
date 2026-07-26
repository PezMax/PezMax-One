// macOS 菜单后端 — muda + NSMenu 集成
//
// 结构：
//   PezMax One (app menu，muda 自动放到首位)
//     ├─ About PezMax One
//     ├─ Services
//     ├─ Hide / Hide Others / Show All
//     └─ Quit PezMax One
//   File   ├─ Open Downloads Folder…  ├─ Clear Cache
//   Edit   ├─ Undo / Redo / Cut / Copy / Paste / Select All  （全部预定义系统项）
//   View   ├─ Toggle Sidebar  ├─ Theme › {System, Light, Dark}  ├─ Accent › {5 项}
//   Go     ├─ Home / Browse / Community / Profile
//   Help   ├─ Project Homepage / Open Log Folder
//
// 线程模型：
//   - install() 必须在主线程且 NSApp 已就绪时调用（eframe 的 Box::new(|cc| ...) 里 OK）
//   - CheckMenuItem::set_checked 也必须在主线程 → 由 PezMaxApp.update()（主线程）驱动
//   - MenuEvent::receiver() 是全局 crossbeam channel；起独立线程转发到我们的 std mpsc

use std::collections::HashMap;
use std::sync::mpsc;

use muda::{
    accelerator::{Accelerator, Code, Modifiers},
    AboutMetadata, CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu,
};

use super::{MenuBackend, MenuCommand};
use crate::app::Section;
use crate::theme::{self, ThemeMode};

// ── 稳定 ID ──────────────────────────────────────────────
const ID_OPEN_DL: &str = "file.open_downloads";
const ID_CLEAR_CACHE: &str = "file.clear_cache";
const ID_TOGGLE_SIDEBAR: &str = "view.toggle_sidebar";
const ID_THEME_SYSTEM: &str = "view.theme.system";
const ID_THEME_LIGHT: &str = "view.theme.light";
const ID_THEME_DARK: &str = "view.theme.dark";
const ID_GO_HOME: &str = "go.home";
const ID_GO_BROWSE: &str = "go.browse";
const ID_GO_COMMUNITY: &str = "go.community";
const ID_GO_PROFILE: &str = "go.profile";
const ID_HOMEPAGE: &str = "help.homepage";
const ID_LOG_DIR: &str = "help.log_dir";
fn accent_id(idx: usize) -> String {
    format!("view.accent.{idx}")
}

pub struct MacMenu {
    /// 主题子菜单的三个 CheckMenuItem，用于 set_checked 时切换勾选
    theme_items: Vec<(ThemeMode, CheckMenuItem)>,
    /// 强调色子菜单，索引对应 ACCENT_PRESETS 位置
    accent_items: Vec<CheckMenuItem>,
    /// 持有 Menu 让它不被 drop；drop Menu 会拆掉 NSMenu
    _menu: Menu,
}

impl MenuBackend for MacMenu {
    fn set_theme_mode(&self, mode: ThemeMode) {
        for (m, item) in &self.theme_items {
            item.set_checked(*m == mode);
        }
    }
    fn set_accent(&self, idx: usize) {
        for (i, item) in self.accent_items.iter().enumerate() {
            item.set_checked(i == idx);
        }
    }
}

pub fn install(tx: mpsc::Sender<MenuCommand>) -> anyhow::Result<Box<dyn MenuBackend>> {
    let menu = Menu::new();

    // ── App menu（muda 会自动认作 NSApp 主菜单第一项） ──
    let app_submenu = Submenu::new("PezMax One", true);
    let about = PredefinedMenuItem::about(
        Some("About PezMax One"),
        Some(AboutMetadata {
            name: Some("PezMax One".into()),
            version: Some(env!("CARGO_PKG_VERSION").into()),
            website: Some("https://github.com/PezMax/PezMax-One".into()),
            ..Default::default()
        }),
    );
    app_submenu.append_items(&[
        &about,
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::services(None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::hide(None),
        &PredefinedMenuItem::hide_others(None),
        &PredefinedMenuItem::show_all(None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::quit(None),
    ])?;
    menu.append(&app_submenu)?;

    // ── File ────────────────────────────────────────────
    let file_menu = Submenu::new("File", true);
    // Modifiers::SUPER 在 muda 里映射到 macOS 的 Cmd 键
    let open_dl = MenuItem::with_id(
        ID_OPEN_DL,
        "Open Downloads Folder…",
        true,
        Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyO)),
    );
    let clear_cache = MenuItem::with_id(ID_CLEAR_CACHE, "Clear Cache", true, None);
    file_menu.append_items(&[&open_dl, &clear_cache])?;
    menu.append(&file_menu)?;

    // ── Edit（全部走系统预定义项，直接接管 responder chain） ──
    let edit_menu = Submenu::new("Edit", true);
    edit_menu.append_items(&[
        &PredefinedMenuItem::undo(None),
        &PredefinedMenuItem::redo(None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::cut(None),
        &PredefinedMenuItem::copy(None),
        &PredefinedMenuItem::paste(None),
        &PredefinedMenuItem::select_all(None),
    ])?;
    menu.append(&edit_menu)?;

    // ── View ────────────────────────────────────────────
    let view_menu = Submenu::new("View", true);
    let toggle_sidebar = MenuItem::with_id(ID_TOGGLE_SIDEBAR, "Toggle Sidebar", true, None);

    let theme_sub = Submenu::new("Theme", true);
    let th_system = CheckMenuItem::with_id(ID_THEME_SYSTEM, "System", true, true, None);
    let th_light = CheckMenuItem::with_id(ID_THEME_LIGHT, "Light", true, false, None);
    let th_dark = CheckMenuItem::with_id(ID_THEME_DARK, "Dark", true, false, None);
    theme_sub.append_items(&[&th_system, &th_light, &th_dark])?;

    let accent_sub = Submenu::new("Accent", true);
    let mut accent_items = Vec::with_capacity(theme::ACCENT_PRESETS.len());
    for (i, preset) in theme::ACCENT_PRESETS.iter().enumerate() {
        let it = CheckMenuItem::with_id(accent_id(i), preset.name, true, i == 0, None);
        accent_sub.append(&it)?;
        accent_items.push(it);
    }

    view_menu.append_items(&[
        &toggle_sidebar,
        &PredefinedMenuItem::separator(),
        &theme_sub,
        &accent_sub,
    ])?;
    menu.append(&view_menu)?;

    // ── Go ──────────────────────────────────────────────
    let go_menu = Submenu::new("Go", true);
    let go_home = MenuItem::with_id(ID_GO_HOME, "Home", true, None);
    let go_browse = MenuItem::with_id(ID_GO_BROWSE, "Browse", true, None);
    let go_community = MenuItem::with_id(ID_GO_COMMUNITY, "Community", true, None);
    let go_profile = MenuItem::with_id(ID_GO_PROFILE, "Profile", true, None);
    go_menu.append_items(&[&go_home, &go_browse, &go_community, &go_profile])?;
    menu.append(&go_menu)?;

    // ── Help ────────────────────────────────────────────
    let help_menu = Submenu::new("Help", true);
    let homepage = MenuItem::with_id(ID_HOMEPAGE, "Project Homepage", true, None);
    let log_dir = MenuItem::with_id(ID_LOG_DIR, "Open Log Folder", true, None);
    help_menu.append_items(&[&homepage, &log_dir])?;
    menu.append(&help_menu)?;

    // ── 挂到 NSApp（主线程，NSApp 已就绪时才能调） ────
    menu.init_for_nsapp();
    log::info!("macOS NSMenu 已挂载");

    // ── 事件转发线程 ────────────────────────────────────
    // muda 的 MenuEvent::receiver() 是全局 crossbeam channel，我们起独立线程
    // 阻塞读取，翻译成 MenuCommand 送到 PezMaxApp。
    let mut id_map: HashMap<String, MenuCommand> = HashMap::new();
    id_map.insert(ID_OPEN_DL.into(), MenuCommand::OpenDownloadDir);
    id_map.insert(ID_CLEAR_CACHE.into(), MenuCommand::ClearCache);
    id_map.insert(ID_TOGGLE_SIDEBAR.into(), MenuCommand::ToggleSidebar);
    id_map.insert(ID_THEME_SYSTEM.into(), MenuCommand::SetThemeMode(ThemeMode::System));
    id_map.insert(ID_THEME_LIGHT.into(), MenuCommand::SetThemeMode(ThemeMode::Light));
    id_map.insert(ID_THEME_DARK.into(), MenuCommand::SetThemeMode(ThemeMode::Dark));
    id_map.insert(ID_GO_HOME.into(), MenuCommand::NavigateTo(Section::Home));
    id_map.insert(ID_GO_BROWSE.into(), MenuCommand::NavigateTo(Section::Browse));
    id_map.insert(ID_GO_COMMUNITY.into(), MenuCommand::NavigateTo(Section::Community));
    id_map.insert(ID_GO_PROFILE.into(), MenuCommand::NavigateTo(Section::Profile));
    id_map.insert(ID_HOMEPAGE.into(), MenuCommand::OpenHomepage);
    id_map.insert(ID_LOG_DIR.into(), MenuCommand::OpenLogDir);
    for i in 0..theme::ACCENT_PRESETS.len() {
        id_map.insert(accent_id(i), MenuCommand::SetAccent(i));
    }

    std::thread::Builder::new()
        .name("pezmax-macos-menu-events".into())
        .spawn(move || {
            let receiver = MenuEvent::receiver();
            while let Ok(event) = receiver.recv() {
                let id: &str = event.id().as_ref();
                if let Some(cmd) = id_map.get(id) {
                    if tx.send(cmd.clone()).is_err() {
                        break; // app 退出
                    }
                }
            }
        })
        .map_err(|e| anyhow::anyhow!("菜单事件线程启动失败: {e}"))?;

    let theme_items = vec![
        (ThemeMode::System, th_system),
        (ThemeMode::Light, th_light),
        (ThemeMode::Dark, th_dark),
    ];

    Ok(Box::new(MacMenu {
        theme_items,
        accent_items,
        _menu: menu,
    }))
}
