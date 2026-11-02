//! Worker dọn dẹp các sự kiện và lần chạy quá hạn lưu trữ (P2-01).

use dandori_module_api::ModuleContext;
use sqlx::SqlitePool;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

/// Thực hiện một lượt dọn dẹp các run_event và run quá hạn lưu trữ.
///
/// Hợp đồng:
/// - Xóa `run_event` có `created_at < cutoff` theo lô nhỏ.
/// - Không xóa `run` đang được tham chiếu bởi task đang mở (`task.status NOT IN ('done', 'cancelled', 'failed')`).
pub async fn cleanup_expired_batch(
    pool: &SqlitePool,
    retention_days: i64,
    batch_size: i64,
) -> Result<(u64, u64), sqlx::Error> {
    let days = retention_days.max(7);
    let cutoff = (chrono::Utc::now() - chrono::Duration::days(days)).to_rfc3339();

    // 1. Xóa run_event quá hạn theo lô nhỏ
    let deleted_events = sqlx::query(
        "DELETE FROM run_event WHERE id IN ( \
             SELECT id FROM run_event \
             WHERE created_at < ? \
             LIMIT ? \
         )",
    )
    .bind(&cutoff)
    .bind(batch_size)
    .execute(pool)
    .await?
    .rows_affected();

    // 2. Xóa run quá hạn theo lô nhỏ CHỈ KHI task của nó đã đóng
    let deleted_runs = sqlx::query(
        "DELETE FROM run WHERE id IN ( \
             SELECT r.id FROM run r \
             JOIN task t ON t.id = r.task_id \
             WHERE r.started_at < ? \
               AND t.status IN ('done', 'cancelled', 'failed') \
             LIMIT ? \
         )",
    )
    .bind(&cutoff)
    .bind(batch_size)
    .execute(pool)
    .await?
    .rows_affected();

    Ok((deleted_events, deleted_runs))
}

/// Chạy vòng lặp worker định kỳ dọn dẹp dữ liệu lịch sử quá hạn.
pub async fn run_retention_worker(ctx: ModuleContext, cancel_token: CancellationToken) {
    let Some(pool) = ctx.pool().cloned() else {
        tracing::warn!("run-history worker: không có database pool, dừng worker");
        return;
    };

    let retention_days = ctx
        .config()
        .get("retentionDays")
        .and_then(|v| v.as_i64())
        .unwrap_or(90)
        .max(7);

    tracing::info!(
        retention_days = retention_days,
        "run-history retention worker khởi động"
    );

    let check_interval = Duration::from_secs(3600); // 1 giờ một lần trong môi trường chạy thật

    loop {
        tokio::select! {
            () = cancel_token.cancelled() => {
                tracing::info!("run-history retention worker nhận tín hiệu dừng");
                break;
            }
            () = tokio::time::sleep(check_interval) => {
                if let Err(err) = cleanup_expired_batch(&pool, retention_days, 500).await {
                    tracing::error!(error = %err, "Lỗi khi dọn dẹp dữ liệu lịch sử quá hạn");
                }
            }
        }
    }
}
