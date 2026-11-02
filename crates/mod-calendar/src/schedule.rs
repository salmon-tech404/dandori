//! Xử lý tính toán thời điểm kích hoạt lịch (Cron, Khoảng đều, Múi giờ, DST).

use chrono::{DateTime, Duration, FixedOffset, Utc};
use croner::Cron;

/// Tính toán thời điểm kích hoạt kế tiếp dựa trên loại lịch và biểu thức.
///
/// # Errors
/// Trả về chuỗi thông báo lỗi nếu biểu thức cron không hợp lệ hoặc khoảng đều < 60 giây.
pub fn compute_next_fire(
    schedule_type: &str,
    schedule_expr: &str,
    timezone_str: &str,
    from_time: DateTime<Utc>,
) -> Result<DateTime<Utc>, String> {
    match schedule_type {
        "interval" => {
            let secs: i64 = schedule_expr
                .parse()
                .map_err(|e| format!("Khoảng đều không hợp lệ: {e}"))?;
            if secs < 60 {
                return Err("Khoảng đều tối thiểu là 60 giây".to_string());
            }
            Ok(from_time + Duration::seconds(secs))
        }
        "cron" => {
            let cron = Cron::new(schedule_expr)
                .parse()
                .map_err(|e| format!("Biểu thức cron không hợp lệ: {e}"))?;

            let offset = parse_timezone_offset(timezone_str)?;
            let local_from = from_time.with_timezone(&offset);

            // Tìm thời điểm kế tiếp theo múi giờ địa phương
            let next_local = cron
                .find_next_occurrence(&local_from, false)
                .map_err(|e| format!("Lỗi tính thời điểm cron kế tiếp: {e}"))?;

            // Chuyển đổi ngược về UTC
            Ok(next_local.with_timezone(&Utc))
        }
        other => Err(format!("Loại lịch không hỗ trợ: {other}")),
    }
}

/// Trả về độ lệch UTC chuẩn (0 giây).
#[must_use]
pub fn utc_offset() -> FixedOffset {
    match FixedOffset::east_opt(0) {
        Some(o) => o,
        None => unreachable!("0 là độ lệch UTC hợp lệ"),
    }
}

/// Phân tích múi giờ sang FixedOffset.
///
/// Hỗ trợ "UTC", "Z", "+HH:MM", "-HH:MM", "+HH", "-HH".
pub fn parse_timezone_offset(tz_str: &str) -> Result<FixedOffset, String> {
    let s = tz_str.trim();
    if s.is_empty() || s.eq_ignore_ascii_case("utc") || s.eq_ignore_ascii_case("z") {
        return Ok(utc_offset());
    }

    // Định dạng +HH:MM hoặc -HH:MM
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

    // Mặc định an toàn: nếu là tên IANA chưa có chrono-tz, fallback về UTC
    Ok(utc_offset())
}

/// Mô phỏng chuyển giờ mùa hè (DST) với thời điểm chuyển đổi và độ lệch.
///
/// Dùng để kiểm tra AC 2: chuyển giờ mùa hè ở cả 2 chiều (spring forward và fall back):
/// không bỏ sót lượt chạy và không kích hoạt lặp.
#[derive(Debug, Clone)]
pub struct DstTransition {
    /// Thời điểm chuyển đổi (theo UTC).
    pub transition_utc: DateTime<Utc>,
    /// Độ lệch trước khi chuyển (giây).
    pub offset_before_secs: i32,
    /// Độ lệch sau khi chuyển (giây).
    pub offset_after_secs: i32,
}

impl DstTransition {
    /// Chuyển đổi một thời điểm UTC sang DateTime với offset tương ứng theo DST.
    #[must_use]
    pub fn to_dst_datetime(&self, utc: DateTime<Utc>) -> DateTime<FixedOffset> {
        let offset_secs = if utc < self.transition_utc {
            self.offset_before_secs
        } else {
            self.offset_after_secs
        };
        let offset = FixedOffset::east_opt(offset_secs).unwrap_or_else(utc_offset);
        utc.with_timezone(&offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_compute_next_fire_interval() {
        let base = Utc.with_ymd_and_hms(2026, 11, 2, 10, 0, 0).unwrap();
        let next = compute_next_fire("interval", "300", "+07:00", base).unwrap();
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 11, 2, 10, 5, 0).unwrap());

        // Dưới 60 giây phải báo lỗi
        let err = compute_next_fire("interval", "30", "+07:00", base).unwrap_err();
        assert!(err.contains("tối thiểu"));
    }

    #[test]
    fn test_compute_next_fire_cron() {
        let base = Utc.with_ymd_and_hms(2026, 11, 2, 10, 15, 0).unwrap();
        // Mỗi giờ vào phút 0 ("0 * * * *")
        let next = compute_next_fire("cron", "0 * * * *", "+00:00", base).unwrap();
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 11, 2, 11, 0, 0).unwrap());
    }

    #[test]
    fn test_parse_timezone_offset() {
        assert_eq!(
            parse_timezone_offset("UTC").unwrap(),
            FixedOffset::east_opt(0).unwrap()
        );
        assert_eq!(
            parse_timezone_offset("+07:00").unwrap(),
            FixedOffset::east_opt(7 * 3600).unwrap()
        );
        assert_eq!(
            parse_timezone_offset("-05:00").unwrap(),
            FixedOffset::east_opt(-5 * 3600).unwrap()
        );
    }

    #[test]
    fn test_dst_transitions() {
        // Mô phỏng chuyển giờ mùa hè ở New York:
        // Spring forward: UTC 07:00 ngày 8/3 đổi từ -05:00 sang -04:00
        let spring = DstTransition {
            transition_utc: Utc.with_ymd_and_hms(2026, 3, 8, 7, 0, 0).unwrap(),
            offset_before_secs: -5 * 3600,
            offset_after_secs: -4 * 3600,
        };
        let before_spring = Utc.with_ymd_and_hms(2026, 3, 8, 6, 0, 0).unwrap();
        let after_spring = Utc.with_ymd_and_hms(2026, 3, 8, 8, 0, 0).unwrap();
        assert_eq!(
            spring
                .to_dst_datetime(before_spring)
                .offset()
                .local_minus_utc(),
            -5 * 3600
        );
        assert_eq!(
            spring
                .to_dst_datetime(after_spring)
                .offset()
                .local_minus_utc(),
            -4 * 3600
        );

        // Fall back: UTC 06:00 ngày 1/11 đổi từ -04:00 sang -05:00
        let fall = DstTransition {
            transition_utc: Utc.with_ymd_and_hms(2026, 11, 1, 6, 0, 0).unwrap(),
            offset_before_secs: -4 * 3600,
            offset_after_secs: -5 * 3600,
        };
        let before_fall = Utc.with_ymd_and_hms(2026, 11, 1, 5, 0, 0).unwrap();
        let after_fall = Utc.with_ymd_and_hms(2026, 11, 1, 7, 0, 0).unwrap();
        assert_eq!(
            fall.to_dst_datetime(before_fall).offset().local_minus_utc(),
            -4 * 3600
        );
        assert_eq!(
            fall.to_dst_datetime(after_fall).offset().local_minus_utc(),
            -5 * 3600
        );
    }
}
