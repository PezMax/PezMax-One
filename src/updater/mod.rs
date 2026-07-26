//! 三平台安装器 · 下载完成后由此接管：
//!   1. 生成 helper 脚本到系统临时目录
//!   2. 用平台 shell 拉起 helper（父进程 detached，与 PezMax 生命周期解耦）
//!   3. 调用方立刻 `std::process::exit(0)` 让 helper 接管安装 + 重启
//!
//! 每个平台的 helper 都会 sleep 一小段让 PezMax 完全退出，然后：
//!   - 跑安装器（弹权限提示由系统负责）
//!   - 重新拉起 PezMax
//!   - 删掉安装包 + 自身
//!
//! Linux 优先 `pkexec`，缺失时降级到 `sudo` + 终端模拟器（考虑到用户排错能力弱，
//! 全靠日志和 stderr 报错，不使用静默失败）。

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// 由 UI 层触发：安装 `installer_path` 指向的资产，然后重启 PezMax。
/// 成功返回 Ok 后调用方应立刻退出进程。
pub fn install_and_restart(installer_path: &Path) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        windows::install_and_restart(installer_path)
    }
    #[cfg(target_os = "macos")]
    {
        macos::install_and_restart(installer_path)
    }
    #[cfg(target_os = "linux")]
    {
        linux::install_and_restart(installer_path)
    }
}

/// 当前进程可执行路径（重启用）。失败时给一个合理的回退。
fn current_exe_string() -> String {
    std::env::current_exe()
        .ok()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "pezmax-one".to_string())
}

// ────────────────────────────────────────────────────────────────────────
// Windows
// ────────────────────────────────────────────────────────────────────────
#[cfg(target_os = "windows")]
mod windows {
    use super::*;

    pub fn install_and_restart(msi: &Path) -> Result<()> {
        let msi_str = msi.to_string_lossy().to_string();
        let exe = super::current_exe_string();

        // 用 cmd 脚本避免 PowerShell 执行策略问题
        // /passive: 显示进度条不询问输入
        // /norestart: 阻止 MSI 自行重启系统
        let script_path = std::env::temp_dir().join("pezmax-updater.cmd");
        let script = format!(
            "@echo off\r\n\
             timeout /t 2 /nobreak >nul\r\n\
             msiexec /i \"{msi}\" /passive /norestart\r\n\
             start \"\" \"{exe}\"\r\n\
             del \"{msi}\" >nul 2>&1\r\n\
             (goto) 2>nul & del \"%~f0\"\r\n",
            msi = msi_str,
            exe = exe,
        );
        std::fs::write(&script_path, script)
            .with_context(|| format!("写入 helper 脚本失败: {}", script_path.display()))?;

        // 拉起 cmd 执行脚本，detach（不能用 wait/output，那会 block）
        // /c 执行完就退出；用 start /b 避免弹出窗口
        Command::new("cmd")
            .args(["/c", "start", "/b", "", script_path.to_str().unwrap()])
            .spawn()
            .context("启动 helper 脚本失败")?;

        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────
// macOS
// ────────────────────────────────────────────────────────────────────────
#[cfg(target_os = "macos")]
mod macos {
    use super::*;

    pub fn install_and_restart(dmg: &Path) -> Result<()> {
        let dmg_str = dmg.to_string_lossy().to_string();
        // 挂载点用固定名，卸载时用回同名
        let mount = "/Volumes/PezMax One Update";
        // 默认安装到 /Applications/PezMax One.app（build-macos.sh 的产物）
        let app_target = "/Applications/PezMax One.app";
        let exe_open = "PezMax One"; // open -a 匹配 CFBundleName

        let script_path = std::env::temp_dir().join("pezmax-updater.sh");
        // 依赖：hdiutil / ditto / open 都是 macOS 内置
        // -nobrowse: 挂载点不出现在 Finder 侧栏
        // ditto 会保留权限 / 扩展属性 / code signature
        let script = format!(
            r#"#!/bin/bash
set -e
sleep 2
MOUNT="{mount}"
DMG="{dmg}"
APP_DEST="{app_dest}"

# 若上次残留就先卸掉
hdiutil detach -quiet "$MOUNT" 2>/dev/null || true

hdiutil attach -nobrowse -quiet -mountpoint "$MOUNT" "$DMG"

# 找 dmg 内的 .app（第一个）
APP_SRC=$(ls -d "$MOUNT"/*.app 2>/dev/null | head -n 1)
if [ -z "$APP_SRC" ]; then
  echo "[updater] dmg 里没找到 .app" >&2
  hdiutil detach -quiet "$MOUNT" || true
  exit 1
fi

rm -rf "$APP_DEST"
ditto "$APP_SRC" "$APP_DEST"

hdiutil detach -quiet "$MOUNT" || true
rm -f "$DMG"

open -a "{exe_open}"

rm -f "$0"
"#,
            mount = mount,
            dmg = dmg_str,
            app_dest = app_target,
            exe_open = exe_open,
        );
        std::fs::write(&script_path, script)
            .with_context(|| format!("写入 helper 脚本失败: {}", script_path.display()))?;

        // chmod +x
        use std::os::unix::fs::PermissionsExt;
        let mut perm = std::fs::metadata(&script_path)?.permissions();
        perm.set_mode(0o755);
        std::fs::set_permissions(&script_path, perm)?;

        // detach 拉起：nohup 避免父进程退出时 SIGHUP；stdin/stdout/stderr 重定向到 /dev/null
        Command::new("bash")
            .arg("-c")
            .arg(format!(
                "nohup '{}' >/tmp/pezmax-updater.log 2>&1 &",
                script_path.display()
            ))
            .spawn()
            .context("启动 helper 脚本失败")?;

        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────
// Linux
// ────────────────────────────────────────────────────────────────────────
#[cfg(target_os = "linux")]
mod linux {
    use super::*;

    pub fn install_and_restart(pkg: &Path) -> Result<()> {
        let pkg_str = pkg.to_string_lossy().to_string();
        let exe = super::current_exe_string();

        let is_deb = pkg_str.ends_with(".deb");
        let is_arch = pkg_str.ends_with(".pkg.tar.zst");
        if !is_deb && !is_arch {
            anyhow::bail!("未知的 Linux 安装包格式: {}", pkg_str);
        }

        // 安装命令（不带提权前缀）
        let install_cmd = if is_deb {
            format!("dpkg -i '{}'", pkg_str)
        } else {
            format!("pacman -U --noconfirm '{}'", pkg_str)
        };

        // 提权前缀选择
        let has_pkexec = which("pkexec").is_some();
        let terminal = if !has_pkexec { pick_terminal() } else { None };

        let script_path = std::env::temp_dir().join("pezmax-updater.sh");

        let full_cmd = if has_pkexec {
            format!("pkexec sh -c \"{cmd}\"", cmd = install_cmd.replace('"', "\\\""))
        } else if let Some(term) = &terminal {
            // sudo + 终端窗口，让用户在图形终端里输密码
            build_terminal_sudo_cmd(term, &install_cmd)
        } else {
            // 极端兜底：直接尝试 sudo（无 tty 会失败，但至少留下日志）
            format!("sudo {cmd}", cmd = install_cmd)
        };

        let script = format!(
            r#"#!/bin/bash
set -e
sleep 2
LOG=/tmp/pezmax-updater.log
echo "[updater] {ts_hint} start" >>"$LOG"

if ! {full_cmd} >>"$LOG" 2>&1; then
  echo "[updater] 安装失败，查看 $LOG" >&2
  # 失败也别删安装包，方便用户手动重试
  exit 1
fi

# 安装成功：删安装包 + 重启
rm -f "{pkg}"
# 优先用系统 pezmax-one wrapper（/usr/bin/pezmax-one），设了 LD_LIBRARY_PATH
if command -v pezmax-one >/dev/null 2>&1; then
  setsid pezmax-one >/dev/null 2>&1 < /dev/null &
else
  setsid '{exe}' >/dev/null 2>&1 < /dev/null &
fi

rm -f "$0"
"#,
            ts_hint = env!("CARGO_PKG_VERSION"),
            full_cmd = full_cmd,
            pkg = pkg_str,
            exe = exe,
        );
        std::fs::write(&script_path, script)
            .with_context(|| format!("写入 helper 脚本失败: {}", script_path.display()))?;

        use std::os::unix::fs::PermissionsExt;
        let mut perm = std::fs::metadata(&script_path)?.permissions();
        perm.set_mode(0o755);
        std::fs::set_permissions(&script_path, perm)?;

        // 用 setsid + nohup 完全脱离父进程 session
        Command::new("bash")
            .arg("-c")
            .arg(format!(
                "setsid nohup '{}' >/tmp/pezmax-updater.log 2>&1 < /dev/null &",
                script_path.display()
            ))
            .spawn()
            .context("启动 helper 脚本失败")?;

        Ok(())
    }

    fn which(name: &str) -> Option<PathBuf> {
        let path = std::env::var_os("PATH")?;
        for dir in std::env::split_paths(&path) {
            let full = dir.join(name);
            if full.is_file() {
                // 简易可执行判断：Linux 上依赖调用 exec 时报错兜底
                return Some(full);
            }
        }
        None
    }

    /// 挑一个存在的图形终端模拟器，返回程序名。找不到返回 None。
    fn pick_terminal() -> Option<String> {
        for cand in [
            "x-terminal-emulator", // Debian 系推荐
            "konsole",             // KDE
            "gnome-terminal",      // GNOME
            "xfce4-terminal",      // XFCE
            "mate-terminal",
            "lxterminal",
            "alacritty",
            "kitty",
            "wezterm",
            "foot",
            "xterm",
        ] {
            if which(cand).is_some() {
                return Some(cand.to_string());
            }
        }
        None
    }

    /// 根据终端类型构造"打开终端 → sudo 跑命令"的一句 shell。
    ///
    /// 大多数终端接受 `-e <cmd>`；gnome-terminal / konsole 需要用不同标志。
    fn build_terminal_sudo_cmd(term: &str, install_cmd: &str) -> String {
        let inner = format!("sudo {cmd}", cmd = install_cmd);
        match term {
            "gnome-terminal" => {
                // gnome-terminal -- bash -c '<inner>; read -p "按回车关闭..."'
                format!(
                    "gnome-terminal -- bash -c '{cmd}; read -p \"按回车关闭...\"'",
                    cmd = inner.replace('\'', "'\\''")
                )
            }
            "konsole" => {
                format!(
                    "konsole -e bash -c '{cmd}; read -p \"按回车关闭...\"'",
                    cmd = inner.replace('\'', "'\\''")
                )
            }
            _ => {
                // xterm / alacritty / kitty / wezterm / foot 等大多接受 -e
                format!(
                    "{term} -e bash -c '{cmd}; read -p \"按回车关闭...\"'",
                    term = term,
                    cmd = inner.replace('\'', "'\\''")
                )
            }
        }
    }
}
