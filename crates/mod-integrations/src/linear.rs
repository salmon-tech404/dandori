//! Adapter kết nối và nhập tác vụ từ Linear GraphQL API.
//! Tài liệu chính thức: <https://developers.linear.app/docs/graphql/working-with-the-graphql-api>

use chrono::Utc;
use reqwest::Client;
use serde_json::{Value, json};
use sqlx::SqlitePool;
use url::Url;

use crate::http_client::{MAX_RESPONSE_SIZE, read_limited_text, sanitize_error_message};
use crate::model::LinearIssue;

/// Client thao tác với Linear GraphQL API.
#[derive(Clone, Debug)]
pub struct LinearClient {
    client: Client,
    endpoint: Url,
    token: String,
}

impl LinearClient {
    /// Khởi tạo một `LinearClient` mới.
    #[must_use]
    pub fn new(client: Client, endpoint: Url, token: String) -> Self {
        Self {
            client,
            endpoint,
            token,
        }
    }

    async fn post_graphql(&self, query: &str, variables: Value) -> Result<Value, String> {
        let body = json!({
            "query": query,
            "variables": variables,
        });

        let resp = self
            .client
            .post(self.endpoint.clone())
            .header("Authorization", &self.token)
            .header("User-Agent", "Dandori-Agent")
            .json(&body)
            .send()
            .await
            .map_err(|e| sanitize_error_message(&e.to_string(), &self.token))?;

        let status = resp.status();
        let text = read_limited_text(resp, MAX_RESPONSE_SIZE).await?;

        if !status.is_success() {
            return Err(format!(
                "Linear API trả về mã lỗi {status}: {}",
                sanitize_error_message(&text, &self.token)
            ));
        }

        let v: Value = serde_json::from_str(&text)
            .map_err(|e| format!("Lỗi parse JSON Linear GraphQL: {e}"))?;

        if let Some(first_err) = v
            .get("errors")
            .and_then(|e| e.as_array())
            .and_then(|arr| arr.first())
        {
            let msg = first_err
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("GraphQL Error");
            return Err(sanitize_error_message(msg, &self.token));
        }

        let data = v.get("data").cloned().unwrap_or(serde_json::Value::Null);
        Ok(data)
    }

    /// Kiểm tra kết nối tới Linear qua truy vấn `viewer`.
    pub async fn test_connection(&self) -> Result<(), String> {
        let query = "query { viewer { id name } }";
        self.post_graphql(query, json!({})).await.map(|_| ())
    }

    /// Lấy danh sách các issue gần nhất từ Linear.
    pub async fn list_issues(&self) -> Result<Vec<LinearIssue>, String> {
        let query = r#"
            query {
                issues(first: 50) {
                    nodes {
                        id
                        identifier
                        title
                        description
                        url
                        state {
                            name
                        }
                    }
                }
            }
        "#;

        let data = self.post_graphql(query, json!({})).await?;
        Ok(parse_linear_issues(&data))
    }

    /// Nhập các issue từ Linear thành task trong backlog của dự án (tránh trùng lặp).
    pub async fn import_issues(
        &self,
        pool: &SqlitePool,
        workspace_id: &str,
        project_id: &str,
        target_issue_ids: &[String],
    ) -> Result<usize, String> {
        let project_ws: Option<String> =
            sqlx::query_scalar("SELECT workspace_id FROM project WHERE id = ?")
                .bind(project_id)
                .fetch_optional(pool)
                .await
                .map_err(|e| format!("Lỗi kiểm tra dự án: {e}"))?;
        if project_ws.as_deref() != Some(workspace_id) {
            return Err("Dự án không thuộc workspace của kết nối".to_string());
        }

        let issues = self.list_issues().await?;
        let mut imported_count = 0;

        for issue in issues {
            if !target_issue_ids.is_empty() && !target_issue_ids.contains(&issue.id) {
                continue;
            }

            // Kiểm tra trùng lặp
            let exists: (i64,) = sqlx::query_as(
                "SELECT count(*) FROM m_integration_issue_link \
                 WHERE workspace_id = ? AND project_id = ? AND issue_id = ?",
            )
            .bind(workspace_id)
            .bind(project_id)
            .bind(&issue.id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Lỗi kiểm tra trùng lặp issue: {e}"))?;

            if exists.0 > 0 {
                continue;
            }

            Self::create_task_and_link(pool, workspace_id, project_id, &issue).await?;
            imported_count += 1;
        }

        Ok(imported_count)
    }

    async fn create_task_and_link(
        pool: &SqlitePool,
        ws_id: &str,
        prj_id: &str,
        issue: &LinearIssue,
    ) -> Result<(), String> {
        let now = Utc::now().to_rfc3339();
        let task_id = uuid::Uuid::new_v4().to_string();
        let link_id = uuid::Uuid::new_v4().to_string();
        let task_title = format!("[{}] {}", issue.identifier, issue.title);
        let task_desc = issue.description.clone().unwrap_or_default();

        sqlx::query(
            "INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) \
             VALUES (?, ?, ?, ?, 'backlog', 'lead', 'worker', ?, ?)",
        )
        .bind(&task_id)
        .bind(prj_id)
        .bind(&task_title)
        .bind(&task_desc)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await
        .map_err(|e| format!("Lỗi tạo task từ Linear: {e}"))?;

        sqlx::query(
            "INSERT INTO m_integration_issue_link \
             (id, workspace_id, project_id, task_id, provider, issue_id, issue_key, issue_title, issue_url, created_at) \
             VALUES (?, ?, ?, ?, 'linear', ?, ?, ?, ?, ?)",
        )
        .bind(&link_id)
        .bind(ws_id)
        .bind(prj_id)
        .bind(&task_id)
        .bind(&issue.id)
        .bind(&issue.identifier)
        .bind(&issue.title)
        .bind(&issue.url)
        .bind(&now)
        .execute(pool)
        .await
        .map_err(|e| format!("Lỗi tạo bản ghi liên kết issue: {e}"))?;

        Ok(())
    }
}

fn parse_linear_issues(data: &Value) -> Vec<LinearIssue> {
    let nodes = data
        .get("issues")
        .and_then(|i| i.get("nodes"))
        .and_then(|n| n.as_array());
    let mut issues = Vec::new();

    if let Some(arr) = nodes {
        for item in arr {
            let id = item
                .get("id")
                .and_then(|i| i.as_str())
                .unwrap_or_default()
                .to_string();
            let identifier = item
                .get("identifier")
                .and_then(|i| i.as_str())
                .unwrap_or_default()
                .to_string();
            let title = item
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or_default()
                .to_string();
            let description = item
                .get("description")
                .and_then(|d| d.as_str())
                .map(String::from);
            let url = item
                .get("url")
                .and_then(|u| u.as_str())
                .unwrap_or_default()
                .to_string();
            let state_name = item
                .get("state")
                .and_then(|s| s.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("Todo")
                .to_string();

            if !id.is_empty() {
                issues.push(LinearIssue {
                    id,
                    identifier,
                    title,
                    description,
                    url,
                    state_name,
                });
            }
        }
    }
    issues
}
