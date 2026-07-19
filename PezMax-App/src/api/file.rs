// 试卷文件 API

use crate::api::client::ApiClient;
use crate::api::models::*;
use anyhow::Result;

impl ApiClient {
    /// 获取试卷文件列表（分页，匿名）
    pub async fn get_file_list(&self, params: &PageParams) -> Result<PageResponse<PaperFile>> {
        let query = vec![
            ("pageNum", params.page_num.to_string()),
            ("pageSize", params.page_size.to_string()),
        ];
        self.get_anonymous("/datum/file/list", Some(query)).await
    }

    /// 获取文件详情（匿名）
    pub async fn get_file_detail(&self, file_id: i64) -> Result<ApiResponse<PaperFile>> {
        self.get_anonymous(&format!("/datum/file/{}", file_id), None).await
    }

    /// 获取文件树形结构（匿名）
    pub async fn get_file_tree(&self) -> Result<ApiResponse<Vec<FileTreeNode>>> {
        self.get_anonymous("/datum/file/tree", None).await
    }

    /// 关键词搜索文件
    pub async fn search_files(&self, keyword: &str) -> Result<Vec<PaperFile>> {
        let params = vec![("keyword", keyword.to_string())];
        self.get("/datum/file/search", Some(params)).await
    }

    /// 获取学科联想列表
    pub async fn get_subjects(&self, keyword: Option<&str>) -> Result<ApiResponse<Vec<String>>> {
        let params = keyword.map(|k| vec![("keyword", k.to_string())]);
        self.get("/datum/file/subjects", params).await
    }

    /// 获取学校联想列表（匿名）
    pub async fn get_schools(&self, keyword: Option<&str>) -> Result<ApiResponse<Vec<String>>> {
        let params = keyword.map(|k| vec![("keyword", k.to_string())]);
        self.get_anonymous("/datum/file/schools", params).await
    }

    /// 新增试卷文件
    pub async fn create_file(&self, file: &PaperFile) -> Result<ApiResponse<serde_json::Value>> {
        self.post("/datum/file", file).await
    }

    /// 修改试卷文件
    pub async fn update_file(&self, file: &PaperFile) -> Result<ApiResponse<serde_json::Value>> {
        self.put("/datum/file", file).await
    }

    /// 删除试卷文件
    pub async fn delete_file(&self, file_ids: &[i64]) -> Result<ApiResponse<serde_json::Value>> {
        let ids = file_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        self.delete(&format!("/datum/file/{}", ids)).await
    }

    /// 下载试卷文件（流）
    pub async fn download_paper(&self, file_id: i64) -> Result<Vec<u8>> {
        self.download_file(&format!("/datum/download/file?fileId={}", file_id)).await
    }
}