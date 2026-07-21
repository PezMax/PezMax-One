# Sokuou Engine 使用记录 — PezMax-One

## 当前状态

Phase 1 完成。核心原语已稳定，已接入前端 UI 组件。

## 已接入的动画（2026-07-20）

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
| `grid_size_anim` | `SpringAnim` | `pdf/mod.rs` | 平摊模式下页面宽度平滑过渡（response=0.4, damping=0.825） |

## 暴露的不足（持续更新）

_待发现_

## 从未使用的 API（持续更新）

- `SpringAnim::set_target_with_velocity` — 暂无用例（无手势驱动交互）
- `Progress::jump_to` — 所有 Progress 实例均使用 `set_target` 平滑过渡
- `Animator` / `Animation` trait — 预留存根，尚未验证

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