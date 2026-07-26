// 应用菜单抽象层
//
// 职责：
//   1. 定义平台无关的 `MenuCommand`（点击菜单项时产生的动作）
//   2. 定义平台无关的 `MenuBackend` trait（应用侧告知菜单"值变了、刷新勾选"）
//   3. 提供 `install(cc)` 工厂：按 target_os 挑选后端
//        - macOS  → muda 直接挂到 NSApp.mainMenu
//        - Linux  → zbus 起 DBusMenu 服务 + Wayland KWin appmenu 协议绑定
//        - Windows/其他 → 无菜单，返回空通道
//
// 使用方式：`main.rs` 里在 eframe::run_native 的 `Box::new(|cc| ...)` 内调用 `install(cc)`，
// 拿到 (rx, backend)，把 rx/backend 塞进 PezMaxApp。每帧 update() 用 `try_recv` 消费命令，
// 主题/强调色变化时调 backend.set_theme_mode() / set_accent() 让菜单勾选跟上。

use std::sync::mpsc;

use crate::app::Section;
use crate::theme::ThemeMode;

#[cfg(target_os = "macos")]
pub mod backend_macos;
#[cfg(target_os = "linux")]
pub mod backend_linux;
#[cfg(target_os = "linux")]
pub mod linux;

/// 菜单点击产生的命令。所有后端把点击事件翻译成此枚举，通过 mpsc 送到 update() 派发。
#[derive(Debug, Clone, PartialEq)]
pub enum MenuCommand {
    // File
    OpenDownloadDir,
    ClearCache,
    Quit,
    // View
    ToggleSidebar,
    SetThemeMode(ThemeMode),
    SetAccent(usize),
    // Go
    NavigateTo(Section),
    // Help
    About,
    OpenLogDir,
    OpenHomepage,
}

/// 应用侧通知菜单"状态变了、更新勾选"。
///
/// 例如用户在设置页把主题从 Light 换成 Dark，应用调 `set_theme_mode(Dark)`，
/// 后端负责把 File→View→主题 子菜单里的勾选切到 Dark 那一项。
///
/// 不要求 Send + Sync——macOS muda 的菜单对象内部用 Rc，只能在主线程持有。
/// PezMaxApp 只在 eframe 主循环（主线程）调用后端方法，因此没有跨线程问题。
pub trait MenuBackend {
    fn set_theme_mode(&self, mode: ThemeMode);
    fn set_accent(&self, idx: usize);
}

/// 安装平台菜单后端。返回 (命令接收端, 后端句柄)。
///
/// - 无菜单平台（Windows/其他）返回空通道 + None 后端
/// - macOS: 立即在主线程创建 NSMenu（必须在 NSApp 初始化后）
/// - Linux: spawn 独立线程跑 zbus + wayland，通过通道把 command 送出
///
/// 传入 `_cc` 是为了未来可能访问 winit 窗口句柄（Linux 拿 wl_surface 要用）。
pub fn install(
    #[cfg_attr(target_os = "windows", allow(unused_variables))]
    cc: &eframe::CreationContext<'_>,
) -> (mpsc::Receiver<MenuCommand>, Option<Box<dyn MenuBackend>>) {
    let (tx, rx) = mpsc::channel::<MenuCommand>();

    #[cfg(target_os = "macos")]
    {
        match backend_macos::install(tx.clone()) {
            Ok(be) => return (rx, Some(be)),
            Err(e) => {
                log::warn!("macOS 菜单初始化失败: {e}");
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        match backend_linux::install(cc, tx.clone()) {
            Ok(be) => return (rx, Some(be)),
            Err(e) => {
                log::warn!("Linux 菜单初始化失败: {e}");
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    let _ = cc;

    // 静音 unused 警告（Windows 分支）
    let _ = tx;
    (rx, None)
}

/// 用 xdg-open / open 在系统文件管理器里打开一个目录。
pub fn open_in_file_manager(path: &std::path::Path) {
    #[cfg(target_os = "macos")]
    let cmd = "open";
    #[cfg(target_os = "linux")]
    let cmd = "xdg-open";
    #[cfg(target_os = "windows")]
    let cmd = "explorer";

    let path = path.to_owned();
    // fork 到后台，不阻塞 UI
    std::thread::spawn(move || {
        if let Err(e) = std::process::Command::new(cmd).arg(&path).spawn() {
            log::warn!("打开 {} 失败: {e}", path.display());
        }
    });
}
