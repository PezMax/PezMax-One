// Linux 菜单后端入口。
//
// 真正的实现在 `linux/` 子模块。这里负责：
// 1. 从 CreationContext 拿 Wayland raw handle
// 2. 起 zbus 服务 + Wayland 绑定（都在 tokio 上）
// 3. 返回 MenuBackend trait 对象供 PezMaxApp 持有

use std::sync::mpsc;

use raw_window_handle::{
    HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle,
};

use super::{MenuBackend, MenuCommand, linux};

/// 由 crate::app_menu::install() 调用。返回值必须是 Send + Sync 的 MenuBackend。
pub fn install(
    cc: &eframe::CreationContext<'_>,
    tx: mpsc::Sender<MenuCommand>,
) -> anyhow::Result<Box<dyn MenuBackend>> {
    let wl_handles = extract_wayland_handles(cc);
    if wl_handles.is_none() {
        log::info!("非 Wayland 会话（或窗口句柄未就绪），跳过 KWin appmenu 绑定，仅上 DBusMenu + Registrar");
    }
    let backend = linux::spawn(tx, wl_handles)?;
    Ok(Box::new(backend))
}

fn extract_wayland_handles(
    cc: &eframe::CreationContext<'_>,
) -> Option<linux::wayland::WaylandHandles> {
    let display_handle = cc.display_handle().ok()?;
    let window_handle = cc.window_handle().ok()?;
    match (display_handle.as_raw(), window_handle.as_raw()) {
        (RawDisplayHandle::Wayland(wd), RawWindowHandle::Wayland(ww)) => {
            Some(linux::wayland::WaylandHandles {
                display: wd.display.as_ptr(),
                surface: ww.surface.as_ptr(),
            })
        }
        _ => None,
    }
}
