// Sokuou Engine — 即応エンジン
// 空间状态解析器，用于 PezMax-One 的动画与视觉效果。
//
// 设计书：SOKUOU_ENGINE.md
// 调用手册：SOKUOU_USAGE.md

pub mod animator;
pub mod easing;
pub mod progress;
pub mod spring;

pub use animator::{Animation, Animator};
pub use easing::{
    ease_in_cubic, ease_in_out_cubic, ease_out_cubic, ease_out_quad, ease_out_quart, Easing,
};
pub use progress::Progress;
pub use spring::SpringAnim;

/// 将 progress [0.0, 1.0] 映射到任意值域 [from, to]。
///
/// ```
/// let alpha = map_range(spring.value(), 0.0, 1.0);
/// let slide = map_range(spring.value(), screen_height, 0.0); // 从底部滑入
/// let scale = map_range(spring.value(), 0.85, 1.0);          // 85% → 100%
/// ```
pub fn map_range(progress: f64, from: f64, to: f64) -> f64 {
    from + (to - from) * progress
}

/// 同 map_range，但先将 progress clamp 到 [0.0, 1.0]。
pub fn map_range_clamped(progress: f64, from: f64, to: f64) -> f64 {
    map_range(progress.clamp(0.0, 1.0), from, to)
}
