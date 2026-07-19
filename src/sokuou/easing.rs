/// 缓动函数枚举。
/// 只包含经过实际使用验证的函数；其余按需添加。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    Linear,
    EaseOutQuad,
    EaseOutCubic,
    EaseOutQuart,
    EaseInCubic,
    EaseInOutCubic,
}

impl Easing {
    pub fn apply(&self, t: f64) -> f64 {
        match self {
            Easing::Linear => t,
            Easing::EaseOutQuad => ease_out_quad(t),
            Easing::EaseOutCubic => ease_out_cubic(t),
            Easing::EaseOutQuart => ease_out_quart(t),
            Easing::EaseInCubic => ease_in_cubic(t),
            Easing::EaseInOutCubic => ease_in_out_cubic(t),
        }
    }
}

impl Default for Easing {
    fn default() -> Self {
        Easing::EaseOutCubic
    }
}

/// 平方缓出：快进慢出，轻量版。
pub fn ease_out_quad(t: f64) -> f64 {
    t * (2.0 - t)
}

/// 立方缓出：标准交互缓动，绝大多数场景首选。
pub fn ease_out_cubic(t: f64) -> f64 {
    let t1 = t - 1.0;
    1.0 + t1 * t1 * t1
}

/// 四次缓出：比立方更激进的减速，用于强调终止。
pub fn ease_out_quart(t: f64) -> f64 {
    let t1 = t - 1.0;
    1.0 - t1 * t1 * t1 * t1
}

/// 立方缓入：慢进快出，仅用于淡出。
pub fn ease_in_cubic(t: f64) -> f64 {
    t * t * t
}

/// 立方缓入缓出：用于循环动画（加载指示器等），日常交互不推荐。
pub fn ease_in_out_cubic(t: f64) -> f64 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        let u = -2.0 * t + 2.0;
        1.0 - u * u * u / 2.0
    }
}
