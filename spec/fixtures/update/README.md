# `fixtures/update/` —— 应用内更新检查（GitHub Releases）

发布走 CrateStack 滚动班次（`docs/RELEASES.md`），所以**不能看 Latest 徽章，也不能依赖数组顺序**，
只能按资产名扫。这份夹具是三端更新检查的唯一判据：同一批 release、同一个当前版本，
三端必须挑出同一个结果。

规则与 Ncrust `spec/fixtures/update/` 同源，差异只有三处：资产前缀 `pezmax-one-`、
多了 `.pkg.tar.zst` 这种多段扩展名、`amd64` / `arm64` **不算** `x86_64` / `aarch64`。

## 形状

```json
{
  "id": "update.train.higher-version",
  "description": "这条用例在防什么（一句话）",
  "source": "docs/RELEASES.md「应用内更新」+ 资产命名一节",
  "input": {
    "currentVersion": "1.1.0",
    "platform": "windows",
    "arch": "x86_64",
    "mirrorBase": null,
    "releases": [ /* GitHub Releases API 响应，只保留用得着的字段 */ ]
  },
  "expect": {
    "version": "1.2.0",
    "releaseTag": "CrateStack-20261102A",
    "releasePageUrl": "https://github.com/PezMax/PezMax-One/releases/tag/CrateStack-20261102A",
    "assets": { "appinstaller": "https://…", "msix": "https://…", "cer": null }
  }
}
```

- `expect: null` = 无更新。
- `expect.version` 是**归一后**的字符串：最多 4 段、去尾部多余的 0、至少保留 3 段
  （`v1.2` → `1.2.0`，`1.2.0.0` → `1.2.0`，`1.2.0.5` → `1.2.0.5`）。界面上只显示前 3 段。
- `expect.assets` 的键是**扩展名**（`appinstaller` / `msix` / `cer` / `deb` / `pkg.tar.zst` / `apk`），
  值是命中的下载地址；该版本没有这类资产时为 `null`（**不得**借用别的版本的资产补齐）。
- `input.mirrorBase` 为 `null` 表示镜像关闭（`ARCHITECTURE.md` D12 的默认值）。

## 规则

1. **资产命名**：`pezmax-one-[平台]-[架构]-[版本].[扩展名]`。
   平台段与架构段必须**同时**逐字符相等（`windows` + `x86_64`、`linux` + `x86_64|aarch64`、
   `android` + `universal`）；跨平台、跨架构的资产一律不算。
2. **扩展名**：只认该平台可安装的扩展名 —— Windows `appinstaller` / `msix` / `cer`，
   Linux `deb` / `pkg.tar.zst`，Android `apk`。签名与校验文件（`.asc`、`.sha256`、`.sig`）不算资产。
   **按已知扩展名做最长后缀匹配**：`…-1.2.0.pkg.tar.zst` 的版本是 `1.2.0`，不是 `1.2.0.pkg.tar`。
3. **版本段**：扩展名之前的最后一段；最多 4 段，缺段补 0，允许前缀 `v`。
   解析不出来（如 `…-1.4.0.apk.sha256` 把扩展名当版本段）→ 忽略该资产，**绝不猜**。
4. **旧命名一律忽略**：`pezmax-one-1.0.0-windows-x86_64.msi`（版本在平台前的 1.0.0 写法）、
   `app-release.apk`、`manifest.json` 这类不合规范的资产不参与挑选。
5. `draft` 与 `prerelease` 的 release **整条忽略**。
6. **班次内取最高**：一个 release 里有多个合规资产时，取版本最高者；同版本的多类资产
   （`.appinstaller` / `.msix` / `.cer`）必须来自**同一个最高版本**。
7. **跨班次取最高**：遍历所有可见 release，取**严格高于**当前版本的最高者。同版本不算更新
   （原地换包的修复靠升版本传递，见 `docs/RELEASES.md`「原地换包」）。
8. **不依赖** `latest`、`created_at` 顺序、`assets` 排列顺序。

## 镜像前缀（D12）

`mirrorBase` 非空时，所有命中的下载地址都换成 `mirrorBase + 原 URL 的 path 与 query`
（去掉原 scheme 与 host）。镜像只改「从哪儿下」，不改「选中哪个版本」——挑选永远在原始 release 上做。
默认关闭；镜像挂掉时用户关掉开关即可，不需要回退逻辑。

## 与版本号的唯一来源

`currentVersion` 由各端从自己的唯一来源读（`docs/RELEASES.md`「版本号」）：
Windows 的 `Package.Id.Version` 前三段、Linux 的 `CARGO_PKG_VERSION`、Android 的 `versionName`。
**不要在代码里写版本常量**，也不要把夹具里的版本号当真版本号——它们只是用例。

## 不覆盖的

- 更新检查的时机与频率（24 小时、启动后延迟）：属于各端调度，不进夹具。
- 各平台的安装动作（`.appinstaller` 交给 App Installer、`pkexec` 装 deb、`FileProvider` 装 apk）：
  平台差异，写在 `docs/RELEASES.md`。
