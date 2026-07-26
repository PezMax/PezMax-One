// DBusMenu 协议：内部菜单表示 + 到 D-Bus 类型的编组。
//
// 规范：https://github.com/AyatanaIndicators/libdbusmenu/blob/master/libdbusmenu-glib/dbus-menu.xml
//
// 关键点：
// - 每一项有 i32 id、a{sv} 属性、v 子项数组（av，每个 v 又包住 (ia{sv}av)——递归通过 variant 装箱解决）
// - toggle-type=radio + toggle-state=1 表示当前选中；同一父节点内的多个 radio 天然构成互斥组
// - visible/enabled 缺省 true；label 里 `_` 是助记符，`__` 才是字面下划线

use std::collections::HashMap;

use zbus::zvariant::{OwnedValue, Value};

/// 菜单项的开关类型（映射到 dbusmenu 的 `toggle-type` 属性）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Checkmark 保留给未来非互斥开关项使用
pub enum ToggleType {
    None,
    Checkmark,
    Radio,
}

/// 内部菜单节点。分隔符用 kind=Separator；子菜单直接看 children 非空。
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub id: i32,
    pub label: String,
    pub is_separator: bool,
    pub enabled: bool,
    pub visible: bool,
    pub toggle_type: ToggleType,
    /// 仅 toggle_type != None 时有意义：true=1, false=0
    pub toggle_state: bool,
    pub children: Vec<MenuItem>,
}

impl MenuItem {
    pub fn standard(id: i32, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            is_separator: false,
            enabled: true,
            visible: true,
            toggle_type: ToggleType::None,
            toggle_state: false,
            children: Vec::new(),
        }
    }

    pub fn separator(id: i32) -> Self {
        Self {
            id,
            label: String::new(),
            is_separator: true,
            enabled: true,
            visible: true,
            toggle_type: ToggleType::None,
            toggle_state: false,
            children: Vec::new(),
        }
    }

    pub fn submenu(id: i32, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            is_separator: false,
            enabled: true,
            visible: true,
            toggle_type: ToggleType::None,
            toggle_state: false,
            children: Vec::new(),
        }
    }

    pub fn radio(id: i32, label: impl Into<String>, checked: bool) -> Self {
        Self {
            id,
            label: label.into(),
            is_separator: false,
            enabled: true,
            visible: true,
            toggle_type: ToggleType::Radio,
            toggle_state: checked,
            children: Vec::new(),
        }
    }

    pub fn push_child(&mut self, child: MenuItem) {
        self.children.push(child);
    }

    /// 深度优先查找并可变返回节点；根节点也参与。
    pub fn find_mut(&mut self, id: i32) -> Option<&mut MenuItem> {
        if self.id == id {
            return Some(self);
        }
        for c in &mut self.children {
            if let Some(found) = c.find_mut(id) {
                return Some(found);
            }
        }
        None
    }

    /// 深度优先查找。
    pub fn find(&self, id: i32) -> Option<&MenuItem> {
        if self.id == id {
            return Some(self);
        }
        for c in &self.children {
            if let Some(found) = c.find(id) {
                return Some(found);
            }
        }
        None
    }

    /// 把节点的属性打包成 dbusmenu a{sv} 字典。
    /// 如果 filter 非空，只包含 filter 里指定的键（DBusMenu GetLayout 允许指定字段）。
    pub fn properties(&self, filter: &[String]) -> HashMap<String, OwnedValue> {
        let mut map: HashMap<String, OwnedValue> = HashMap::new();
        let want = |k: &str| filter.is_empty() || filter.iter().any(|f| f == k);

        if self.is_separator {
            if want("type") {
                map.insert("type".to_string(), str_val("separator"));
            }
        } else {
            if want("label") {
                // dbusmenu 用 _ 作助记符；我们没有助记符需求，保险起见把裸 _ 转义为 __
                let escaped = self.label.replace('_', "__");
                map.insert("label".to_string(), str_val(&escaped));
            }
            if !self.children.is_empty() && want("children-display") {
                map.insert("children-display".to_string(), str_val("submenu"));
            }
            match self.toggle_type {
                ToggleType::Checkmark => {
                    if want("toggle-type") {
                        map.insert("toggle-type".to_string(), str_val("checkmark"));
                    }
                    if want("toggle-state") {
                        map.insert(
                            "toggle-state".to_string(),
                            i32_val(if self.toggle_state { 1 } else { 0 }),
                        );
                    }
                }
                ToggleType::Radio => {
                    if want("toggle-type") {
                        map.insert("toggle-type".to_string(), str_val("radio"));
                    }
                    if want("toggle-state") {
                        map.insert(
                            "toggle-state".to_string(),
                            i32_val(if self.toggle_state { 1 } else { 0 }),
                        );
                    }
                }
                ToggleType::None => {}
            }
        }

        if want("enabled") && !self.enabled {
            map.insert("enabled".to_string(), bool_val(false));
        }
        if want("visible") && !self.visible {
            map.insert("visible".to_string(), bool_val(false));
        }

        map
    }

    /// 递归打包为 GetLayout 返回类型：(i32, a{sv}, av)
    /// depth: -1 表示无限；0 表示只当前节点，不含子；>0 表示还剩几层。
    pub fn to_layout(&self, depth: i32, filter: &[String]) -> LayoutItem {
        let props = self.properties(filter);
        let children = if depth == 0 {
            Vec::new()
        } else {
            let next_depth = if depth < 0 { -1 } else { depth - 1 };
            self.children
                .iter()
                .map(|c| c.to_layout(next_depth, filter))
                .collect()
        };
        LayoutItem {
            id: self.id,
            props,
            children,
        }
    }
}

/// GetLayout 返回的一层布局节点。序列化时通过 [`layout_to_value`] 装成 D-Bus 递归 variant。
#[derive(Debug)]
pub struct LayoutItem {
    pub id: i32,
    pub props: HashMap<String, OwnedValue>,
    pub children: Vec<LayoutItem>,
}

// ── zvariant 辅助 ─────────────────────────────────────────

fn str_val(s: &str) -> OwnedValue {
    Value::from(s.to_owned())
        .try_to_owned()
        .expect("str Value cannot fail to own")
}
fn bool_val(b: bool) -> OwnedValue {
    Value::from(b)
        .try_to_owned()
        .expect("bool Value cannot fail to own")
}
fn i32_val(v: i32) -> OwnedValue {
    Value::from(v)
        .try_to_owned()
        .expect("i32 Value cannot fail to own")
}

/// 把 `LayoutItem` 递归编码成 (i32, a{sv}, av) 的 zvariant Value。
/// 内层 children 数组元素类型为 v，每个 v 装箱一个 (ia{sv}av)。
///
/// 注意：如果签名不对，Plasma 的 GlobalMenu 会静默丢弃。签名必须严格为
/// `(ia{sv}av)`，av 里的每个 v 也必须包含 `(ia{sv}av)`。
pub fn layout_to_value(item: &LayoutItem) -> OwnedValue {
    use std::str::FromStr;
    use zbus::zvariant::{Array, Dict, Signature, StructureBuilder};

    let variant_sig = Signature::from_str("v").expect("valid signature");
    let key_sig = Signature::from_str("s").expect("valid signature");
    let val_sig = Signature::from_str("v").expect("valid signature");

    // 构造子项数组 av：元素签名声明为 v，因此每个元素必须显式装箱成
    // `Value::Value(Box<..>)`——否则 Array::append 会以 SignatureMismatch 报错。
    let mut av = Array::new(&variant_sig);
    for child in &item.children {
        let child_owned = layout_to_value(child);
        let child_value: Value<'static> = Value::from(child_owned);
        let boxed: Value<'static> = Value::Value(Box::new(child_value));
        if let Err(e) = av.append(boxed) {
            log::error!("append child variant 失败: {e}");
        }
    }

    // 构造 a{sv}：同理，value_sig=v，val 必须装箱。
    let mut props = Dict::new(&key_sig, &val_sig);
    for (k, v) in &item.props {
        let key_value: Value<'static> = Value::from(k.clone());
        let val_inner: Value<'static> = Value::from(v.clone());
        let val_boxed: Value<'static> = Value::Value(Box::new(val_inner));
        if let Err(e) = props.append(key_value, val_boxed) {
            log::error!("append prop {k} 失败: {e}");
        }
    }

    let structure = StructureBuilder::new()
        .add_field(item.id)
        .append_field(Value::from(props))
        .append_field(Value::from(av))
        .build()
        .expect("build layout struct 不会失败");

    Value::from(structure)
        .try_to_owned()
        .expect("layout Value 不会失败")
}
