# `uiwalk/` —— 用 UI Automation 真点一遍客户端

给**打包后的 UWP 客户端**用：`PezMax.Core` 的夹具管判定，`tools/apiwalk` 管协议，
这里管「点了有没有反应」。2026-11-07 就是靠它发现文件树点了不响应、详情栏被压扁这类
夹具测不出来的缺陷。

## 前提

1. 客户端已装好（松散注册也行，见 `docs/ENVIRONMENT.md`）。
2. 服务端在本机跑着，且包已加**环回豁免**——不加的话界面只显示「验证码获取失败」，
   而服务端日志里一条请求都没有。命令也写在 `docs/ENVIRONMENT.md`。
3. 应用窗口标题就是 `PezMax One`；改了显示名要同步 `uiwalk.ps1` 里的 `$AppWindowName`。

## 用法

```powershell
. .\windows\tools\uiwalk\uiwalk.ps1      # 点号加载，函数进当前会话

Dump-Ui                                   # 先看看现在有哪些元素、叫什么
Set-Text -Name '用户名' -Text 'demo'
Set-Text -Name '密码'   -Text 'demo1234'
Set-Text -Name '验证码' -Text '1234'      # devserver 不校验
Click-El -Name '登录'
Shot -Path "$env:TEMP\home.png"

Select-Item -Name '资料'                  # 左侧导航是 ListItem
Click-Real  -Name '化学' -Type TreeItem   # 树节点必须真点击：ItemInvoked 不认 UIA 的 Select
```

| 函数 | 用在什么上 |
|---|---|
| `Dump-Ui` | 界面里有什么（类型 / 名字 / 当前值），写脚本前先跑它 |
| `Set-Text` | `TextBox` / `PasswordBox`（按 Header 找） |
| `Click-El` | `Button` / `Hyperlink`（走 `InvokePattern`） |
| `Select-Item` | `ListItem`（导航项、列表行） |
| `Expand-Tree` | `TreeItem` 的展开箭头 |
| `Click-Real` | 需要 `ItemInvoked` 的东西——`TreeView` 的节点必须用这个 |
| `Shot` | 截图到指定路径（会先把它置顶） |

## 两条容易踩的

- **同名元素**：覆盖层（注册 / 找回密码）盖住登录页时，两层都在 UIA 树里，`用户名` 有两个。
  `Find-El` 先排除屏幕外的、再取最后一个；自己写脚本时别用 `FindFirst`。
- **`Shot` 抓的是屏幕**：窗口被别的窗口盖住就抓到别的。`Shot` 里已经先 `SetWindowPos` 置顶，
  自己另外截图时记得也这么做。

## 它不做什么

它不建基线、不做断言、不进 CI——它是**放大镜**，不是测试。看见不对的地方，
要么在 `PezMax.Core` 里补一条夹具，要么在这里留一张截图，然后人去改。
