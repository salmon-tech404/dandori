//! Bộ quét và đồng bộ các tệp `.cron/jobs/*.md` trong kho mã nguồn dự án (RS-SEC-004).

#![expect(
    clippy::too_many_lines,
    reason = "Đồng bộ thư mục và phân loại trạng thái CSDL"
)]

use chrono::Utc;
use sqlx::SqlitePool;
use std::fs;
use std::path::Path;
use tracing::{info, warn};
use uuid::Uuid;

use crate::frontmatter::parse_job_markdown;
use crate::model::MdJobRecord;

/// Kích thước tối đa của một tệp job Markdown (64 KiB).
const MAX_JOB_FILE_BYTES: u64 = 64 * 1024;

/// Quét thư mục `.cron/jobs` của một dự án và đồng bộ vào bảng `m_md_job`.
///
/// # Errors
/// Trả về lỗi chuỗi nếu repo_path không tồn tại hoặc lỗi truy vấn CSDL.
pub async fn sync_project_jobs(
    pool: &SqlitePool,
    workspace_id: &str,
    project_id: &str,
    repo_path_str: &str,
) -> Result<usize, String> {
    let repo_path = Path::new(repo_path_str);
    if !repo_path.exists() || !repo_path.is_dir() {
        return Err(format!("Đường dẫn repo không hợp lệ: {repo_path_str}"));
    }

    let canonical_repo = fs::canonicalize(repo_path)
        .map_err(|e| format!("Không thể chuẩn hóa đường dẫn repo: {e}"))?;

    let jobs_dir = repo_path.join(".cron").join("jobs");
    if !jobs_dir.exists() {
        // Thư mục .cron/jobs không tồn tại -> không có job nào để quét
        return Ok(0);
    }

    // `.cron` hoặc `.cron/jobs` có thể là symlink/junction trong kho không đáng tin: thư mục thật phải nằm trong repo.
    let canonical_jobs = fs::canonicalize(&jobs_dir)
        .map_err(|e| format!("Không thể chuẩn hóa thư mục jobs: {e}"))?;
    if !canonical_jobs.starts_with(&canonical_repo) {
        return Err("Thư mục .cron/jobs trỏ ra ngoài repo, bị từ chối".to_string());
    }

    let entries = fs::read_dir(&jobs_dir)
        .map_err(|e| format!("Không thể đọc thư mục jobs {}: {e}", jobs_dir.display()))?;

    let mut scanned_count = 0;
    let mut found_relative_paths: Vec<String> = Vec::new();

    for entry_res in entries {
        let entry = match entry_res {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }

        // RS-SEC-004: Kiểm tra symlink và bảo vệ chống Path Traversal
        // Nếu là symlink, kiểm tra đích đến thực tế có nằm trong canonical_repo không
        let is_symlink = fs::symlink_metadata(&path)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false);
        if is_symlink {
            if let Ok(target_canonical) = fs::canonicalize(&path) {
                if !target_canonical.starts_with(&canonical_repo) {
                    warn!(
                        "Bỏ qua symlink trỏ ra ngoài repo: {} -> {}",
                        path.display(),
                        target_canonical.display()
                    );
                    continue;
                }
            } else {
                warn!("Bỏ qua symlink bị lỗi: {}", path.display());
                continue;
            }
        }

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.md");
        let rel_path = format!(".cron/jobs/{file_name}");
        found_relative_paths.push(rel_path.clone());

        // Đọc nội dung tệp với giới hạn kích thước an toàn
        if fs::metadata(&path).map_or(true, |m| m.len() > MAX_JOB_FILE_BYTES) {
            warn!(
                "Bỏ qua tệp job quá lớn hoặc không đọc được: {}",
                path.display()
            );
            continue;
        }
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                warn!("Không thể đọc tệp {}: {e}", path.display());
                continue;
            }
        };

        process_single_job_file(
            pool,
            workspace_id,
            project_id,
            &rel_path,
            file_name,
            &content,
        )
        .await?;

        scanned_count += 1;
    }

    // Xử lý các job trong CSDL mà tệp đã bị xóa khỏi đĩa: xóa bỏ khỏi CSDL
    let existing_jobs: Vec<MdJobRecord> = sqlx::query_as(
        "SELECT id, workspace_id, project_id, file_path, job_name, schedule_cron, \
         agent_role, account_id, prompt, content_hash, approved_hash, status, \
         next_fire, last_fire, created_at, updated_at \
         FROM m_md_job WHERE project_id = ?",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    for job in existing_jobs {
        if !found_relative_paths.contains(&job.file_path) {
            let _ = sqlx::query("DELETE FROM m_md_job WHERE id = ?")
                .bind(&job.id)
                .execute(pool)
                .await;
            info!(
                "Đã xóa job {} do tệp {} không còn tồn tại",
                job.id, job.file_path
            );
        }
    }

    Ok(scanned_count)
}

/// Xử lý nội dung của một tệp Markdown và cập nhật vào CSDL.
async fn process_single_job_file(
    pool: &SqlitePool,
    workspace_id: &str,
    project_id: &str,
    file_path: &str,
    file_name: &str,
    content: &str,
) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();

    // 1. Kiểm tra bản ghi hiện tại trong DB
    let existing: Option<MdJobRecord> = sqlx::query_as(
        "SELECT id, workspace_id, project_id, file_path, job_name, schedule_cron, \
         agent_role, account_id, prompt, content_hash, approved_hash, status, \
         next_fire, last_fire, created_at, updated_at \
         FROM m_md_job WHERE project_id = ? AND file_path = ?",
    )
    .bind(project_id)
    .bind(file_path)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    // 2. Phân tích nội dung Markdown
    match parse_job_markdown(content) {
        Ok(parsed) => {
            if let Some(record) = existing {
                // Tệp đã tồn tại: so sánh mã băm
                if record.content_hash == parsed.content_hash {
                    // Nội dung không đổi, giữ nguyên trạng thái
                    return Ok(());
                }

                // Tiêu chí AC 2: Tệp bị sửa đổi sau khi duyệt -> quay về 'pending_approval'
                // và xóa next_fire để không tự động chạy bản mới cho đến khi được duyệt lại
                let new_status = if let Some(ref approved) = record.approved_hash {
                    if approved == &parsed.content_hash {
                        "approved"
                    } else {
                        "pending_approval"
                    }
                } else {
                    "pending_approval"
                };

                let next_fire_val: Option<String> = if new_status == "approved" {
                    record.next_fire
                } else {
                    None
                };

                sqlx::query(
                    r#"
                    UPDATE m_md_job
                    SET job_name = ?, schedule_cron = ?, agent_role = ?, account_id = ?,
                        prompt = ?, content_hash = ?, status = ?, next_fire = ?, updated_at = ?
                    WHERE id = ?
                    "#,
                )
                .bind(&parsed.name)
                .bind(&parsed.schedule)
                .bind(&parsed.agent)
                .bind(&parsed.account)
                .bind(&parsed.prompt)
                .bind(&parsed.content_hash)
                .bind(new_status)
                .bind(&next_fire_val)
                .bind(&now)
                .bind(&record.id)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;
            } else {
                // Tiêu chí AC 1: Job mới phát hiện -> tạo với 'pending_approval', KHÔNG CHẠY cho tới khi duyệt
                let job_id = Uuid::new_v4().to_string();
                sqlx::query(
                    r#"
                    INSERT INTO m_md_job (
                        id, workspace_id, project_id, file_path, job_name, schedule_cron,
                        agent_role, account_id, prompt, content_hash, approved_hash,
                        status, next_fire, last_fire, created_at, updated_at
                    )
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, 'pending_approval', NULL, NULL, ?, ?)
                    "#,
                )
                .bind(&job_id)
                .bind(workspace_id)
                .bind(project_id)
                .bind(file_path)
                .bind(&parsed.name)
                .bind(&parsed.schedule)
                .bind(&parsed.agent)
                .bind(&parsed.account)
                .bind(&parsed.prompt)
                .bind(&parsed.content_hash)
                .bind(&now)
                .bind(&now)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }
        Err(parse_err) => {
            // Tiêu chí AC 3: Lỗi cú pháp/trường lạ/quá lớn -> ghi nhận 'syntax_error' an toàn, không panic
            let err_msg = parse_err.to_string();
            let dummy_hash = "syntax_error";

            if let Some(record) = existing {
                sqlx::query(
                    r#"
                    UPDATE m_md_job
                    SET prompt = ?, content_hash = ?, status = 'syntax_error', next_fire = NULL, updated_at = ?
                    WHERE id = ?
                    "#,
                )
                .bind(&err_msg)
                .bind(dummy_hash)
                .bind(&now)
                .bind(&record.id)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;
            } else {
                let job_id = Uuid::new_v4().to_string();
                sqlx::query(
                    r#"
                    INSERT INTO m_md_job (
                        id, workspace_id, project_id, file_path, job_name, schedule_cron,
                        agent_role, account_id, prompt, content_hash, approved_hash,
                        status, next_fire, last_fire, created_at, updated_at
                    )
                    VALUES (?, ?, ?, ?, ?, '', '', NULL, ?, ?, NULL, 'syntax_error', NULL, NULL, ?, ?)
                    "#,
                )
                .bind(&job_id)
                .bind(workspace_id)
                .bind(project_id)
                .bind(file_path)
                .bind(file_name)
                .bind(&err_msg)
                .bind(dummy_hash)
                .bind(&now)
                .bind(&now)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(())
}
