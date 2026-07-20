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
| `display_scale_anim` | `SpringAnim` | `pdf/mod.rs:165` | PDF 缩放平滑过渡（response=0.4, damping=0.8） |
| `search_hint_anim` | `SpringAnim` | `app.rs` | 搜索框提示文字滑入/滑出（response=0.4, damping=0.825） |

## 暴露的不足（持续更新）

_待发现_

## 从未使用的 API（持续更新）

- `SpringAnim::set_target_with_velocity` — 暂无用例（无手势驱动交互）
- `Progress::jump_to` — 所有 Progress 实例均使用 `set_target` 平滑过渡
- `Animator` / `Animation` trait — 预留存根，尚未验证

## 需要新增的原语（持续更新）

_待定_

---

更新规则：每新增或修改 Sokuou 动画后，同步更新此表。
由 CLAUDE.md 中的规则强制执行。