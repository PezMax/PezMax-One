---
kind: configuration_system
name: Spring Boot + Vite 多环境配置体系
category: configuration_system
scope:
    - '**'
source_files:
    - PezMax-Backend/ruoyi-admin/src/main/resources/application.yml
    - PezMax-Backend/ruoyi-admin/src/main/resources/application-druid.yml
    - PezMax-Backend/ruoyi-admin/src/main/resources/logback.xml
    - PezMax-Backend/ruoyi-common/src/main/java/com/ruoyi/common/config/RuoYiConfig.java
    - PezMax-Backend/ruoyi-common/src/main/java/com/ruoyi/common/config/MinioConfig.java
    - PezMax-Backend/ruoyi-framework/src/main/java/com/ruoyi/framework/config/properties/DruidProperties.java
    - PezMax-Backend/ruoyi-framework/src/main/java/com/ruoyi/framework/config/properties/PermitAllUrlProperties.java
    - PezMax-Backend/ruoyi-framework/src/main/java/com/ruoyi/framework/config/SecurityConfig.java
    - PezMax-Backend/ruoyi-framework/src/main/java/com/ruoyi/framework/config/RedisConfig.java
    - PezMax-Backend/ruoyi-generator/src/main/resources/generator.yml
    - PezMax-Desktop/.env.development
    - PezMax-Desktop/.env.production
---

## 系统概述
本项目采用 Spring Boot（后端）+ Electron-vite Vue3（桌面客户端）双端架构，各自维护独立的环境配置体系，通过 YAML/Properties 与 .env 文件分层管理运行时参数。

## 后端配置体系（Spring Boot / 若依）

### 配置文件分层
- **application.yml**：主配置入口，定义项目元信息、服务器端口、Redis、MinIO、XSS、Referer、token、MyBatis、PageHelper、Springdoc 等通用配置；敏感项通过 `${ENV_VAR:default}` 占位符注入环境变量。
- **application-druid.yml**：数据源 profile，按 `spring.profiles.active=druid` 激活，包含主库/从库连接池、Druid 监控与 SQL 审计参数。
- **logback.xml**：Logback 日志配置，按 INFO/ERROR/用户操作三类输出到不同滚动文件，保留 60 天历史。
- **generator.yml**：代码生成器专属配置（包名、表前缀、覆盖策略），位于 `ruoyi-generator` 模块。

### 配置绑定方式
| 方式 | 使用位置 | 说明 |
|---|---|---|
| `@ConfigurationProperties(prefix="ruoyi")` | `RuoYiConfig.java` | 将 `ruoyi.*` 批量映射为静态字段，提供 `getUploadPath()` 等派生路径方法 |
| `@Value("${minio.url}")` | `MinioConfig.java` | 单值注入 MinIO 客户端 Bean |
| `@Value("${spring.datasource.druid.*}")` | `DruidProperties.java` | 逐个读取 Druid 连接池参数并回写 DataSource |
| `PermitAllUrlProperties` | 启动时扫描 `@Anonymous` 注解，动态构建匿名访问 URL 列表供 Security 使用 |

### 关键配置域
- **应用元信息**：`ruoyi.name/version/copyrightYear/profile`（上传根目录，默认 `/home/ruoyi/uploadPath`）
- **安全**：`token.header/secret/expireTime`、`xss.enabled/excludes/urlPatterns`、`referer.allowed-domains`
- **存储**：`minio.url/accessKey/secretKey/bucketName`；本地文件路径由 `RuoYiConfig.getUploadPath()` 等统一计算
- **缓存**：`spring.data.redis.host/port/database/password`，Lettuce 连接池参数
- **数据库**：`DB_HOST` 环境变量驱动 JDBC URL，默认 `ptmj-platform` 库
- **业务开关**：`ptmj.file.allow-format/min-year/type-map` 控制学习资料格式与分类映射

### 配置加载约定
1. 所有可外部化的值优先读环境变量，其次取 YAML 中默认值（`${VAR:default}` 语法）
2. 需要跨模块共享的配置集中在 `ruoyi-common.config` 下以 `@Component` 暴露
3. 第三方组件（MinIO、Redis、Druid）通过独立 `@Configuration` 类装配 Bean，避免在业务类中直接 `@Value`
4. 安全白名单不硬编码，通过 `@Anonymous` 注解 + `PermitAllUrlProperties` 自动发现

## 前端配置体系（Electron-vite Vue3）

### 环境变量文件
- `.env.development`：开发模式，`VITE_APP_BASE_API=/dev-api` 配合 Vite 代理转发到 `VITE_APP_TARGET_URL`（默认 `http://154.8.39.48:8080`）
- `.env.production`：生产模式，`VITE_APP_BASE_API` 直连后端地址，启用 gzip 压缩与 electron-updater generic 更新源
- `.env.staging`：预发布环境（存在但未在上述文件中展开）

### 变量命名约定
- `VITE_APP_*`：Vue 侧全局常量，通过 `import.meta.env.VITE_APP_TITLE` 等形式注入
- `PTMJ_UPDATE_PROVIDER/URL/GH_OWNER/GH_REPO`：electron-updater 更新源选择（generic/github）
- `VITE_APP_TARGET_URL`：Electron 主进程直连后端的真实地址（绕过浏览器同源限制）

### 前后端配置差异对比
| 维度 | 后端（Spring Boot） | 前端（Electron-vite） |
|---|---|---|
| 文件格式 | YAML + XML + Java `@Value` | `.env.*` 明文键值对 |
| 环境切换 | `spring.profiles.active=druid` | 构建时选择 `.env.development` / `.env.production` |
| 敏感信息 | 环境变量占位符（如 `REDIS_HOST`） | 仅存非敏感目标地址，密钥不在前端 |
| 动态发现 | `@Anonymous` 注解扫描匿名路由 | 无，API 基址固定于环境变量 |

## 开发者应遵循的规则
1. **新增配置项**：优先放入 `application.yml` 对应分组，并在 `ruoyi-common.config` 中补充 `@ConfigurationProperties` 或 `@Value` 绑定类
2. **敏感值外置**：密码、密钥一律使用 `${ENV_VAR:default}` 形式，禁止硬编码进仓库
3. **Profile 隔离**：数据库、MinIO、Redis 等基础设施配置放在 `application-{profile}.yml`，通过 `profiles.active` 切换
4. **前端环境变量**：新增 `VITE_APP_*` 需同步更新 `.env.development` 与 `.env.production`，保持两套一致
5. **安全白名单**：通过 `@Anonymous` 标注接口，不要手动修改 `SecurityConfig` 中的 permitAll 列表
6. **日志路径**：`logback.xml` 中 `/home/ruoyi/logs` 需在部署环境提前创建并赋予写入权限