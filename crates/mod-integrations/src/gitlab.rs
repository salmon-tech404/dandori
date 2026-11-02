//! Adapter `GitLabProvider` hiện thực `GitHostProvider` cho GitLab REST API v4.
//! Tài liệu chính thức: <https://docs.gitlab.com/ee/api/merge_requests.html>

use axum::http::StatusCode;
use dandori_module_api::{
    BoxFuture, CiState, CiStatusSummary, CreatePullRequestPayload, GitHostError, GitHostProvider,
    MergeMethod, MergeResult, PullRequestState, RemotePullRequest,
};
use reqwest::Client;
use serde_json::Value;
use std::fmt;
use url::Url;

use crate::http_client::{MAX_RESPONSE_SIZE, read_limited_text, sanitize_error_message};

/// Adapter kết nối tới GitLab (gitlab.com hoặc máy chủ self-hosted).
#[derive(Clone)]
pub struct GitLabProvider {
    client: Client,
    base_url: Url,
    token: String,
}

impl fmt::Debug for GitLabProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GitLabProvider")
            .field("base_url", &self.base_url.as_str())
            .field("token", &"[REDACTED]")
            .finish()
    }
}

impl GitLabProvider {
    /// Khởi tạo một `GitLabProvider` mới.
    #[must_use]
    pub fn new(client: Client, base_url: Url, token: String) -> Self {
        Self {
            client,
            base_url,
            token,
        }
    }

    fn build_url(&self, path: &str) -> Result<Url, GitHostError> {
        let trimmed = path.trim_start_matches('/');
        self.base_url
            .join(trimmed)
            .map_err(|e| GitHostError::InvalidParameter(e.to_string()))
    }

    fn encode_project_id(repo: &str) -> String {
        url::form_urlencoded::byte_serialize(repo.as_bytes()).collect::<String>()
    }

    fn parse_mr_json(v: &Value) -> Result<RemotePullRequest, GitHostError> {
        let number = v.get("iid").and_then(|n| n.as_u64()).unwrap_or(0);
        let url = v
            .get("web_url")
            .and_then(|u| u.as_str())
            .unwrap_or_default()
            .to_string();
        let title = v
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or_default()
            .to_string();
        let state_str = v.get("state").and_then(|s| s.as_str()).unwrap_or("opened");

        let state = match state_str {
            "merged" => PullRequestState::Merged,
            "closed" | "locked" => PullRequestState::Closed,
            _ => PullRequestState::Open,
        };

        let source_branch = v
            .get("source_branch")
            .and_then(|b| b.as_str())
            .unwrap_or_default()
            .to_string();
        let target_branch = v
            .get("target_branch")
            .and_then(|b| b.as_str())
            .unwrap_or_default()
            .to_string();
        let has_conflicts = v.get("has_conflicts").and_then(|c| c.as_bool());
        let mergeable = has_conflicts.map(|c| !c);

        Ok(RemotePullRequest {
            number,
            url,
            title,
            state,
            source_branch,
            target_branch,
            mergeable,
        })
    }
}

impl GitHostProvider for GitLabProvider {
    fn name(&self) -> &'static str {
        "gitlab"
    }

    fn test_connection(&self) -> BoxFuture<'static, Result<(), GitHostError>> {
        let this = self.clone();
        Box::pin(async move {
            let url = this.build_url("api/v4/user")?;
            let resp = this
                .client
                .get(url)
                .header("PRIVATE-TOKEN", &this.token)
                .header("User-Agent", "Dandori-Agent")
                .send()
                .await
                .map_err(|e| {
                    GitHostError::Network(sanitize_error_message(&e.to_string(), &this.token))
                })?;

            if resp.status().is_success() {
                Ok(())
            } else if resp.status() == StatusCode::UNAUTHORIZED
                || resp.status() == StatusCode::FORBIDDEN
            {
                Err(GitHostError::AuthenticationFailed(
                    "Token GitLab không hợp lệ hoặc không có quyền truy cập".to_string(),
                ))
            } else {
                Err(GitHostError::Other(format!(
                    "GitLab API trả về mã lỗi: {}",
                    resp.status()
                )))
            }
        })
    }

    fn create_pull_request(
        &self,
        repo: &str,
        payload: &CreatePullRequestPayload,
    ) -> BoxFuture<'static, Result<RemotePullRequest, GitHostError>> {
        let this = self.clone();
        let encoded_repo = Self::encode_project_id(repo);
        let payload = payload.clone();

        Box::pin(async move {
            let path = format!("api/v4/projects/{encoded_repo}/merge_requests");
            let url = this.build_url(&path)?;
            let body = serde_json::json!({
                "title": payload.title,
                "description": payload.body,
                "source_branch": payload.source_branch,
                "target_branch": payload.target_branch,
            });

            let resp = this
                .client
                .post(url)
                .header("PRIVATE-TOKEN", &this.token)
                .header("User-Agent", "Dandori-Agent")
                .json(&body)
                .send()
                .await
                .map_err(|e| {
                    GitHostError::Network(sanitize_error_message(&e.to_string(), &this.token))
                })?;

            let status = resp.status();
            let text = read_limited_text(resp, MAX_RESPONSE_SIZE)
                .await
                .map_err(GitHostError::Other)?;

            if status.is_success() {
                let v: Value = serde_json::from_str(&text)
                    .map_err(|e| GitHostError::Other(format!("Lỗi parse JSON MR: {e}")))?;
                Self::parse_mr_json(&v)
            } else {
                Err(GitHostError::Other(format!(
                    "Tạo MR thất bại (mã {status}): {}",
                    sanitize_error_message(&text, &this.token)
                )))
            }
        })
    }

    fn get_pull_request(
        &self,
        repo: &str,
        number: u64,
    ) -> BoxFuture<'static, Result<RemotePullRequest, GitHostError>> {
        let this = self.clone();
        let encoded_repo = Self::encode_project_id(repo);

        Box::pin(async move {
            let path = format!("api/v4/projects/{encoded_repo}/merge_requests/{number}");
            let url = this.build_url(&path)?;

            let resp = this
                .client
                .get(url)
                .header("PRIVATE-TOKEN", &this.token)
                .header("User-Agent", "Dandori-Agent")
                .send()
                .await
                .map_err(|e| {
                    GitHostError::Network(sanitize_error_message(&e.to_string(), &this.token))
                })?;

            if resp.status() == StatusCode::NOT_FOUND {
                return Err(GitHostError::NotFound(format!(
                    "Không tìm thấy MR #{number}"
                )));
            }

            let text = read_limited_text(resp, MAX_RESPONSE_SIZE)
                .await
                .map_err(GitHostError::Other)?;
            let v: Value = serde_json::from_str(&text)
                .map_err(|e| GitHostError::Other(format!("Lỗi parse JSON MR: {e}")))?;
            Self::parse_mr_json(&v)
        })
    }

    fn get_ci_status(
        &self,
        repo: &str,
        _git_ref: &str,
    ) -> BoxFuture<'static, Result<CiStatusSummary, GitHostError>> {
        let this = self.clone();
        let encoded_repo = Self::encode_project_id(repo);

        Box::pin(async move {
            let path = format!("api/v4/projects/{encoded_repo}/pipelines");
            let url = this.build_url(&path)?;

            let resp = this
                .client
                .get(url)
                .header("PRIVATE-TOKEN", &this.token)
                .header("User-Agent", "Dandori-Agent")
                .send()
                .await
                .map_err(|e| {
                    GitHostError::Network(sanitize_error_message(&e.to_string(), &this.token))
                })?;

            let text = read_limited_text(resp, MAX_RESPONSE_SIZE)
                .await
                .map_err(GitHostError::Other)?;
            let v: Value = serde_json::from_str(&text).unwrap_or(serde_json::json!([]));

            let first_pipeline = v.as_array().and_then(|arr| arr.first());
            let raw_status = first_pipeline
                .and_then(|p| p.get("status"))
                .and_then(|s| s.as_str())
                .unwrap_or("unknown");

            let state = match raw_status {
                "success" => CiState::Success,
                "failed" => CiState::Failure,
                "running" | "pending" => CiState::Pending,
                _ => CiState::Error,
            };

            Ok(CiStatusSummary {
                state,
                description: format!("Pipeline status: {raw_status}"),
                success_count: if state == CiState::Success { 1 } else { 0 },
                failure_count: if state == CiState::Failure { 1 } else { 0 },
                pending_count: if state == CiState::Pending { 1 } else { 0 },
            })
        })
    }

    fn merge_pull_request(
        &self,
        repo: &str,
        number: u64,
        method: MergeMethod,
    ) -> BoxFuture<'static, Result<MergeResult, GitHostError>> {
        let this = self.clone();
        let encoded_repo = Self::encode_project_id(repo);

        Box::pin(async move {
            let path = format!("api/v4/projects/{encoded_repo}/merge_requests/{number}/merge");
            let url = this.build_url(&path)?;
            let squash = matches!(method, MergeMethod::Squash);

            let resp = this
                .client
                .put(url)
                .header("PRIVATE-TOKEN", &this.token)
                .header("User-Agent", "Dandori-Agent")
                .json(&serde_json::json!({ "squash": squash }))
                .send()
                .await
                .map_err(|e| {
                    GitHostError::Network(sanitize_error_message(&e.to_string(), &this.token))
                })?;

            let status = resp.status();
            let text = read_limited_text(resp, MAX_RESPONSE_SIZE)
                .await
                .map_err(GitHostError::Other)?;

            if status == StatusCode::METHOD_NOT_ALLOWED || status == StatusCode::CONFLICT {
                return Err(GitHostError::MergeConflict(
                    "Không thể merge MR do xung đột hoặc pipeline chưa qua".to_string(),
                ));
            }

            let v: Value = serde_json::from_str(&text).unwrap_or(serde_json::json!({}));
            let state_str = v.get("state").and_then(|s| s.as_str()).unwrap_or("");
            let merged = state_str == "merged" || status.is_success();
            let sha = v
                .get("merge_commit_sha")
                .and_then(|s| s.as_str())
                .map(String::from);
            let message = v
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or("Merged")
                .to_string();

            Ok(MergeResult {
                merged,
                sha,
                message,
            })
        })
    }
}
