# AGENTS.md —— PezMax-One for Windows（C# + UWP + WinUI 2）

本文件是 `windows/` 目录的权威指南。仓库级约定见仓根 `AGENTS.md`，跨端架构见 `docs/ARCHITECTURE.md`。

**状态**（2026-10-19）：规划稿，尚无代码。下一步见 `docs/PLAN.md`「下一轮」。

> 样板是兄弟仓库 Ncrust 的 Windows 端（`../../Ncrust/windows/AGENTS.md`）：**凡本文没写的工程细节，按 Ncrust 的做法办**，
> 尤其是它的「已知的坑」一节——那 40 条大多来自 arc-deck 与 Ncrust 的本机实测，在这里同样成立，不再复抄。

## 立项决策

| 决策 | 结论 | 原因 |
|---|---|---|
| UI 框架 | UWP + WinUI 2（`Microsoft.UI.Xaml` 2.8.x） | 平台免费提供文本、IME、滚动、虚拟化、无障碍；arc-deck / Ncrust 已验证 |
| WinUI 3 | 不考虑，不引入，不提议迁移 | 负责人决定 |
| 运行时 | .NET Native（`UseDotNetNativeToolchain=true`），C# `LangVersion` 10，只建 `Release|x64` | Ncrust 实测：Debug 的 .NET Native 依赖未预装的调试运行时 |
| 业务核心 | `PezMax.Core`：netstandard2.0，**不依赖 WinRT 与 UI**；`dotnet test` 秒级跑 spec 夹具 | 同 Ncrust.Core |
| JSON | 读用零反射只读 DOM（同 Ncrust `JsonValue` 思路），写用手写序列化；不引入 System.Text.Json / Newtonsoft | .NET Native 对反射敏感 |
| 绑定 | 只用 `x:Bind`；`INotifyPropertyChanged` 手写；无 DI / MVVM 框架，单例服务定位 | 依赖越少，.NET Native 问题越少 |
| 视觉 | Kanesumi.Xaml（抽成独立仓后按相对路径引用，`ARCHITECTURE.md` D3）+ 平台控件优先；`XamlControlsResources ControlsResourcesVersion="Version1"` | Windows 10 直角样式；Version2 的圆角与 Kanesumi 冲突 |
| 按钮 / 图标 | 微软原生样式：主操作 `AccentButtonStyle`、图标按钮 `AppBarButton`；图标 Segoe MDL2 Assets | Ncrust 负责人 2026-09-27 定下的做法 |
| 强调色 | 跟随系统强调色，读不到时回落钴蓝 `#3B82F6`（D11） | 与 Ncrust 一致 |
| 底色 | 页面背景**显式设置**为 token，不依赖系统材质 | arc-deck 实测窗口背景会被混成中灰 |

## 解决方案结构

```
windows/
├── AGENTS.md
├── PezMax.Windows.sln            # 手写（dotnet sln 加不了 UWP 项目），仅 Release|x64
├── src/
│   ├── PezMax.Core/              # netstandard2.0
│   └── PezMax.App/               # UWP 应用
├── tests/PezMax.Core.Tests/      # net9.0 + xUnit，读 ../../spec/
└── tools/{shot,release,icons}/
```

依赖方向：`PezMax.App → Kanesumi.Xaml`、`PezMax.App → PezMax.Core ← PezMax.Core.Tests → spec/`，不得反向。

### PezMax.Core

| 模块 | 内容 |
|---|---|
| `Net/` | `DatumHttp`：Base URL、Bearer 注入、信封解析、HTTP 状态 → 错误类型、401 事件；`HttpClient` 设 `UseCookies = false` |
| `Json/` | 只读 JSON DOM + 写入器 |
| `Api/` | 按域划分的端点封装与响应模型：`Auth`、`Profile`、`Files`、`Ebooks`、`Bookmarks`、`Favorites`、`Downloads`、`Reports`、`Community`、`Notifications` |
| `State/` | 哈希缓存 `StateCache<T>`（文件树、排行）、分页聚合 `PageFetcher` |
| `Files/` | 文件树模型（学科 / 学校 / 类型 / 年份）、筛选与排序 |
| `Transfer/` | 下载状态机（续传、ETag、取消、串行队列）、上传校验（白名单、大小、编码要求）；不含 I/O 实现 |
| `Urls/` | 文件 URL 归一（过渡规则，见 `spec/fixtures/url`） |
| `Notify/` | 通知可见性（发布窗口、维护 / 故障时间段）、本地已读集合 |
| `Update/` | CrateStack 资产解析与版本比较 |
| `Validation/` | 注册 / 改密 / 密保表单规则 |
| `Platform/` | 接口：`ISettingsStore`、`IFileStore`、`ICredentialStore`、`IClock` |

规则：Core 里的 `await` 一律 `ConfigureAwait(false)`；由 App 负责切回 Dispatcher。

### PezMax.App

| 目录 | 内容 |
|---|---|
| `Shell/` | `ShellPage`：`NavigationView` + 内容 Frame + 自定义标题栏 + 滚动提醒条 + 下载浮层 + 登录层；全局快捷键；`AppTheme` |
| `Pages/` | 首页、资料（树 + 列表 + 搜索）、电子书、书签、收藏、社区（排行）、我的上传 / 下载 / 举报、个人中心、设置 |
| `Reader/` | `PdfReader`（网格 / 连续两种视图）、`ImageViewer`、`TextViewer`、`MarkdownViewer`、`EpubReader` |
| `Transfer/` | `DownloadService`（`BackgroundDownloader`）、`UploadService`（`HttpMultipartFormDataContent` + 进度） |
| `Login/` | 登录、注册（三步：账号 → 密保 → 完成）、找回密码（两步）独立覆盖层 |
| `Platform/` | Core 平台接口实现：`LocalSettings`、`LocalFolder`、`PasswordVault` |
| `Update/` | `UpdateService`：下载 `.appinstaller` 交给 App Installer |
| `I18n/` | `Strings` 类；界面文字一律不许写死。M1 只做 zh-CN |

## 外壳与导航

- 标准汉堡菜单：WinUI 2 `NavigationView`，≥ 1008 展开、600 ~ 1008 紧凑、< 600 最小。
- 面板：顶部搜索框（Ctrl+F）→ 首页 → 分组「资料库」：资料、电子书、书签 → 分组「我的」：收藏、下载、上传、举报 →
  社区；底部账户项（未登录显示「登录」，登录后头像 + 昵称进个人中心）；最下方 NavigationView 自带「设置」。
- 内容延伸进标题栏（`ExtendViewIntoTitleBar`），覆盖层让出标题栏高度（Ncrust `ApplyTitleBarHeight` 同款坑）。
- 滚动提醒（`scroll` 通知）是外壳顶部一条可关闭的窄条；弹窗通知（`popup`）在首次进入首页时用 `ContentDialog` 逐条显示，
  铃铛入口在标题栏右侧，未读计数用 `InfoBadge`。
- 会话失效（401）：关闭所有覆盖层，打开登录层，登录成功后回到原页面。

## 列表与浏览

- 资料页左侧 `TreeView`（文件树，来自哈希缓存）+ 右侧 `ListView` 详情列（文件名、科目、学校、年份、大小、格式），
  窄窗口退化为面包屑（`BreadcrumbBar`）+ 单列。
- 列表全部用平台 `ListView` / `GridView` 的虚拟化，**不重写 `ListViewItem` 模板的交互部分**，只换视觉资源。
- 详情：右侧 `SplitView` 面板或独立详情页（宽窗口面板、窄窗口页面），含预览、下载、收藏、举报、举报时间线。
- 右键菜单 `MenuFlyout`：打开、下载、收藏 / 取消收藏、复制链接、举报；属主另有编辑、删除。

## 阅读器

| 格式 | 实现 | 备注 |
|---|---|---|
| PDF | `Windows.Data.Pdf.PdfDocument.LoadFromStreamAsync` → `PdfPage.RenderToStreamAsync`（按显示尺寸 × DPI 渲染）→ `BitmapImage` | 两种视图沿用 egui 版：**网格**（缩略图，点击进连续视图该页）、**连续**（纵向滚动 + 左侧概览）。虚拟化：只渲染可视区 ± 2 页，限 3 并发；缩放用 Composition `Scale` 先行、松手后按新尺寸重渲染。页缓存写 `LocalCache/pdf/{fileId}/p{idx}_s{scale}.png` |
| 图片 | `Image` + `ScrollViewer` 缩放 | 按显示尺寸解码（`DecodePixelWidth`） |
| TXT | `RichTextBlock`，大文件分段加载 | 后端保证 UTF-8 |
| Markdown | M0 二选一：自研块级渲染到 `RichTextBlock`，或 WebView2 + 本地打包的渲染页 | 与 EPUB 同用 WebView2 时，两者共用一个阅读器宿主 |
| EPUB | WebView2（`Microsoft.UI.Xaml.Controls.WebView2`）+ 本地打包的阅读器页面（epub.js / foliate-js 二选一，查许可证）；`SetVirtualHostNameToFolderMapping` 映射本地资源，书本内容由应用下载后经虚拟主机提供 | M0 必须验证：UWP 中 WebView2 虚拟主机映射、离线运行、翻页与目录、CJK 排版 |
| MOBI / AZW3 | 下载后 `Launcher.LaunchFileAsync` 交给默认程序 | 无默认程序时提示并打开所在文件夹 |

预览一律先**下载到 `LocalCache`**（电子书走 `/datum/ebook/content`，资料走下载流）再交给阅读器；阅读器不直接拿 URL。

## 下载

- `Windows.Networking.BackgroundTransfer.BackgroundDownloader`：应用挂起后继续、系统级续传；`SetRequestHeader("Authorization", …)`。
  M0 验证：带 Bearer 的 Range 续传、ETag 变化后的行为、与 edge 每 IP 1 连接限制的配合。
- 不满足时回退到自管 `HttpClient` 流式下载 + `.part` 文件 + `Range`（egui 版的做法），状态机仍在 Core。
- 下载目录：默认 `下载\PezMax`（`DownloadsFolder`）；用户可用 `FolderPicker` 改目录，授权存进 `FutureAccessList`。
  **不申请 `broadFileSystemAccess`**（受限能力，且 UWP 设置里默认关闭，用户体验差）。
- 队列默认串行；下载浮层显示进度、速度、取消；完成后系统 Toast（可在设置关闭，对应「静默下载」）。
- 本地记录写 `LocalFolder/downloads.json`；服务端记录由下载流端点自动落库，「我的下载」页读服务端列表并与本地记录按 `fileId` 合并（本地有路径则可「打开 / 在文件夹中显示」）。

## 上传

- `FileOpenPicker`（多选）与 `FolderPicker`（文件夹上传，递归枚举）；上传前由 Core 校验扩展名、大小、编码要求，给出逐个文件的结果。
- `Windows.Web.Http.HttpClient` + `HttpMultipartFormDataContent`，`IProgress<HttpProgress>` 驱动进度；串行上传，单个失败不影响其余。
- 元信息（科目、学校、类型、年份）表单：科目 / 学校来自 `/datum/file/subjects`、`/schools`，学校用 `AutoSuggestBox` + `/schools/check`。
- 上传成功提示「已提交，审核通过后公开」。

## 设置

按 Windows 设置应用的分组，用平台控件：

| 分组 | 内容 |
|---|---|
| 账户 | 头像、用户名、改密码、密保问题、登出 |
| 外观 | 跟随系统 / 深色 / 浅色；强调色跟随系统（色块 + 打开 `ms-settings:colors`） |
| 下载 | 下载目录、完成后通知、下载完成后自动打开 |
| 阅读 | PDF 默认视图（网格 / 连续）、默认缩放 |
| 启动 | 开机启动（`StartupTask` 扩展，需在清单声明） |
| 存储与缓存 | 占用大小；清除缓存（不动登录态与下载记录） |
| 高级 | 服务器地址（D13），改后清会话 |
| 关于 | 版本号（`Package.Current.Id.Version`）、检查更新、GitHub、开源许可 |

## 快捷键

| 键 | 作用 |
|---|---|
| Ctrl+F | 聚焦搜索框（紧凑 / 最小模式先展开面板） |
| Alt+← / 鼠标侧键 / Backspace（焦点不在输入框） | 后退 |
| Ctrl+D | 下载当前文件 |
| Ctrl+ = / Ctrl+ - / Ctrl+0 | 阅读器缩放 / 复位 |
| F11 | 阅读器全屏 |
| Esc | 逐层退出：全屏 → 阅读器 → 覆盖层 |

挂在 `ShellPage.KeyboardAccelerators`，`KeyboardAcceleratorPlacementMode = Hidden`。

## 能力声明

`internetClient`；下载目录用 `DownloadsFolder` 与 Picker，不需要库能力。若后端部署在内网（如 `10.*`），
需要 `privateNetworkClientServer`——**这是 UWP 特有的坑**：AppContainer 默认不能访问私有网络地址，测试服务器在内网时首先检查它。

## 与 Electron 官方端的映射

| Electron（`PezMax-Desktop`） | Windows |
|---|---|
| 主进程下载 + `.part` 续传（`src/main/index.js`） | `BackgroundDownloader`，回退 `HttpClient` + `.part` |
| sql.js 本地库（下载记录、`notice_reads`） | `LocalFolder/*.json` |
| `ptmj-settings.json` | `LocalSettings` |
| `normalizeFileUrl` | Core `Urls/`（过渡规则） |
| `fetchAllPages` | Core `PageFetcher` |
| `SessionExpiredModal` | 401 → 登录层 |
| electron-updater + `latest.yml` | CrateStack + `.appinstaller` |
| 主题及背景自定义 | 明暗三模式 + 系统强调色；不做背景图（Kanesumi 纯色原则） |

## M0 验证项（Windows）

| # | 验证项 | 通过标准 |
|---|---|---|
| W1 | `BackgroundDownloader` + Bearer + Range | 断网恢复后续传成功；应用挂起后继续；经 edge 时不被每 IP 连接限制拒绝 |
| W2 | `Windows.Data.Pdf` 大文件 | 100 页以上试卷 PDF：首屏 < 1 s，连续视图滚动无白屏、内存稳定 |
| W3 | WebView2 EPUB 阅读器 | 离线打开 EPUB，翻页 / 目录 / CJK 排版正常；Release + .NET Native 下运行 |
| W4 | 内网后端访问 | 带 `privateNetworkClientServer` 时能连内网测试服务器 |
| W5 | Kanesumi.Xaml 引用 | 抽仓后按相对路径的 ProjectReference 能在 MSBuild Release 下编过 |
