# Sokuou Engine 使用记录 — PezMax-One

## 当前状态

Phase 1 初始实现。核心原语已完成，编排层预留。
尚未接入任何实际 UI 组件（所有页面当前无动画）。

## 高频使用预测（待验证）

- SpringAnim：页面切换、侧边栏开合
- Progress + EaseOutCubic：Toast 淡入淡出、内容加载后渐显
- map_range：progress → alpha / slide offset / scale

## 暴露的不足（持续更新）

_尚未有实际使用，此栏待填写_

## 从未使用的 API（持续更新）

_待统计_

## 需要新增的原语（持续更新）

_待统计_

---

更新规则：每完成一轮 UI 开发后在此记录发现。
这些记录是 Sokuou Engine 最终统一到独立 crate 时的第一手需求文档。
