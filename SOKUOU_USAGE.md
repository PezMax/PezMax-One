# Sokuou Engine 调用手册 — PezMax-One

> 核心哲学：**动画不是播放，而是空间状态的连续解析。**
> `state → progress [0.0, 1.0] → resolved spatial state → render`
> 用户感受到的不是"动画在播放"，而是"空间在自然变化"。

---

## 1 · 快速入门

### 导入

```rust
use crate::sokuou::{SpringAnim, Progress, Easing, map_range};
```

### 在 egui 中获取 dt

```rust
impl eframe::App for PezMaxApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dt = ctx.input(|i| i.stable_dt) as f64;
        // dt 已经是帧间时间（秒），直接传给 update()
        // 无需手动 clamp——SpringAnim 内部已处理 dt.min(0.05)

        self.sidebar_anim.update(dt);
        self.page_fade.update(dt);

        // 有动画运行时请求下一帧重绘
        if !self.sidebar_anim.is_steady() || !self.page_fade.is_steady() {
            ctx.request_repaint();
        }
    }
}
```

### 关键规则

- `update(dt)` 每帧调用，稳态时自动跳过（无开销）
- 有动画运行时必须 `ctx.request_repaint()`，否则 egui 不会继续绘制
- 渲染函数只**读** `value()`，不操作动画本身

---

## 2 · 选择哪种原语

| 场景 | 原语 | 缓动 |
|------|------|------|
| 面板滑入/滑出 | `SpringAnim` | 内置物理 |
| 窗口/弹窗弹入 | `SpringAnim` | 内置物理 |
| 页面切换位移 | `SpringAnim` | 内置物理 |
| 透明度渐变 | `Progress` | `EaseOutCubic` |
| 颜色过渡 | `Progress` | `Linear` |
| 图标旋转（非弹性） | `Progress` | `EaseOutQuart` |
| 加载指示器循环 | `Progress` | `EaseInOutCubic` |

**原则**：主运动（位置、大小）用弹簧；次要属性（透明度、颜色）用 Progress。

---

## 3 · SpringAnim 场景示例

### 3.1 侧边栏开合

```rust
// 在 PezMaxApp 中添加字段
pub sidebar_anim: SpringAnim,

// 初始化（在 PezMaxApp::new()）
sidebar_anim: SpringAnim::new(0.5, 0.825, 1.0), // 初始展开

// 切换时
pub fn toggle_sidebar(&mut self) {
    self.sidebar_open = !self.sidebar_open;
    let target = if self.sidebar_open { 1.0 } else { 0.0 };
    self.sidebar_anim.set_target(target);
}

// 渲染时（sidebar.rs）
fn render(app: &mut PezMaxApp, ctx: &egui::Context) {
    let p = app.sidebar_anim.value();
    let sidebar_width = map_range(p, 0.0, 240.0); // 0px → 240px

    // progress < 0.01 时可跳过渲染
    if p < 0.01 { return; }

    egui::SidePanel::left("sidebar")
        .exact_width(sidebar_width as f32)
        .show(ctx, |ui| { /* ... */ });
}
```

### 3.2 页面切换（进场动画）

```rust
// 在 PezMaxApp 中
pub page_enter_anim: SpringAnim,

// navigate() 时触发进场
pub fn navigate(&mut self, page: Page) {
    self.page_history.push(self.current_page.clone());
    self.current_page = page;
    // 从 0 开始，弹入到 1
    self.page_enter_anim = SpringAnim::with_target(0.4, 0.8, 0.0, 0.0, 1.0);
}

// 渲染时（在各页面 render 函数顶部）
fn render(app: &mut PezMaxApp, ui: &mut egui::Ui) {
    let p = app.page_enter_anim.value();
    let alpha = ease_out_cubic(p) as f32;
    let offset_y = map_range(p, 20.0, 0.0) as f32; // 向上滑入 20px

    ui.add_space(offset_y);
    // 设置全局透明度
    let mut visuals = ui.visuals().clone();
    visuals.override_text_color = Some(visuals.text_color().linear_multiply(alpha));
    ui.set_visuals(visuals);

    // 正常渲染页面内容...
}
```

### 3.3 弹窗/对话框弹入

```rust
// response=0.45, damping=0.7：更弹，强调"弹入"感
let dialog_anim = SpringAnim::with_target(0.45, 0.7, 0.0, 0.0, 1.0);

// 渲染
let p = dialog_anim.value();
let scale = map_range(p, 0.85, 1.0) as f32; // 85% → 100%
let alpha = ease_out_cubic(p) as f32;

// egui 目前不直接支持 transform，可用 Rect 偏移模拟：
let panel_rect = ui.available_rect_before_wrap();
let center = panel_rect.center();
let scaled_size = panel_rect.size() * scale;
let scaled_rect = egui::Rect::from_center_size(center, scaled_size);
ui.allocate_rect(scaled_rect, egui::Sense::hover());
```

---

## 4 · Progress 场景示例

### 4.1 Toast 通知进场/离场

```rust
// toast.rs 中的 Toast 结构体
pub struct AnimatedToast {
    pub message: String,
    pub level: ToastLevel,
    pub created_at: std::time::Instant,
    pub enter_anim: Progress,  // 进场（0→1）
    pub exit_anim: Progress,   // 离场（0→1，触发后开始）
    pub exiting: bool,
}

impl AnimatedToast {
    pub fn new(message: String, level: ToastLevel) -> Self {
        let mut enter_anim = Progress::with_easing(0.25, Easing::EaseOutCubic);
        enter_anim.set_target(1.0); // 立即开始进场
        Self {
            message, level,
            created_at: std::time::Instant::now(),
            enter_anim,
            exit_anim: Progress::with_easing(0.2, Easing::EaseInCubic),
            exiting: false,
        }
    }

    pub fn dismiss(&mut self) {
        if !self.exiting {
            self.exiting = true;
            self.exit_anim.set_target(1.0);
        }
    }

    pub fn update(&mut self, dt: f64) {
        self.enter_anim.update(dt);
        self.exit_anim.update(dt);
    }

    pub fn is_gone(&self) -> bool {
        self.exiting && self.exit_anim.is_steady()
    }
}

// 渲染时
fn render_toast(toast: &AnimatedToast, ui: &mut egui::Ui) {
    let enter_p = toast.enter_anim.value();
    let exit_p = toast.exit_anim.value();

    // 进场：从右侧滑入；离场：向上滑出
    let slide_x = map_range(enter_p, 320.0, 0.0);       // 右侧滑入
    let slide_y = map_range(exit_p, 0.0, -60.0);         // 向上离场
    let alpha = enter_p * (1.0 - exit_p);                 // 进场亮起，离场暗下

    // 用 ui.put() 或 painter 在绝对位置渲染
}
```

### 4.2 内容加载后渐显

```rust
// 在 PezMaxApp 或页面状态中
pub content_fade: Progress,

// API 响应回调中
self.file_list = response.data;
self.content_fade.jump_to(0.0);          // 先重置为不可见
self.content_fade.set_target(1.0);       // 然后渐入

// 渲染
let alpha = app.content_fade.value() as f32;
ui.set_opacity(alpha); // egui 0.31+ 支持 set_opacity
```

---

## 5 · map_range 速查

```rust
use crate::sokuou::map_range;

let p = spring.value(); // 0.0 → 1.0

// 透明度：完全不可见 → 完全可见
let alpha = map_range(p, 0.0, 1.0) as f32;

// 向上滑入（从屏幕外 +40px 到原位）
let offset_y = map_range(p, 40.0, 0.0) as f32;

// 向左滑入（侧边栏 240px 宽）
let sidebar_w = map_range(p, 0.0, 240.0) as f32;

// 缩放：0.85 → 1.0
let scale = map_range(p, 0.85, 1.0) as f32;

// 颜色插值（手动对每个通道调用）
let r = map_range(p, from_color.r(), to_color.r());
```

---

## 6 · 主次动画分离原则

```rust
fn update(&mut self, dt: f64) {
    self.spring_anim.update(dt); // 主运动：弹簧控制位置

    // 次要属性从同一个 progress 派生，不单独建动画
    let p = self.spring_anim.value();
    self.alpha = ease_out_cubic(p);    // 透明度跟随弹簧进度
    self.scale = map_range(p, 0.9, 1.0); // 缩放跟随弹簧进度
}
```

同一个 `progress` 驱动多个属性，保证视觉一致性——不要为透明度和位置各建一个独立动画。

---

## 7 · 推荐参数速查

```rust
// 通用交互（点击、面板切换）
SpringAnim::new(0.5, 0.825, initial)

// 快速交互（按钮按下、图标反馈）
SpringAnim::new(0.3, 0.6, initial)

// 慢速展示（通知横幅）
SpringAnim::new(0.65, 0.85, initial)

// 弹窗强调弹入
SpringAnim::with_target(0.45, 0.7, 0.0, 0.0, 1.0)

// 非交互淡入（200ms）
Progress::with_easing(0.2, Easing::EaseOutCubic)

// 淡出（稍慢）
Progress::with_easing(0.15, Easing::EaseInCubic)
```

---

## 8 · 性能守则

```rust
// 1. 稳态不重绘
if !self.anim.is_steady() {
    ctx.request_repaint();
}

// 2. progress 极低时跳过渲染
if app.page_enter_anim.value() < 0.01 { return; }

// 3. 稳态时用精确值
if self.spring.is_steady() {
    draw_at_exact_target();
}

// 4. SpringAnim 内部已 clamp dt.min(0.05)，无需外部保护
```

---

## 9 · 预留 API 说明

`animator.rs` 中的 `Animation` trait 和 `Animator` 编排器目前**仅为预留存根**，
不可用于生产代码。当出现以下真实需求时再实现：

- 需要"面板升起后内容才开始渐显"的串行依赖
- 需要多个卡片错位延迟入场（stagger）

发现需求时，先在 `src/sokuou/NOTE.md` 记录具体场景，再实现对应原语。

---

## 10 · 设计不变量（禁止事项）

| 禁止 | 原因 |
|------|------|
| `play_animation()` 风格 API | 违反状态驱动原则 |
| 在渲染函数中修改动画状态 | 渲染函数应只读 `value()` |
| pop-in（突然出现） | 违反空间连续性——用 `Progress::jump_to(0.0)` 初始化 |
| 多个独立动画驱动同一元素的不同属性 | 改为从单一 `progress` 派生所有属性 |
| 忘记 `ctx.request_repaint()` | 动画会卡在第一帧 |
