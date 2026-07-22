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
| `accent_transition` | `MetroAnim` | `theme/mod.rs` | 强调色切换 RGB 插值（0.3s, Quadratic/EaseOut） |
| `dark_transition` | `MetroAnim` | `theme/mod.rs` | 深色/浅色模式切换全颜色插值（0.3s, Quadratic/EaseOut） |

## 新增原语

- **`MetroAnim`** (`src/sokuou/uwp.rs`) — 从 sokuou-engine-toolkit 复制，UWP 缓动函数体系（11 种变体 × 3 种方向），默认 0.25s Quadratic/EaseOut。目前仅用于 accent_transition 和 dark_transition，但后续可用于任何短时色彩/透明度动画。
- **`AccentTransition`** (`src/theme/mod.rs`) — 基于 MetroAnim 的强调色 RGB 插值动画，interrupt-safe（中途切换颜色从中断位置继续）。
- **`DarkTransition`** (`src/theme/mod.rs`) — 基于 MetroAnim 的深色/浅色模式全颜色插值动画。`dark_progress()` 返回 0.0（浅色）到 1.0（深色），`colors` 模块中所有 13 个颜色函数使用 `lerp_dark(light, dark)` 在深浅色之间平滑过渡。

## 暴露的不足（持续更新）

- **`MetroAnim::set_target` 早期返回陷阱**（2026-07-22 修复）— `set_target` 在 `is_steady() && target == self.target` 时直接返回，不重置 `elapsed`。强调色第二次切换时，`AccentTransition` 的 `from/to` 已更新，但 `set_target(1.0)` 为空操作，导致动画不播放、颜色跳变。修复：在 `set_target` 前调用 `jump_to(0.0)` 强制重置。（`theme/mod.rs:120`）

## 从未使用的 API（持续更新）

- `SpringAnim::set_target_with_velocity` — 暂无用例（无手势驱动交互）
- `Progress::jump_to` — 所有 Progress 实例均使用 `set_target` 平滑过渡
- `Animator` / `Animation` trait — 预留存根，尚未验证
- `MetroAnim::default_metro` / `MetroAnim::jump_to` — 仅使用 `new` + `set_target` + `update`
- `UwpEasing` 非-Quadratic 变体（Cubic, Sine, Back, Bounce, Elastic 等）— 暂无用例，但保留供后续参考

## 需要新增的原语（持续更新）

_待定_

## 已移除的动画（2026-07-21）

| 动画 | 类型 | 之前位置 | 移除原因 |
|------|------|----------|---------|
| `page_enter_anim` | `SpringAnim` | `pdf/mod.rs:172` | PDF 阅读器改为全文档纵向滚动，不再需要单页翻页过渡 |
| `page_exit_anim` | `SpringAnim` | `pdf/mod.rs:173` | 同上 |
| `is_animating_out` | `bool` | `pdf/mod.rs:174` | 同上 |

---

更新规则：每新增或修改 Sokuou 动画后，同步更新此表。
由 CLAUDE.md 中的规则强制执行。