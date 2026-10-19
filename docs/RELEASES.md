# PezMax-One 发布索引（CrateStack）

PezMax-One 在同一个仓库（`PezMax/PezMax-One`）里发布多个平台，各平台**版本线独立**。发布走 Ncrust 同款的
**CrateStack 滚动多平台班次（release train）**：一个 Release 装载当日各平台的最新版资产，tag 形如
`CrateStack-YYYYMMDD<序号>`，GitHub 的 Latest 徽章指向当前班次。

> 某平台的最新版一律以本表为准，**不要看 Latest 徽章判断**——徽章只指向最近一班。
> 本文是规范；各平台的打包与签名细节见 `<平台>/AGENTS.md`。

## 各平台最新版

| 平台 | 版本 | 所在班次 | 下载 | 日期 |
|---|---|---|---|---|
| Windows（UWP，x86_64） | — | — | — | 计划中（M1） |
| Linux（x86_64 / aarch64） | — | — | — | 计划中 |
| Android（通用包） | — | — | — | 计划中 |
| 旧版 egui（Windows / Linux / macOS） | 1.0.0 | `v1.0.0`（旧式 tag） | [Release 页](https://github.com/PezMax/PezMax-One/releases/tag/v1.0.0) | 2026-07-27，**停止维护**，对接的旧后端契约已废弃 |

发布时更新本表。

## 班次

- tag：`CrateStack-YYYYMMDD<A..Z>`，同日多班依次编号；Release 标题写日期（如「2026-11-02 更新」），正文按平台分节。
- 每班装载**当时各平台的最新版资产**；某平台本班没有新版本时，**沿用上一班的同名资产**一并装入，
  保证任何一班都能装到任何平台的最新版。
- 每班附 `manifest.json`（机器可读，程序化检查以它为准），形状与 Ncrust 一致：

```json
{
  "crate": "CrateStack-20261102A",
  "date": "2026-11-02",
  "platforms": {
    "windows": {
      "version": "1.1.0",
      "assets": [
        "pezmax-one-windows-x86_64-1.1.0.msix",
        "pezmax-one-windows-x86_64-1.1.0.appinstaller",
        "pezmax-one-windows-x86_64-1.1.0.cer"
      ]
    },
    "linux": {
      "version": "1.1.0",
      "assets": [
        "pezmax-one-linux-x86_64-1.1.0.deb",
        "pezmax-one-linux-x86_64-1.1.0.pkg.tar.zst",
        "pezmax-one-linux-aarch64-1.1.0.deb",
        "pezmax-one-linux-aarch64-1.1.0.pkg.tar.zst"
      ]
    }
  }
}
```

- 创建时一律 `--draft --latest=false`，负责人冒烟测试后手动发布并设为 Latest。

## 资产命名

`pezmax-one-[平台]-[架构]-[版本].[扩展名]`。

| 平台段 | 架构段 | 可安装扩展名 | 附属文件 |
|---|---|---|---|
| `windows` | `x86_64`（M2 起加 `aarch64`） | `.appinstaller`（主入口）、`.msix` | `.cer`（自签证书公钥） |
| `linux` | `x86_64`、`aarch64` | `.deb`、`.pkg.tar.zst` | —— |
| `android` | `universal` | `.apk` | —— |

- 架构段**统一写 `x86_64` / `aarch64`**，不写 `amd64` / `arm64`（1.0.0 的 deb 混用了两种写法）。
  deb 包内部的 `Architecture:` 字段照 Debian 惯例仍是 `amd64` / `arm64`，与文件名无关。
- 版本段：前缀之后到**最后一个点**之间（`.pkg.tar.zst` 这类多段扩展名按整体扩展名剥离，规则见
  `spec/fixtures/update/README.md`）。最多 4 段，缺段补 0，允许前缀 `v`；解析失败的资产忽略，绝不猜。
- 1.0.0 的旧资产（`pezmax-one-1.0.0-windows-x86_64.msi` 这种版本在前的写法）**不匹配新规则**，新客户端一律忽略。

## 版本号

各平台各自维护，唯一来源：

| 平台 | 唯一来源 | 显示位置读取 |
|---|---|---|
| Windows | `windows/src/PezMax.App/Package.appxmanifest` 的 `Identity Version`（`X.Y.Z.0`，第 4 段为开发构建号） | `Package.Current.Id.Version` |
| Linux | `linux/crates/pezmax-app/Cargo.toml` 的 `version` | `env!("CARGO_PKG_VERSION")` |
| Android | `android/app/build.gradle.kts` 的 `versionName` / `versionCode` | `BuildConfig.VERSION_NAME` |

**不要在任何地方写死版本常量。** 新客户端的版本线从 **`1.1.0`** 起（高于 egui 1.0.0，避免同号歧义）。

## 原地换包与应用内更新

- 修复类改动允许不升版本、在班次里原地替换同名资产，但必须在 Release 正文注明替换时间与原因，并同步本表。
- **原地换包不会被应用内更新发现**：应用内检查只认**严格高于当前版本**的版本。要让已安装用户收到修复，
  必须把版本号与资产名的版本段一起递增。该契约由 `spec/fixtures/update/` 的 `update.train.same-version` 锁死，
  不要把检查改成支持同版本更新。

## 应用内更新（三端同一规则）

1. 请求 `GET https://api.github.com/repos/PezMax/PezMax-One/releases`（未认证 60 次 / 小时 / IP，够用）；
   **忽略 draft 与 prerelease**；不依赖 Latest 徽章，也不依赖数组顺序。
2. 在所有可见 release 里按资产名找本平台、本架构、可安装扩展名的资产，取**严格大于当前版本**的最高者。
3. 检查时机：启动后首屏渲染完成再查，**每 24 小时最多一次**（Windows 可沿用 Ncrust 的 6 小时缓存，统一前以 spec 为准）；
   设置 → 关于里可手动检查。
4. 提示方式：不弹模态框、不打断阅读 / 下载；在导航或设置入口挂小圆点 + 一行提示，「稍后」后该版本不再提示。
5. 安装：

| 平台 | 方式 |
|---|---|
| Windows | 下载 `.appinstaller` 到临时目录，`Launcher.LaunchFileAsync` 交给系统 App Installer；后台自动更新由 `.appinstaller` 的 `<UpdateSettings><OnLaunch/>` 承担（UWP 不能静默自装） |
| Linux | 下载对应包到缓存目录，按发行版调用 `pkexec` 安装（`apt` / `pacman -U`）或打开所在目录让用户安装；无法判断包管理器时只给出 Release 页链接 |
| Android | 应用内断点下载 → 校验签名与当前应用一致、`versionCode` 更大 → `FileProvider` 交给系统安装器（Ncrust `update/` 同款） |

6. 下载源：GitHub 为唯一真源；设置里可开启镜像前缀（默认关闭，`ARCHITECTURE.md` D12），拼接规则见 `spec/fixtures/update/`。

## Windows 签名与包身份

- 发三样：签名 MSIX、`.appinstaller`、公开证书 `.cer`。**请用 `.appinstaller` 安装**，直接装 `.msix` 不会自动更新。
- 首次安装需把 `.cer` 导入「受信任人」（`windows/tools/release/install-cert.ps1`）。换成正规代码签名证书后此步可省。
- 包身份（`Identity Name` + `Publisher`）一经发布**永不改变**，否则老版本无法升级。Publisher 取值待定
  （Ncrust 用 `CN=TakahashiRinta`；PezMax 是组织项目，可能需要组织名），**首次发布前必须定下来**。
- `.pfx` 私钥不入库（`.gitignore` 排除 `*.pfx` / `*.pvk`），私钥与口令存密码管理器 / CI secret。

## 从 egui 1.0.0 迁移

egui 1.0.0 对接的是 Java 旧后端契约，后端切到 PezMax-Go 后它本身就无法继续工作，因此**不做桥接版本**：

- 1.0.0 的更新器请求 `/releases/latest` 并在 Windows 上兜底找任意 `.msi`。第一班 CrateStack 设为 Latest 后，
  1.0.0 Windows 用户点「检查更新」会看到「release 中没有 Windows(.msi) 安装包」。这是预期行为，在 README、
  Release 正文与上游通知（类型 1「版本更新」，需上游运营配合）里说明：**卸载旧版，从 Release 页安装新版**。
- Linux 1.0.0 用户同理，新包名与旧包名一致（`pezmax-one`）则可直接覆盖安装；包名若变化需先卸载。
- macOS 1.0.0 没有后继版本，Release 正文写明暂停支持。
- 旧的 `v1.0.0` tag 与资产保留，不删除。
