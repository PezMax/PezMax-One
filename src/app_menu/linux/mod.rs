// Linux 菜单后端子模块
//
// dbusmenu：实现 com.canonical.dbusmenu，供 Plasma Global Menu 读取
// proto：MenuItem 类型 + DBusMenu 序列化辅助
// tree：菜单结构 + 稳定 ID + 命令映射
// wayland：Task #6 会填充——绑定 KWin org_kde_kwin_appmenu 协议

pub mod dbusmenu;
pub mod proto;
pub mod tree;
pub mod wayland;

use std::sync::Arc;
use std::sync::mpsc as std_mpsc;

use tokio::sync::{Mutex, mpsc as tokio_mpsc};

use super::{MenuBackend, MenuCommand};
use crate::theme::{self, ThemeMode};

pub use dbusmenu::{DBusMenuIface, DBusMenuState};

/// D-Bus 上服务菜单的对象路径。Plasma / DBusMenu 客户端约定挂在 /MenuBar。
pub const MENUBAR_OBJECT_PATH: &str = "/MenuBar";

/// 后端 → tokio 任务的通知。用来触发 D-Bus 信号 / Wayland 重新 set_address。
pub enum BackendNotify {
    ThemeChanged(ThemeMode),
    AccentChanged(usize),
}

/// 平台句柄。持有 tokio 任务的通知 tx；drop 时任务由 sender-close 自然退出。
pub struct LinuxMenu {
    tx: tokio_mpsc::UnboundedSender<BackendNotify>,
}

impl MenuBackend for LinuxMenu {
    fn set_theme_mode(&self, mode: ThemeMode) {
        let _ = self.tx.send(BackendNotify::ThemeChanged(mode));
    }
    fn set_accent(&self, idx: usize) {
        let _ = self.tx.send(BackendNotify::AccentChanged(idx));
    }
}

/// 启动 D-Bus 服务 + Wayland 绑定。
/// - `menu_tx`: 用户点菜单项时把 MenuCommand 送到这里
/// - `wl_display` / `wl_surface`: winit 提供的原始指针（Wayland-only），
///   由外层调用点在窗口创建后传入。为 None 时跳过 Wayland 绑定，只上 DBus + Registrar。
pub fn spawn(
    menu_tx: std_mpsc::Sender<MenuCommand>,
    wl_handles: Option<wayland::WaylandHandles>,
) -> anyhow::Result<LinuxMenu> {
    let (notify_tx, notify_rx) = tokio_mpsc::unbounded_channel();

    // 初始菜单树用当前主题/强调色的默认值构造；PezMaxApp 起来后会同步一次
    let initial_tree = tree::build(ThemeMode::System, 0);
    let state = Arc::new(Mutex::new(DBusMenuState {
        tree: initial_tree,
        revision: 1,
        tx: menu_tx,
    }));

    let state_for_task = state.clone();
    tokio::spawn(async move {
        if let Err(e) = run_service(state_for_task, notify_rx, wl_handles).await {
            log::warn!("Linux 菜单服务异常退出: {e:#}");
        }
    });

    Ok(LinuxMenu { tx: notify_tx })
}

async fn run_service(
    state: Arc<Mutex<DBusMenuState>>,
    mut notify_rx: tokio_mpsc::UnboundedReceiver<BackendNotify>,
    wl_handles: Option<wayland::WaylandHandles>,
) -> anyhow::Result<()> {
    let iface = DBusMenuIface {
        state: state.clone(),
    };

    let conn = zbus::connection::Builder::session()?
        .serve_at(MENUBAR_OBJECT_PATH, iface)?
        .build()
        .await?;

    let unique_name = conn
        .unique_name()
        .map(|n| n.to_string())
        .unwrap_or_else(|| ":pezmax.menu".to_string());

    log::info!(
        "DBusMenu 服务已启动: service={} path={}",
        unique_name,
        MENUBAR_OBJECT_PATH
    );

    // ── Wayland 绑定（Task #6）─────────────────────────
    if let Some(handles) = wl_handles {
        match wayland::bind_appmenu(handles, &unique_name, MENUBAR_OBJECT_PATH) {
            Ok(_) => log::info!("KWin appmenu 协议绑定成功"),
            Err(e) => log::warn!("KWin appmenu 绑定失败（可能不是 KDE Wayland 会话）: {e:#}"),
        }
    }

    // ── AppMenu Registrar 兜底 ────────────────────────
    // 部分 Global Menu 实现（老 Plasma / Unity / 独立 dbusmenu 面板）走 Registrar。
    // KDE Wayland 走 KWin 协议，Registrar 只是保险，无从获取真实 window_id
    // 就传 0；Plasma 在 Wayland 下不会用这个值。失败静默跳过。
    if let Err(e) = register_with_registrar(&conn, MENUBAR_OBJECT_PATH).await {
        log::debug!("AppMenu Registrar 注册跳过: {e:#}");
    }

    // ── 主循环：消费 backend 通知 → 更新状态 + 发信号 ────
    while let Some(notify) = notify_rx.recv().await {
        match notify {
            BackendNotify::ThemeChanged(mode) => {
                update_theme_mode(&state, mode).await;
                emit_theme_updated(&conn, mode).await;
            }
            BackendNotify::AccentChanged(idx) => {
                update_accent(&state, idx).await;
                emit_accent_updated(&conn, idx).await;
            }
        }
    }
    // sender 关闭 → 任务退出
    Ok(())
}

/// 修改主题勾选：把三个主题项的 toggle-state 更新。
async fn update_theme_mode(state: &Arc<Mutex<DBusMenuState>>, mode: ThemeMode) {
    let mut s = state.lock().await;
    let target = tree::theme_mode_id(mode);
    for &id in tree::THEME_ALL_IDS {
        if let Some(node) = s.tree.find_mut(id) {
            node.toggle_state = id == target;
        }
    }
    s.revision = s.revision.wrapping_add(1);
}

/// 修改强调色勾选。
async fn update_accent(state: &Arc<Mutex<DBusMenuState>>, idx: usize) {
    let mut s = state.lock().await;
    let target = tree::ID_ACCENT_BASE + idx as i32;
    for i in 0..theme::ACCENT_PRESETS.len() {
        let id = tree::ID_ACCENT_BASE + i as i32;
        if let Some(node) = s.tree.find_mut(id) {
            node.toggle_state = id == target;
        }
    }
    s.revision = s.revision.wrapping_add(1);
}

/// 发信号：主题子菜单三项 toggle-state 变化。
async fn emit_theme_updated(conn: &zbus::Connection, mode: ThemeMode) {
    use std::collections::HashMap;
    use zbus::zvariant::{OwnedValue, Value};

    let target = tree::theme_mode_id(mode);
    let updated: Vec<(i32, HashMap<String, OwnedValue>)> = tree::THEME_ALL_IDS
        .iter()
        .map(|&id| {
            let mut props = HashMap::new();
            let on = if id == target { 1i32 } else { 0i32 };
            props.insert(
                "toggle-state".to_string(),
                Value::from(on).try_to_owned().unwrap(),
            );
            (id, props)
        })
        .collect();

    if let Err(e) = emit_items_properties_updated(conn, updated).await {
        log::warn!("发送主题更新信号失败: {e:#}");
    }
}

/// 发信号：强调色子菜单 n 项 toggle-state 变化。
async fn emit_accent_updated(conn: &zbus::Connection, idx: usize) {
    use std::collections::HashMap;
    use zbus::zvariant::{OwnedValue, Value};

    let target = tree::ID_ACCENT_BASE + idx as i32;
    let updated: Vec<(i32, HashMap<String, OwnedValue>)> = (0..theme::ACCENT_PRESETS.len())
        .map(|i| {
            let id = tree::ID_ACCENT_BASE + i as i32;
            let mut props = HashMap::new();
            let on = if id == target { 1i32 } else { 0i32 };
            props.insert(
                "toggle-state".to_string(),
                Value::from(on).try_to_owned().unwrap(),
            );
            (id, props)
        })
        .collect();

    if let Err(e) = emit_items_properties_updated(conn, updated).await {
        log::warn!("发送强调色更新信号失败: {e:#}");
    }
}

async fn emit_items_properties_updated(
    conn: &zbus::Connection,
    updated: Vec<(i32, std::collections::HashMap<String, zbus::zvariant::OwnedValue>)>,
) -> zbus::Result<()> {
    let iface_ref = conn
        .object_server()
        .interface::<_, DBusMenuIface>(MENUBAR_OBJECT_PATH)
        .await?;
    DBusMenuIface::items_properties_updated(iface_ref.signal_emitter(), updated, Vec::new())
        .await
}

async fn register_with_registrar(conn: &zbus::Connection, object_path: &str) -> zbus::Result<()> {
    use zbus::zvariant::ObjectPath;
    let proxy = zbus::Proxy::new(
        conn,
        "com.canonical.AppMenu.Registrar",
        "/com/canonical/AppMenu/Registrar",
        "com.canonical.AppMenu.Registrar",
    )
    .await?;
    // window_id: Wayland 环境下无真实 XID。传 0——Plasma applet 在 Wayland 会用 KWin
    // 协议路径拿菜单地址，Registrar 只是保险。
    let _: () = proxy
        .call(
            "RegisterWindow",
            &(0u32, ObjectPath::try_from(object_path).unwrap()),
        )
        .await?;
    Ok(())
}
