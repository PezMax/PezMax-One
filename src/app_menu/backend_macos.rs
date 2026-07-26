// macOS 菜单后端（muda + NSMenu）
// 由 Task #3 实现，当前仅提供占位符使 cfg 分支可编译。

use std::sync::mpsc;

use super::{MenuBackend, MenuCommand};
use crate::theme::ThemeMode;

pub struct MacMenu;

impl MenuBackend for MacMenu {
    fn set_theme_mode(&self, _mode: ThemeMode) {}
    fn set_accent(&self, _idx: usize) {}
}

pub fn install(_tx: mpsc::Sender<MenuCommand>) -> anyhow::Result<Box<dyn MenuBackend>> {
    // TODO(#3): 用 muda 构建 NSMenu。当前先返回占位符，让整个流程可编译运行。
    log::info!("macOS 菜单后端未实现（Task #3），使用占位符");
    Ok(Box::new(MacMenu))
}
