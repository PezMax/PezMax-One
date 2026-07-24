/// 机械电表风格数字显示 — 轮盘式垂直滚动动画
///
/// 每个数位独立驱动，使用 MetroAnim (0.3s, Quadratic EaseOut) 实现垂直滚动效果。
/// 纯色文字，无渐变，同步多位数跳变。
///
/// # 使用示例
///
/// ```ignore
/// let mut counter = AnimatedCounter::new();
/// counter.set_target(42);  // 首次 → 直接跳转
/// counter.set_target(47);  // 后续 → 个位 2→7 滚动动画
/// ```
///
/// # 渲染
///
/// ```ignore
/// render_odometer_value(ui.painter(), pos, &counter, 28.0, color);
/// ```

use crate::sokuou::{EasingMode, MetroAnim, UwpEasing};
use egui::{Color32, FontId, Painter, UiBuilder, Pos2, Rect, Vec2};

/// 动画时长（秒）
const ANIM_DURATION: f64 = 0.3;

/// 数位渲染信息
pub struct DigitInfo {
    pub from: u8,      // 旧数字 0-9
    pub to: u8,        // 新数字 0-9
    pub progress: f64, // 滚动进度 0.0→1.0
}

/// 单个数位动画状态
struct DigitAnim {
    from: u8,
    to: u8,
    anim: MetroAnim,
}

impl DigitAnim {
    fn steady(digit: u8) -> Self {
        let mut anim = MetroAnim::new(ANIM_DURATION, UwpEasing::Quadratic, EasingMode::EaseOut);
        anim.jump_to(1.0);
        Self { from: digit, to: digit, anim }
    }
}

/// 机械电表风格动画计数器
pub struct AnimatedCounter {
    digits: Vec<DigitAnim>, // 从右到左，索引 0 = 个位
    old_value: i64,
    target_value: i64,
    steady: bool, // true = 首次加载直接跳转
}

impl AnimatedCounter {
    pub fn new() -> Self {
        Self {
            digits: vec![DigitAnim::steady(0)],
            old_value: 0,
            target_value: 0,
            steady: true,
        }
    }

    /// 设置目标值。首次调用直接跳转，后续触发滚动动画。
    pub fn set_target(&mut self, target: i64) {
        if self.steady {
            self.jump_to(target);
            self.steady = false;
            return;
        }
        if target == self.target_value {
            return;
        }

        let old_str = format!("{}", self.target_value);
        let new_str = format!("{}", target);
        let max_len = old_str.len().max(new_str.len()).max(1);

        let old_padded = format!("{:0>width$}", old_str, width = max_len);
        let new_padded = format!("{:0>width$}", new_str, width = max_len);

        self.old_value = self.target_value;

        let mut new_digits: Vec<DigitAnim> = Vec::with_capacity(max_len);
        for i in 0..max_len {
            let old_ch = old_padded.chars().nth(max_len - 1 - i).unwrap();
            let new_ch = new_padded.chars().nth(max_len - 1 - i).unwrap();
            let old_digit = old_ch.to_digit(10).unwrap() as u8;
            let new_digit = new_ch.to_digit(10).unwrap() as u8;

            if old_digit == new_digit {
                new_digits.push(DigitAnim::steady(old_digit));
            } else {
                let mut anim = MetroAnim::new(ANIM_DURATION, UwpEasing::Quadratic, EasingMode::EaseOut);
                // set_target(1.0) 设置 elapsed=0，使 update() 实际推进动画
                anim.set_target(1.0);
                new_digits.push(DigitAnim { from: old_digit, to: new_digit, anim });
            }
        }

        self.digits = new_digits;
        self.target_value = target;
    }

    /// 立即跳转到目标值（无动画）
    pub fn jump_to(&mut self, target: i64) {
        let s = format!("{}", target);
        let len = s.len().max(1);
        let mut digits = Vec::with_capacity(len);
        for ch in s.chars().rev() {
            let d = ch.to_digit(10).unwrap_or(0) as u8;
            digits.push(DigitAnim::steady(d));
        }
        self.digits = digits;
        self.old_value = target;
        self.target_value = target;
        self.steady = true;
    }

    /// 每帧更新
    pub fn update(&mut self, dt: f64) {
        for digit in &mut self.digits {
            digit.anim.update(dt);
        }
    }

    /// 目标值（兼容旧代码）
    pub fn value(&self) -> i64 {
        self.target_value
    }

    /// 所有数位动画是否已完成
    pub fn is_steady(&self) -> bool {
        self.digits.iter().all(|d| d.anim.is_steady())
    }

    /// 获取每个数位的渲染信息（从右到左，索引 0 = 个位）
    pub fn digit_info(&self) -> Vec<DigitInfo> {
        self.digits
            .iter()
            .map(|d| DigitInfo {
                from: d.from,
                to: d.to,
                progress: d.anim.value(),
            })
            .collect()
    }

    /// 数位数量
    pub fn digit_count(&self) -> usize {
        self.digits.len()
    }
}

// ── 渲染辅助 ──────────────────────────────────────────────────────────────────

/// 渲染电表风格数字，返回总宽度
///
/// 每个数位位独立渲染，使用 child UI 的 clip rect 实现垂直滚动遮挡。
/// 纯色文字，无渐变，同步多位数跳变。
///
/// - `ui`: egui UI（用于创建带 clip rect 的子 UI）
/// - `pos`: 数字左上角位置（最左侧数位的左上角）
/// - `counter`: 动画计数器
/// - `font_size`: 字号（px）
/// - `color`: 文字颜色
pub fn render_odometer_value(
    ui: &mut egui::Ui,
    pos: Pos2,
    counter: &AnimatedCounter,
    font_size: f32,
    color: Color32,
) -> f32 {
    let (dw, dh) = digit_size(font_size);
    let info = counter.digit_info();
    let total_w = info.len() as f32 * dw;

    for (i, digit_info) in info.iter().enumerate().rev() {
        let x = pos.x + i as f32 * dw;
        let digit_rect = Rect::from_min_size(Pos2::new(x, pos.y), Vec2::new(dw, dh));

        // 用 child UI 的 clip rect 实现数位遮挡
        ui.scope(|ui| {
            ui.set_clip_rect(digit_rect);
            render_digit_inner(ui.painter(), digit_rect, digit_info, font_size, color);
        });
    }

    total_w
}

/// 计算数字字符的渲染尺寸
fn digit_size(font_size: f32) -> (f32, f32) {
    let w = font_size * 0.58;
    let h = font_size * 1.15;
    (w, h)
}

/// 在给定 rect 区域内绘制两个数字（旧数字滚出 + 新数字滚入）
fn render_digit_inner(
    painter: &Painter,
    rect: Rect,
    info: &DigitInfo,
    font_size: f32,
    color: Color32,
) {
    let (_, dh) = digit_size(font_size);
    let progress = info.progress as f32;
    let center_x = rect.center().x;

    // 旧数字：向上滚出，从 y = center_y 到 y = center_y - dh
    let old_y = rect.center().y - progress * dh;
    painter.text(
        Pos2::new(center_x, old_y),
        egui::Align2::CENTER_CENTER,
        format!("{}", info.from),
        FontId::new(font_size, egui::FontFamily::Proportional),
        color,
    );

    // 新数字：从下方滚入，从 y = center_y + dh 到 y = center_y
    let new_y = rect.center().y + (1.0 - progress) * dh;
    painter.text(
        Pos2::new(center_x, new_y),
        egui::Align2::CENTER_CENTER,
        format!("{}", info.to),
        FontId::new(font_size, egui::FontFamily::Proportional),
        color,
    );
}