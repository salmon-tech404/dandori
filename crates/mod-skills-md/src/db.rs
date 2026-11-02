//! Các hàm thao tác cơ sở dữ liệu SQLite cho `skills-md` (P3-08).

#![expect(
    clippy::too_many_lines,
    reason = "Thao tác truy vấn và cập nhật bảng SQLite cho kỹ năng"
)]

use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::model::{ParsedSkill, SkillBinding, SkillDetail, SkillStatus, SkillSummary};

#[derive(sqlx::FromRow)]
struct SkillRow {
    id: String,
    workspace_id: String,
    name: String,
    description: String,
    version: Option<String>,
    author: Option<String>,
    tags: Option<String>,
    tools: Option<String>,
    content: String,
    content_hash: String,
    approved_hash: Option<String>,
    has_scripts: i64,
    script_files: Option<String>,
    status: String,
    created_at: String,
    updated_at: String,
}

#[derive(sqlx::FromRow)]
struct BindingRow {
    id: String,
    workspace_id: String,
    skill_id: String,
    project_id: Option<String>,
    role: Option<String>,
    embed_full: i64,
    created_at: String,
}

fn parse_json_list(val: Option<String>) -> Vec<String> {
    val.and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// Lưu hoặc cập nhật một kỹ năng vào cơ sở dữ liệu.
pub async fn upsert_skill(
    pool: &SqlitePool,
    workspace_id: &str,
    skill: &ParsedSkill,
) -> Result<String, sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    let tags_json = serde_json::to_string(&skill.tags).unwrap_or_else(|_| "[]".to_string());
    let tools_json = serde_json::to_string(&skill.tools).unwrap_or_else(|_| "[]".to_string());
    let script_files_json =
        serde_json::to_string(&skill.script_files).unwrap_or_else(|_| "[]".to_string());

    #[derive(sqlx::FromRow)]
    struct ExistingRow {
        id: String,
        approved_hash: Option<String>,
    }

    let existing: Option<ExistingRow> = sqlx::query_as(
        "SELECT id, approved_hash FROM m_skills_md_skill WHERE workspace_id = ? AND name = ?",
    )
    .bind(workspace_id)
    .bind(&skill.name)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = existing {
        // Cập nhật: nếu băm thay đổi so với approved_hash -> quay về pending_approval
        let status = if row.approved_hash.as_deref() == Some(&skill.content_hash) {
            SkillStatus::Approved.as_str()
        } else {
            SkillStatus::PendingApproval.as_str()
        };

        sqlx::query(
            r#"
            UPDATE m_skills_md_skill
            SET description = ?,
                version = ?,
                author = ?,
                tags = ?,
                tools = ?,
                content = ?,
                content_hash = ?,
                has_scripts = ?,
                script_files = ?,
                status = ?,
                updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&skill.description)
        .bind(&skill.version)
        .bind(&skill.author)
        .bind(&tags_json)
        .bind(&tools_json)
        .bind(&skill.content)
        .bind(&skill.content_hash)
        .bind(i64::from(skill.has_scripts))
        .bind(&script_files_json)
        .bind(status)
        .bind(&now)
        .bind(&row.id)
        .execute(pool)
        .await?;

        Ok(row.id)
    } else {
        let new_id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO m_skills_md_skill (
                id, workspace_id, name, description, version, author, tags, tools,
                content, content_hash, approved_hash, has_scripts, script_files,
                status, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, ?, ?, 'pending_approval', ?, ?)
            "#,
        )
        .bind(&new_id)
        .bind(workspace_id)
        .bind(&skill.name)
        .bind(&skill.description)
        .bind(&skill.version)
        .bind(&skill.author)
        .bind(&tags_json)
        .bind(&tools_json)
        .bind(&skill.content)
        .bind(&skill.content_hash)
        .bind(i64::from(skill.has_scripts))
        .bind(&script_files_json)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await?;

        Ok(new_id)
    }
}

/// Lấy danh sách tóm tắt toàn bộ kỹ năng trong workspace.
pub async fn list_skills(
    pool: &SqlitePool,
    workspace_id: &str,
) -> Result<Vec<SkillSummary>, sqlx::Error> {
    let rows: Vec<SkillRow> = sqlx::query_as(
        r#"
        SELECT id, workspace_id, name, description, version, author, tags, tools,
               content, content_hash, approved_hash, has_scripts, script_files,
               status, created_at, updated_at
        FROM m_skills_md_skill
        WHERE workspace_id = ?
        ORDER BY name ASC
        "#,
    )
    .bind(workspace_id)
    .fetch_all(pool)
    .await?;

    let mut summaries = Vec::with_capacity(rows.len());
    for r in rows {
        let count_row: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM m_skills_md_binding WHERE skill_id = ?")
                .bind(&r.id)
                .fetch_one(pool)
                .await?;

        let is_matching = r.approved_hash.as_deref() == Some(&r.content_hash);
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "Số lượng binding nhỏ"
        )]
        summaries.push(SkillSummary {
            id: r.id,
            workspace_id: r.workspace_id,
            name: r.name,
            description: r.description,
            version: r.version,
            author: r.author,
            tags: parse_json_list(r.tags),
            has_scripts: r.has_scripts > 0,
            status: SkillStatus::parse(&r.status),
            content_hash: r.content_hash,
            is_hash_matching_approved: is_matching,
            binding_count: count_row.0 as usize,
            created_at: r.created_at,
            updated_at: r.updated_at,
        });
    }

    Ok(summaries)
}

/// Lấy chi tiết đầy đủ một kỹ năng kèm các gắn kết.
pub async fn get_skill(
    pool: &SqlitePool,
    workspace_id: &str,
    skill_id: &str,
) -> Result<Option<SkillDetail>, sqlx::Error> {
    let maybe_row: Option<SkillRow> = sqlx::query_as(
        r#"
        SELECT id, workspace_id, name, description, version, author, tags, tools,
               content, content_hash, approved_hash, has_scripts, script_files,
               status, created_at, updated_at
        FROM m_skills_md_skill
        WHERE workspace_id = ? AND id = ?
        "#,
    )
    .bind(workspace_id)
    .bind(skill_id)
    .fetch_optional(pool)
    .await?;

    let Some(r) = maybe_row else {
        return Ok(None);
    };

    let b_rows: Vec<BindingRow> = sqlx::query_as(
        r#"
        SELECT id, workspace_id, skill_id, project_id, role, embed_full, created_at
        FROM m_skills_md_binding
        WHERE skill_id = ?
        ORDER BY created_at ASC
        "#,
    )
    .bind(skill_id)
    .fetch_all(pool)
    .await?;

    let bindings = b_rows
        .into_iter()
        .map(|b| SkillBinding {
            id: b.id,
            workspace_id: b.workspace_id,
            skill_id: b.skill_id,
            project_id: b.project_id,
            role: b.role,
            embed_full: b.embed_full > 0,
            created_at: b.created_at,
        })
        .collect();

    Ok(Some(SkillDetail {
        id: r.id,
        workspace_id: r.workspace_id,
        name: r.name,
        description: r.description,
        version: r.version,
        author: r.author,
        tags: parse_json_list(r.tags),
        tools: parse_json_list(r.tools),
        content: r.content,
        content_hash: r.content_hash,
        approved_hash: r.approved_hash,
        has_scripts: r.has_scripts > 0,
        script_files: parse_json_list(r.script_files),
        status: SkillStatus::parse(&r.status),
        bindings,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }))
}

/// Phê duyệt kỹ năng (chỉ Quản trị viên).
pub async fn approve_skill(
    pool: &SqlitePool,
    workspace_id: &str,
    skill_id: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE m_skills_md_skill
        SET status = 'approved',
            approved_hash = content_hash,
            updated_at = ?
        WHERE workspace_id = ? AND id = ?
        "#,
    )
    .bind(Utc::now().to_rfc3339())
    .bind(workspace_id)
    .bind(skill_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Xóa kỹ năng khỏi workspace (cascade bindings).
pub async fn delete_skill(
    pool: &SqlitePool,
    workspace_id: &str,
    skill_id: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM m_skills_md_skill WHERE workspace_id = ? AND id = ?")
        .bind(workspace_id)
        .bind(skill_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Tạo gắn kết kỹ năng với dự án hoặc vai trò.
pub async fn create_binding(
    pool: &SqlitePool,
    workspace_id: &str,
    skill_id: &str,
    project_id: Option<&str>,
    role: Option<&str>,
    embed_full: bool,
) -> Result<String, sqlx::Error> {
    let new_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO m_skills_md_binding (
            id, workspace_id, skill_id, project_id, role, embed_full, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(skill_id, project_id, role) DO UPDATE SET
            embed_full = excluded.embed_full
        "#,
    )
    .bind(&new_id)
    .bind(workspace_id)
    .bind(skill_id)
    .bind(project_id)
    .bind(role)
    .bind(i64::from(embed_full))
    .bind(&now)
    .execute(pool)
    .await?;

    Ok(new_id)
}

/// Xóa gắn kết kỹ năng.
pub async fn delete_binding(
    pool: &SqlitePool,
    workspace_id: &str,
    binding_id: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM m_skills_md_binding WHERE workspace_id = ? AND id = ?")
        .bind(workspace_id)
        .bind(binding_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Thông tin kỹ năng khả dụng để đóng góp vào prompt.
#[derive(Debug, Clone)]
pub struct ActivePromptSkill {
    pub name: String,
    pub description: String,
    pub content: String,
    pub embed_full: bool,
}

/// Lấy danh sách kỹ năng đã duyệt và hợp lệ gắn với dự án/vai trò cho prompt.
pub async fn fetch_active_skills_for_prompt(
    pool: &SqlitePool,
    project_id: &str,
    role: &str,
) -> Result<Vec<ActivePromptSkill>, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct Row {
        name: String,
        description: String,
        content: String,
        embed_full: i64,
    }

    let rows: Vec<Row> = sqlx::query_as(
        r#"
        SELECT s.name, s.description, s.content, MAX(b.embed_full) as embed_full
        FROM m_skills_md_binding b
        JOIN m_skills_md_skill s ON b.skill_id = s.id
        WHERE s.status = 'approved'
          AND s.approved_hash IS NOT NULL
          AND s.approved_hash = s.content_hash
          AND (b.project_id IS NULL OR b.project_id = '' OR b.project_id = ?)
          AND (b.role IS NULL OR b.role = '' OR b.role = ?)
        GROUP BY s.id
        ORDER BY s.name ASC
        "#,
    )
    .bind(project_id)
    .bind(role)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ActivePromptSkill {
            name: r.name,
            description: r.description,
            content: r.content,
            embed_full: r.embed_full > 0,
        })
        .collect())
}
