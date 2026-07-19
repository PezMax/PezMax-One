// 下载与收藏 API

use crate::api::client::ApiClient;
use crate::api::models::*;
use anyhow::Result;

impl ApiClient {
    // ── 下载 ──

    /// 获取当前用户下载记录（需传入 userId）
    pub async fn get_download_list(&self, user_id: i64, params: &PageParams) -> Result<PageResponse<DownloadRecord>> {
        let query = vec![
            ("pageNum", params.page_num.to_string()),
            ("pageSize", params.page_size.to_string()),
        ];
        self.get(&format!("/datum/desktop/download/list/{}", user_id), Some(query)).await
    }

    /// 隐藏下载记录
    pub async fn hide_download(&self, user_id: i64, file_id: i64) -> Result<ApiResponse<serde_json::Value>> {
        self.delete(&format!("/datum/desktop/download/{}/{}", user_id, file_id)).await
    }

    // ── 收藏 ──

    /// 获取当前用户收藏列表（需传入 userId）
    pub async fn get_favorite_list(&self, user_id: i64, params: &PageParams) -> Result<PageResponse<FavoriteRecord>> {
        let query = vec![
            ("pageNum", params.page_num.to_string()),
            ("pageSize", params.page_size.to_string()),
        ];
        self.get(&format!("/datum/desktop/favorite/list/{}", user_id), Some(query)).await
    }

    /// 新增收藏
    pub async fn add_favorite(&self, file_id: i64) -> Result<ApiResponse<serde_json::Value>> {
        let body = serde_json::json!({ "fileId": file_id });
        self.post("/datum/favorite", &body).await
    }

    /// 删除收藏（需传入 userId）
    pub async fn remove_favorite(&self, user_id: i64, file_id: i64) -> Result<ApiResponse<serde_json::Value>> {
        self.delete(&format!("/datum/desktop/favorite/{}/{}", user_id, file_id)).await
    }
}