# Sokuou Engine 使用记录 — PezMax-One

## 当前状态

Phase 1 完成。核心原语已稳定，已接入前端 UI 组件。

## 已接入的动画（2026-07-22）

| 动画 | 类型 | 位置 | 用途 |
|------|------|------|------|
| `sidebar_anim` | `SpringAnim` | `app.rs:340` | 侧边栏展开/收起（48px ↔ 200px） |
| `sidebar_indicator_anim` | `SpringAnim` | `app.rs:341` | 侧边栏导航指示器滑动 |
| `subtab_indicator_anim` | `SpringAnim` | `app.rs:343` | 子标签指示器滑动 |
| `preview_anim` | `SpringAnim` | `app.rs:354` / `browse.rs:257` | 预览面板 slide-in/out |
| `page_enter_anim` | `SpringAnim` | `app.rs:358` | 页面进入过渡 |
| `auth_anim` | `Progress` | `app.rs:360` | 登录/注册页淡入 |
| Toast `enter` | `Progress` | `app.rs:184` | Toast 滑入 |
| Toast `exit` | `Progress` | `app.rs:185` | Toast 滑出 |
| `display_scale_anim` | `SpringAnim` | `pdf/mod.rs:168` | PDF 缩放平滑过渡（response=0.4, damping=0.8） |
| `search_hint_anim` | `SpringAnim` | `app.rs` | 🔍 左滑出场 + 占位文字右滑入场（response=0.25, damping=0.7） |
| `bookmark_detail_anim` | `SpringAnim` | `app.rs` / `browse.rs` | 书签详情页入场（20px 下滑 + 透明度）response=0.4, damping=0.8 |
| `grid_size_anim` | `SpringAnim` | `pdf/mod.rs` | 平摊模式下页面宽度平滑过渡（response=0.4, damping=0.825） |
| `overview_anim` | `SpringAnim` | `pdf/mod.rs` | Line 模式左侧总览面板宽度过渡（response=0.4, damping=0.8）；0.0=收起、1.0=展开（OVERVIEW_PANEL_WIDTH=150px）。内容按满宽布局，通过 `set_clip_rect` 裁到实际可见区域，收起时缩略图向左滑出 |
| `accent_transition` | `MetroAnim` | `theme/mod.rs` | 强调色切换 RGB 插值（0.3s, Quadratic/EaseOut） |
| `dark_transition` | `MetroAnim` | `theme/mod.rs` | 深色/浅色模式切换全颜色插值（0.3s, Quadratic/EaseOut） |
| 设置页 toggle 开关滑块 | `Progress` | `pages/profile.rs::render_toggle_switch` | 开关滑块横向位移 + 背景色插值（0.22s, EaseOutCubic）。每开关按 `id_source` 独立存储到 `ctx.data_mut`，首次渲染 `jump_to` 避免开场动画。 |
| `auth_step_anim` | `SpringAnim` | `app.rs` / `components/step_indicator.rs` | 注册 4 步、找回密码 3 步向导的步骤指示器滑动（0.3s, damping 0.85） |
| `register_disclaimer_countdown` | `Progress` | `app.rs` / `components/disclaimer_dialog.rs` | 免责声明弹窗 1 秒倒计时门（Linear）；`value() >= 1.0` 才可点确认 |
| `report_timeline_anim` | `SpringAnim` | `app.rs` / `components/timeline_panel.rs` | 举报时间线弹窗入场（0.4/0.8，24px 下滑 + alpha 淡入）。用 `SpringAnim::with_target` 重置状态而不加 `jump_to` API |
| `subsection_transition_anim` | `MetroAnim` | `app.rs` | 子分页切换（浏览/社区/个人内的水平标签）内容区左右滑入 + 淡入（0.28s, Quadratic/EaseOut）。通过 `inner_margin` 对称增减实现横向位移 48px；`navigate_subsection` / 同 Section 内 `navigate_to` 触发；方向由目标索引与当前索引对比决定（右移→从右滑入，左移→从左滑入） |

## 新增原语

- **`MetroAnim`** (`src/sokuou/uwp.rs`) — 从 sokuou-engine-toolkit 复制，UWP 缓动函数体系（11 种变体 × 3 种方向），默认 0.25s Quadratic/EaseOut。目前仅用于 accent_transition 和 dark_transition，但后续可用于任何短时色彩/透明度动画。
- **`AccentTransition`** (`src/theme/mod.rs`) — 基于 MetroAnim 的强调色 RGB 插值动画，interrupt-safe（中途切换颜色从中断位置继续）。
- **`DarkTransition`** (`src/theme/mod.rs`) — 基于 MetroAnim 的深色/浅色模式全颜色插值动画。`dark_progress()` 返回 0.0（浅色）到 1.0（深色），`colors` 模块中所有 13 个颜色函数使用 `lerp_dark(light, dark)` 在深浅色之间平滑过渡。

## 暴露的不足（持续更新）

- **`MetroAnim::set_target` 早期返回陷阱**（2026-07-22 修复）— `set_target` 在 `is_steady() && target == self.target` 时直接返回，不重置 `elapsed`。强调色第二次切换时，`AccentTransition` 的 `from/to` 已更新，但 `set_target(1.0)` 为空操作，导致动画不播放、颜色跳变。修复：在 `set_target` 前调用 `jump_to(0.0)` 强制重置。（`theme/mod.rs:120`）

## 从未使用的 API（持续更新）

- `SpringAnim::set_target_with_velocity` — 暂无用例（无手势驱动交互）
- `Progress::jump_to` — `render_toggle_switch` 首次渲染时用它跳到当前布尔态，避免开场播动画；其余 Progress 实例均使用 `set_target` 平滑过渡
- `Animator` / `Animation` trait — 预留存根，尚未验证
- `MetroAnim::default_metro` — 各接入点显式传入 duration/variant/mode
- `MetroAnim::jump_to` — `subsection_transition_anim` 每次切换用它归零重播；初始化时也用它跳到 1.0 稳态
- `UwpEasing` 非-Quadratic 变体（Cubic, Sine, Back, Bounce, Elastic 等）— 暂无用例，但保留供后续参考

## 需要新增的原语（持续更新）

- `SpringAnim::jump_to(value)` — 用于弹窗二次入场时"从头播动画"。目前通过 `SpringAnim::with_target(response, damping, 0, 0, 1)` 重新构造实例来绕过（见 `report_timeline_anim` 的入场逻辑）。

## 已移除的动画（2026-07-21）

| 动画 | 类型 | 之前位置 | 移除原因 |
|------|------|----------|---------|
| `page_enter_anim` | `SpringAnim` | `pdf/mod.rs:172` | PDF 阅读器改为全文档纵向滚动，不再需要单页翻页过渡 |
| `page_exit_anim` | `SpringAnim` | `pdf/mod.rs:173` | 同上 |
| `is_animating_out` | `bool` | `pdf/mod.rs:174` | 同上 |

---

更新规则：每新增或修改 Sokuou 动画后，同步更新此表。
由 CLAUDE.md 中的规则强制执行。