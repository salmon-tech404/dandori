//! Thao tác cơ sở dữ liệu SQLite cho module `eta` (P3-07).

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use crate::calculator::calculate_median;
use crate::model::{EtaAccuracyStats, EtaPredictionRecord};

#[derive(sqlx::FromRow)]
struct RunRow {
    started_at: String,
    finished_at: Option<String>,
    agent: String,
    worker_agent: String,
    lead_agent: String,
}

fn parse_duration(started_at: &str, finished_at: Option<&str>) -> Option<f64> {
    let finished_str = finished_at?;
    let start = DateTime::parse_from_rfc3339(started_at).ok()?;
    let end = DateTime::parse_from_rfc3339(finished_str).ok()?;
    let diff = (end - start).num_milliseconds();
    if diff > 0 {
        #[expect(clippy::cast_precision_loss, reason = "Mili-giây chuyển sang giây f64")]
        Some(diff as f64 / 1000.0)
    } else {
        None
    }
}

fn matches_filter(row: &RunRow, role: &str, provider: &str) -> bool {
    let matches_provider =
        row.agent.contains(provider) || provider.contains(&row.agent) || provider == "all";
    if !matches_provider {
        return false;
    }

    match role {
        "worker" => row.agent == row.worker_agent || role == "worker",
        "lead" => row.agent == row.lead_agent,
        _ => true,
    }
}

/// Lấy danh sách thời lượng (giây) của các lần chạy đã hoàn tất thành công cùng (dự án, vai trò, nhà cung cấp).
pub async fn fetch_matching_run_durations(
    pool: &SqlitePool,
    project_id: &str,
    role: &str,
    provider: &str,
) -> Result<Vec<f64>, sqlx::Error> {
    let rows: Vec<RunRow> = sqlx::query_as(
        r#"
        SELECT r.started_at, r.finished_at, r.agent, t.worker_agent, t.lead_agent
        FROM run r
        JOIN task t ON r.task_id = t.id
        WHERE t.project_id = ?
          AND r.status = 'success'
          AND r.finished_at IS NOT NULL
        ORDER BY r.started_at DESC
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    let mut durations = Vec::new();
    for row in rows {
        if matches_filter(&row, role, provider)
            && let Some(dur) = parse_duration(&row.started_at, row.finished_at.as_deref())
        {
            durations.push(dur);
        }
    }

    Ok(durations)
}

/// Lưu một dự đoán ETA vào bảng `m_eta_prediction`.
pub async fn save_prediction(
    pool: &SqlitePool,
    record: &EtaPredictionRecord,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO m_eta_prediction (
            id, workspace_id, project_id, task_id, run_id, role, provider,
            sample_count, predicted_median_secs, predicted_p80_secs, confidence,
            actual_duration_secs, absolute_error_secs, created_at, completed_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            predicted_median_secs = excluded.predicted_median_secs,
            predicted_p80_secs = excluded.predicted_p80_secs,
            confidence = excluded.confidence,
            sample_count = excluded.sample_count
        "#,
    )
    .bind(&record.id)
    .bind(&record.workspace_id)
    .bind(&record.project_id)
    .bind(&record.task_id)
    .bind(&record.run_id)
    .bind(&record.role)
    .bind(&record.provider)
    .bind(record.sample_count)
    .bind(record.predicted_median_secs)
    .bind(record.predicted_p80_secs)
    .bind(&record.confidence)
    .bind(record.actual_duration_secs)
    .bind(record.absolute_error_secs)
    .bind(&record.created_at)
    .bind(&record.completed_at)
    .execute(pool)
    .await?;

    Ok(())
}

/// Cập nhật thời lượng thực tế và tính sai số tuyệt đối khi một lần chạy hoàn tất.
pub async fn record_actual_duration(
    pool: &SqlitePool,
    run_id: &str,
    actual_duration_secs: f64,
) -> Result<bool, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct PredRow {
        id: String,
        predicted_median_secs: f64,
    }

    let maybe_pred: Option<PredRow> =
        sqlx::query_as("SELECT id, predicted_median_secs FROM m_eta_prediction WHERE run_id = ?")
            .bind(run_id)
            .fetch_optional(pool)
            .await?;

    if let Some(pred) = maybe_pred {
        let abs_error = (actual_duration_secs - pred.predicted_median_secs).abs();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE m_eta_prediction
            SET actual_duration_secs = ?,
                absolute_error_secs = ?,
                completed_at = ?
            WHERE id = ?
            "#,
        )
        .bind(actual_duration_secs)
        .bind(abs_error)
        .bind(&now)
        .bind(&pred.id)
        .execute(pool)
        .await?;

        Ok(true)
    } else {
        Ok(false)
    }
}

/// Tính tỷ lệ sai số tuyệt đối trung vị trên thời lượng trung vị của 10 lần chạy gần nhất.
pub async fn get_recent_mae_ratio(
    pool: &SqlitePool,
    project_id: &str,
    role: &str,
    provider: &str,
) -> Result<Option<f64>, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct Row {
        absolute_error_secs: Option<f64>,
        actual_duration_secs: Option<f64>,
    }

    let rows: Vec<Row> = sqlx::query_as(
        r#"
        SELECT absolute_error_secs, actual_duration_secs
        FROM m_eta_prediction
        WHERE project_id = ? AND role = ? AND provider = ?
          AND actual_duration_secs IS NOT NULL
        ORDER BY completed_at DESC
        LIMIT 10
        "#,
    )
    .bind(project_id)
    .bind(role)
    .bind(provider)
    .fetch_all(pool)
    .await?;

    if rows.len() < 3 {
        return Ok(None);
    }

    let mut errors = Vec::new();
    let mut actuals = Vec::new();
    for r in rows {
        if let (Some(err), Some(act)) = (r.absolute_error_secs, r.actual_duration_secs) {
            errors.push(err);
            actuals.push(act);
        }
    }

    let median_err = calculate_median(&errors);
    let median_act = calculate_median(&actuals);

    match (median_err, median_act) {
        (Some(err), Some(act)) if act > 0.0 => Ok(Some(err / act)),
        _ => Ok(None),
    }
}

#[derive(sqlx::FromRow)]
struct AccuracyRow {
    absolute_error_secs: Option<f64>,
    actual_duration_secs: Option<f64>,
}

fn calculate_accuracy_summary(rows: &[AccuracyRow]) -> EtaAccuracyStats {
    let total = rows.len();
    let mut completed_errors = Vec::new();
    let mut completed_actuals = Vec::new();

    for r in rows {
        if let (Some(err), Some(act)) = (r.absolute_error_secs, r.actual_duration_secs) {
            completed_errors.push(err);
            completed_actuals.push(act);
        }
    }

    let completed_count = completed_errors.len();
    let median_error = calculate_median(&completed_errors);
    let mean_error = if completed_count > 0 {
        let sum: f64 = completed_errors.iter().sum();
        #[expect(clippy::cast_precision_loss, reason = "Đếm số phần tử")]
        Some(sum / completed_count as f64)
    } else {
        None
    };

    let median_actual = calculate_median(&completed_actuals);
    let ratio = match (median_error, median_actual) {
        (Some(err), Some(act)) if act > 0.0 => Some(err / act),
        _ => None,
    };

    let is_degraded = ratio.is_some_and(|r| r > 0.5);

    EtaAccuracyStats {
        total_predictions: total,
        completed_predictions: completed_count,
        median_absolute_error_secs: median_error.map(|v| (v * 10.0).round() / 10.0),
        mean_absolute_error_secs: mean_error.map(|v| (v * 10.0).round() / 10.0),
        recent_mae_ratio: ratio.map(|v| (v * 100.0).round() / 100.0),
        is_accuracy_degraded: is_degraded,
    }
}

/// Lấy thống kê độ chính xác tự đo toàn workspace hoặc dự án.
pub async fn get_accuracy_stats(
    pool: &SqlitePool,
    workspace_id: &str,
    project_id: Option<&str>,
) -> Result<EtaAccuracyStats, sqlx::Error> {
    let rows: Vec<AccuracyRow> = if let Some(pid) = project_id {
        sqlx::query_as(
            r#"
            SELECT absolute_error_secs, actual_duration_secs
            FROM m_eta_prediction
            WHERE workspace_id = ? AND project_id = ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(workspace_id)
        .bind(pid)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as(
            r#"
            SELECT absolute_error_secs, actual_duration_secs
            FROM m_eta_prediction
            WHERE workspace_id = ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(workspace_id)
        .fetch_all(pool)
        .await?
    };

    Ok(calculate_accuracy_summary(&rows))
}
