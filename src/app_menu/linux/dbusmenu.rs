// com.canonical.dbusmenu 服务实现
//
// 挂在 /MenuBar，供 KWin 的 Global Menu 组件 / AppMenu-Registrar 客户端读取。
// 只实现被 Plasma 实际调用的方法。ItemsPropertiesUpdated 用于勾选状态刷新。

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc as std_mpsc;

use tokio::sync::Mutex;
use zbus::interface;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::{OwnedValue, Value};

use super::proto::{MenuItem, layout_to_value};
use super::tree;
use crate::app_menu::MenuCommand;

/// DBusMenu 服务内部状态
pub struct DBusMenuState {
    /// 菜单树（勾选状态可变，因此包 Mutex）
    pub tree: MenuItem,
    /// 每次结构或属性更新递增；GetLayout 返回此值供客户端做失效比较。
    pub revision: u32,
    /// 点击命令送到主线程的通道
    pub tx: std_mpsc::Sender<MenuCommand>,
}

/// zbus interface 实现挂在 Arc<Mutex> 上，允许后端异步任务在别处修改状态并发信号。
pub struct DBusMenuIface {
    pub state: Arc<Mutex<DBusMenuState>>,
}

#[interface(name = "com.canonical.dbusmenu")]
impl DBusMenuIface {
    // ── 属性 ─────────────────────────────────────────────

    #[zbus(property)]
    async fn version(&self) -> u32 {
        3
    }

    #[zbus(property)]
    async fn status(&self) -> String {
        "normal".to_string()
    }

    #[zbus(property)]
    async fn text_direction(&self) -> String {
        "ltr".to_string()
    }

    #[zbus(property)]
    async fn icon_theme_path(&self) -> Vec<String> {
        Vec::new()
    }

    // ── 方法 ─────────────────────────────────────────────

    /// 返回子树布局。
    ///
    /// - `parent_id` = 0 时返回整棵
    /// - `recursion_depth` = -1 表示无限层；0 表示只当前节点；>0 表示还剩几层
    /// - `property_names` 为空 = 返回全部支持的属性
    async fn get_layout(
        &self,
        parent_id: i32,
        recursion_depth: i32,
        property_names: Vec<String>,
    ) -> (u32, (i32, HashMap<String, OwnedValue>, Vec<OwnedValue>)) {
        let state = self.state.lock().await;
        let node = state.tree.find(parent_id).unwrap_or(&state.tree);
        let layout = node.to_layout(recursion_depth, &property_names);

        // 顶层 struct 手动拆开——DBus 签名对根节点用具体类型而非 variant，
        // 只有 av 里的子项需要 layout_to_value 装箱。
        let children_variants: Vec<OwnedValue> = layout
            .children
            .iter()
            .map(|c| layout_to_value(c))
            .collect();

        (state.revision, (layout.id, layout.props, children_variants))
    }

    /// 返回一组节点的属性。property_names 为空 = 返回全部。
    async fn get_group_properties(
        &self,
        ids: Vec<i32>,
        property_names: Vec<String>,
    ) -> Vec<(i32, HashMap<String, OwnedValue>)> {
        let state = self.state.lock().await;
        ids.into_iter()
            .filter_map(|id| {
                state
                    .tree
                    .find(id)
                    .map(|node| (id, node.properties(&property_names)))
            })
            .collect()
    }

    /// 返回单个节点的一个属性。
    async fn get_property(&self, id: i32, name: String) -> OwnedValue {
        let state = self.state.lock().await;
        if let Some(node) = state.tree.find(id) {
            let props = node.properties(std::slice::from_ref(&name));
            if let Some(v) = props.get(&name) {
                return v.clone();
            }
        }
        // 未知则返回空字符串（避免报错让客户端断连）
        Value::from(String::new()).try_to_owned().unwrap()
    }

    /// 菜单事件——点击 / hover。我们只处理 clicked。
    async fn event(
        &self,
        id: i32,
        event_id: String,
        _data: OwnedValue,
        _timestamp: u32,
    ) {
        if event_id != "clicked" {
            return;
        }
        let state = self.state.lock().await;
        if let Some(cmd) = tree::id_to_command(id) {
            if let Err(e) = state.tx.send(cmd) {
                log::warn!("菜单命令发送失败: {e}");
            }
        }
    }

    /// 批量事件版本。返回未找到的 id 列表（我们从不失败，返回空即可）。
    async fn event_group(
        &self,
        events: Vec<(i32, String, OwnedValue, u32)>,
    ) -> Vec<i32> {
        for (id, event_id, data, ts) in events {
            self.event(id, event_id, data, ts).await;
        }
        Vec::new()
    }

    /// 客户端准备展开子菜单前的钩子。返回 true 表示需要重新拉 layout。
    /// 我们的菜单结构不动，永远返回 false。
    async fn about_to_show(&self, _id: i32) -> bool {
        false
    }

    /// 批量版本。第一个返回值是"需要更新的 id 列表"，第二个是"未找到的 id 列表"。
    async fn about_to_show_group(&self, _ids: Vec<i32>) -> (Vec<i32>, Vec<i32>) {
        (Vec::new(), Vec::new())
    }

    // ── 信号 ─────────────────────────────────────────────
    // ItemsPropertiesUpdated 和 LayoutUpdated 由外部代码通过 SignalEmitter 直接触发。

    #[zbus(signal)]
    pub async fn items_properties_updated(
        emitter: &SignalEmitter<'_>,
        updated: Vec<(i32, HashMap<String, OwnedValue>)>,
        removed: Vec<(i32, Vec<String>)>,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    pub async fn layout_updated(
        emitter: &SignalEmitter<'_>,
        revision: u32,
        parent: i32,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    pub async fn item_activation_requested(
        emitter: &SignalEmitter<'_>,
        id: i32,
        timestamp: u32,
    ) -> zbus::Result<()>;
}
