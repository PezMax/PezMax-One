---
kind: dependency_management
name: 多语言依赖管理：Maven + NPM/Yarn 双栈策略
category: dependency_management
scope:
    - '**'
source_files:
    - PezMax-Backend/pom.xml
    - PezMax-Backend/ptmj-datum/pom.xml
    - PezMax-Backend/ruoyi-admin/pom.xml
    - PezMax-Desktop/package.json
    - PezMax-Desktop/yarn.lock
    - PezMax-Desktop/.npmrc
---

## 1. 系统概览

本项目采用**双栈依赖管理**：后端使用 Maven（Spring Boot 多模块），桌面客户端使用 npm/yarn（Electron-vite + Vue3）。两套体系各自独立，通过各自的锁文件保证可重复构建。

- **后端（Java）**：Maven 多模块项目，基于若依（RuoYi）框架，版本 `3.9.2`，使用 Spring Boot `4.0.3`、MyBatis、Druid、Fastjson2、JWT、SpringDoc 等组件。
- **前端/桌面（Node.js）**：Electron-vite + Vue3，依赖由 `package.json` 声明，锁定在 `yarn.lock`（同时存在 `package-lock.json`，但实际使用 yarn）。

## 2. 关键文件与位置

| 技术栈 | 核心文件 | 作用 |
|--------|----------|------|
| Maven | `PezMax-Backend/pom.xml` | 根 POM，集中声明所有第三方库版本与 `<dependencyManagement>` |
| Maven 子模块 | `PezMax-Backend/*/pom.xml` | 各模块仅声明依赖坐标，不写版本号（继承父 POM） |
| Maven 仓库 | `PezMax-Backend/.mvn/wrapper/maven-wrapper.properties` | Maven Wrapper 配置 |
| NPM | `PezMax-Desktop/package.json` | 依赖声明（dependencies + devDependencies） |
| Yarn 锁 | `PezMax-Desktop/yarn.lock` | 精确锁定所有依赖树版本与 integrity |
| NPM 镜像 | `PezMax-Desktop/.npmrc` | Electron/electron-builder 二进制下载镜像（npmmirror.com） |

## 3. 架构与约定

### 3.1 Maven 多模块分层

```
ruoyi (根 POM)
├── ruoyi-common      # 通用工具、常量、异常、过滤器
├── ruoyi-framework   # 安全、缓存、AOP、配置、Web 基础
├── ruoyi-system      # 用户、角色、菜单、字典等系统功能
├── ruoyi-quartz      # 定时任务
├── ruoyi-generator   # 代码生成器
├── ptmj-datum        # 业务领域：学习资料、书签、收藏、举报、通知
└── ruoyi-admin       # Web 入口，聚合以上模块并打包为可执行 jar
```

- **版本统一**：所有第三方库版本集中在根 `pom.xml` 的 `<properties>` 中定义，并通过 `<dependencyManagement>` 引入；子模块引用时**不写 `<version>`**。
- **内部模块版本**：所有 `com.ruoyi:*` 内部模块统一使用 `${ruoyi.version}`（当前 `3.9.2`）。
- **仓库源**：仅配置阿里云公共仓库 `https://maven.aliyun.com/repository/public`，无私有仓库或 Nexus 代理。
- **插件仓库**：单独配置 `<pluginRepositories>` 指向同一阿里云地址。

### 3.2 Node.js 依赖管理

- **包管理器**：项目同时包含 `yarn.lock` 和 `package-lock.json`，但从 `.npmrc` 中的镜像配置以及 lock 文件头部注释 `# yarn lockfile v1` 可知，**实际使用 yarn**。
- **镜像加速**：通过 `.npmrc` 将 Electron 及 electron-builder 的二进制下载源替换为 npmmirror.com，解决国内网络问题。
- **依赖分类**：
  - `dependencies`：运行时依赖（axios、element-plus、vue-router、pinia、electron-updater 等）
  - `devDependencies`：构建/开发期依赖（electron、vite、eslint、prettier、sass 等）
- **自动安装**：`postinstall` 脚本调用 `electron-builder install-app-deps`，确保原生模块正确编译。

## 4. 开发者应遵循的规则

1. **新增 Java 依赖**：
   - 先在根 `pom.xml` 的 `<properties>` 中声明版本号，再在 `<dependencyManagement>` 中引入。
   - 子模块中仅写 `<groupId>/<artifactId>`，**禁止自行指定 version**。
   - 如需新增内部模块，需在根 POM 的 `<modules>` 中注册。

2. **新增 Node.js 依赖**：
   - 使用 `yarn add <pkg>` 而非 npm，以确保写入 `yarn.lock`。
   - 区分 `dependencies` 与 `devDependencies`，构建期工具放入后者。
   - 不要手动编辑 `yarn.lock`。

3. **仓库与镜像**：
   - Maven 侧如需接入私有仓库，应在根 POM 的 `<repositories>` 中统一配置，避免在各子模块重复声明。
   - Node 侧如需切换镜像源，修改 `.npmrc` 即可全局生效。

4. **版本升级注意事项**：
   - Spring Boot 已升级到 `4.0.3`（较新大版本），升级时需关注兼容性变更。
   - 前后端均使用较新版本生态（Vue 3.5、Element Plus 2.x、Electron 39），升级前需评估对现有 API 的影响。