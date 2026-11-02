//! Hiện thực điểm mở rộng `PromptContributor` cho module `skills-md` (P3-08).

use dandori_module_api::{AgentRole, BoxFuture, PromptContributor, PromptPart};
use dandori_types::Task;
use sqlx::SqlitePool;

use crate::db::fetch_active_skills_for_prompt;

/// Chuyển đổi AgentRole sang chuỗi định danh vai trò.
#[must_use]
pub fn role_to_str(role: &AgentRole) -> &'static str {
    match role {
        AgentRole::Lead => "lead",
        AgentRole::Worker => "worker",
        AgentRole::Custom(s) if s == "test" => "test",
        AgentRole::Custom(s) if s == "doc" => "doc",
        AgentRole::Custom(_) => "custom",
    }
}

/// Hiện thực `PromptContributor` đóng góp kỹ năng đã phê duyệt vào prompt của agent.
#[derive(Debug, Clone)]
pub struct SkillsMdContributor {
    pool: Option<SqlitePool>,
}

impl SkillsMdContributor {
    /// Khởi tạo instance mới.
    #[must_use]
    pub fn new(pool: Option<SqlitePool>) -> Self {
        Self { pool }
    }
}

async fn build_skills_prompt(pool: SqlitePool, task: Task, role: AgentRole) -> Vec<PromptPart> {
    let role_key = role_to_str(&role);
    let proj_id = task.project_id.to_string();

    let skills = match fetch_active_skills_for_prompt(&pool, &proj_id, role_key).await {
        Ok(s) => s,
        Err(err) => {
            tracing::warn!("Lỗi nạp kỹ năng cho prompt: {err}");
            return Vec::new();
        }
    };

    if skills.is_empty() {
        return Vec::new();
    }

    let mut lines = Vec::new();
    lines.push("## Kỹ năng Khả dụng (Skills Catalog)".to_string());
    lines.push(
        "Agent có thể tham khảo hoặc áp dụng các kỹ năng chuyên môn sau cho nhiệm vụ này:"
            .to_string(),
    );

    let mut full_embeds = Vec::new();

    for s in skills {
        lines.push(format!("- **{}**: {}", s.name, s.description));
        if s.embed_full && !s.content.is_empty() {
            full_embeds.push((s.name, s.content));
        }
    }

    if !full_embeds.is_empty() {
        lines.push(String::new());
        lines.push("### Hướng dẫn Kỹ năng Chi tiết (Full Skill Instructions)".to_string());
        for (name, content) in full_embeds {
            lines.push(format!("#### Skill `{name}`\n{content}"));
        }
    }

    let rendered = lines.join("\n");
    vec![PromptPart::new("skills-md:catalog", 25, rendered)]
}

impl PromptContributor for SkillsMdContributor {
    fn name(&self) -> &'static str {
        "skills-md"
    }

    fn contribute(&self, task: &Task, role: &AgentRole) -> BoxFuture<'static, Vec<PromptPart>> {
        let pool_opt = self.pool.clone();
        let task_clone = task.clone();
        let role_clone = role.clone();

        Box::pin(async move {
            match pool_opt {
                Some(pool) => build_skills_prompt(pool, task_clone, role_clone).await,
                None => Vec::new(),
            }
        })
    }
}
