//! Adapter `GitHubProvider` hiện thực `GitHostProvider` cho GitHub REST API.
//! Tài liệu chính thức: <https://docs.github.com/en/rest>

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

/// Adapter kết nối tới GitHub.
#[derive(Clone)]
pub struct GitHubProvider {
    client: Client,
    base_url: Url,
    token: String,
}

impl fmt::Debug for GitHubProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GitHubProvider")
            .field("base_url", &self.base_url.as_str())
            .field("token", &"[REDACTED]")
            .finish()
    }
}

impl GitHubProvider {
    /// Khởi tạo một `GitHubProvider` mới.
    #[must_use]
    pub fn new(client: Client, base_url: Url, token: String) -> Self {
        Self {
            client,
            base_url,
            token,
        }
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.token)
    }

    fn build_url(&self, path: &str) -> Result<Url, GitHostError> {
        let trimmed = path.trim_start_matches('/');
        self.base_url
            .join(trimmed)
            .map_err(|e| GitHostError::InvalidParameter(e.to_string()))
    }

    fn parse_pr_json(v: &Value) -> Result<RemotePullRequest, GitHostError> {
        let number = v.get("number").and_then(|n| n.as_u64()).unwrap_or(0);
        let url = v
            .get("html_url")
            .and_then(|u| u.as_str())
            .unwrap_or_default()
            .to_string();
        let title = v
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or_default()
            .to_string();
        let state_str = v.get("state").and_then(|s| s.as_str()).unwrap_or("open");
        let merged = v.get("merged").and_then(|m| m.as_bool()).unwrap_or(false);

        let state = if merged {
            PullRequestState::Merged
        } else if state_str == "closed" {
            PullRequestState::Closed
        } else {
            PullRequestState::Open
        };

        let source_branch = v
            .get("head")
            .and_then(|h| h.get("ref"))
            .and_then(|r| r.as_str())
            .unwrap_or_default()
            .to_string();
        let target_branch = v
            .get("base")
            .and_then(|b| b.get("ref"))
            .and_then(|r| r.as_str())
            .unwrap_or_default()
            .to_string();
        let mergeable = v.get("mergeable").and_then(|m| m.as_bool());

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

fn parse_check_runs(runs: &[Value]) -> (u32, u32, u32) {
    let mut success: u32 = 0;
    let mut failure: u32 = 0;
    let mut pending: u32 = 0;
    for r in runs {
        let conclusion = r.get("conclusion").and_then(|c| c.as_str()).unwrap_or("");
        let status = r.get("status").and_then(|s| s.as_str()).unwrap_or("");
        if status != "completed" {
            pending += 1;
        } else if conclusion == "success" {
            success += 1;
        } else {
            failure += 1;
        }
    }
    (success, failure, pending)
}

impl GitHostProvider for GitHubProvider {
    fn name(&self) -> &'static str {
        "github"
    }

    fn test_connection(&self) -> BoxFuture<'static, Result<(), GitHostError>> {
        let this = self.clone();
        Box::pin(async move {
            let url = this.build_url("user")?;
            let resp = this
                .client
                .get(url)
                .header("Authorization", this.auth_header())
                .header("User-Agent", "Dandori-Agent")
                .header("Accept", "application/vnd.github+json")
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
                    "Token GitHub không hợp lệ hoặc không có quyền truy cập".to_string(),
                ))
            } else {
                Err(GitHostError::Other(format!(
                    "GitHub API trả về mã lỗi: {}",
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
        let repo = repo.to_string();
        let payload = payload.clone();

        Box::pin(async move {
            let path = format!("repos/{repo}/pulls");
            let url = this.build_url(&path)?;
            let body = serde_json::json!({
                "title": payload.title,
                "body": payload.body,
                "head": payload.source_branch,
                "base": payload.target_branch,
            });

            let resp = this
                .client
                .post(url)
                .header("Authorization", this.auth_header())
                .header("User-Agent", "Dandori-Agent")
                .header("Accept", "application/vnd.github+json")
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
                    .map_err(|e| GitHostError::Other(format!("Lỗi parse JSON PR: {e}")))?;
                Self::parse_pr_json(&v)
            } else {
                Err(GitHostError::Other(format!(
                    "Tạo PR thất bại (mã {status}): {}",
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
        let repo = repo.to_string();

        Box::pin(async move {
            let path = format!("repos/{repo}/pulls/{number}");
            let url = this.build_url(&path)?;

            let resp = this
                .client
                .get(url)
                .header("Authorization", this.auth_header())
                .header("User-Agent", "Dandori-Agent")
                .header("Accept", "application/vnd.github+json")
                .send()
                .await
                .map_err(|e| {
                    GitHostError::Network(sanitize_error_message(&e.to_string(), &this.token))
                })?;

            if resp.status() == StatusCode::NOT_FOUND {
                return Err(GitHostError::NotFound(format!(
                    "Không tìm thấy PR #{number}"
                )));
            }

            let text = read_limited_text(resp, MAX_RESPONSE_SIZE)
                .await
                .map_err(GitHostError::Other)?;
            let v: Value = serde_json::from_str(&text)
                .map_err(|e| GitHostError::Other(format!("Lỗi parse JSON PR: {e}")))?;
            Self::parse_pr_json(&v)
        })
    }

    fn get_ci_status(
        &self,
        repo: &str,
        git_ref: &str,
    ) -> BoxFuture<'static, Result<CiStatusSummary, GitHostError>> {
        let this = self.clone();
        let repo = repo.to_string();
        let git_ref = git_ref.to_string();

        Box::pin(async move {
            let path = format!("repos/{repo}/commits/{git_ref}/check-runs");
            let url = this.build_url(&path)?;

            let resp = this
                .client
                .get(url)
                .header("Authorization", this.auth_header())
                .header("User-Agent", "Dandori-Agent")
                .header("Accept", "application/vnd.github+json")
                .send()
                .await
                .map_err(|e| {
                    GitHostError::Network(sanitize_error_message(&e.to_string(), &this.token))
                })?;

            let text = read_limited_text(resp, MAX_RESPONSE_SIZE)
                .await
                .map_err(GitHostError::Other)?;
            let v: Value = serde_json::from_str(&text).unwrap_or(serde_json::json!({}));

            let (success, failure, pending) = v
                .get("check_runs")
                .and_then(|c| c.as_array())
                .map(|runs| parse_check_runs(runs))
                .unwrap_or((0, 0, 0));

            let state = if failure > 0 {
                CiState::Failure
            } else if pending > 0 {
                CiState::Pending
            } else {
                CiState::Success
            };

            Ok(CiStatusSummary {
                state,
                description: format!("{success} passed, {failure} failed, {pending} pending"),
                success_count: success,
                failure_count: failure,
                pending_count: pending,
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
        let repo = repo.to_string();

        Box::pin(async move {
            let path = format!("repos/{repo}/pulls/{number}/merge");
            let url = this.build_url(&path)?;
            let method_str = match method {
                MergeMethod::Merge => "merge",
                MergeMethod::Squash => "squash",
                MergeMethod::Rebase => "rebase",
            };

            let resp = this
                .client
                .put(url)
                .header("Authorization", this.auth_header())
                .header("User-Agent", "Dandori-Agent")
                .header("Accept", "application/vnd.github+json")
                .json(&serde_json::json!({ "merge_method": method_str }))
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
                    "Không thể merge PR do xung đột hoặc chưa đạt điều kiện bắt buộc".to_string(),
                ));
            }

            let v: Value = serde_json::from_str(&text).unwrap_or(serde_json::json!({}));
            let merged = v
                .get("merged")
                .and_then(|m| m.as_bool())
                .unwrap_or(status.is_success());
            let sha = v.get("sha").and_then(|s| s.as_str()).map(String::from);
            let message = v
                .get("message")
                .and_then(|m| m.as_str())
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
