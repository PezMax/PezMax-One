// 通用 API 响应模型
// 对应后端 AjaxResult / TableDataInfo 统一格式

use serde::{Deserialize, Serialize};

/// 后端统一响应格式 (AjaxResult)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
pub struct ApiResponse<T> {
    pub code: i32,
    pub msg: String,
    #[serde(default)]
    pub data: Option<T>,
}

/// 分页响应格式 (TableDataInfo)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
pub struct PageResponse<T> {
    pub code: i32,
    pub msg: String,
    #[serde(default)]
    pub rows: Vec<T>,
    #[serde(default)]
    pub total: i64,
}

/// 分页请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageParams {
    #[serde(default = "default_page_num")]
    pub page_num: i32,
    #[serde(default = "default_page_size")]
    pub page_size: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by_column: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_asc: Option<String>,
}

fn default_page_num() -> i32 { 1 }
fn default_page_size() -> i32 { 20 }

impl Default for PageParams {
    fn default() -> Self {
        Self {
            page_num: 1,
            page_size: 20,
            order_by_column: None,
            is_asc: None,
        }
    }
}

/// 登录请求体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
}

/// 登录响应 (桌面端)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

/// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    #[serde(default)]
    pub user_id: i64,
    #[serde(default)]
    pub user_name: String,
    #[serde(default)]
    pub nick_name: String,
    #[serde(default)]
    pub avatar: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub phonenumber: String,
    #[serde(default)]
    pub sex: String,
    #[serde(default)]
    pub status: String,
}

/// getInfo 响应结构（用户信息嵌套在 data.user 中）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfoResponse {
    pub user: UserInfo,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub is_default_modify_pwd: bool,
    #[serde(default)]
    pub is_password_expired: bool,
}

/// 验证码响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaResponse {
    #[serde(default = "default_true", alias = "captchaEnabled")]
    pub captcha_enabled: bool,
    #[serde(default)]
    pub uuid: String,
    #[serde(default)]
    pub img: String, // base64
}

fn default_true() -> bool { true }

/// 注册请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub nickname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    pub security_questions: Vec<SecurityQuestion>,
}

/// 密保问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityQuestion {
    pub question: String,
    pub answer: String,
}

/// 试卷文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperFile {
    #[serde(default)]
    pub file_id: i64,
    #[serde(default)]
    pub file_name: String,
    #[serde(default)]
    pub file_format: String,
    #[serde(default)]
    pub file_size: i64,
    #[serde(default)]
    pub file_url: String,
    #[serde(default)]
    pub file_subject: String,
    #[serde(default)]
    pub file_year: String,
    #[serde(default)]
    pub school_name: String,
    #[serde(default)]
    pub create_by: String,
    #[serde(default)]
    pub create_time: String,
    #[serde(default)]
    pub file_cover: String,
    #[serde(default)]
    pub file_uploader: String,
}

/// 文件树节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTreeNode {
    pub id: i64,
    pub label: String,
    #[serde(default)]
    pub children: Vec<FileTreeNode>,
    #[serde(rename = "type", default)]
    pub node_type: String,
}

/// 书签
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub bookmark_name: String,
    #[serde(default)]
    pub bookmark_url: String,
    #[serde(default)]
    pub resource_type: String,
    #[serde(default)]
    pub cover_url: String,
    #[serde(default)]
    pub create_time: String,
}

/// 通知
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    #[serde(default)]
    pub notify_id: i64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub notify_type: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub create_time: String,
}

/// 下载记录
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DownloadRecord {
    #[serde(default)]
    pub download_id: i64,
    #[serde(default)]
    pub file_id: i64,
    #[serde(default)]
    pub file_name: String,
    #[serde(default)]
    pub user_id: i64,
    #[serde(default)]
    pub download_time: String,
    #[serde(default)]
    pub file_format: String,
}

/// 收藏记录
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FavoriteRecord {
    #[serde(default)]
    pub file_id: i64,
    #[serde(default)]
    pub file_name: String,
    #[serde(default)]
    pub file_subject: String,
    #[serde(default)]
    pub create_time: String,
}

/// 用户统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStats {
    #[serde(default)]
    pub favorite_count: i64,
    #[serde(default)]
    pub download_count: i64,
    #[serde(default)]
    pub upload_count: i64,
}

/// 举报
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    #[serde(default)]
    pub report_id: i64,
    #[serde(default)]
    pub report_type: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub create_time: String,
}

/// 收藏/下载/举报的通用操作响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResponse {
    pub code: i32,
    pub msg: String,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}