/// 弹簧动画状态机。
///
/// 基于阻尼振荡器的**解析解**（非欧拉数值积分）：
/// - 帧率无关：60Hz 与 120Hz 轨迹完全一致
/// - 无条件稳定：dt 再大也不会发散
/// - 可中断：任意时刻 set_target，从当前 pos/vel 继续
///
/// 参数使用 Apple 风格 response/damping_ratio：
/// - `response`：响应时间（秒），近似"动画持续时间"
/// - `damping_ratio`：阻尼比，0.825 为标准系统弹簧，<1 有过冲，>1 无过冲
pub struct SpringAnim {
    response: f64,
    damping_ratio: f64,
    from: f64,
    from_vel: f64,
    target: f64,
    elapsed: f64,
    pos: f64,
    vel: f64,
    steady: bool,
}

const SNAP_THRESHOLD: f64 = 0.001;

impl SpringAnim {
    /// 创建弹簧，初始静止于 `initial`，目标也为 `initial`。
    pub fn new(response: f64, damping_ratio: f64, initial: f64) -> Self {
        Self {
            response,
            damping_ratio,
            from: initial,
            from_vel: 0.0,
            target: initial,
            elapsed: 0.0,
            pos: initial,
            vel: 0.0,
            steady: true,
        }
    }

    /// 创建弹簧，同时指定起始条件和目标。
    pub fn with_target(
        response: f64,
        damping_ratio: f64,
        from: f64,
        from_vel: f64,
        target: f64,
    ) -> Self {
        Self {
            response,
            damping_ratio,
            from,
            from_vel,
            target,
            elapsed: 0.0,
            pos: from,
            vel: from_vel,
            steady: false,
        }
    }

    /// 设定新目标，继承当前 pos/vel（不回头，不卡顿）。
    pub fn set_target(&mut self, new_target: f64) {
        if self.steady && (new_target - self.target).abs() < f64::EPSILON {
            return;
        }
        self.from = self.pos;
        self.from_vel = self.vel;
        self.target = new_target;
        self.elapsed = 0.0;
        self.steady = false;
    }

    /// 设定新目标，并指定起始速度（用于手势速度继承）。
    pub fn set_target_with_velocity(&mut self, new_target: f64, from_vel: f64) {
        self.from = self.pos;
        self.from_vel = from_vel;
        self.target = new_target;
        self.elapsed = 0.0;
        self.steady = false;
    }

    /// 动态调整物理参数，从当前状态重新计算。
    pub fn set_params(&mut self, response: f64, damping_ratio: f64) {
        self.response = response;
        self.damping_ratio = damping_ratio;
        self.from = self.pos;
        self.from_vel = self.vel;
        self.elapsed = 0.0;
    }

    /// 每帧调用。`dt` 单位为秒（来自 `ctx.input(|i| i.stable_dt) as f64`）。
    pub fn update(&mut self, dt: f64) {
        if self.steady {
            return;
        }
        self.elapsed += dt.min(0.05);
        let (pos, vel) = evaluate(
            self.response,
            self.damping_ratio,
            self.from,
            self.from_vel,
            self.target,
            self.elapsed,
        );
        self.pos = pos;
        self.vel = vel;

        if (self.pos - self.target).abs() < SNAP_THRESHOLD && self.vel.abs() < SNAP_THRESHOLD {
            self.snap();
        }
    }

    /// 立即吸附到目标，标记为稳态。
    pub fn snap(&mut self) {
        self.pos = self.target;
        self.vel = 0.0;
        self.steady = true;
    }

    pub fn value(&self) -> f64 {
        self.pos
    }

    pub fn velocity(&self) -> f64 {
        self.vel
    }

    pub fn target(&self) -> f64 {
        self.target
    }

    pub fn is_steady(&self) -> bool {
        self.steady
    }
}

/// 阻尼振荡器解析解。
///
/// 设误差 y(t) = target - x(t)，初始条件：
///   y(0) = target - from，  ẏ(0) = -from_vel
///
/// 返回 (pos, vel)。
fn evaluate(
    response: f64,
    zeta: f64,
    from: f64,
    from_vel: f64,
    target: f64,
    t: f64,
) -> (f64, f64) {
    let omega0 = std::f64::consts::TAU / response; // 2π / response

    let y0 = target - from;
    let dy0 = -from_vel;

    if zeta < 1.0 - 1e-10 {
        // 欠阻尼：最常用，有过冲
        let omega_n = omega0 * (1.0 - zeta * zeta).sqrt();
        let decay = (-zeta * omega0 * t).exp();
        let a = y0;
        let b = (dy0 + zeta * omega0 * y0) / omega_n;

        let cos_t = (omega_n * t).cos();
        let sin_t = (omega_n * t).sin();

        let y = decay * (a * cos_t + b * sin_t);
        let dy = decay
            * ((-zeta * omega0 * a + omega_n * b) * cos_t
                - (zeta * omega0 * b + omega_n * a) * sin_t);

        (target - y, -dy)
    } else if zeta < 1.0 + 1e-10 {
        // 临界阻尼：无过冲，最快到达
        let decay = (-omega0 * t).exp();
        let a = y0;
        let b = dy0 + omega0 * y0;

        let y = decay * (a + b * t);
        let dy = decay * (-omega0 * (a + b * t) + b);

        (target - y, -dy)
    } else {
        // 过阻尼：慢于临界，无过冲
        let omega_n = omega0 * (zeta * zeta - 1.0).sqrt();
        let decay = (-zeta * omega0 * t).exp();
        let a = y0;
        let b = (dy0 + zeta * omega0 * y0) / omega_n;

        let cosh_t = (omega_n * t).cosh();
        let sinh_t = (omega_n * t).sinh();

        let y = decay * (a * cosh_t + b * sinh_t);
        let dy = decay
            * ((-zeta * omega0 * a + omega_n * b) * cosh_t
                + (omega_n * a - zeta * omega0 * b) * sinh_t);

        (target - y, -dy)
    }
}
