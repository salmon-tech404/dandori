//! Bộ tính toán thống kê phân vị và đánh giá ETA (P3-07).

use chrono::{DateTime, Duration, Utc};

use crate::model::{EtaConfidence, EtaResult};

/// Số lượng mẫu tối thiểu để kích hoạt ước lượng ETA.
pub const MIN_SAMPLE_THRESHOLD: usize = 5;

/// Tính toán giá trị trung vị (Median / P50) từ danh sách mẫu.
#[must_use]
pub fn calculate_median(samples: &[f64]) -> Option<f64> {
    if samples.is_empty() {
        return None;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let len = sorted.len();
    if len % 2 == 1 {
        sorted.get(len / 2).copied()
    } else {
        let left = sorted.get(len / 2 - 1).copied()?;
        let right = sorted.get(len / 2).copied()?;
        Some((left + right) / 2.0)
    }
}

/// Tính toán phân vị 80 (P80) từ danh sách mẫu bằng phương pháp nội suy tuyến tính.
#[must_use]
pub fn calculate_p80(samples: &[f64]) -> Option<f64> {
    if samples.is_empty() {
        return None;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let len = sorted.len();
    if len == 1 {
        return sorted.first().copied();
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "Chuyển độ dài tập mẫu sang f64 để tính index"
    )]
    let rank = 0.8 * (len as f64 - 1.0);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "Rank index luôn dương và nhỏ hơn len"
    )]
    let low = rank.floor() as usize;
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "Rank index luôn dương và nhỏ hơn len"
    )]
    let high = rank.ceil() as usize;

    let val_low = sorted.get(low).copied()?;
    let val_high = sorted.get(high).copied()?;

    if low == high {
        Some(val_low)
    } else {
        let weight = rank - rank.floor();
        Some(val_low + (val_high - val_low) * weight)
    }
}

/// Tính độ tin cậy và kiểm tra hạ bậc.
fn determine_confidence(
    count: usize,
    median: f64,
    p80: f64,
    recent_mae_ratio: Option<f64>,
) -> (EtaConfidence, bool) {
    let mut conf = if count >= 20 {
        EtaConfidence::High
    } else if count >= 10 {
        EtaConfidence::Medium
    } else {
        EtaConfidence::Low
    };

    if median > 0.0 && (p80 / median) > 2.5 {
        conf = conf.downgrade();
    }

    let mut downgraded = false;
    if let Some(ratio) = recent_mae_ratio
        && ratio > 0.5
    {
        conf = conf.downgrade();
        downgraded = true;
    }

    (conf, downgraded)
}

/// Tính toán mốc thời gian hoàn tất ước tính.
fn calculate_completion_times(
    start_time: Option<DateTime<Utc>>,
    median: f64,
    p80: f64,
) -> (Option<String>, Option<String>) {
    if let Some(st) = start_time {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "Thời lượng task nằm trong phạm vi i64 giây"
        )]
        let est_med = st + Duration::seconds(median.round() as i64);
        #[expect(
            clippy::cast_possible_truncation,
            reason = "Thời lượng task nằm trong phạm vi i64 giây"
        )]
        let est_p80 = st + Duration::seconds(p80.round() as i64);
        (Some(est_med.to_rfc3339()), Some(est_p80.to_rfc3339()))
    } else {
        (None, None)
    }
}

/// Đánh giá và sinh kết quả ETA cho một tác vụ dựa trên các mẫu thời lượng và sai số tự đo.
#[must_use]
pub fn evaluate_eta(
    task_id: &str,
    run_id: Option<&str>,
    samples: &[f64],
    recent_mae_ratio: Option<f64>,
    start_time: Option<DateTime<Utc>>,
) -> EtaResult {
    let count = samples.len();

    if count < MIN_SAMPLE_THRESHOLD {
        return EtaResult {
            task_id: task_id.to_string(),
            run_id: run_id.map(ToString::to_string),
            has_estimate: false,
            reason: Some(format!(
                "chưa đủ dữ liệu (có {count}/{MIN_SAMPLE_THRESHOLD} mẫu hoàn tất cùng dự án, vai trò và nhà cung cấp)"
            )),
            sample_count: count,
            median_secs: None,
            p80_secs: None,
            confidence: None,
            confidence_downgraded: false,
            eta_range_text: None,
            predicted_completion_median: None,
            predicted_completion_p80: None,
        };
    }

    let median = calculate_median(samples).unwrap_or(0.0);
    let p80 = calculate_p80(samples).unwrap_or(median);
    let (confidence, downgraded) = determine_confidence(count, median, p80, recent_mae_ratio);

    let median_round = (median * 10.0).round() / 10.0;
    let p80_round = (p80 * 10.0).round() / 10.0;
    let range_text = format!("{median_round:.0}s - {p80_round:.0}s");
    let (pred_med, pred_p80) = calculate_completion_times(start_time, median, p80);

    EtaResult {
        task_id: task_id.to_string(),
        run_id: run_id.map(ToString::to_string),
        has_estimate: true,
        reason: None,
        sample_count: count,
        median_secs: Some(median_round),
        p80_secs: Some(p80_round),
        confidence: Some(confidence),
        confidence_downgraded: downgraded,
        eta_range_text: Some(range_text),
        predicted_completion_median: pred_med,
        predicted_completion_p80: pred_p80,
    }
}
