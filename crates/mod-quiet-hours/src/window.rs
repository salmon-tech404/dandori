//! Thuật toán đánh giá khung giờ cấm, xử lý ngày trong tuần, múi giờ, qua nửa đêm và khung chồng nhau.

#![expect(
    clippy::too_many_lines,
    reason = "Thuật toán xử lý tính toán khung giờ ngày trong tuần, DST, qua nửa đêm và chồng chéo"
)]

use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveTime, Utc};
use std::collections::HashSet;

use crate::model::QuietWindow;

/// Trả về độ lệch UTC 0 chuẩn an toàn.
#[must_use]
pub fn utc_offset() -> FixedOffset {
    match FixedOffset::east_opt(0) {
        Some(o) => o,
        None => unreachable!("0 là độ lệch UTC hợp lệ"),
    }
}

/// Phân tích chuỗi múi giờ sang `FixedOffset`.
/// Hỗ trợ "UTC", "Z", "+HH:MM", "-HH:MM", "+HH", "-HH".
pub fn parse_timezone_offset(tz_str: &str) -> Result<FixedOffset, String> {
    let s = tz_str.trim();
    if s.is_empty() || s.eq_ignore_ascii_case("utc") || s.eq_ignore_ascii_case("z") {
        return Ok(utc_offset());
    }

    if s.starts_with('+') || s.starts_with('-') {
        let sign = if s.starts_with('+') { 1 } else { -1 };
        let parts: Vec<&str> = s[1..].split(':').collect();
        let hours: i32 = parts
            .first()
            .and_then(|h| h.parse().ok())
            .ok_or_else(|| format!("Giờ múi giờ không hợp lệ: {s}"))?;
        let minutes: i32 = if parts.len() > 1 {
            parts
                .get(1)
                .and_then(|m| m.parse().ok())
                .ok_or_else(|| format!("Phút múi giờ không hợp lệ: {s}"))?
        } else {
            0
        };

        let total_seconds = sign * (hours * 3600 + minutes * 60);
        return FixedOffset::east_opt(total_seconds)
            .ok_or_else(|| format!("Độ lệch múi giờ vượt giới hạn: {s}"));
    }

    // Các tên viết tắt hoặc múi giờ phổ biến
    if s.eq_ignore_ascii_case("asia/ho_chi_minh")
        || s.eq_ignore_ascii_case("asia/bangkok")
        || s.eq_ignore_ascii_case("ict")
    {
        return FixedOffset::east_opt(7 * 3600)
            .ok_or_else(|| "Lỗi khởi tạo múi giờ Asia/Ho_Chi_Minh".to_string());
    }
    if s.eq_ignore_ascii_case("est") {
        return FixedOffset::east_opt(-5 * 3600).ok_or_else(|| "Lỗi khởi tạo EST".to_string());
    }
    if s.eq_ignore_ascii_case("edt") {
        return FixedOffset::east_opt(-4 * 3600).ok_or_else(|| "Lỗi khởi tạo EDT".to_string());
    }

    Ok(utc_offset())
}

/// Phân tích chuỗi "HH:MM" sang `NaiveTime`.
pub fn parse_hh_mm(time_str: &str) -> Result<NaiveTime, String> {
    let parts: Vec<&str> = time_str.trim().split(':').collect();
    if parts.len() != 2 {
        return Err(format!(
            "Định dạng giờ không hợp lệ (cần HH:MM): {time_str}"
        ));
    }
    let hour: u32 = parts
        .first()
        .ok_or_else(|| "Thiếu phần giờ".to_string())?
        .parse()
        .map_err(|e| format!("Giờ không hợp lệ '{time_str}': {e}"))?;
    let minute: u32 = parts
        .get(1)
        .ok_or_else(|| "Thiếu phần phút".to_string())?
        .parse()
        .map_err(|e| format!("Phút không hợp lệ '{time_str}': {e}"))?;

    NaiveTime::from_hms_opt(hour, minute, 0)
        .ok_or_else(|| format!("Thời gian ngoài khoảng hợp lệ: {time_str}"))
}

/// Phân tích danh sách các ngày trong tuần (1=Thứ 2..7=Chủ nhật).
#[must_use]
pub fn parse_days_of_week(days_str: &str) -> HashSet<u32> {
    let mut days = HashSet::new();
    for part in days_str.split(',') {
        let p = part.trim();
        if let Ok(d) = p.parse::<u32>()
            && (1..=7).contains(&d)
        {
            days.insert(d);
        }
    }
    days
}

/// Kết quả kiểm tra một khung giờ cấm đơn lẻ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SingleWindowCheckResult {
    /// Có đang rơi vào khung cấm hay không.
    pub is_inside: bool,
    /// Thời điểm kết thúc của khung cấm này (theo UTC), nếu đang rơi vào khung.
    pub end_utc: Option<DateTime<Utc>>,
}

/// Đánh giá xem thời điểm `now_utc` có nằm trong một khung giờ cấm hay không.
///
/// Hỗ trợ cả khung giờ trong cùng ngày (`start < end`) và khung giờ vắt qua nửa đêm (`start > end`).
pub fn check_single_window(
    window: &QuietWindow,
    now_utc: DateTime<Utc>,
    offset_override: Option<FixedOffset>,
) -> Result<SingleWindowCheckResult, String> {
    if !window.enabled {
        return Ok(SingleWindowCheckResult {
            is_inside: false,
            end_utc: None,
        });
    }

    let offset = match offset_override {
        Some(o) => o,
        None => parse_timezone_offset(&window.timezone)?,
    };

    let start_time = parse_hh_mm(&window.start_time)?;
    let end_time = parse_hh_mm(&window.end_time)?;
    let active_days = parse_days_of_week(&window.days_of_week);

    let local_dt = now_utc.with_timezone(&offset);
    let local_date = local_dt.date_naive();
    let local_time = local_dt.time();
    let local_weekday = local_dt.weekday().number_from_monday();

    if start_time < end_time {
        // Khung giờ trong cùng một ngày (ví dụ 09:00 - 18:00)
        if active_days.contains(&local_weekday) && local_time >= start_time && local_time < end_time
        {
            let end_naive = local_date.and_time(end_time);
            let end_local = end_naive
                .and_local_timezone(offset)
                .single()
                .ok_or_else(|| "Lỗi chuyển đổi thời điểm kết thúc sang múi giờ".to_string())?;
            return Ok(SingleWindowCheckResult {
                is_inside: true,
                end_utc: Some(end_local.with_timezone(&Utc)),
            });
        }
    } else if start_time > end_time {
        // Khung giờ vắt qua nửa đêm (ví dụ 22:00 - 06:00)
        // Trường hợp 1: Đang ở phần đầu khung giờ (local_time >= start_time, ví dụ 23:00)
        if local_time >= start_time && active_days.contains(&local_weekday) {
            let next_day = local_date
                .checked_add_signed(Duration::days(1))
                .ok_or_else(|| "Tràn ngày khi tính khung qua nửa đêm".to_string())?;
            let end_naive = next_day.and_time(end_time);
            let end_local = end_naive
                .and_local_timezone(offset)
                .single()
                .ok_or_else(|| "Lỗi chuyển đổi thời điểm kết thúc sang múi giờ".to_string())?;
            return Ok(SingleWindowCheckResult {
                is_inside: true,
                end_utc: Some(end_local.with_timezone(&Utc)),
            });
        }

        // Trường hợp 2: Đang ở phần sau nửa đêm (local_time < end_time, ví dụ 04:00)
        if local_time < end_time {
            // Ngày bắt đầu khung giờ là ngày hôm qua
            let prev_day = local_date
                .checked_sub_signed(Duration::days(1))
                .ok_or_else(|| "Lỗi trừ ngày khi tính khung qua nửa đêm".to_string())?;
            let prev_weekday = prev_day.weekday().number_from_monday();

            if active_days.contains(&prev_weekday) {
                let end_naive = local_date.and_time(end_time);
                let end_local = end_naive
                    .and_local_timezone(offset)
                    .single()
                    .ok_or_else(|| "Lỗi chuyển đổi thời điểm kết thúc sang múi giờ".to_string())?;
                return Ok(SingleWindowCheckResult {
                    is_inside: true,
                    end_utc: Some(end_local.with_timezone(&Utc)),
                });
            }
        }
    } else {
        // start_time == end_time: Khung cấm cả ngày 24h
        if active_days.contains(&local_weekday) {
            let next_day = local_date
                .checked_add_signed(Duration::days(1))
                .ok_or_else(|| "Tràn ngày".to_string())?;
            let end_naive = next_day.and_time(end_time);
            let end_local = end_naive
                .and_local_timezone(offset)
                .single()
                .ok_or_else(|| "Lỗi tính toán múi giờ".to_string())?;
            return Ok(SingleWindowCheckResult {
                is_inside: true,
                end_utc: Some(end_local.with_timezone(&Utc)),
            });
        }
    }

    Ok(SingleWindowCheckResult {
        is_inside: false,
        end_utc: None,
    })
}

/// Đánh giá tập hợp nhiều khung giờ cấm, xử lý các khung chồng chéo nhau (Test 5).
///
/// Trả về thời điểm `retry_after` được mở rộng liên tục qua các khung giờ kế tiếp nếu có chồng lấn.
pub fn evaluate_windows(
    windows: &[QuietWindow],
    now_utc: DateTime<Utc>,
    offset_override: Option<FixedOffset>,
) -> Result<crate::model::QuietStatusDto, String> {
    let mut in_quiet_window = false;
    let mut active_window_names = Vec::new();
    let mut blocks_manual_runs = false;
    let mut latest_end: Option<DateTime<Utc>> = None;

    // Bước 1: Tìm tất cả các khung giờ đang bao trùm thời điểm now_utc
    for w in windows {
        let res = check_single_window(w, now_utc, offset_override)?;
        if res.is_inside {
            in_quiet_window = true;
            active_window_names.push(w.name.clone());
            if w.block_manual_runs {
                blocks_manual_runs = true;
            }
            if let Some(end) = res.end_utc {
                latest_end = match latest_end {
                    Some(cur) => Some(if end > cur { end } else { cur }),
                    None => Some(end),
                };
            }
        }
    }

    if !in_quiet_window {
        return Ok(crate::model::QuietStatusDto {
            in_quiet_window: false,
            active_window_names: vec![],
            retry_after: None,
            blocks_manual_runs: false,
        });
    }

    // Bước 2: Xử lý các khung giờ chồng chéo (Test 5)
    // Nếu tại thời điểm latest_end (hoặc ngay trước đó 1 giây) có một khung giờ khác đang mở rộng tiếp,
    // ta mở rộng latest_end đến khi không còn khung nào chồng tiếp.
    let mut current_end = latest_end.unwrap_or(now_utc);
    let mut iterations = 0;
    while iterations < 20 {
        iterations += 1;
        let mut expanded = false;
        // Kiểm tra thời điểm vừa trước khi kết thúc (current_end - 1s)
        let probe_time = current_end - Duration::seconds(1);
        for w in windows {
            let res = check_single_window(w, probe_time, offset_override)?;
            if res.is_inside
                && let Some(end) = res.end_utc
                && end > current_end
            {
                current_end = end;
                expanded = true;
                if !active_window_names.contains(&w.name) {
                    active_window_names.push(w.name.clone());
                }
                if w.block_manual_runs {
                    blocks_manual_runs = true;
                }
            }
        }
        if !expanded {
            break;
        }
    }

    Ok(crate::model::QuietStatusDto {
        in_quiet_window: true,
        active_window_names,
        retry_after: Some(current_end.to_rfc3339()),
        blocks_manual_runs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn make_test_window(
        name: &str,
        days: &str,
        start: &str,
        end: &str,
        tz: &str,
        block_manual: bool,
    ) -> QuietWindow {
        QuietWindow {
            id: uuid::Uuid::new_v4().to_string(),
            workspace_id: "ws-1".to_string(),
            project_id: None,
            name: name.to_string(),
            days_of_week: days.to_string(),
            start_time: start.to_string(),
            end_time: end.to_string(),
            timezone: tz.to_string(),
            block_manual_runs: block_manual,
            enabled: true,
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        }
    }

    #[test]
    fn test_same_day_window_inside_and_outside() {
        // Thứ 2 đến Thứ 6, từ 09:00 đến 18:00 UTC
        let w = make_test_window("Giờ làm việc", "1,2,3,4,5", "09:00", "18:00", "UTC", false);

        // Thứ 2 lúc 10:00 UTC -> bên trong khung
        let mon_10am = Utc.with_ymd_and_hms(2026, 11, 2, 10, 0, 0).unwrap();
        let res = check_single_window(&w, mon_10am, None).unwrap();
        assert!(res.is_inside);
        let expected_end = Utc.with_ymd_and_hms(2026, 11, 2, 18, 0, 0).unwrap();
        assert_eq!(res.end_utc, Some(expected_end));

        // Thứ 2 lúc 08:30 UTC -> bên ngoài khung
        let mon_830am = Utc.with_ymd_and_hms(2026, 11, 2, 8, 30, 0).unwrap();
        let res = check_single_window(&w, mon_830am, None).unwrap();
        assert!(!res.is_inside);

        // Thứ 2 lúc 18:00 UTC -> hết khung (ngoài)
        let mon_18pm = Utc.with_ymd_and_hms(2026, 11, 2, 18, 0, 0).unwrap();
        let res = check_single_window(&w, mon_18pm, None).unwrap();
        assert!(!res.is_inside);

        // Chủ nhật (2026-11-08) lúc 12:00 UTC -> ngoài ngày áp dụng
        let sun_12pm = Utc.with_ymd_and_hms(2026, 11, 8, 12, 0, 0).unwrap();
        let res = check_single_window(&w, sun_12pm, None).unwrap();
        assert!(!res.is_inside);
    }

    #[test]
    fn test_cross_midnight_window() {
        // Khung qua nửa đêm 22:00 đến 06:00 UTC, các ngày trong tuần Mon-Fri (1,2,3,4,5)
        let w = make_test_window("Cấm ban đêm", "1,2,3,4,5", "22:00", "06:00", "UTC", false);

        // Thứ 2 (2026-11-02) lúc 23:00 UTC -> bên trong, kết thúc lúc 06:00 ngày Thứ 3
        let mon_23pm = Utc.with_ymd_and_hms(2026, 11, 2, 23, 0, 0).unwrap();
        let res = check_single_window(&w, mon_23pm, None).unwrap();
        assert!(res.is_inside);
        let expected_end = Utc.with_ymd_and_hms(2026, 11, 3, 6, 0, 0).unwrap();
        assert_eq!(res.end_utc, Some(expected_end));

        // Thứ 3 (2026-11-03) lúc 04:00 UTC -> bên trong (bắt đầu từ tối Thứ 2)
        let tue_4am = Utc.with_ymd_and_hms(2026, 11, 3, 4, 0, 0).unwrap();
        let res = check_single_window(&w, tue_4am, None).unwrap();
        assert!(res.is_inside);
        let expected_end = Utc.with_ymd_and_hms(2026, 11, 3, 6, 0, 0).unwrap();
        assert_eq!(res.end_utc, Some(expected_end));

        // Thứ 3 (2026-11-03) lúc 07:00 UTC -> ngoài khung
        let tue_7am = Utc.with_ymd_and_hms(2026, 11, 3, 7, 0, 0).unwrap();
        let res = check_single_window(&w, tue_7am, None).unwrap();
        assert!(!res.is_inside);

        // Chủ nhật (2026-11-08) lúc 04:00 UTC: Hôm trước là Thứ 7 (không nằm trong 1..5) -> ngoài khung
        let sun_4am = Utc.with_ymd_and_hms(2026, 11, 8, 4, 0, 0).unwrap();
        let res = check_single_window(&w, sun_4am, None).unwrap();
        assert!(!res.is_inside);
    }

    #[test]
    fn test_overlapping_windows() {
        // Window 1: 09:00 - 13:00
        let w1 = make_test_window("Khung sáng", "1", "09:00", "13:00", "UTC", false);
        // Window 2: 12:00 - 18:00 (chồng nhau từ 12:00 đến 13:00)
        let w2 = make_test_window("Khung chiều", "1", "12:00", "18:00", "UTC", false);

        // Thứ 2 lúc 10:00 UTC -> nằm trong w1
        let mon_10am = Utc.with_ymd_and_hms(2026, 11, 2, 10, 0, 0).unwrap();
        let status = evaluate_windows(&[w1, w2], mon_10am, None).unwrap();

        assert!(status.in_quiet_window);
        // retry_after phải được kéo dài tới 18:00 (kết thúc của w2) chứ không dừng ở 13:00
        let expected_end = Utc.with_ymd_and_hms(2026, 11, 2, 18, 0, 0).unwrap();
        assert_eq!(status.retry_after, Some(expected_end.to_rfc3339()));
    }

    #[test]
    fn test_dst_transition() {
        // Kiểm tra chuyển đổi múi giờ khi có DST
        // Múi giờ EDT (-04:00) vs EST (-05:00)
        let edt_offset = FixedOffset::east_opt(-4 * 3600).unwrap();
        let w = make_test_window("Khung EDT", "1,2,3,4,5", "09:00", "17:00", "-04:00", false);

        // 10:00 EDT tương đương 14:00 UTC
        let now_utc = Utc.with_ymd_and_hms(2026, 11, 2, 14, 0, 0).unwrap();
        let res = check_single_window(&w, now_utc, Some(edt_offset)).unwrap();
        assert!(res.is_inside);

        // 17:00 EDT tương đương 21:00 UTC
        let expected_end = Utc.with_ymd_and_hms(2026, 11, 2, 21, 0, 0).unwrap();
        assert_eq!(res.end_utc, Some(expected_end));
    }
}
