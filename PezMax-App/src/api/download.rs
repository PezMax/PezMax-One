// 下载与收藏 API

use crate::api::client::ApiClient;
use crate::api::models::*;
use anyhow::Result;

impl ApiClient {
    // ── 下载 ──

    /// 获取当前用户下载记录
    pub async fn get_download_list(&self, params: &PageParams) -> Result<PageResponse<DownloadRecord>> {
        let query = vec![
            ("pageNum", params.page_num.to_string()),
            ("pageSize", params.page_size.to_string()),
        ];
        self.get("/datum/desktop/download/list/{userId}", Some(query)).await
    }

    /// 隐藏下载记录
    pub async fn hide_download(&self, file_id: i64) -> Result<ApiResponse<serde_json::Value>> {
        let user_id = self.get_user_id().await;
        self.delete(&format!("/datum/desktop/download/{}/{}", user_id, file_id)).await
    }

    /// 获取用户 ID（内部辅助）
    async fn get_user_id(&self) -> i64 {
        match self.get_profile().await {
            Ok(resp) => resp.data.map(|u| u.user_id).unwrap_or(0),
            Err(_) => 0,
        }
    }

    // ── 收藏 ──

    /// 获取当前用户收藏列表
    pub async fn get_favorite_list(&self, params: &PageParams) -> Result<PageResponse<FavoriteRecord>> {
        let query = vec![
            ("pageNum", params.page_num.to_string()),
            ("pageSize", params.page_size.to_string()),
        ];
        // 注意：需要先获取 userId，这里简化处理
        self.get("/datum/desktop/favorite/list/{userId}", Some(query)).await
    }

    /// 新增收藏
    pub async fn add_favorite(&self, file_id: i64) -> Result<ApiResponse<serde_json::Value>> {
        let body = serde_json::json!({ "fileId": file_id });
        self.post("/datum/favorite", &body).await
    }

    /// 删除收藏
    pub async fn remove_favorite(&self, file_id: i64) -> Result<ApiResponse<serde_json::Value>> {
        let user_id = self.get_user_id().await;
        self.delete(&format!("/datum/desktop/favorite/{}/{}", user_id, file_id)).await
    }
}