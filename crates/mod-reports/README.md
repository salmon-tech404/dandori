# Module `reports`

Module tổng hợp báo cáo ngày và tuần cho không gian làm việc (`workspace`):
- Báo cáo xác định, hoàn toàn không do AI bịa số liệu: tổng hợp từ bảng `task`, `run` và sự kiện hệ thống.
- Thống kê chi tiết: số lượng task theo trạng thái (`completed`, `failed`, `needs_human`, `running`), thời lượng tổng và trung vị.
- Tích hợp số liệu chi phí (`cost-tracking`) và sự kiện chuyển tài khoản (`auto-switch`) khi các module đó được kích hoạt.
- Hỗ trợ xuất định dạng Markdown với bộ lọc khử lộ bí mật/khóa API tự động.
- Worker nền tự động chụp báo cáo định kỳ cuối ngày/cuối tuần theo múi giờ cấu hình.
