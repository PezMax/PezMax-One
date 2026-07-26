// KWin Wayland org_kde_kwin_appmenu 协议绑定
//
// 目标：在 winit 已经建好 wl_display / wl_surface 之后，从同一个 libwayland
// 连接上绑定 KWin 的私有全局 `org_kde_kwin_appmenu_manager`，给我们的
// wl_surface 挂一个 appmenu 对象，并 set_address(service_name, "/MenuBar")。
// Plasma 的 Global Menu applet 会从这个地址拉 DBusMenu。
//
// 关键约束：
// 1. 必须复用 winit 的 wl_display——appmenu 只对同连接的 wl_surface 生效
// 2. 用 `Backend::from_foreign_display` 借用现有连接（不 own，drop 不关闭）
// 3. 用 `ObjectId::from_ptr` 把 winit 的裸 wl_surface 指针包成 Rust proxy
// 4. 用独立的 event_queue 处理 registry 事件——libwayland 支持多队列，
//    winit 的默认队列不受影响
// 5. 我们 send 的 set_address 会走 libwayland 的通用 flush，此后 winit 的
//    每次 wl_display_dispatch/flush 都会把我们的请求带出去，所以设完就 OK
// 6. appmenu proxy 一 drop 就走 release()——所以我们把状态 Box::leak 长驻

use std::ffi::c_void;
use std::sync::mpsc as std_mpsc;
use std::sync::Arc;

use anyhow::{anyhow, Context};
use wayland_backend::sys::client::{Backend, ObjectId};
use wayland_client::protocol::wl_registry::{Event as RegistryEvent, WlRegistry};
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{Connection, Dispatch, Proxy, QueueHandle};

// ── KWin appmenu 协议 —— 从 resources/wayland-protocols/appmenu.xml 生成 ──

pub mod appmenu {
    use wayland_client;
    use wayland_client::protocol::*;

    pub mod __interfaces {
        use wayland_client::protocol::__interfaces::*;
        wayland_scanner::generate_interfaces!("resources/wayland-protocols/appmenu.xml");
    }
    use self::__interfaces::*;

    wayland_scanner::generate_client_code!("resources/wayland-protocols/appmenu.xml");
}

use appmenu::org_kde_kwin_appmenu::OrgKdeKwinAppmenu;
use appmenu::org_kde_kwin_appmenu_manager::OrgKdeKwinAppmenuManager;

/// winit 提供的 Wayland 原始指针。发送到工作线程使用，因此加 unsafe Send/Sync 承诺。
#[allow(dead_code)]
pub struct WaylandHandles {
    pub display: *mut c_void,
    pub surface: *mut c_void,
}

// SAFETY: 指针来自 winit，winit 保证它们在窗口生命周期内有效。
// 我们只在窗口创建后使用；退出前不会 drop 它们。
unsafe impl Send for WaylandHandles {}
unsafe impl Sync for WaylandHandles {}

/// 绑定 appmenu。同步阻塞（内部 spawn 独立线程完成绑定，主 tokio 线程
/// 立刻拿到结果不被阻塞）。返回后 appmenu 对象由后台线程持有直到进程结束。
///
/// 失败原因常见：
///   - 不是 Wayland 会话
///   - Wayland 会话不是 KDE（`org_kde_kwin_appmenu_manager` 全局不存在）
///   - libwayland-client.so.0 加载失败
pub fn bind_appmenu(
    handles: WaylandHandles,
    service_name: &str,
    object_path: &str,
) -> anyhow::Result<()> {
    let service_name = service_name.to_owned();
    let object_path = object_path.to_owned();
    let (tx, rx) = std_mpsc::channel::<anyhow::Result<()>>();

    // Spawn independent std thread — blocking_dispatch 不能在 tokio worker 上跑
    std::thread::Builder::new()
        .name("pezmax-kwin-appmenu".into())
        .spawn(move || {
            let result = do_bind(&handles, &service_name, &object_path, tx);
            if let Err(e) = &result {
                log::warn!("KWin appmenu 绑定线程退出: {e:#}");
            }
        })
        .map_err(|e| anyhow!("无法启动 wayland 绑定线程: {e}"))?;

    // 等待绑定完成（成功或失败）；线程随后自己 park 保持 appmenu 存活
    rx.recv()
        .map_err(|_| anyhow!("wayland 绑定线程意外退出"))?
}

/// 在专用线程内执行 bind。返回后线程 park 保持全部 wayland 对象存活。
fn do_bind(
    handles: &WaylandHandles,
    service_name: &str,
    object_path: &str,
    report: std_mpsc::Sender<anyhow::Result<()>>,
) -> anyhow::Result<()> {
    // SAFETY: winit 保证 display 指针在整个窗口生命周期内有效；我们不释放它。
    // 用 as *mut _ 让编译器推断出 wayland_sys 需要的 wl_display 类型。
    let backend = unsafe { Backend::from_foreign_display(handles.display as *mut _) };
    let conn = Connection::from_backend(backend);

    let mut event_queue = conn.new_event_queue::<AppState>();
    let qh = event_queue.handle();

    // 1. 拉全局列表
    let display = conn.display();
    let _registry = display.get_registry(&qh, ());
    let mut state = AppState {
        manager: None,
        error: None,
    };
    event_queue
        .roundtrip(&mut state)
        .context("wayland registry roundtrip 失败")?;
    if let Some(e) = state.error.take() {
        let _ = report.send(Err(anyhow!(e)));
        return Ok(());
    }

    let manager = state
        .manager
        .clone()
        .ok_or_else(|| anyhow!("org_kde_kwin_appmenu_manager 全局不存在——非 KDE 会话？"))?;

    // 2. 把 winit 的 wl_surface 包成 Rust proxy
    // SAFETY: surface 指针来自 winit 的 raw-window-handle，winit 保证它是有效
    // wl_proxy 且属于同一 wl_display。ObjectId 只是记 pointer + interface。
    let surface_id = unsafe {
        ObjectId::from_ptr(WlSurface::interface(), handles.surface as *mut _)
            .context("wl_surface ObjectId::from_ptr 失败")?
    };
    let surface = WlSurface::from_id(&conn, surface_id)
        .context("WlSurface::from_id 失败")?;

    // 3. 创建 appmenu 对象、set_address
    let appmenu = manager.create(&surface, &qh, ());
    appmenu.set_address(service_name.to_owned(), object_path.to_owned());

    // 4. 显式 flush 一次，把请求推到 wire——之后 winit 每次 dispatch 都会带出
    conn.flush().context("wayland flush 失败")?;

    log::info!(
        "KWin appmenu 已 set_address: service={} path={}",
        service_name,
        object_path
    );
    let _ = report.send(Ok(()));

    // 5. 长驻线程：leak 掉整套状态，保持 appmenu 对象生存直到进程退出。
    //    不能让 appmenu 或 manager Drop——proxy Drop 会 send release()。
    //    也保留 conn 和 event_queue，虽然我们不再 dispatch，但 conn Drop
    //    对 foreign display 是无害的（不 own display）。
    let leaked = Box::new((conn, event_queue, manager, appmenu, surface));
    let _static: &'static mut _ = Box::leak(leaked);
    // 线程返回；leak 的对象随进程结束自然释放。
    Ok(())
}

// ── event dispatcher ─────────────────────────────────────

struct AppState {
    manager: Option<OrgKdeKwinAppmenuManager>,
    error: Option<String>,
}

impl Dispatch<WlRegistry, ()> for AppState {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: RegistryEvent,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let RegistryEvent::Global { name, interface, version } = event {
            if interface == "org_kde_kwin_appmenu_manager" {
                let v = version.min(1);
                match registry.bind::<OrgKdeKwinAppmenuManager, _, _>(name, v, qh, ()) {
                    m => state.manager = Some(m),
                }
            }
        }
    }
}

// appmenu manager 和 appmenu 都没有 event，用 delegate_noop
wayland_client::delegate_noop!(AppState: OrgKdeKwinAppmenuManager);
wayland_client::delegate_noop!(AppState: OrgKdeKwinAppmenu);

// 静音 unused_imports 警告（Arc / Proxy 用于生成代码内部）
const _: fn() = || {
    let _ = std::mem::size_of::<Arc<()>>();
    let _ = <WlSurface as Proxy>::interface();
};
