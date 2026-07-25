// 本地下载记录 SQLite 数据库
//
// 后端下载记录只能反映"点了下载按钮"，无法记录本地保存路径或文件是否还在。
// 这里维护一份本地表，用于：
//   - 记录实际写盘成功的下载（#22：失败下载不进表）
//   - 从下载记录里"打开本地文件"（#14）
//   - 与服务器下载记录合并去重，作为下载量的权威数据源（#15）
//
// 数据库文件位于 `{data_dir}/.cache/downloads.db`。

use anyhow::Result;
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct LocalDownload {
    pub id: i64,
    pub file_id: i64,
    pub file_name: String,
    pub path: String,
    /// Unix epoch 秒
    pub downloaded_at: i64,
    /// 是否被用户隐藏
    pub hidden: bool,
}

/// SQLite 连接的线程安全包装
#[derive(Clone)]
pub struct DownloadDb {
    conn: Arc<Mutex<Connection>>,
}

impl DownloadDb {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS downloads (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                file_id       INTEGER NOT NULL,
                file_name     TEXT    NOT NULL,
                path          TEXT    NOT NULL,
                downloaded_at INTEGER NOT NULL,
                hidden_at     INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_downloads_file_id
                ON downloads(file_id);
            CREATE INDEX IF NOT EXISTS idx_downloads_downloaded_at
                ON downloads(downloaded_at DESC);
            "#,
        )?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// 追加一条下载记录。返回新纪录的 rowid。
    /// 仅应在文件真的写盘成功后调用。
    pub fn insert(&self, file_id: i64, file_name: &str, path: &Path) -> Result<i64> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let path_str = path.to_string_lossy().to_string();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO downloads (file_id, file_name, path, downloaded_at) \
             VALUES (?1, ?2, ?3, ?4)",
            params![file_id, file_name, path_str, now],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// 列出所有未被隐藏的记录，最新的在前面。
    pub fn list_visible(&self) -> Result<Vec<LocalDownload>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, file_id, file_name, path, downloaded_at, hidden_at \
             FROM downloads WHERE hidden_at IS NULL ORDER BY downloaded_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(LocalDownload {
                id: row.get(0)?,
                file_id: row.get(1)?,
                file_name: row.get(2)?,
                path: row.get(3)?,
                downloaded_at: row.get(4)?,
                hidden: row.get::<_, Option<i64>>(5)?.is_some(),
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    /// 隐藏一条本地记录（软删）。
    pub fn hide(&self, id: i64) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE downloads SET hidden_at = ?1 WHERE id = ?2 AND hidden_at IS NULL",
            params![now, id],
        )?;
        Ok(())
    }

    /// 硬删除一条记录。
    pub fn delete(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM downloads WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// 统计未隐藏的下载数量。
    pub fn count_visible(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM downloads WHERE hidden_at IS NULL",
            [],
            |row| row.get(0),
        )?;
        Ok(n)
    }
}

/// 计算数据库默认路径 `{cache_dir}/downloads.db`
pub fn default_db_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join("downloads.db")
}
