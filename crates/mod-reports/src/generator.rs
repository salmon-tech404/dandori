//! Trình tạo báo cáo xác định và chuyển đổi định dạng Markdown an toàn.

use chrono::{DateTime, Datelike, Duration, FixedOffset, Offset, TimeZone, Utc};
use sqlx::SqlitePool;
use std::collections::HashMap;

use crate::model::{
    AccountSwitchStats, CostStats, DurationStats, ProviderAccountUsage, ReportCounts, ReportData,
    TopTaskItem,
};

fn zero_offset() -> FixedOffset {
    Utc.fix()
}

/// Phân tích múi giờ từ chuỗi (mặc định UTC).
#[must_use]
pub fn parse_timezone(tz_str: &str) -> FixedOffset {
    if tz_str.eq_ignore_ascii_case("utc") || tz_str.eq_ignore_ascii_case("z") {
        return zero_offset();
    }
    if let Ok(offset) = tz_str.parse::<FixedOffset>() {
        return offset;
    }
    // Hỗ trợ dạng "+07:00", "+0700", "+7", "-05:00"
    let clean = tz_str.trim().trim_start_matches('+');
    clean
        .parse::<i32>()
        .ok()
        .filter(|h| (-23..=23).contains(h))
        .and_then(|h| FixedOffset::east_opt(h * 3600))
        .unwrap_or_else(zero_offset)
}

/// Tính toán khoảng thời gian (start, end) theo UTC cho kỳ báo cáo theo ngày hoặc tuần.
#[must_use]
pub fn calculate_period_bounds(
    now: DateTime<Utc>,
    period_type: &str,
    tz: FixedOffset,
) -> (DateTime<Utc>, DateTime<Utc>) {
    let local_now = now.with_timezone(&tz);
    let local_date = local_now.date_naive();

    if period_type == "weekly" {
        // Tuần bắt đầu từ Thứ 2 (iso_weekday() = 1)
        let days_from_monday = i64::from(local_date.weekday().num_days_from_monday());
        let monday = local_date - Duration::days(days_from_monday);
        let sunday = monday + Duration::days(6);

        let start_local = monday.and_hms_opt(0, 0, 0).unwrap_or_default();
        let end_local = sunday.and_hms_opt(23, 59, 59).unwrap_or_default();

        let start_utc = tz
            .from_local_datetime(&start_local)
            .single()
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(now);
        let end_utc = tz
            .from_local_datetime(&end_local)
            .single()
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(now);

        (start_utc, end_utc)
    } else {
        // Daily: Từ 00:00:00 đến 23:59:59 của ngày local_date
        let start_local = local_date.and_hms_opt(0, 0, 0).unwrap_or_default();
        let end_local = local_date.and_hms_opt(23, 59, 59).unwrap_or_default();

        let start_utc = tz
            .from_local_datetime(&start_local)
            .single()
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(now);
        let end_utc = tz
            .from_local_datetime(&end_local)
            .single()
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(now);

        (start_utc, end_utc)
    }
}

/// Tính thời lượng trung vị từ danh sách thời lượng đã sắp xếp.
#[must_use]
pub fn calculate_median(sorted: &[i64]) -> i64 {
    let n = sorted.len();
    if n == 0 {
        return 0;
    }
    if n % 2 == 1 {
        sorted.get(n / 2).copied().unwrap_or(0)
    } else {
        let mid1 = sorted.get(n / 2 - 1).copied().unwrap_or(0);
        let mid2 = sorted.get(n / 2).copied().unwrap_or(0);
        (mid1 + mid2) / 2
    }
}

/// Khử các chuỗi nhạy cảm và token khóa API trong chuỗi văn bản Markdown (AC 4).
#[must_use]
pub fn redact_secrets(input: &str) -> String {
    let mut out = input.to_string();

    // Khử khóa dạng sk-..., ghp_..., dandori_..., Bearer ...
    let prefixes = &["sk-", "ghp_", "dandori_", "gho_", "xoxb-", "xoxp-"];
    for prefix in prefixes {
        let mut search_from = 0;
        while let Some(rel) = out[search_from..].find(prefix) {
            let start_idx = search_from + rel;
            let rest = &out[start_idx..];
            let end_idx = rest
                .find(|c: char| c.is_whitespace() || c == '`' || c == '"' || c == '\'' || c == ')')
                .unwrap_or(rest.len());
            if end_idx >= 8 {
                out.replace_range(start_idx..start_idx + end_idx, "[REDACTED_SECRET]");
                search_from = start_idx + "[REDACTED_SECRET]".len();
            } else {
                // Khớp ngắn (ví dụ "task-1"): bỏ qua và tiếp tục quét phần còn lại, không dừng sớm.
                search_from = start_idx + prefix.len();
            }
        }
    }

    // Khử khối private key nhiều dòng
    while let Some(pk_start) = out.find("-----BEGIN") {
        if let Some(pk_end) = out[pk_start..].find("-----END") {
            let after_end = pk_start + pk_end;
            let line_end = out[after_end..]
                .find('\n')
                .map_or(out[after_end..].len(), |idx| idx + 1);
            let total_end = after_end + line_end;
            out.replace_range(pk_start..total_end, "[REDACTED_PRIVATE_KEY]");
            continue;
        }
        // Khóa bị cắt cụt (không có dòng END): che toàn bộ phần còn lại, không để lộ thân khóa.
        out.replace_range(pk_start.., "[REDACTED_PRIVATE_KEY]");
        break;
    }

    out
}

/// Sinh dữ liệu báo cáo xác định từ cơ sở dữ liệu lõi SQLite.
#[expect(
    clippy::too_many_lines,
    reason = "Tổng hợp dữ liệu báo cáo đầy đủ từ các bảng task, run và module phụ thuộc"
)]
pub async fn generate_report_data(
    pool: &SqlitePool,
    workspace_id: &str,
    period_type: &str,
    start_utc: DateTime<Utc>,
    end_utc: DateTime<Utc>,
    timezone: &str,
) -> Result<ReportData, sqlx::Error> {
    let start_str = start_utc.to_rfc3339();
    let end_str = end_utc.to_rfc3339();

    // 1. Lấy danh sách nhiệm vụ của workspace được cập nhật/tạo trong kỳ
    let tasks: Vec<(String, String, String, Option<i64>)> = sqlx::query_as(
        r#"
        SELECT t.id, t.title, t.status,
               (
                   SELECT CAST((strftime('%s', finished_at) - strftime('%s', started_at)) AS INTEGER)
                   FROM run
                   WHERE task_id = t.id AND finished_at IS NOT NULL
                   ORDER BY started_at DESC LIMIT 1
               ) AS duration
        FROM task t
        JOIN project p ON t.project_id = p.id
        WHERE p.workspace_id = ?
          AND (
              (t.updated_at >= ? AND t.updated_at <= ?)
              OR (t.created_at >= ? AND t.created_at <= ?)
          )
        ORDER BY t.updated_at DESC
        "#,
    )
    .bind(workspace_id)
    .bind(&start_str)
    .bind(&end_str)
    .bind(&start_str)
    .bind(&end_str)
    .fetch_all(pool)
    .await?;

    let mut counts = ReportCounts::default();
    let mut top_tasks = Vec::new();

    for (t_id, title, status, dur) in &tasks {
        counts.total += 1;
        match status.as_str() {
            "done" | "completed" => counts.completed += 1,
            "failed" => counts.failed += 1,
            "needs_human" => counts.needs_human += 1,
            "in_progress" | "running" => counts.running += 1,
            _ => {}
        }
        if top_tasks.len() < 5 {
            top_tasks.push(TopTaskItem {
                task_id: t_id.clone(),
                title: title.clone(),
                status: status.clone(),
                duration_secs: *dur,
            });
        }
    }

    // 2. Thống kê thời lượng chạy từ bảng `run`
    let runs: Vec<(String, String, Option<i64>)> = sqlx::query_as(
        r#"
        SELECT r.agent, r.account,
               CAST((strftime('%s', r.finished_at) - strftime('%s', r.started_at)) AS INTEGER) AS dur
        FROM run r
        JOIN task t ON r.task_id = t.id
        JOIN project p ON t.project_id = p.id
        WHERE p.workspace_id = ?
          AND r.started_at >= ? AND r.started_at <= ?
        "#,
    )
    .bind(workspace_id)
    .bind(&start_str)
    .bind(&end_str)
    .fetch_all(pool)
    .await?;

    let mut durations: Vec<i64> = Vec::new();
    let mut usage_map: HashMap<(String, String), usize> = HashMap::new();

    for (agent, account, dur_opt) in &runs {
        match dur_opt {
            Some(d) if *d >= 0 => durations.push(*d),
            _ => {}
        }
        *usage_map
            .entry((agent.clone(), account.clone()))
            .or_insert(0) += 1;
    }

    durations.sort_unstable();
    let total_dur: i64 = durations.iter().sum();
    let median_dur = calculate_median(&durations);
    let avg_dur = if durations.is_empty() {
        0
    } else {
        total_dur / durations.len() as i64
    };

    let duration_stats = DurationStats {
        total_duration_secs: total_dur,
        median_duration_secs: median_dur,
        avg_duration_secs: avg_dur,
    };

    let mut usage: Vec<ProviderAccountUsage> = usage_map
        .into_iter()
        .map(|((agent, account), run_count)| ProviderAccountUsage {
            agent,
            account,
            run_count,
        })
        .collect();
    usage.sort_by_key(|a| std::cmp::Reverse(a.run_count));

    // 3. Thống kê chuyển đổi tài khoản (auto-switch) nếu bảng tồn tại
    let switches = query_account_switches(pool, workspace_id, &start_str, &end_str).await;

    // 4. Thống kê chi phí từ module cost-tracking nếu có
    let cost = query_cost_stats(pool, workspace_id, &start_str, &end_str).await;

    Ok(ReportData {
        workspace_id: workspace_id.to_string(),
        period_type: period_type.to_string(),
        period_start: start_str,
        period_end: end_str,
        timezone: timezone.to_string(),
        counts,
        duration: duration_stats,
        usage,
        switches,
        cost,
        top_tasks,
        generated_at: Utc::now().to_rfc3339(),
    })
}

async fn query_account_switches(
    pool: &SqlitePool,
    workspace_id: &str,
    start_str: &str,
    end_str: &str,
) -> AccountSwitchStats {
    let check: Result<i64, _> = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM run_event re
        JOIN run r ON re.run_id = r.id
        JOIN task t ON r.task_id = t.id
        JOIN project p ON t.project_id = p.id
        WHERE p.workspace_id = ?
          AND re.event_type = 'account.switched'
          AND re.created_at >= ? AND re.created_at <= ?
        "#,
    )
    .bind(workspace_id)
    .bind(start_str)
    .bind(end_str)
    .fetch_one(pool)
    .await;

    match check {
        Ok(count) => AccountSwitchStats {
            switch_count: Some(count as usize),
        },
        Err(_) => AccountSwitchStats { switch_count: None },
    }
}

async fn query_cost_stats(
    pool: &SqlitePool,
    workspace_id: &str,
    start_str: &str,
    end_str: &str,
) -> CostStats {
    // Thử truy vấn bảng `m_cost_record` nếu module cost-tracking đã chạy migration
    let cost_sum: Result<Option<f64>, _> = sqlx::query_scalar(
        r#"
        SELECT SUM(amount)
        FROM m_cost_record
        WHERE workspace_id = ?
          AND created_at >= ? AND created_at <= ?
        "#,
    )
    .bind(workspace_id)
    .bind(start_str)
    .bind(end_str)
    .fetch_one(pool)
    .await;

    match cost_sum {
        Ok(Some(total)) => CostStats {
            total_cost: Some(total),
            currency: Some("USD".to_string()),
            has_data: true,
            missing_reason: None,
        },
        Ok(None) => CostStats {
            total_cost: None,
            currency: None,
            has_data: false,
            missing_reason: Some("không có số liệu chi phí".to_string()),
        },
        Err(_) => CostStats {
            total_cost: None,
            currency: None,
            has_data: false,
            missing_reason: Some("module cost-tracking chưa kích hoạt".to_string()),
        },
    }
}

/// Chuyển đổi dữ liệu báo cáo sang văn bản Markdown có cấu trúc, áp dụng bộ lọc bí mật (AC 2, AC 4).
#[expect(
    clippy::too_many_lines,
    reason = "Định dạng các mục báo cáo sang văn bản Markdown có cấu trúc"
)]
#[must_use]
pub fn export_report_to_markdown(report: &ReportData) -> String {
    let period_title = if report.period_type == "weekly" {
        "Tuần"
    } else {
        "Ngày"
    };

    let mut md = format!(
        "# Báo cáo Tổng kết {} ({})\n\n",
        period_title, report.workspace_id
    );
    md.push_str(&format!(
        "> Kỳ báo cáo: `{}` đến `{}` (Múi giờ: `{}`)\n\n",
        report.period_start, report.period_end, report.timezone
    ));

    // 1. Tổng quan số lượng
    md.push_str("## 1. Tổng quan Nhiệm vụ\n\n");
    md.push_str("| Trạng thái | Số lượng |\n| :--- | :--- |\n");
    md.push_str(&format!(
        "| Đã hoàn thành (`done`) | {} |\n",
        report.counts.completed
    ));
    md.push_str(&format!(
        "| Thất bại (`failed`) | {} |\n",
        report.counts.failed
    ));
    md.push_str(&format!(
        "| Cần can thiệp (`needs_human`) | {} |\n",
        report.counts.needs_human
    ));
    md.push_str(&format!(
        "| Đang chạy (`running`) | {} |\n",
        report.counts.running
    ));
    md.push_str(&format!(
        "| **Tổng cộng** | **{}** |\n\n",
        report.counts.total
    ));

    // 2. Thời lượng thực thi
    md.push_str("## 2. Thời lượng Thực thi\n\n");
    md.push_str(&format!(
        "- **Tổng thời gian chạy:** {} giây (~{:.2} phút)\n",
        report.duration.total_duration_secs,
        report.duration.total_duration_secs as f64 / 60.0
    ));
    md.push_str(&format!(
        "- **Thời lượng trung vị (Median):** {} giây\n",
        report.duration.median_duration_secs
    ));
    md.push_str(&format!(
        "- **Thời lượng trung bình (Average):** {} giây\n\n",
        report.duration.avg_duration_secs
    ));

    // 3. Sử dụng Nhà cung cấp & Tài khoản
    md.push_str("## 3. Nhà cung cấp & Tài khoản Agent\n\n");
    if report.usage.is_empty() {
        md.push_str("_Không có lần chạy nào trong kỳ._\n\n");
    } else {
        md.push_str("| Nhà cung cấp | Tài khoản | Số lần chạy |\n| :--- | :--- | :--- |\n");
        for u in &report.usage {
            md.push_str(&format!(
                "| {} | {} | {} |\n",
                u.agent, u.account, u.run_count
            ));
        }
        md.push('\n');
    }

    // 4. Chuyển đổi tài khoản (Auto-switch)
    md.push_str("## 4. Chuyển đổi Tài khoản\n\n");
    if let Some(sw) = report.switches.switch_count {
        md.push_str(&format!(
            "- Số lần chuyển tài khoản tự động: **{}**\n\n",
            sw
        ));
    } else {
        md.push_str("_Không có số liệu chuyển đổi tài khoản (chưa phát sinh hoặc module auto-switch chưa bật)._\n\n");
    }

    // 5. Chi phí & Hạn mức
    md.push_str("## 5. Chi phí & Hạn mức\n\n");
    if report.cost.has_data {
        if let Some(total) = report.cost.total_cost {
            let curr = report.cost.currency.as_deref().unwrap_or("USD");
            md.push_str(&format!("- **Tổng chi phí:** {:.4} {}\n\n", total, curr));
        }
    } else {
        let reason = report
            .cost
            .missing_reason
            .as_deref()
            .unwrap_or("không có số liệu chi phí");
        md.push_str(&format!("- _Chi phí: {}._\n\n", reason));
    }

    // 6. Nhiệm vụ nổi bật
    md.push_str("## 6. Nhiệm vụ Nổi bật\n\n");
    if report.top_tasks.is_empty() {
        md.push_str("_Chưa có nhiệm vụ nổi bật._\n\n");
    } else {
        md.push_str(
            "| ID | Tiêu đề | Trạng thái | Thời lượng (s) |\n| :--- | :--- | :--- | :--- |\n",
        );
        for t in &report.top_tasks {
            let dur_str = t.duration_secs.map_or("-".to_string(), |d| d.to_string());
            md.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                t.task_id, t.title, t.status, dur_str
            ));
        }
        md.push('\n');
    }

    redact_secrets(&md)
}
