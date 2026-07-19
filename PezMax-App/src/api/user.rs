// 用户个人中心 API

use crate::api::client::ApiClient;
use crate::api::models::*;
use anyhow::Result;

impl ApiClient {
    /// 获取当前用户信息（桌面端个人中心）
    pub async fn get_profile(&self) -> Result<ApiResponse<UserInfo>> {
        self.get("/datum/desktop/user/profile", None).await
    }

    /// 获取用户统计
    pub async fn get_user_stats(&self) -> Result<ApiResponse<UserStats>> {
        self.get("/datum/desktop/user/profile/stats", None).await
    }

    /// 更新用户名
    pub async fn update_username(&self, name: &str) -> Result<ApiResponse<serde_json::Value>> {
        let body = serde_json::json!({ "userName": name });
        self.put("/datum/desktop/user/profile/username", &body).await
    }

    /// 更新头像地址
    pub async fn update_avatar_url(&self, url: &str) -> Result<ApiResponse<serde_json::Value>> {
        let body = serde_json::json!({ "avatar": url });
        self.put("/datum/desktop/user/profile/avatar", &body).await
    }

    /// 上传头像文件
    pub async fn upload_avatar(&self, file_path: &str) -> Result<ApiResponse<serde_json::Value>> {
        self.upload_file("/datum/desktop/user/profile/avatar/upload", file_path, "file", None).await
    }

    /// 验证密码
    pub async fn verify_password(&self, password: &str) -> Result<ApiResponse<serde_json::Value>> {
        let body = serde_json::json!({ "password": password });
        self.post("/datum/desktop/user/profile/password/verify", &body).await
    }

    /// 更新密码
    pub async fn update_password(&self, old: &str, new: &str) -> Result<ApiResponse<serde_json::Value>> {
        let body = serde_json::json!({ "oldPassword": old, "newPassword": new });
        self.put("/datum/desktop/user/profile/password", &body).await
    }

    /// 获取密保问题
    pub async fn get_security(&self) -> Result<ApiResponse<serde_json::Value>> {
        self.get("/datum/desktop/user/profile/security", None).await
    }

    /// 更新密保
    pub async fn update_security(&self, data: &serde_json::Value) -> Result<ApiResponse<serde_json::Value>> {
        self.put("/datum/desktop/user/profile/security", data).await
    }
}