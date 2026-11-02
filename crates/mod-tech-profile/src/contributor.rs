//! Điểm mở rộng PromptContributor cho module tech-profile (P1-08).

use dandori_module_api::{AgentRole, BoxFuture, PromptContributor, PromptPart};
use dandori_types::Task;
use sqlx::{Row, SqlitePool};
use std::fmt::Write as _;

use crate::validation::{MAX_PROFILE_PROMPT_LENGTH, TRUNCATION_MARKER};

/// Cấu trúc mục hồ sơ kỹ thuật đọc từ cơ sở dữ liệu.
#[derive(Debug, Clone)]
pub struct TechProfileRow {
    /// Nhóm phân loại.
    pub category: String,
    /// Khóa thuộc tính.
    pub key: String,
    /// Giá trị cấu hình/quy ước.
    pub value: String,
}

/// Hiện thực `PromptContributor` đóng góp khối hồ sơ kỹ thuật vào mọi lượt chạy của agent.
#[derive(Debug, Clone)]
pub struct TechProfileContributor {
    pool: Option<SqlitePool>,
}

impl TechProfileContributor {
    /// Khởi tạo instance mới.
    #[must_use]
    pub fn new(pool: Option<SqlitePool>) -> Self {
        Self { pool }
    }
}

/// Định dạng danh sách mục hồ sơ thành chuỗi Markdown với thứ tự ổn định và giới hạn độ dài.
#[must_use]
pub fn format_tech_profile(rows: &[TechProfileRow]) -> String {
    if rows.is_empty() {
        return String::new();
    }

    let mut buf = String::with_capacity(1024);
    buf.push_str("### Hồ sơ kỹ thuật dự án (Mọi agent đều đọc)\n");

    let mut current_cat = "";
    for r in rows {
        if r.category != current_cat {
            current_cat = &r.category;
            let _ = writeln!(buf, "\n[{current_cat}]");
        }
        let _ = writeln!(buf, "- {}: {}", r.key, r.value);
    }

    if buf.len() > MAX_PROFILE_PROMPT_LENGTH {
        let mut truncated = buf[..MAX_PROFILE_PROMPT_LENGTH].to_string();
        truncated.push_str(TRUNCATION_MARKER);
        truncated
    } else {
        buf
    }
}

async fn fetch_profile_entries(pool: &SqlitePool, project_id: &str) -> Vec<TechProfileRow> {
    let query = "SELECT category, key, value
         FROM m_tech_profile_entry
         WHERE project_id = ?
         ORDER BY category ASC, sort_order ASC, key ASC";

    let rows = match sqlx::query(query).bind(project_id).fetch_all(pool).await {
        Ok(r) => r,
        Err(err) => {
            tracing::warn!("Lỗi nạp hồ sơ kỹ thuật: {}", err);
            return Vec::new();
        }
    };

    rows.into_iter()
        .map(|r| TechProfileRow {
            category: r.get(0),
            key: r.get(1),
            value: r.get(2),
        })
        .collect()
}

impl PromptContributor for TechProfileContributor {
    fn name(&self) -> &'static str {
        "tech-profile"
    }

    fn contribute(&self, task: &Task, _role: &AgentRole) -> BoxFuture<'static, Vec<PromptPart>> {
        let pool_opt = self.pool.clone();
        let proj_id = task.project_id.to_string();

        Box::pin(async move {
            let pool = match pool_opt {
                Some(p) => p,
                None => return Vec::new(),
            };

            let entries = fetch_profile_entries(&pool, &proj_id).await;
            if entries.is_empty() {
                return Vec::new();
            }

            let formatted = format_tech_profile(&entries);
            let source = format!("tech-profile:project:{proj_id}");
            vec![PromptPart::new(source, 10, formatted)]
        })
    }
}
