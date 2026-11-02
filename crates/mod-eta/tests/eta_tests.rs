//! Các bài kiểm thử đơn vị cho module `eta` (P3-07).

#![expect(clippy::unwrap_used, reason = "Kiểm thử đơn vị dùng unwrap")]

use chrono::Utc;
use mod_eta::calculator::{calculate_median, calculate_p80, evaluate_eta};
use mod_eta::db::{
    fetch_matching_run_durations, get_accuracy_stats, record_actual_duration, save_prediction,
};
use mod_eta::model::{EtaConfidence, EtaPredictionRecord};
use sqlx::SqlitePool;

async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    let core_schema = include_str!("../../../crates/dandori-db/migrations/0001_core_schema.sql");
    sqlx::query(core_schema).execute(&pool).await.unwrap();

    let eta_schema = include_str!("../migrations/0001_eta_schema.sql");
    sqlx::query(eta_schema).execute(&pool).await.unwrap();

    pool
}

#[test]
fn test_ac1_median_and_p80_calculation_even_and_odd() {
    // 1. Tập mẫu lẻ (5 phần tử): [10, 20, 30, 40, 50]
    let odd_samples = vec![50.0, 10.0, 40.0, 20.0, 30.0];
    let med_odd = calculate_median(&odd_samples).unwrap();
    assert_eq!(med_odd, 30.0);

    let p80_odd = calculate_p80(&odd_samples).unwrap();
    // rank = 0.8 * 4 = 3.2 -> low=3 (40), high=4 (50), weight=0.2 -> 40 + 10*0.2 = 42.0
    assert_eq!(p80_odd, 42.0);

    // 2. Tập mẫu chẵn (6 phần tử): [10, 20, 30, 40, 50, 60]
    let even_samples = vec![60.0, 10.0, 50.0, 20.0, 40.0, 30.0];
    let med_even = calculate_median(&even_samples).unwrap();
    assert_eq!(med_even, 35.0);

    let p80_even = calculate_p80(&even_samples).unwrap();
    // rank = 0.8 * 5 = 4.0 -> low=4 (50), high=4 (50) -> 50.0
    assert_eq!(p80_even, 50.0);
}

#[test]
fn test_ac2_under_sample_threshold_refuses_estimate() {
    // Dưới 5 mẫu (ví dụ 4 mẫu)
    let samples = vec![100.0, 120.0, 110.0, 115.0];
    let res = evaluate_eta("t-1", Some("r-1"), &samples, None, Some(Utc::now()));

    assert!(!res.has_estimate);
    assert_eq!(res.sample_count, 4);
    assert!(res.median_secs.is_none());
    assert!(res.p80_secs.is_none());
    assert!(res.confidence.is_none());
    assert!(res.eta_range_text.is_none());
    assert!(res.predicted_completion_median.is_none());

    let reason = res.reason.expect("Phải có lý do từ chối");
    assert!(reason.contains("chưa đủ dữ liệu"));
    assert!(reason.contains("4/5"));
}

#[test]
fn test_ac3_self_measurement_and_confidence_downgrade() {
    // Đủ 10 mẫu -> độ tin cậy ban đầu Medium
    let samples = vec![
        100.0, 105.0, 110.0, 108.0, 112.0, 115.0, 107.0, 109.0, 111.0, 114.0,
    ];

    // Trường hợp 1: Không có sai số lịch sử hoặc sai số nhỏ (< 50%) -> giữ nguyên Medium
    let res_ok = evaluate_eta("t-1", Some("r-1"), &samples, Some(0.15), Some(Utc::now()));
    assert!(res_ok.has_estimate);
    assert_eq!(res_ok.confidence, Some(EtaConfidence::Medium));
    assert!(!res_ok.confidence_downgraded);

    // Trường hợp 2: Sai số lịch sử gần đây lớn (> 50%, ví dụ 65%) -> tự động hạ bậc xuống Low
    let res_degraded = evaluate_eta("t-1", Some("r-1"), &samples, Some(0.65), Some(Utc::now()));
    assert!(res_degraded.has_estimate);
    assert_eq!(res_degraded.confidence, Some(EtaConfidence::Low));
    assert!(res_degraded.confidence_downgraded);
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "Test kịch bản lọc dự án, vai trò, nhà cung cấp và loại trừ lỗi"
)]
async fn test_ac4_filter_by_project_role_provider_and_exclude_failures() {
    let pool = setup_test_db().await;

    // Thiết lập dữ liệu: workspace và project
    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES ('ws-1', 'WS 1', '2026-11-02T00:00:00Z')")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES ('prj-1', 'ws-1', 'Prj 1', '/tmp', 'main', '2026-11-02T00:00:00Z')")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES ('prj-2', 'ws-1', 'Prj 2', '/tmp', 'main', '2026-11-02T00:00:00Z')")
        .execute(&pool)
        .await
        .unwrap();

    // 1. Task hợp lệ thuộc prj-1
    for i in 1..=5 {
        let task_id = format!("t-valid-{i}");
        let run_id = format!("r-valid-{i}");
        let start = "2026-11-02T10:00:00Z";
        let end = format!("2026-11-02T10:0{i}:00Z"); // 1, 2, 3, 4, 5 phút = 60s, 120s, 180s, 240s, 300s

        sqlx::query(
            "INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) VALUES (?, 'prj-1', 'Task', 'Desc', 'done', 'lead', 'claude', '2026-11-02T00:00:00Z', '2026-11-02T00:00:00Z')"
        )
        .bind(&task_id)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO run (id, task_id, agent, account, status, started_at, finished_at, exit_code) VALUES (?, ?, 'claude', 'acc-1', 'success', ?, ?, 0)"
        )
        .bind(&run_id)
        .bind(&task_id)
        .bind(start)
        .bind(&end)
        .execute(&pool)
        .await
        .unwrap();
    }

    // 2. Run thuộc dự án khác (prj-2) -> không tính
    sqlx::query("INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) VALUES ('t-other-prj', 'prj-2', 'Task', 'Desc', 'done', 'lead', 'claude', '2026-11-02T00:00:00Z', '2026-11-02T00:00:00Z')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO run (id, task_id, agent, account, status, started_at, finished_at, exit_code) VALUES ('r-other-prj', 't-other-prj', 'claude', 'acc-1', 'success', '2026-11-02T10:00:00Z', '2026-11-02T10:10:00Z', 0)")
        .execute(&pool)
        .await
        .unwrap();

    // 3. Run bị failed -> không tính
    sqlx::query("INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) VALUES ('t-failed', 'prj-1', 'Task', 'Desc', 'failed', 'lead', 'claude', '2026-11-02T00:00:00Z', '2026-11-02T00:00:00Z')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO run (id, task_id, agent, account, status, started_at, finished_at, exit_code) VALUES ('r-failed', 't-failed', 'claude', 'acc-1', 'failed', '2026-11-02T10:00:00Z', '2026-11-02T10:02:00Z', 1)")
        .execute(&pool)
        .await
        .unwrap();

    // 4. Run bị cancelled -> không tính
    sqlx::query("INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) VALUES ('t-cancel', 'prj-1', 'Task', 'Desc', 'cancelled', 'lead', 'claude', '2026-11-02T00:00:00Z', '2026-11-02T00:00:00Z')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO run (id, task_id, agent, account, status, started_at, finished_at, exit_code) VALUES ('r-cancel', 't-cancel', 'claude', 'acc-1', 'cancelled', '2026-11-02T10:00:00Z', '2026-11-02T10:01:00Z', 130)")
        .execute(&pool)
        .await
        .unwrap();

    // 5. Run dùng provider khác (codex) -> không tính vào mẫu của claude
    sqlx::query("INSERT INTO task (id, project_id, title, description, status, lead_agent, worker_agent, created_at, updated_at) VALUES ('t-other-prov', 'prj-1', 'Task', 'Desc', 'done', 'lead', 'codex', '2026-11-02T00:00:00Z', '2026-11-02T00:00:00Z')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO run (id, task_id, agent, account, status, started_at, finished_at, exit_code) VALUES ('r-other-prov', 't-other-prov', 'codex', 'acc-2', 'success', '2026-11-02T10:00:00Z', '2026-11-02T10:05:00Z', 0)")
        .execute(&pool)
        .await
        .unwrap();

    let durations = fetch_matching_run_durations(&pool, "prj-1", "worker", "claude")
        .await
        .unwrap();

    // Chỉ đúng 5 lần chạy hợp lệ được lọc vào mẫu
    assert_eq!(durations.len(), 5);
    assert_eq!(durations, vec![60.0, 120.0, 180.0, 240.0, 300.0]);
}

#[tokio::test]
async fn test_ac5_accuracy_stats_tracking() {
    let pool = setup_test_db().await;

    // Lưu một dự đoán
    let pred = EtaPredictionRecord {
        id: "pred-1".into(),
        workspace_id: "ws-1".into(),
        project_id: "prj-1".into(),
        task_id: "t-1".into(),
        run_id: "r-1".into(),
        role: "worker".into(),
        provider: "claude".into(),
        sample_count: 5,
        predicted_median_secs: 180.0,
        predicted_p80_secs: 240.0,
        confidence: "low".into(),
        actual_duration_secs: None,
        absolute_error_secs: None,
        created_at: Utc::now().to_rfc3339(),
        completed_at: None,
    };
    save_prediction(&pool, &pred).await.unwrap();

    // Ghi nhận hoàn thành thực tế với thời lượng 200s (sai số |200 - 180| = 20s)
    let updated = record_actual_duration(&pool, "r-1", 200.0).await.unwrap();
    assert!(updated);

    let stats = get_accuracy_stats(&pool, "ws-1", Some("prj-1"))
        .await
        .unwrap();
    assert_eq!(stats.total_predictions, 1);
    assert_eq!(stats.completed_predictions, 1);
    assert_eq!(stats.median_absolute_error_secs, Some(20.0));
    assert_eq!(stats.mean_absolute_error_secs, Some(20.0));
    assert_eq!(stats.recent_mae_ratio, Some(0.1));
    assert!(!stats.is_accuracy_degraded);
}
