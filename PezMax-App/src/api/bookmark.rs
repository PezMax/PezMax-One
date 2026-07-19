// 书签 API

use crate::api::client::ApiClient;
use crate::api::models::*;
use anyhow::Result;

impl ApiClient {
    /// 获取书签列表（分页，匿名）
    pub async fn get_bookmark_list(&self, params: &PageParams) -> Result<PageResponse<Bookmark>> {
        let query = vec![
            ("pageNum", params.page_num.to_string()),
            ("pageSize", params.page_size.to_string()),
        ];
        self.get_anonymous("/datum/bookmark/list", Some(query)).await
    }

    /// 获取书签详情（匿名）
    pub async fn get_bookmark_detail(&self, id: i64) -> Result<ApiResponse<Bookmark>> {
        self.get_anonymous(&format!("/datum/bookmark/{}", id), None).await
    }

    /// 新增书签
    pub async fn create_bookmark(&self, bookmark: &Bookmark) -> Result<ApiResponse<serde_json::Value>> {
        self.post("/datum/bookmark", bookmark).await
    }

    /// 修改书签
    pub async fn update_bookmark(&self, bookmark: &Bookmark) -> Result<ApiResponse<serde_json::Value>> {
        self.put("/datum/bookmark", bookmark).await
    }

    /// 删除书签
    pub async fn delete_bookmark(&self, id: i64) -> Result<ApiResponse<serde_json::Value>> {
        self.delete(&format!("/datum/bookmark/{}", id)).await
    }
}