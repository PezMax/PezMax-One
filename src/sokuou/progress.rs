use crate::sokuou::easing::Easing;

/// 时长驱动的线性缓动值 [0.0, 1.0]，支持可中断。
///
/// 适用于透明度、颜色等不需要弹性的过渡。
/// 位移/缩放等需要"自然感"的属性请用 SpringAnim。
pub struct Progress {
    value: f64,
    target: f64,
    from: f64,
    elapsed: f64,
    duration: f64,
    easing: Easing,
}

impl Progress {
    /// 创建，默认 EaseOutCubic。
    pub fn new(duration_secs: f64) -> Self {
        Self {
            value: 0.0,
            target: 0.0,
            from: 0.0,
            elapsed: duration_secs, // 初始为稳态
            duration: duration_secs,
            easing: Easing::EaseOutCubic,
        }
    }

    pub fn with_easing(duration_secs: f64, easing: Easing) -> Self {
        Self {
            easing,
            ..Self::new(duration_secs)
        }
    }

    /// 设定新目标，从当前 value 开始过渡（可中断）。
    pub fn set_target(&mut self, target: f64) {
        if self.is_steady() && (target - self.target).abs() < f64::EPSILON {
            return;
        }
        self.from = self.value;
        self.target = target;
        self.elapsed = 0.0;
    }

    pub fn set_duration(&mut self, duration_secs: f64) {
        self.duration = duration_secs;
    }

    pub fn set_easing(&mut self, easing: Easing) {
        self.easing = easing;
    }

    /// 立即跳到指定值，无过渡。
    pub fn jump_to(&mut self, value: f64) {
        self.value = value;
        self.target = value;
        self.from = value;
        self.elapsed = self.duration;
    }

    /// 每帧调用。`dt` 单位为秒。
    pub fn update(&mut self, dt: f64) {
        if self.is_steady() {
            return;
        }
        self.elapsed = (self.elapsed + dt).min(self.duration);
        let t = if self.duration > 0.0 {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let t_eased = self.easing.apply(t);
        self.value = self.from + (self.target - self.from) * t_eased;
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn target(&self) -> f64 {
        self.target
    }

    /// 已到达目标时为 true，此帧无需 request_repaint。
    pub fn is_steady(&self) -> bool {
        self.elapsed >= self.duration
    }
}
