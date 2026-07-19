---
kind: build_system
name: 构建与制品管理：Maven 多模块 + Docker Compose + Electron-builder
category: build_system
scope:
    - '**'
source_files:
    - PezMax-Backend/pom.xml
    - PezMax-Backend/Dockerfile
    - PezMax-Backend/compose.yaml
    - PezMax-Backend/ry.sh
    - PezMax-Backend/bin/package.bat
    - PezMax-Desktop/package.json
    - PezMax-Desktop/electron.vite.config.mjs
    - PezMax-Desktop/electron-builder.yml
---

## 1. 使用的系统/工具链

- **后端（Java）**：基于 Maven 的多模块项目，使用 Spring Boot 3.x 打包为可执行 JAR；通过 `mvnw` Wrapper 保证构建一致性。
- **容器化**：Dockerfile 采用多阶段构建 + Spring Boot Layered Jar 分层优化；Compose 编排 MySQL、Redis、MinIO 与应用服务。
- **桌面客户端（Electron + Vue3）**：electron-vite 负责主进程/预加载/渲染进程构建，electron-builder 打包为 Windows NSIS、macOS DMG、Linux AppImage/snap/deb，并配置 GitHub Releases 作为自动更新源。
- **本地运行脚本**：提供 `ry.sh`（Linux/macOS）和 `bin/package.bat`（Windows）简化启动/打包流程。

## 2. 关键文件与位置

- 后端构建
  - `PezMax-Backend/pom.xml`：顶层聚合 POM，声明 Java 17、Spring Boot 4.0.3、依赖版本与 7 个子模块。
  - `PezMax-Backend/Dockerfile`：多阶段镜像（deps → package → extract → final），安装 LibreOffice 与中文字体，以非 root 用户运行。
  - `PezMax-Backend/compose.yaml`：MySQL 8.0.44 / Redis / MinIO / server 四服务编排，含健康检查与数据卷挂载。
  - `PezMax-Backend/ry.sh`：JVM 参数固定（-Xms512m/-Xmx1024m，ParallelGC），支持 start/stop/restart/status。
  - `PezMax-Backend/bin/package.bat`：调用 `mvn clean package -Dmaven.test.skip=true`。
- 前端（Web 管理端）
  - `PezMax-Backend/ruoyi-ui/package.json`：Vite 构建，`build.bat`/`package.bat`/`run-web.bat` 辅助脚本。
- 桌面客户端
  - `PezMax-Desktop/package.json`：electron-vite 脚本（dev/build:win/macos/linux）、electron-updater 集成。
  - `PezMax-Desktop/electron.vite.config.mjs`：main/preload/renderer 三入口 Vite 配置，开发代理 `/dev-api` 到后端。
  - `PezMax-Desktop/electron-builder.yml`：NSIS 差分包、generic 发布到 GitHub Releases、各平台 artifactName 模板。

## 3. 架构与约定

### 后端 Maven 多模块
- 顶层 `packaging=pom`，子模块包括 `ruoyi-admin`、`ruoyi-framework`、`ruoyi-system`、`ruoyi-quartz`、`ruoyi-generator`、`ruoyi-common`、`ptmj-datum`。
- 所有子模块统一通过父 POM 的 `<dependencyManagement>` 锁定版本，避免版本漂移。
- 仓库指向阿里云公共镜像，插件仓库同样配置，加速国内下载。
- 构建目标仅针对 `ruoyi-admin` 模块（`-pl ruoyi-admin -am`），其他模块作为依赖被一起编译。

### Docker 分层与运行
- 依赖缓存：`--mount=type=cache,target=/root/.m2` 复用 Maven 本地仓库。
- 分层提取：`java -Djarmode=layertools` 将 dependencies、spring-boot-loader、snapshot-dependencies、application 拆成独立层，提升增量构建效率。
- 运行时镜像最小化：eclipse-temurin:17-jre-jammy，仅拷贝应用层，暴露 8080 端口，以 UID 10001 非特权用户运行。
- 额外依赖：LibreOffice 与 `fonts-wqy-zenhei` 用于文档预览/转换。

### Compose 环境
- 默认数据库名 `ptmj-platform`，根密码 `123456`，并通过 `sql/pezmax.sql` 初始化。
- 环境变量注入：`DB_HOST`、`REDIS_HOST`、`UPLOAD_PATH`、`JAVA_OPTS`，便于覆盖。
- 健康检查确保 MySQL/Redis/MinIO 就绪后再启动 server。

### 桌面客户端构建
- electron-vite 分别构建 main、preload、renderer 三个产物，输出到 `out/`。
- 生产构建启用 gzip 压缩（`vite-plugin-compression`），关闭 sourcemap。
- electron-builder 按平台生成安装包，NSIS 开启 `differentialPackage: true` 支持增量更新；publish 统一指向 GitHub Releases 的 generic 地址。
- 通过 `VITE_AUTH_ENTRY_MODE` 环境变量切换 client/admin 两种入口模式。

## 4. 开发者应遵循的规则

- **后端**
  - 新增模块需在顶层 `pom.xml` 的 `<modules>` 中注册，并在 `dependencyManagement` 中声明版本。
  - 本地打包优先使用 `./mvnw`，避免本地 Maven 版本差异导致构建失败。
  - 容器化部署时通过环境变量覆盖数据库/Redis/上传路径，不要修改 Dockerfile 中的硬编码值。
  - 直接 JVM 部署时使用 `ry.sh` 或自行封装 systemd/supervisor，保持 JVM 参数一致。
- **桌面客户端**
  - 新增平台打包使用 `npm run build:win|mac|linux`，产物由 electron-builder 根据 `electron-builder.yml` 命名。
  - 发布前确认 `dev-app-update.yml` 与 `electron-builder.yml` 的 `publish.url` 指向正确的 GitHub Releases 地址。
  - 开发时通过 `VITE_APP_TARGET_URL` 指定后端地址，或通过 `.env.*` 文件管理不同环境。
- **前后端联调**
  - 开发期 renderer 通过 Vite proxy 将 `/dev-api` 转发到后端，无需在浏览器侧处理 CORS。
  - 若修改后端端口或部署域名，需同步更新 `electron.vite.config.mjs` 中的代理 target 及前端环境变量。
