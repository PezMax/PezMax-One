# 开发环境

各平台的构建工具链与本机现状。本机检查于 **2026-10-19**（Windows 11 26200）；工具链有变化时更新本表。

## 兄弟仓库

本仓按相对路径引用以下检出，它们必须与 `PezMax-One/` 并列放在同一目录下：

| 目录 | 用途 | 本机 |
|---|---|---|
| `../Kanesumi-sec-a` | Android 端 composite build | ✅ |
| `../Kanesumi-xaml` | Windows 端视觉底座（D3）：从 Ncrust 抽出，两仓都按相对路径引用 | ✅ 已建立（`git filter-repo` 导出，保留历史） |
| `../Ether/shared/kanesumi` | Linux 端本机开发时的 `[patch]` 目标（正式构建用 git rev） | ✅ |
| `../PezMax-Go` | 上游后端，契约来源（只读） | ✅ 基线 `061a888` |
| `../Ncrust` | 样板工程（Windows 端做法、CrateStack、spec 约定） | ✅ |

## Windows 端

| 需要 | 本机 |
|---|---|
| VS 2022 Build Tools + UWP 工作负载（MSBuild 17.14+，含 `Microsoft\WindowsXaml` targets） | ✅ MSBuild 17.14.51 |
| Windows SDK 10.0.22621（UAP） | ✅ 22621、26100 |
| .NET 9 SDK（Core 测试） | ✅ 9.0.316 |
| 开发者模式（本地 `Add-AppxPackage -Register`） | ✅ 已开启 |
| 自签代码签名证书（`.pfx`，不入库） | ⏳ 首次发布前生成，Subject 必须等于包 `Publisher` |

```powershell
$msbuild = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\MSBuild\Current\Bin\MSBuild.exe"
& $msbuild windows\PezMax.Windows.sln /t:Restore /p:Configuration=Release /p:Platform=x64
& $msbuild windows\PezMax.Windows.sln /p:Configuration=Release /p:Platform=x64
dotnet test windows\tests\PezMax.Core.Tests
```

`dotnet test` 在**受限沙箱**（agent 以受限令牌跑命令时）会崩在 testhost 启动阶段：

```
System.ComponentModel.Win32Exception (5): 拒绝访问
   at System.Diagnostics.Process.set_EnableRaisingEvents(...)
```

这不是测试本身的问题——vstest 的 testhost 要 `OpenProcess` 父进程句柄来挂退出回调，被沙箱拦了。
换普通终端跑即可；实在要在沙箱里验证，把测试产物目录里的程序集当成库、自己反射调用 `[Fact]` 方法。

### 免签名本地运行（验证界面）

还没有代码签名证书，`Add-AppxPackage -Path *.msix` 会被拒。可绕开签名做**松散文件注册**，
但**必须用 `AppPackages\..._Test\` 里 `.msix` 的内容**，不能注册 `bin\Release\AppxManifest.xml`：
后者指向 IL 布局，激活期直接崩（`0xe0434352`，没有任何托管日志）。

```powershell
$dest = "$env:TEMP\pezmax-run"
Expand-Archive windows\src\PezMax.App\AppPackages\PezMax.App_*_x64_Test\PezMax.App_*_x64.msix -DestinationPath $dest -Force
Add-AppxPackage -Register "$dest\AppxManifest.xml"
Start-Process "shell:AppsFolder\PezMax.One_0say9r0m9v83e!App"   # AUMID 用 Get-AppxPackage 查
```

每次改了代码要重新解包再注册（注册记录指向那份解包目录，不指向仓库）。验证完 `Remove-AppxPackage` 收拾干净。
截图不必上工具：`user32!PrintWindow` 对 UWP 的 `Windows.UI.Core.CoreWindow` 用 `PW_RENDERFULLCONTENT(2)`
能抓到位图，被遮挡也能抓。

## Linux 端

Kanesumi harness 只在 Linux（Wayland）上编译与运行。

| 需要 | 本机 |
|---|---|
| Rust stable（edition 2024） | ✅ cargo 1.95.0（Windows 主机，只能 `cargo test -p pezmax-core`） |
| Linux 主机 + Wayland 会话（Plasma 与 GNOME 各一，验证 L1） | ❌ 本机无 WSL；`~/.ssh/config` 中有 `ether`、`ether-arch`、`ether-debian`、`ether-deb6` |
| `.deb` 打包（`dpkg-deb` / `cargo-deb`） | 在 `ether-debian` 上做（待确认工具链） |
| `.pkg.tar.zst` 打包（`makepkg`） | 在 `ether-arch` 上做（待确认工具链） |
| aarch64 产物 | 交叉编译或 aarch64 构建机（待定） |
| pdfium 预编译库（x86_64 / aarch64） | 构建脚本下载，不入库 |

可选：本机装 WSL2，WSLg 自带 Wayland 合成器，可用于 `pezmax-app` 的日常调试（不替代 Plasma / GNOME 实机验证）。

## Android 端

| 需要 | 本机 |
|---|---|
| JDK 17（运行 Gradle；字节码目标 Java 11） | ✅ 17.0.12 |
| Android SDK，platform 36，build-tools 36 | ✅ `D:\Applications\AndroidSDK`（`android/local.properties` 写 `sdk.dir`，该文件不入库） |
| Gradle | 随 wrapper，与 Ncrust 同版本（9.3.1） |
| release 签名 `keystore.properties` | ⏳ 发布前准备，不入库 |

## 本机跑后端（两条路）

**快路：本地开发服务器**（不需要 Postgres / Redis / MinIO，只要 Go）：

```powershell
cd spec\tools\devserver
go run . -addr 127.0.0.1:9033
# 客户端「设置 → 高级 → 服务器地址」填 http://127.0.0.1:9033
# 种子账号 demo / demo1234（id 7）；也可以在客户端注册新号（验证码填任意四位数）
```

它按 `spec/api/endpoints.md` 的形状提供最小实现，并且**现场生成**一份 120 页 PDF
（`/datum/download/file?fileId=101&pages=120`，验 W2）和一份三章 EPUB
（`/datum/ebook/content?ebookId=202`，验 W3）。细节与已知差别见 `spec/tools/devserver/README.md`。
它**不是**后端：不鉴权、内存态、通知与收藏恒为空——契约核对到哪一步算哪一步。

### 用 UWP 客户端连本机服务端：先豁免回环

**这一步只有 UWP 需要，控制台不需要。** AppContainer 到**本机**地址的连接一律被挡——
不只是 `127.0.0.1`，填本机的局域网地址（如 `10.173.79.224:8080`）同样被挡，
因为报文在本机内部绕回来，还是回环。症状是登录页显示「验证码获取失败」，
而服务端日志里**一条请求都没有**（不是 404、不是 500，是根本没发出去），
异常是传输层的，所以走不到 `DatumException` 的文案分支。

```powershell
# 一次就够，重启后仍在；需要**管理员**终端（UAC 提权那条命令）
CheckNetIsolation LoopbackExempt -a -n=PezMax.One_0say9r0m9v83e
CheckNetIsolation LoopbackExempt -s          # 看它在不在名单里
```

包全名用 `Get-AppxPackage PezMax.One | Select-Object PackageFamilyName` 现取，重新注册后若
`PackageFamilyName` 变了要重加。豁免是**本地联调手段**，不进产品要求、不写进发布说明；
发布环境是真实后端地址，不涉及回环。

不需要改清单：`privateNetworkClientServer` 已在 `Package.appxmanifest` 里（W4 需要它连内网 10.x）。
把 devserver 绑到局域网地址并不会绕过上面那条限制，也不该这么做——它没有鉴权，
`spec/tools/devserver/README.md` 明确要求只绑 `127.0.0.1`。

不装 UWP 也能核对协议：`dotnet run --project windows\tools\apiwalk -- http://127.0.0.1:9033`
用 `PezMax.Core` 自己的协议代码走一遍注册到下载，不需要打包也不需要豁免。

**真路：跑 PezMax-Go 本体**（契约的最后一道核对）。条件见下表，注意缺 Redis 起不来：

| 需要 | 本机 | 说明 |
|---|---|---|
| PostgreSQL | ✅ 18，`postgresql-x64-18` 服务在 5432 | 建一个空库即可，schema 由后端启动时自建 |
| Redis | ❌ 没有运行中的实例 | 会话、文件树 / 排行缓存都用它；**缺它后端起不来** |
| 对象存储 | ✅ 可选 | 配 `KADMIN_FILE_LOCAL_ROOT` 走本地目录，不需要 MinIO |
| Go | ✅ 1.26.5 | `go run .` 直接跑 |

```powershell
psql -h 127.0.0.1 -U postgres -c "CREATE DATABASE kadmin;"
# 起 Redis（自己装 / 便携版 / 让负责人给测试服务器地址）
# 在 PezMax-Go/k_admin 下填 .env（DB、Redis、KADMIN_FILE_LOCAL_ROOT），然后：
go run .
pwsh -File ..\PezMax-One\spec\tools\probe.ps1 -BaseUrl http://127.0.0.1:9033
```

接上真实后端之后才有意义的三件事：W1（断网续传 / 每 IP 一连接）、W2（大 PDF）、W4（内网）。
会话类端点还要一个**只读测试账号**——注册一个即可，但别用真人的账号。

## 通用

| 需要 | 本机 |
|---|---|
| `gh`（发布 CrateStack 班次） | ✅ 2.96.0 |
| 推送权限：`PezMax/PezMax-One` | 待确认 |
