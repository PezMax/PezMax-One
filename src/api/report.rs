// 举报 API

use crate::api::client::ApiClient;
use crate::api::models::*;
use anyhow::Result;

impl ApiClient {
    /// 新增举报
    pub async fn create_report(&self, report: &Report) -> Result<ApiResponse<serde_json::Value>> {
        self.post("/datum/report", report).await
    }

    /// 获取举报处理时间线
    pub async fn get_report_timeline(&self, report_id: i64) -> Result<ApiResponse<serde_json::Value>> {
        self.get(&format!("/datum/report/timeline/{}", report_id), None).await
    }
}