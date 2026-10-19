# 开发环境

各平台的构建工具链与本机现状。本机检查于 **2026-10-19**（Windows 11 26200）；工具链有变化时更新本表。

## 兄弟仓库

本仓按相对路径引用以下检出，它们必须与 `PezMax-One/` 并列放在同一目录下：

| 目录 | 用途 | 本机 |
|---|---|---|
| `../Kanesumi-sec-a` | Android 端 composite build | ✅ |
| `../Kanesumi-xaml` | Windows 端视觉底座（D3，待从 Ncrust 抽出） | ⏳ 尚未建立；当前在 `../Ncrust/windows/src/Kanesumi.Xaml` |
| `../Ether/shared/kanesumi` | Linux 端本机开发时的 `[patch]` 目标（正式构建用 git rev） | ✅ |
| `../PezMax-Go` | 上游后端，契约来源（只读） | ✅ 基线 `f79a4e0` |
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

## 通用

| 需要 | 本机 |
|---|---|
| `gh`（发布 CrateStack 班次） | ✅ 2.96.0 |
| 推送权限：`PezMax/PezMax-One` | 待确认 |
