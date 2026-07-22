/// UWP Metro 缓动函数体系 + MetroAnim 动画原语。
///
/// 对应 Windows.UI.Xaml.Media.Animation 命名空间下的 EasingFunctionBase 派生类。
/// 所有函数支持 EasingMode（EaseIn / EaseOut / EaseInOut）。
///
/// 设计目标：轻盈、短时、60Hz 友好。
/// 默认 duration 0.25s，默认变体 Quadratic/EaseOut。
///
/// 从 sokuou-engine-toolkit 复制，用于 PezMax-One 的色彩过渡动画。

/// 缓动方向模式。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EasingMode {
    EaseIn,
    EaseOut,
    EaseInOut,
}

/// UWP 缓动函数变体。
#[derive(Debug, Clone, PartialEq)]
pub enum UwpEasing {
    Quadratic,
    Cubic,
    Quartic,
    Quintic,
    Sine,
    /// 参数为 Exponent（指数）
    Power(f64),
    /// 参数为 Exponent（指数乘数）
    Exponential(f64),
    Circle,
    /// 参数为 Amplitude（振幅），控制过冲量
    Back(f64),
    /// Bounces：弹跳次数，Bounciness：弹性系数
    Bounce { bounces: i32, bounciness: f64 },
    /// Oscillations：振荡次数，Springiness：弹簧刚度
    Elastic { oscillations: i32, springiness: f64 },
}

impl Default for UwpEasing {
    fn default() -> Self {
        UwpEasing::Quadratic
    }
}

// ── 原始缓动函数（EaseIn 方向） ─────────────────────────────

/// 原始缓动函数 f(t)，t∈[0,1]，返回 [0,1] 或过冲值。
fn ease_in_raw(t: f64, variant: &UwpEasing) -> f64 {
    let t = t.clamp(0.0, 1.0);
    match *variant {
        UwpEasing::Quadratic => t * t,
        UwpEasing::Cubic => t * t * t,
        UwpEasing::Quartic => t * t * t * t,
        UwpEasing::Quintic => t * t * t * t * t,
        UwpEasing::Sine => 1.0 - (std::f64::consts::FRAC_PI_2 * (1.0 - t)).sin(),
        UwpEasing::Power(exp) => t.powf(exp),
        UwpEasing::Exponential(exp) => {
            if t <= 0.0 {
                0.0
            } else {
                (2.0_f64).powf(10.0 * exp * (t - 1.0))
            }
        }
        UwpEasing::Circle => 1.0 - (1.0 - t * t).sqrt(),
        UwpEasing::Back(amplitude) => {
            let c = 1.70158 * amplitude;
            let c1 = c + 1.0;
            c1 * t * t * t - c * t * t
        }
        UwpEasing::Bounce { bounces, bounciness } => {
            if t <= 0.0 || t >= 1.0 {
                return t;
            }
            let b = bounces.max(1) as usize;
            let c = bounciness.max(0.001);
            let mut total = 0.0;
            let mut pow = 1.0;
            for _ in 0..b {
                total += pow;
                pow *= c;
            }
            let t_scaled = t * total;
            pow = 1.0;
            let mut acc = 0.0;
            for i in 0..b {
                let dur = pow;
                if t_scaled <= acc + dur {
                    let local = (t_scaled - acc) / dur;
                    if i == 0 {
                        return 1.0 - (1.0 - local).powi(2);
                    } else {
                        let height = 1.0 / c.powi(i as i32);
                        return 1.0 - 4.0 * local * (1.0 - local) * height;
                    }
                }
                acc += dur;
                pow *= c;
            }
            t
        }
        UwpEasing::Elastic { oscillations, springiness } => {
            if t <= 0.0 {
                return 0.0;
            }
            if t >= 1.0 {
                return 1.0;
            }
            let osc = oscillations.max(1) as f64;
            let spring = springiness.max(0.001);
            let phase = osc * std::f64::consts::PI * t;
            let decay = (-spring * t).exp();
            1.0 - decay * phase.cos()
        }
    }
}

// ── 公开 API ────────────────────────────────────────────────

/// 应用 UWP 缓动函数，支持 EasingMode 转换。
pub fn apply_uwp(t: f64, variant: &UwpEasing, mode: EasingMode) -> f64 {
    match mode {
        EasingMode::EaseIn => ease_in_raw(t, variant),
        EasingMode::EaseOut => {
            if t <= 0.0 {
                return 0.0;
            }
            if t >= 1.0 {
                return 1.0;
            }
            1.0 - ease_in_raw(1.0 - t, variant)
        }
        EasingMode::EaseInOut => {
            if t <= 0.0 {
                return 0.0;
            }
            if t >= 1.0 {
                return 1.0;
            }
            if t < 0.5 {
                0.5 * ease_in_raw(2.0 * t, variant)
            } else {
                1.0 - 0.5 * ease_in_raw(2.0 - 2.0 * t, variant)
            }
        }
    }
}

/// 时长驱动的 UWP Metro 缓动值。
///
/// 与 `Progress` 结构对齐，但使用 UWP 缓动函数体系。
/// 默认时长为 0.25s（Metro 风格），默认变体 Quadratic/EaseOut。
pub struct MetroAnim {
    value: f64,
    target: f64,
    from: f64,
    elapsed: f64,
    duration: f64,
    variant: UwpEasing,
    mode: EasingMode,
}

impl MetroAnim {
    pub const fn new(duration: f64, variant: UwpEasing, mode: EasingMode) -> Self {
        Self {
            value: 0.0,
            target: 0.0,
            from: 0.0,
            elapsed: duration,
            duration,
            variant,
            mode,
        }
    }

    /// 默认构造：0.25s, Quadratic, EaseOut
    #[allow(dead_code)]
    pub fn default_metro() -> Self {
        Self::new(0.25, UwpEasing::Quadratic, EasingMode::EaseOut)
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
        let t_eased = apply_uwp(t, &self.variant, self.mode);
        self.value = self.from + (self.target - self.from) * t_eased;
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    /// 已到达目标时为 true。
    pub fn is_steady(&self) -> bool {
        self.elapsed >= self.duration
    }
}