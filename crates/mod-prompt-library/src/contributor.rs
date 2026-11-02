//! Điểm mở rộng PromptContributor cho module prompt-library (P1-07).

use dandori_module_api::{AgentRole, BoxFuture, PromptContributor, PromptPart};
use dandori_types::Task;
use sqlx::{Row, SqlitePool};

/// Chuyển đổi AgentRole sang chuỗi định danh vai trò trong thư viện prompt.
#[must_use]
pub fn role_to_str(role: &AgentRole) -> &'static str {
    match role {
        AgentRole::Lead => "pm",
        AgentRole::Worker => "worker",
        AgentRole::Custom(s) if s == "test" => "test",
        AgentRole::Custom(s) if s == "doc" => "doc",
        AgentRole::Custom(_) => "custom",
    }
}

/// Hiện thực `PromptContributor` đóng góp chỉ thị từ thư viện prompt vào lần chạy agent.
#[derive(Debug, Clone)]
pub struct PromptLibraryContributor {
    pool: Option<SqlitePool>,
}

impl PromptLibraryContributor {
    /// Khởi tạo instance mới.
    #[must_use]
    pub fn new(pool: Option<SqlitePool>) -> Self {
        Self { pool }
    }
}

async fn fetch_and_render_prompts(
    pool: SqlitePool,
    task: Task,
    role: AgentRole,
) -> Vec<PromptPart> {
    let role_key = role_to_str(&role);
    let proj_id_str = task.project_id.to_string();

    let query = "SELECT p.name, p.current_version, v.content, COALESCE(pr.name, '') as project_name
         FROM m_prompt_library_prompt p
         JOIN m_prompt_library_version v ON v.prompt_id = p.id AND v.version = p.current_version
         LEFT JOIN project pr ON pr.id = p.project_id
         WHERE (p.role IS NULL OR p.role = '' OR p.role = ?)
           AND (p.project_id IS NULL OR p.project_id = '' OR p.project_id = ?)
         ORDER BY p.name ASC";

    let rows = match sqlx::query(query)
        .bind(role_key)
        .bind(proj_id_str)
        .fetch_all(&pool)
        .await
    {
        Ok(r) => r,
        Err(err) => {
            tracing::warn!("Lỗi nạp thư viện prompt: {}", err);
            return Vec::new();
        }
    };

    let mut parts = Vec::with_capacity(rows.len());
    for row in rows {
        let name: String = row.get(0);
        let version: i64 = row.get(1);
        let content: String = row.get(2);
        let project_name: String = row.get(3);

        let rendered = crate::template::render_template(
            &content,
            &task.title,
            &task.description,
            &project_name,
            role_key,
        );

        let source = format!("prompt-library:{name}:v{version}");
        parts.push(PromptPart::new(source, 20, rendered));
    }

    parts
}

impl PromptContributor for PromptLibraryContributor {
    fn name(&self) -> &'static str {
        "prompt-library"
    }

    fn contribute(&self, task: &Task, role: &AgentRole) -> BoxFuture<'static, Vec<PromptPart>> {
        let pool_opt = self.pool.clone();
        let task_clone = task.clone();
        let role_clone = role.clone();

        Box::pin(async move {
            match pool_opt {
                Some(pool) => fetch_and_render_prompts(pool, task_clone, role_clone).await,
                None => Vec::new(),
            }
        })
    }
}
