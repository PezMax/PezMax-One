// PezMax API 客户端
// 基于后端接口列表.md 的完整 API 封装
// 使用 reqwest 异步 HTTP 客户端，支持 Bearer 鉴权、分页、文件上传/下载

pub mod client;
pub mod models;
pub mod auth;
pub mod bookmark;
pub mod file;
pub mod notification;
pub mod user;
pub mod report;
pub mod download;
pub mod favorite;

pub use client::ApiClient;
pub use models::*;