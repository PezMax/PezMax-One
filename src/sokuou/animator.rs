// RESERVED — 编排层预留接口
//
// Animation trait 和 Animator 编排器在此预留，但尚未有实际应用验证。
// 设计书 §3.5 明确指出：不要预先实现编排器。
//
// 何时启用：
//   当 PezMax 出现"多个动画需要串行/并行依赖"的真实需求时，
//   在 NOTE.md 中记录具体场景，然后在此实现。
//
// 当前状态：仅定义 trait 和空结构体，不可用于生产代码。

/// 动画对象的统一接口（预留）。
#[allow(dead_code)]
pub trait Animation {
    fn update(&mut self, dt: f64);
    fn value(&self) -> f64;
    fn is_steady(&self) -> bool;
}

/// 多动画编排器（预留）。
#[allow(dead_code)]
pub struct Animator {
    _reserved: (),
}

#[allow(dead_code)]
impl Animator {
    pub fn new() -> Self {
        Self { _reserved: () }
    }
}
