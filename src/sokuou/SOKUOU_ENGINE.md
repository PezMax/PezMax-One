# Sokuou Engine 设计书

> 即応エンジン — 进度驱动的空间状态解析器。
>
> 面向 Rust 应用开发者的动画引擎，追求 Apple 级别的流畅与优雅，跨 Linux/Windows/macOS 三平台。

---

## 目录

1. [哲学](#1-哲学)
2. [引擎定义](#2-引擎定义)
3. [发展策略](#3-发展策略)
4. [核心概念](#4-核心概念)
5. [Progress 原语](#5-progress-原语)
6. [Spring 原语](#6-spring-原语)
7. [缓动函数库](#7-缓动函数库)
8. [动画编排](#8-动画编排)
9. [与渲染循环的集成](#9-与渲染循环的集成)
10. [API 参考](#10-api-参考)
11. [最佳实践](#11-最佳实践)
12. [设计不变量](#12-设计不变量)
13. [附录：数学推导](#13-附录数学推导)

---

## 1 · 哲学

### 1.1 动画不是"播放"

Sokuou Engine 不存在传统意义上的"动画播放"。

系统不使用：
- timeline animation
- keyframe playback
- clip-based transition
- UI 特效式 motion system

Sokuou 将动画定义为：**空间状态解析过程**。

动画并非"被播放"——而是系统状态被连续计算后之结果。

### 1.2 核心原则

**Progress-based Spatial Resolution**（进度驱动的空间解析）

系统中所有空间元件：
- 位置
- 透明度
- 缩放
- 旋转
- 层级状态
- 空间存在感

均由统一 progress 参数解析得出。

### 1.3 状态驱动渲染

```
state → progress → resolved spatial state → render result
```

而非：

```
play_animation() → timeline → render
```

### 1.4 连续性

空间状态始终保持连续。系统不得出现：
- pop-in
- sudden spawn
- abrupt destruction
- transition discontinuity

---

## 2 · 引擎定义

### 2.1 Sokuou Engine 是什么

**Sokuou Engine 是一个「空间状态解析器」——不是动画播放器，不是缓动函数库，不是运动引擎。**

```
Sokuou Engine = 
  一个接受「空间状态变化意图」并输出「连续 progress 序列」的解析器
```

它的存在理由：当系统中某个状态需要从 A 变成 B，Sokuou Engine 负责回答"从 A 到 B 的过程中，每一帧的状态应该是什么"。

它不是"让东西动起来"的工具，而是"让空间状态自然变化"的契约。

### 2.2 命名拆解

即応（Sokuou）——"即"为即时，"応"为响应。

```
即時（そくじ）—— 立即
応答（おうとう）—— 响应
即応（そくおう）—— 即时响应
```

Sokuou Engine 的核心不在于"如何播放动画"，而在于"如何即时响应状态变化"。

### 2.3 职责范围

Sokuou Engine 管理以下内容：

| 职责 | 说明 | 体现 |
|------|------|------|
| Progress 变量 | 系统唯一的空间状态标识，[0.0, 1.0] | `Progress::value()` |
| 物理运动 | 弹簧、阻尼、惯性，二阶物理系统 | `SpringAnim` |
| 缓动映射 | 从 progress 到视觉属性的非线性映射 | `Easing` 枚举 |
| 动画编排 | 多个动画的并行、串行、链式关系 | `Animator` |
| 时间与帧率 | dt 管理、帧率无关性保证 | `update(dt)` |
| 稳态检测 | 判断动画是否完成，临界吸附 | `is_steady()`, `snap_if_close()` |

Sokuou Engine 不管理以下内容：

| 非职责 | 说明 | 由谁负责 |
|--------|------|----------|
| 渲染 | 不绘制任何像素 | 应用层的渲染管线 |
| 输入 | 不处理鼠标/键盘/触摸事件 | 应用层的输入系统 |
| 布局 | 不决定元素的位置和尺寸 | 应用层的布局系统 |
| 数据流 | 不管理应用状态树 | 应用层的状态管理 |
| 纹理/资源 | 不加载图片、字体 | 应用层的资源系统 |
| 音效/触觉 | 不驱动非视觉反馈 | 应用层的反馈系统 |

**核心分界线**：Sokuou Engine 输出 `f64` 值，应用层将这些值解释为位置、透明度、缩放等视觉属性。

```
Sokuou Engine 的输出边界：
  f64 ∈ [0.0, 1.0]  →  应用层自行映射到具体属性
  (progress)         →  position / alpha / scale / rotation / ...
```

### 2.4 如何存在

Sokuou Engine 不是一个运行时服务、守护进程或全局单例。它的存在方式：

**作为库嵌入：**

```
┌─────────────────────────────────────┐
│           应用层 (你的 App)           │
│  ┌─────────────────────────────────┐│
│  │  Sokuou Engine (嵌入为库)       ││
│  │  ┌──────────┐ ┌──────────────┐ ││
│  │  │ Progress │ │ SpringAnim   │ ││
│  │  │ 实例 A   │ │ 实例 B       │ ││
│  │  └──────────┘ └──────────────┘ ││
│  │  ┌──────────────────────────┐  ││
│  │  │ Animator (编排器)        │  ││
│  │  └──────────────────────────┘  ││
│  └─────────────────────────────────┘│
│  ┌─────────────────────────────────┐│
│  │ 你的渲染循环                     ││
│  │ 每帧: engine.update(dt) → render││
│  └─────────────────────────────────┘│
└─────────────────────────────────────┘
```

**每个组件持有自己的动画实例：**

```rust
struct DockPanel {
    // Sokuou Engine 的存在方式：作为字段嵌入
    slide_anim: SpringAnim,    // 这个面板的滑入动画
    fade_anim: Progress,       // 这个面板的淡入动画
    app_anims: Vec<SpringAnim>, // 每个图标有自己的弹跳动画
}

impl DockPanel {
    fn update(&mut self, dt: f64) {
        self.slide_anim.update(dt);
        self.fade_anim.update();
        for anim in &mut self.app_anims {
            anim.update(dt);
        }
    }
}
```

**没有全局状态，没有注册中心，没有隐式依赖。**

### 2.5 生命周期契约

Sokuou Engine 的每个动画实例遵循以下生命周期：

```
构造 ──→ 运行 ──→ 稳态 ──→ 中断 ──→ 运行 ──→ 稳态 ──→ 析构
  │        │         │        │
  │        │         ├── snap_if_close 直接落地
  │        │         └── 触发回调（如 transition complete）
  │        │
  │        └── set_target(new_target)  → 中断当前状态，重新开始
  │
  └── new() 设定初始条件 from, from_vel, target
```

关键规则：
- **构造时即确定初始条件**，不依赖外部隐性状态
- **运行中可任意中断**，从中断时的 pos/vel 继续
- **稳态后应 snap**，但不强制销毁——可随时 set_target 重新激活
- **析构即释放**，无善后逻辑

### 2.6 三层架构

Sokuou Engine 内部由三个层次组成：

```
┌──────────────────────────────────────┐
│  Layer 3: 集成层 (Integration)       │
│  与渲染循环的契约、dt 管理、进度查询  │
│  → update(dt), value(), is_steady()  │
├──────────────────────────────────────┤
│  Layer 2: 编排层 (Orchestration)      │
│  多动画管理、时序关系、组合模式       │
│  → Animator, Parallel, Sequence,Chain│
├──────────────────────────────────────┤
│  Layer 1: 物理层 (Physics)            │
│  原始运动原语、物理参数化、解析解     │
│  → Progress, SpringAnim, Easing      │
└──────────────────────────────────────┘
```

应用层可直接使用 Layer 1（简单场景），也可通过 Layer 2 编排复杂动画序列。Layer 3 是契约规范，不提供单独的运行时结构体。

### 2.7 与现有 ether-motion 的关系

```
ether-motion (共享库 crate)          ← 包名
  └── Sokuou Engine (产品品牌名)     ← 设计理念名
       ├── 现有: Progress (线性, 时长驱动)
       ├── 现有: Spring (stiffness/damping, 欧拉积分)
       ├── 新增: SpringAnim (response/damping_ratio, 解析解)
       ├── 新增: Easing (缓动函数库)
       └── 新增: Animator (编排器)
```

Sokuou Engine 是 `ether-motion` 的产品品牌和设计理念，而不是一个独立的 crate。`ether-motion` 是它在 Rust 生态中的包名，`Sokuou Engine` 是它在设计文档和架构讨论中的名称。

---

## 3 · 发展策略

### 3.1 核心原则：通过使用来发现引擎

Sokuou Engine 不是预先设计再实现的。它的正确形状**只有在多个真实应用中被使用后，才能被揭示**。

```
设计一个引擎   → 写一个应用   → 发现 API 有问题 → 修改引擎
（错误路径）      （一次性验证）

写一个应用     → 使用 Sokuou  → 记录不足       → 写下一个应用
（正确路径）      核心原语       积累经验          揭示更多需求
                                                      │
                                                      ▼
                                              经过 n 个应用后
                                              整合所有经验
                                              回灌 Ether 本体
```

Sokuou Engine 的最终形态不是由构想的"完美设计"决定的，而是由它在 PezMax-One、ether-launcher、ether-settings、ether-librarian 以及未来更多应用的**实际使用痕迹**决定的。

### 3.2 每个应用拥有自己的 Sokuou 副本

在引擎稳定之前，每个应用内联一个 `src/sokuou/` 目录：

```
src/sokuou/
├── mod.rs           // 公共 API 重导出
├── progress.rs      // Progress 原语
├── spring.rs        // SpringAnim 原语（解析解）
├── easing.rs        // 缓动函数（按需添加）
├── animator.rs      // 编排器（按需添加）
└── NOTE.md          // 这个应用中对 Sokuou 的修改和发现
```

**关键规则：**

| 规则 | 原因 |
|------|------|
| 每个应用拥有自己的副本 | 独立迭代，不受其他应用进度约束 |
| 跨应用共享经验，不共享代码 | 避免"为兼容而妥协"——每个应用的 API 只为它的需求服务 |
| 等稳定后再统一为独立 crate | 统一太早会冻结 API，抑制进化 |

### 3.3 NOTE.md 记录机制

每个应用结束一轮开发时，在 `src/sokuou/NOTE.md` 中记录：

```markdown
# Sokuou Engine 使用记录 — PezMax-One

## 暴露的不足
- StaggeredAnim 的延迟启动需要负 elapsed 技巧，不优雅
- 没有"动画完成回调"机制，页面切换需要手动 polling
- Progress 的 set_target 在中途重新设置时，尾迹不自然

## 高频使用的 API
- SpringAnim::new() / update() / value() / is_steady()
- ease_out_cubic()
- Progress::set_target() / update()

## 从未使用的 API
- Animator::add_parallel()（PezMax 不需要编排器）
- Easing::EaseInOutQuint（永远用不到）

## 需要新增的原语
- 动画完成回调（用于页面切换）
- 延迟启动的原语支持（替代负 elapsed 技巧）
```

这些记录是 Sokuou Engine 最终统一时的**第一手需求文档**。

### 3.4 最小可用核心

早期 Sokuou 只包含绝对必要的原语：

```
Layer 1 (物理层):  Progress + SpringAnim + easing 函数
  └── 没有编排器，没有 Animator trait，没有链式动画
```

其余一切**按需生长**：

```
PezMax 需要错位卡片入场
  → 发现需要延迟启动
  → 在 ease_out_cubic 中加 delay 参数
  → 记录到 NOTE.md
  → 下一个应用可能把这个模式抽象为 DelayedAnim

ether-launcher 需要 Dock 图标逐个弹入
  → 发现需要多个动画的时序关系
  → 实现 StaggeredAnim
  → 记录到 NOTE.md
  → 对比 PezMax 的 delay 参数，统一 API
```

### 3.5 不要预先实现什么

| 不要预先实现 | 原因 |
|-------------|------|
| `Animator` 编排器 | 没有应用验证过它的 API 是否正确 |
| `Animation` trait | 不确定 trait 的 method 签名是否正确 |
| 完整的缓动函数枚举 | 不知道哪些缓动函数真正需要 |
| 链式动画 | 不知道链式依赖的语义是什么 |
| 泛型（`Animation<T>`） | 不知道 f64 以外是否真的需要 |

### 3.6 Sokuou Engine 的成长路径

```
Phase 0 (当前)
  ├── ether-motion: 两个原语 (Progress + Spring 欧拉)
  └── 仅用于 Ether 合成器

Phase 1 (早期应用驱动)
  ├── 每个应用内联 src/sokuou/
  ├── 核心: Progress + SpringAnim (解析解) + easing
  ├── 应用 1: PezMax-One
  ├── 应用 2: ???
  └── 应用 3: ???

Phase 2 (经验整合)
  ├── 汇集所有 NOTE.md
  ├── 识别交集: 所有应用都用的 API → 保留
  ├── 识别差异: 只有特定应用需要的 → 保持可选
  └── 识别缺失: 多个应用用不同方式解决了同一问题 → 需要统一原语

Phase 3 (回灌 Ether)
  ├── 提取为独立 crate (shared/ether-motion v2)
  ├── 替换 Ether 合成器中的现有动画
  ├── 替换 ether-launcher 中的现有动画
  └── 替换 ether-settings 中的现有动画
```

---

## 4 · 核心概念

### 4.1 Progress 的本质

```
progress ∈ [0.0, 1.0]
```

progress 并非：
- 时间
- 帧计数
- 动画播放百分比
- timeline 位置

其本质为：**当前空间状态值**。

`progress = 0.25` 并不意味着"动画已播放 25%"，而是"系统当前真实处于 25% 的空间状态"。

### 4.2 状态解析的确定性

```
render_state = resolve(progress)
```

任意 progress 值，系统必须存在唯一确定的渲染结果。系统不得依赖：
- 历史关键帧
- timeline cache
- animation clip
- 上一帧动画状态

### 4.3 时间与 progress 的关系

```
time → affects progress
progress → determines state
```

时间只影响 progress 的变化速度（加速度、阻尼、速度、弹簧行为），但空间状态本身永远只由 progress 决定。

### 4.4 关键公式

所有动画最终表达为：

```
对任意 t (时间): 
  progress = f(t)          // 从时间到进度的映射
  state    = resolve(progress)  // 从进度到空间状态的映射
```

其中 `f(t)` 可以是：
- 线性插值（Progress）
- 弹簧物理（Spring）
- 任意缓动函数（Easing）

---

## 5 · Progress 原语

### 5.1 职责

Progress 提供**时长驱动**的线性缓动。适用于：
- 透明度变化
- 简单的位移
- 不需要弹性效果的过渡

### 5.2 设计

```rust
/// 进度驱动的线性缓动值 [0.0, 1.0]，支持中断。
pub struct Progress {
    /// 当前值
    value: f64,
    /// 目标值
    target: f64,
    /// 本段起始值（set_target 时的 value 快照）
    from: f64,
    /// 本段起始时间
    start_time: Option<Instant>,
    /// 时长
    duration: Duration,
    /// 缓动函数
    easing: Easing,
}
```

### 5.3 生命周期

```
set_target(1.0)     update() 每帧   到达目标
    │                   │                │
    ▼                   ▼                ▼
┌────────┐    ┌───────────────┐    ┌────────┐
│  from  │───→│  线性插值推进   │───→│ target │
│ value  │    │  value = lerp  │    │ value  │
└────────┘    └───────────────┘    └────────┘
                  │
                  │ 任意时刻可中断：
                  ▼
            set_target(new_target)
            → 记录当前 value 为新的 from
            → 重置计时
```

### 5.4 插值算法

```
t = elapsed / duration
t_clamped = clamp(t, 0.0, 1.0)
progress = easing(t_clamped)  // 应用缓动函数
value = from + (target - from) * progress
```

### 5.5 可中断性

`set_target(new_target)` 在任意时刻调用：
- 当前 `value` 成为新的 `from`
- 当前时间戳记录为新的 `start_time`
- 向 `new_target` 运动，不从零或原始位置重新开始

---

## 6 · Spring 原语

### 6.1 职责

Spring 提供**物理驱动的弹簧动画**，基于二阶阻尼振荡器。适用于：
- 窗口弹入/弹出
- 面板滑入/滑出
- 视图切换
- 任何需要"自然感"的过渡

### 6.2 Apple 风格参数化

Apple 在 iOS 10+ 中引入的 `response` / `dampingRatio` 参数化：

| 参数 | 含义 | 范围 | 物理直觉 |
|------|------|------|----------|
| `response` | 响应时间（秒） | (0, ∞) | 近似"动画持续时间" |
| `damping_ratio` | 阻尼比 | [0, ∞) | 0=无阻尼无限弹跳, 1=临界阻尼, >1=过阻尼 |

**为什么这种参数化更好？**

传统 stiffness/damping 对时长的影响是非线性的：

```
stiffness = (2π / response)²
damping   = 4π * damping_ratio / response
```

直接调 stiffness 无法直觉地控制动画时长。`response` 直截了当：设 0.3 就是约 0.3 秒到达。

### 6.3 典型值

| 场景 | response | damping_ratio | 效果 |
|------|----------|---------------|------|
| 页面转场 | 0.4s | 0.8 | 快速，略欠阻尼 |
| 键盘弹出 | 0.5s | 0.825 | 标准系统弹簧 |
| 弹窗显示 | 0.45s | 0.7 | 更弹，强调"弹入" |
| 通知横幅 | 0.65s | 0.85 | 更慢，更克制 |
| 图标长按菜单 | 0.3s | 0.6 | 极快，弹跳明显 |
| 窗口最小化 | 0.5s | 0.6 | 表演性弹跳 |
| Dock 图标弹跳 | 0.3s | 0.5 | 很弹，吸引注意 |
| 非交互淡入 | 0.2s | 1.0 | 临界阻尼，无过冲 |

### 6.4 解析解

Sokuou Engine 的 Spring 使用**阻尼振荡器的解析解**，而非数值积分（欧拉法）。

**为什么必须用解析解？**

| 特性 | 解析解 | 欧拉数值积分 |
|------|--------|-------------|
| 帧率无关 | 精确——120Hz 与 60Hz 轨迹完全一致 | 帧率越低误差越大 |
| 稳定性 | 无条件稳定 | dt 过大时发散 |
| 可预测性 | 确定性的位置函数 | 随帧率变化 |
| 计算量 | O(1)（一次 exp + 一次 sin/cos） | O(1)（几次乘加） |

**解析解公式：**

设 `ω₀ = 2π / response`（无阻尼角频率），`ζ = damping_ratio`（阻尼比），`ωₙ = ω₀ · √|1 - ζ²|`（有阻尼角频率）。

定义误差 `y(t) = target - x(t)`，初始条件 `y(0) = target - from`，`ẏ(0) = -from_vel`。

**欠阻尼 (ζ < 1)** —— 最常用：

```
y(t) = e^(-ζω₀t) · [A · cos(ωₙt) + B · sin(ωₙt)]

其中：
  A = y(0) = target - from
  B = (ẏ(0) + ζω₀ · y(0)) / ωₙ
    = (-from_vel + ζω₀ · (target - from)) / ωₙ
```

**临界阻尼 (ζ = 1)**：

```
y(t) = e^(-ω₀t) · (A + B · t)

其中：
  A = y(0) = target - from
  B = ẏ(0) + ω₀ · y(0) = -from_vel + ω₀ · (target - from)
```

**过阻尼 (ζ > 1)**：

```
y(t) = e^(-ζω₀t) · [A · cosh(ωₙt) + B · sinh(ωₙt)]

其中：
  ωₙ = ω₀ · √(ζ² - 1)
  A = y(0) = target - from
  B = (ẏ(0) + ζω₀ · y(0)) / ωₙ
```

**最终位置：**

```
x(t) = target - y(t)
```

**速度：**

```
ẋ(t) = -ẏ(t)   （欠阻尼情况）
ẏ(t) = e^(-ζω₀t) · [(-ζω₀A + ωₙB) · cos(ωₙt) - (ζω₀B + ωₙA) · sin(ωₙt)]
```

### 6.5 初始速度继承

Apple 动画的关键特性：当手势结束后，动画继承手势的当前速度。

```
设用户手指抬起时：
  - 界面位置在 from
  - 界面速度在 from_vel（来自手势速度）
  - 目标位置为 target

则弹簧的初始条件为：
  x(0) = from
  ẋ(0) = from_vel
  x(∞) = target
```

这创造了一种"物理惯性"——物体仿佛拥有真实动量，不会因手指离开而突然卡停。

**实现：** 初始速度 `from_vel` 直接代入解析解的常数项 `B` 中。

### 6.6 临界吸附

当弹簧接近稳定时，直接吸附到目标，消除不可见的尾部微振：

```rust
/// 阈值：位移 < 0.001 且 速度 < 0.001
fn snap_if_close(pos: f64, vel: f64, target: f64) -> bool {
    let error = (pos - target).abs();
    error < 0.001 && vel.abs() < 0.001
}
```

吸附后：
- `pos = target`
- `vel = 0.0`
- 标记为稳态

### 6.7 帧率无关性

解析解不依赖 `dt` 做积分，只依赖经过的**绝对时间** `t`：

```rust
// 每帧：
self.elapsed += dt;  // 累计时间
let (pos, vel) = self.spring.evaluate(self.elapsed, ...);  // 解析解直接计算

// 无论 60Hz、120Hz、144Hz，轨迹完全一致
// 120Hz 只是在相同轨迹上取了更多采样点
```

### 6.8 设计

```rust
/// 弹簧动画状态机。
pub struct SpringAnim {
    // 弹簧参数
    response: f64,
    damping_ratio: f64,
    // 本段起始条件
    from: f64,
    from_vel: f64,
    target: f64,
    // 时间跟踪
    elapsed: f64,
    // 输出
    pos: f64,
    vel: f64,
}
```

---

## 7 · 缓动函数库

### 7.1 设计

```rust
/// 缓动函数枚举。
pub enum Easing {
    /// 线性：t
    Linear,
    /// 平方缓出：t · (2 - t)
    EaseOutQuad,
    /// 立方缓出：1 - (1 - t)³
    EaseOutCubic,
    /// 四次缓出：1 - (1 - t)⁴
    EaseOutQuart,
    /// 五次缓出：1 - (1 - t)⁵
    EaseOutQuint,
    /// 立方缓入：t³
    EaseInCubic,
    /// 立方缓入缓出：t < 0.5 ? 4t³ : 1 - (1 - t)³ / 4
    EaseInOutCubic,
    /// 弹性缓出（超出目标后回弹，适合强调动画）
    EaseOutElastic,
    /// 回弹缓出（过冲后回正，比弹簧轻量）
    EaseOutBack,
    /// 弹跳缓出（模拟重力弹跳）
    EaseOutBounce,
    /// 自定义三次贝塞尔
    CubicBezier { x1: f64, y1: f64, x2: f64, y2: f64 },
}
```

### 7.2 缓出（Ease Out）—— 最常用

Apple 推荐的缓出原则：**不缓入，只缓出**。

原因：
- 人眼对"突然出现"不敏感，但对"突然消失"敏感
- 快速开始 → 慢速结束，感觉"自然"
- 慢速开始 → 快速结束，感觉"拖沓"

```rust
// ease-out 的通用形式：t 从 0→1，输出从 0→1，尾部斜率趋近 0
fn ease_out_cubic(t: f64) -> f64 {
    1.0 - (1.0 - t).powi(3)
}
```

### 7.3 缓入（Ease In）—— 仅用于淡出

淡出（fade-out）使用缓入，因为淡出时**内容消失**，缓慢开始更自然。

### 7.4 缓入缓出（Ease In Out）—— 用于循环动画

仅用于持续循环的动画（如加载指示器、呼吸光效），**日常交互不推荐**。

### 7.5 自定义三次贝塞尔

```rust
// 映射到三次贝塞尔曲线 B(t) = (1-t)³·P₀ + 3(1-t)²·t·P₁ + 3(1-t)·t²·P₂ + t³·P₃
// 其中 P₀ = (0,0), P₃ = (1,1), P₁ = (x1,y1), P₂ = (x2,y2)
// 通过牛顿法求解 t 使得 Bx(t) = progress，然后取 By(t)
```

---

## 8 · 动画编排

### 8.1 设计

```rust
/// 动画编排器。
pub struct Animator {
    /// 并行运行的动画
    parallels: Vec<Box<dyn Animation>>,
    /// 串行运行的动画队列
    sequence: Vec<Box<dyn Animation>>,
}
```

### 8.2 三种编排模式

**并行（Parallel）**——多个动画同时运行：

```
time ──►
  A: ████████████
  B: ████████████
  C: ████████████
```

适用场景：同一个面板的位移 + 透明度 + 缩放同时变化。

**串行（Sequence）**——动画依次执行：

```
time ──►
  A: ████████
  B:          ████████
  C:                   ████████
```

适用场景：面板升起 → 内容渐显 → 按钮高亮。

**链式（Chain）**——前一个动画完成后触发下一个，但属性上有依赖关系：

```
time ──►
  A: ████████
  B:          ████████  (依赖 A 的结果)
  C:                   ████████  (依赖 B 的结果)
```

适用场景：Dock 图标逐个错位展开。

### 8.3 延迟与偏移

```rust
struct Delayed {
    delay: Duration,
    animation: Box<dyn Animation>,
}
```

错位延迟：同一组元素，每个元素延迟 `i * stagger_ms` 后开始。

---

## 9 · 与渲染循环的集成

### 9.1 标准模式

```rust
// 每帧调用（伪代码）
fn frame(dt: f64) {
    let dt = dt.min(0.05);  // dt 上限保护

    // 1. 更新所有动画
    launchpad_anim.update(dt);
    dock_anim.update(dt);
    fade_anim.update(dt);

    // 2. 从 progress 解析渲染状态
    let launchpad_pos = resolve_position(launchpad_anim.value());
    let launchpad_alpha = resolve_alpha(launchpad_anim.value());

    // 3. 渲染
    render(launchpad_pos, launchpad_alpha, ...);
}
```

### 9.2 dt 处理

```rust
fn update(&mut self, dt: f64) {
    // dt 上限保护：防止跳帧（如调试时断点恢复）导致数值异常
    let dt = dt.min(0.05);

    // Spring 解析解：累计 elapsed，直接计算位置
    self.elapsed += dt;
    let (pos, vel) = self.spring.evaluate(self.elapsed, self.from, self.from_vel, self.target);
    self.pos = pos;
    self.vel = vel;

    // 临界吸附
    if self.snap_if_close() {
        self.pos = self.target;
        self.vel = 0.0;
    }
}
```

### 9.3 帧率无关性

解析解保证：**轨迹完全由物理参数决定，与帧率无关**。

```rust
// 60Hz 下经过 0.5s → 10 次 update，每次 dt=0.0167
// 120Hz 下经过 0.5s → 20 次 update，每次 dt=0.0083
// 结果完全相同：pos, vel 在 t=0.5s 时一致
```

### 9.4 稳态检测与渲染优化

```rust
fn render(&self) {
    // progress 极低时跳过渲染
    if self.pos < 0.01 { return; }

    // 稳态时使用精确值
    if self.is_steady() {
        self.draw_at_target();  // 直接使用 target 值，无需 progress 派生
        return;
    }

    // 动画进行中：从 progress 派生
    let slide = (1.0 - self.pos) * output_h;
    let alpha = self.content_alpha();
    self.draw_at(slide, alpha);
}
```

---

## 10 · API 参考

### 10.1 Progress

```rust
// ── 构造 ──
Progress::new(duration: Duration) -> Self
Progress::with_easing(duration: Duration, easing: Easing) -> Self

// ── 控制 ──
set_target(&mut self, target: f64)     // 设定目标，可中断
set_duration(&mut self, duration: Duration)  // 动态调整时长
set_easing(&mut self, easing: Easing)  // 动态调整缓动函数

// ── 每帧 ──
update(&mut self)                      // 推进动画

// ── 查询 ──
value() -> f64                         // 当前值 [0.0, 1.0]
is_steady() -> bool                    // 是否稳定
target() -> f64                        // 当前目标值
```

### 10.2 Spring

```rust
// ── 构造 ──
SpringAnim::new(
    response: Duration,
    damping_ratio: f64,
    from: f64,
    from_vel: f64,
    target: f64,
) -> Self

// ── 控制 ──
set_target(&mut self, new_target: f64)  // 设定新目标，继承当前速度
set_target_with_velocity(&mut self, new_target: f64, from_vel: f64)  // 设定新目标，指定速度
set_params(&mut self, response: Duration, damping_ratio: f64)  // 动态调整物理参数

// ── 每帧 ──
update(&mut self, dt: f64)              // 推进动画

// ── 查询 ──
value() -> f64                          // 当前位置 [0.0, 1.0]
velocity() -> f64                       // 当前速度
target() -> f64                         // 当前目标
is_steady() -> bool                     // 是否稳定
snap_if_close(&mut self) -> bool        // 临界吸附，返回是否吸附
```

### 10.3 Easing

```rust
// ── 缓动函数（直接使用） ──
Easing::ease(t: f64) -> f64              // 对 [0.0, 1.0] 的 t 应用缓动

// ── 静态函数（独立使用） ──
ease_out_cubic(t: f64) -> f64
ease_out_elastic(t: f64) -> f64
ease_out_back(t: f64) -> f64
cubic_bezier(t: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64
```

### 10.4 Animator

```rust
// ── 构造 ──
Animator::new() -> Self

// ── 编排 ──
add_parallel(&mut self, anim: Box<dyn Animation>)   // 添加并行动画
add_sequence(&mut self, anim: Box<dyn Animation>)   // 添加串行动画

// ── 每帧 ──
update(&mut self, dt: f64)              // 推进所有动画

// ── 查询 ──
is_steady() -> bool                     // 所有动画是否均稳定
```

```rust
/// 动画 trait（用于 Animator 编排）
pub trait Animation {
    fn update(&mut self, dt: f64);
    fn value(&self) -> f64;
    fn is_steady(&self) -> bool;
}
```

### 10.5 辅助工具

```rust
// ── 延迟动画 ──
DelayedAnim::new(delay: Duration, inner: Box<dyn Animation>) -> Self

// ── 错位序列（stagger） ──
StaggeredAnim::new(
    count: usize,
    stagger: Duration,
    factory: Box<dyn Fn(usize) -> Box<dyn Animation>>,
) -> Self

// ── 范围映射 ──
// 将 progress [0.0, 1.0] 映射到任意值域
fn map_range(progress: f64, from: f64, to: f64) -> f64;
fn map_range_clamped(progress: f64, from: f64, to: f64) -> f64;
```

---

## 11 · 最佳实践

### 11.1 默认推荐参数

```rust
// 通用交互（点击、弹窗、面板切换）
let spring = SpringAnim::new(
    Duration::from_secs_f64(0.5),  // response: 0.5s
    0.825,                            // damping_ratio: 0.825
    from, from_vel, target,
);

// 快速交互（按钮按下、图标弹跳）
let spring = SpringAnim::new(
    Duration::from_secs_f64(0.3),  // response: 0.3s
    0.6,                              // damping_ratio: 0.6
    from, from_vel, target,
);

// 慢速展示（通知横幅、引导提示）
let spring = SpringAnim::new(
    Duration::from_secs_f64(0.65), // response: 0.65s
    0.85,                             // damping_ratio: 0.85
    from, from_vel, target,
);

// 非交互淡入（透明度变化，无弹簧）
let progress = Progress::with_easing(
    Duration::from_millis(200),
    Easing::EaseOutCubic,
);
```

### 11.2 感知时长

动画时长应因触发方式不同而变化：

| 触发方式 | 时长 | 原因 |
|----------|------|------|
| 用户直接操作（点击、按钮） | 快（0.3-0.4s） | 用户期待即时反馈 |
| 用户间接操作（弹窗、菜单） | 中（0.4-0.5s） | 需要被注意到 |
| 系统自动触发（通知、提示） | 慢（0.5-0.7s） | 用户需要时间理解 |
| 手势驱动（滑动返回） | 跟随手指速度 | 物理惯性 |

### 11.3 速度继承

```rust
// 手势结束时，将手势速度作为弹簧初始速度
fn on_gesture_end(&mut self, gesture_velocity: f64) {
    let target = compute_target_from_gesture();
    self.spring_anim.set_target_with_velocity(target, gesture_velocity);
}
```

### 11.4 主次动画分离

```rust
// 主要运动（位置、大小）→ Spring（物理感）
// 次要属性（透明度、颜色）→ Progress + EaseOut（非物理感）

// 正确：
fn update(&mut self, dt: f64) {
    self.spring_anim.update(dt);         // 主运动：弹簧物理
    let p = self.spring_anim.value();
    self.alpha = ease_out_cubic(p);      // 次要属性：从 progress 派生
    self.scale = 0.8 + 0.2 * p;          // 缩放：从 progress 派生
}
```

### 11.5 不滥用弹簧

| 场景 | 推荐工具 | 原因 |
|------|----------|------|
| 窗口弹入 | Spring (response=0.4, ratio=0.8) | 物理感 |
| 透明度变化 | Progress + EaseOutCubic | 透明度不需要弹跳 |
| 面板滑入 | Spring (response=0.5, ratio=0.825) | 物理感 |
| 图标旋转 | Progress + EaseOutBack | 轻微过冲即可 |
| 颜色过渡 | Progress + Linear | 颜色插值不需要弹簧 |
| 加载指示器 | EaseInOutCubic | 循环动画 |

### 11.6 性能守则

```
1. Spring 解析解 O(1) — 一次 exp + 一次 sin/cos，放心用
2. 每帧 update() 只做数值计算，不做 I/O 或纹理上传
3. 稳态时及时 snap，减少不必要的渲染
4. progress < 0.01 时可跳过渲染
5. dt 上限保护（dt.min(0.05)）防止跳帧
```

---

## 12 · 设计不变量

| 原则 | 含义 | 执行 |
|------|------|------|
| 可中断性 | 随时中断、反向、接管、回弹，不锁定 | `set_target()` 在任意时刻可用 |
| 连续空间存在 | 无 pop-in、sudden spawn、abrupt destruction | 所有动画从 0→1 或 1→0，永不消失 |
| 渐隐即存在转移 | Fade 不是透明度渐变，而是空间存在感转移 | fade-out 同时 fade-in 另一个元素 |
| 不可伪造进度 | progress 必须映射真实时间 | 解析解基于真实 elapsed 时间，不缩放 |
| 状态驱动渲染 | 不是 `play_animation()`，而是 `state → progress → render` | 渲染函数只读 progress，不操作动画 |
| 确定性解析器 | 必须可逆、连续、支持任意状态停留 | 解析解定义域为所有 t ≥ 0 |
| 帧率无关 | 轨迹不依赖帧率 | 解析解基于绝对时间，不是帧计数 |
| 空间真实性 | 用户不应感到"动画正在播放"，而应感到"空间正在变化" | 弹簧物理模拟真实世界运动 |

---

## 13 · 附录：数学推导

### 13.1 阻尼振荡器

标准二阶系统：

```
ẍ + 2ζω₀ẋ + ω₀²x = ω₀² · target
```

其中：
- `ω₀ = 2π / response`：无阻尼角频率
- `ζ = damping_ratio`：阻尼比

令 `y(t) = target - x(t)`，则：

```
ÿ + 2ζω₀ẏ + ω₀²y = 0
```

### 13.2 特征方程

```
r² + 2ζω₀r + ω₀² = 0
r = -ζω₀ ± ω₀ · √(ζ² - 1)
```

### 13.3 三种情况

**欠阻尼 (ζ < 1)**：

```
r = -ζω₀ ± iωₙ, 其中 ωₙ = ω₀ · √(1 - ζ²)
y(t) = e^(-ζω₀t) · [A · cos(ωₙt) + B · sin(ωₙt)]
```

**临界阻尼 (ζ = 1)**：

```
r = -ω₀（重复根）
y(t) = e^(-ω₀t) · (A + B · t)
```

**过阻尼 (ζ > 1)**：

```
r = -ζω₀ ± ωₙ, 其中 ωₙ = ω₀ · √(ζ² - 1)
y(t) = e^(-ζω₀t) · [A · cosh(ωₙt) + B · sinh(ωₙt)]
```

### 13.4 常数项求解

初始条件 `y(0) = target - from`，`ẏ(0) = -from_vel`。

以欠阻尼为例：

```
y(0) = A = target - from
ẏ(0) = -ζω₀A + ωₙB = -from_vel
→ B = (ζω₀A - ẏ(0)) / ωₙ
    = (ζω₀ · (target - from) + from_vel) / ωₙ
```

### 13.5 速度公式

以欠阻尼为例：

```
ẏ(t) = e^(-ζω₀t) · [(-ζω₀A + ωₙB) · cos(ωₙt) - (ζω₀B + ωₙA) · sin(ωₙt)]
ẋ(t) = -ẏ(t)
```

### 13.6 参数转换

```
response  →  ω₀ = 2π / response
damping_ratio  →  ζ = damping_ratio

stiffness = ω₀² = (2π / response)²
damping   = 2ζω₀ = 4π · damping_ratio / response
```

---

## 附录：与现有代码的兼容性

Sokuou Engine 早期版应保持与 `ether-motion` 现有 API 的兼容：

```rust
// 现有 API 保持不变
mod legacy {
    pub struct Progress { /* ... */ }
    pub struct Spring { stiffness, damping }
}

// 新增 API
pub struct SpringAnim { response, damping_ratio }

// 便利转换
impl SpringAnim {
    pub fn from_legacy(stiffness: f64, damping: f64) -> Self {
        let omega0 = stiffness.sqrt();
        let response = 2.0 * std::f64::consts::PI / omega0;
        let damping_ratio = damping / (2.0 * omega0);
        Self { response, damping_ratio }
    }
}
```

---

> **Sokuou Engine 的目标并非炫技——而是维持空间真实性。**
>
> 用户不应感受到"动画正在播放"，而应感受到"空间正在变化"。
>
> —— Sokuou Engine · 即応エンジン