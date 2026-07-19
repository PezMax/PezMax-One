// 通知 API

use crate::api::client::ApiClient;
use crate::api::models::*;
use anyhow::Result;

impl ApiClient {
    /// 获取弹窗通知
    pub async fn get_popup_notifications(&self, user_id: i64) -> Result<ApiResponse<Vec<Notification>>> {
        let params = vec![("userId", user_id.to_string())];
        self.get("/system/notification/user/popup", Some(params)).await
    }

    /// 获取滚动通知
    pub async fn get_scroll_notifications(&self) -> Result<ApiResponse<Vec<Notification>>> {
        self.get("/system/notification/user/scroll", None).await
    }
}