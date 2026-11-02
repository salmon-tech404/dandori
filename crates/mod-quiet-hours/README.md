# mod-quiet-hours

Module quản lý khung giờ cấm chạy (`Quiet Hours`) cho hệ thống Dandori.

## Tính năng chính
- Cấu hình khung giờ cấm chạy theo dự án hoặc toàn không gian làm việc.
- Hỗ trợ các ngày trong tuần, giờ bắt đầu/kết thúc (HH:MM), múi giờ và chuyển đổi giờ mùa hè (DST).
- Xử lý chính xác khung giờ vắt qua nửa đêm (ví dụ: 22:00 - 06:00).
- Hiện thực điểm mở rộng `TransitionGuard`: từ chối chuyển tác vụ sang `running` từ nguồn `scheduler:*` trong giờ cấm kèm `retry_after`.
- Chạy tay thủ công mặc định không bị chặn, có tùy chọn cấu hình chặn cả chạy tay.
- Tự động gộp các khung giờ cấm chồng chéo nhau để tính toán `retry_after` chính xác.
