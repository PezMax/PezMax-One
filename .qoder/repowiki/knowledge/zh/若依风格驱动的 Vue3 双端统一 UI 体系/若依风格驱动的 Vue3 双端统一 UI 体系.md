---
kind: frontend_style
name: 若依风格驱动的 Vue3 双端统一 UI 体系
category: frontend_style
scope:
    - '**'
source_files:
    - PezMax-Backend/ruoyi-ui/src/settings.js
    - PezMax-Desktop/src/renderer/settings.js
    - PezMax-Backend/ruoyi-ui/src/utils/theme.js
    - PezMax-Backend/ruoyi-ui/src/layout/index.vue
    - PezMax-Backend/ruoyi-ui/src/main.js
    - PezMax-Backend/ruoyi-ui/vite/plugins/svg-icon.js
    - PezMax-Backend/ruoyi-ui/vite/config.js
---

本项目前后端均基于若依（RuoYi）生态，采用统一的 Vue3 + Element Plus 管理后台风格，通过配置化主题与布局实现 Web 与桌面客户端视觉一致性。

**1. 使用的系统与工具**
- 前端框架：Vue3 + Vite（Web 端 `PezMax-Backend/ruoyi-ui`，桌面端 `PezMax-Desktop/src/renderer`）
- UI 组件库：Element Plus（若依默认），配合自定义 SVG 图标系统（`assets/icons/svg`）
- 构建插件：`vite-plugin-auto-import`、`vite-plugin-svg-icon`、`vite-plugin-compression` 等
- 样式方案：原生 CSS + SCSS，无 Tailwind / UnoCSS；通过全局 CSS 变量与主题类名切换明暗主题
- 状态持久化：侧边栏主题、导航模式、tagsView 等通过 `settings.js` 集中导出，由 `utils/theme.js` 在运行时注入 `<html>` 的 class 控制

**2. 关键文件与包**
- 主题与布局开关：`ruoyi-ui/src/settings.js`、`src/utils/theme.js`、`src/layout/index.vue`
- 全局样式入口：`ruoyi-ui/src/main.js` 中引入若依基础样式与自定义覆盖
- 图标资源：`assets/icons/svg/*.svg`（90+ 个业务图标）与 `SvgIcon` 组件
- 桌面端额外样式：`src/renderer/assets/styles/` 下存放 Electron 窗口适配、通知弹窗等局部样式
- 打包产物：`vite.config.js` 中配置 SVG 自动导入、压缩与代理，保证开发/生产一致体验

**3. 架构与约定**
- **单源主题配置**：两个前端的 `settings.js` 字段完全对齐（`sideTheme`、`navType`、`tagsView`、`fixedHeader`、`sidebarLogo`、`footerVisible` 等），仅 `footerContent` 不同（Web 为 RuoYi 版权，桌面为 PTMJ 版权），确保双端外观一致。
- **主题切换机制**：通过向 `<html>` 根节点添加 `theme-dark` / `theme-light` class，配合全局 CSS 变量实现一键换肤，无需重新编译。
- **布局骨架复用**：`layout/components/*`（Sidebar、Navbar、TagsView、AppMain、Hamburger、TopNav 等）在 Web 与桌面端结构几乎相同，遵循若依经典的「左侧菜单 + 顶部导航 + 标签页」三栏布局。
- **图标规范**：所有图标以 SVG 形式存放在 `assets/icons/svg`，通过 `SvgIcon` 组件按需渲染，避免字体图标带来的额外依赖。
- **响应式策略**：主要面向桌面浏览器与 Electron 窗口，未使用媒体查询做移动端适配，布局宽度以固定像素 + flex 为主。

**4. 开发者应遵守的规则**
- 新增页面优先复用若依布局组件（`Breadcrumb`、`RightToolbar`、`Pagination`、`Editor` 等），不要自行重复实现表格/分页/富文本。
- 颜色、字号、间距等设计令牌一律通过全局 CSS 变量或 Element Plus 主题变量引用，禁止在组件内硬编码色值。
- 新增图标必须放入 `assets/icons/svg` 并通过 `SvgIcon` 组件使用，保持命名语义化（kebab-case）。
- 如需调整主题行为，修改 `settings.js` 对应字段而非直接改 CSS；动态主题切换只允许通过 `utils/theme.js` 暴露的 API。
- 桌面端新增的局部样式放在 `src/renderer/assets/styles/` 下，按功能模块分包，避免污染全局样式。