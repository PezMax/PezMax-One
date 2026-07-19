---
kind: logging_system
name: 基于 Logback + SLF4J 的若依操作日志体系
category: logging_system
scope:
    - '**'
source_files:
    - PezMax-Backend/ruoyi-admin/src/main/resources/logback.xml
    - PezMax-Backend/ruoyi-common/src/main/java/com/ruoyi/common/annotation/Log.java
    - PezMax-Backend/ruoyi-framework/src/main/java/com/ruoyi/framework/aspectj/LogAspect.java
    - PezMax-Backend/ruoyi-common/src/main/java/com/ruoyi/common/utils/LogUtils.java
    - PezMax-Backend/ruoyi-common/src/main/java/com/ruoyi/common/core/controller/BaseController.java
---

## 1. 使用的框架与工具
- 日志门面：SLF4J（`org.slf4j.Logger` / `LoggerFactory`）
- 日志实现：Logback（`ch.qos.logback.core.*`），通过 `ruoyi-admin/src/main/resources/logback.xml` 集中配置
- 操作审计：基于 AspectJ AOP 的自定义注解 `@Log` + `LogAspect`，将关键 Controller 方法调用持久化到数据库 `sys_oper_log` 表
- 辅助工具：`com.ruoyi.common.utils.LogUtils` 提供简单的消息包装；`BaseController` 暴露 `logger` 供子类复用

## 2. 核心文件与包
- 配置中心：`PezMax-Backend/ruoyi-admin/src/main/resources/logback.xml`
- 注解定义：`PezMax-Backend/ruoyi-common/src/main/java/com/ruoyi/common/annotation/Log.java`
- AOP 切面：`PezMax-Backend/ruoyi-framework/src/main/java/com/ruoyi/framework/aspectj/LogAspect.java`
- 通用基类：`PezMax-Backend/ruoyi-common/src/main/java/com/ruoyi/common/core/controller/BaseController.java`
- 业务日志使用示例：各 `ptmj-datum` 下 `service/impl/*ServiceImpl.java` 中通过 `Logger log = LoggerFactory.getLogger(...)` 记录业务日志
- 操作日志实体：`ruoyi-system` 模块中的 `SysOperLog`（由 `LogAspect` 组装并异步写入）

## 3. 架构与约定
### 3.1 输出通道与级别策略
- 控制台：`console` appender，默认 root level=INFO
- 系统信息日志：`file_info` RollingFileAppender，按天滚动，保留 60 天，仅 ACCEPT INFO
- 错误日志：`file_error` RollingFileAppender，单独输出 ERROR，按天滚动，保留 60 天
- 用户访问日志：`sys-user` RollingFileAppender，独立文件 `sys-user.log`，用于用户行为追踪
- 包级控制：`com.ruoyi` 设为 info，`org.springframework` 设为 warn，避免第三方噪音

### 3.2 结构化字段（操作日志）
`LogAspect` 在 `@Before/@AfterReturning/@AfterThrowing` 三个阶段收集以下字段并写入 `SysOperLog`：
- 请求元信息：IP、URL、HTTP 方法、耗时（毫秒）
- 操作主体：当前登录用户名、部门名称（从 SecurityContext 提取）
- 业务描述：`@Log.title()`、`businessType`、`operatorType`
- 参数快照：自动序列化请求参数（排除 `password` 等敏感字段及 `MultipartFile`、`HttpServletRequest` 等对象），最大长度限制为 2000 字符
- 响应快照：可选保存 JSON 响应体（同样受长度限制）
- 异常堆栈摘要：失败时截取异常消息前 2000 字符

所有入库操作通过 `AsyncManager.execute(AsyncFactory.recordOper(...))` 异步执行，避免阻塞主流程。

### 3.3 业务日志使用模式
- Service 层普遍采用 `private static final Logger log = LoggerFactory.getLogger(XxxServiceImpl.class);` 直接调用 `log.info/warn/error` 记录业务事件
- 统一通过 SLF4J 占位符传参，避免字符串拼接开销
- 未引入 Lombok `@Slf4j`，保持原生写法以兼容现有代码风格

## 4. 开发者应遵循的规则
1. **Controller 操作日志**：对涉及增删改查、导出、上传等关键接口方法添加 `@Log(title="...", businessType=..., operatorType=...)`，并在需要时设置 `isSaveRequestData=false` 或 `excludeParamNames` 过滤敏感参数。
2. **Service 日志**：使用 `LoggerFactory.getLogger(getClass())` 获取 logger，优先用 `info/warn/error` 表达业务语义，错误路径必须附带异常对象以便堆栈输出。
3. **敏感信息保护**：不要手动记录密码、token、完整请求体；如需记录请求参数，依赖 `@Log` 的自动脱敏与长度截断机制。
4. **日志级别规范**：
   - `info`：正常业务流程节点（如“外部书签导入完成”）
   - `warn`：可恢复异常或降级场景（如“未能获取当前用户信息，跳过记录下载流水”）
   - `error`：不可恢复异常，需附带异常对象
5. **性能注意**：避免在高频路径上打印超大对象；必要时先判断 `log.isDebugEnabled()` 再构造详细日志。
6. **日志轮转与留存**：生产环境日志目录 `/home/ruoyi/logs` 需确保 Java 进程有写权限，且不被外部工具锁定，否则 Logback 滚动重命名会失败。