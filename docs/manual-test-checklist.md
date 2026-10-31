# Sổ tay Kiểm thử Thủ công (Manual Test Checklist)

Tài liệu này tổng hợp toàn bộ các hạng mục kiểm thử cần sự can thiệp hoặc tài nguyên thật của người dùng (tài khoản đăng nhập CLI thật, dịch vụ đám mây, thông báo qua kênh chat, máy chủ từ xa SSH, thiết bị di động, kiểm tra giao diện trực tiếp trên trình duyệt).

Mỗi hạng mục được gán mã định danh duy nhất theo định dạng `DAN-VERIFY-nnn`.

---

## Danh mục Hạng mục Chờ Người Dùng Kiểm

| Mã | Task liên quan | Hạng mục cần kiểm thử | Điều kiện tiên quyết | Hướng dẫn kiểm thử thủ công | Trạng thái |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `DAN-VERIFY-001` | T01B / T02 | Kiểm tra hiển thị trang chào trên trình duyệt thực tế | Đã chạy `pnpm --filter app dev` | Mở `http://localhost:5173`, xác nhận giao diện hiển thị đúng layout, màu sắc, phông chữ và nút bấm | Chờ kiểm |
| `DAN-VERIFY-002` | T02 | Kiểm thử cổng verify trên hệ điều hành Linux / macOS | Máy trạm hoặc VM Linux có cài đặt Rust & Node | Clone dự án và chạy `pnpm run verify`, xác nhận 15 bước đều qua | Chờ kiểm |
| `DAN-VERIFY-003` | P1-02 | Kết nối subscription CLI agent thật (Claude / Gemini / Codex) | CLI đã đăng nhập tài khoản trả phí trên máy | Chạy lệnh tương tác với CLI thật, xác nhận lấy được hạn mức và thông tin model | Chờ kiểm (P1) |
| `DAN-VERIFY-004` | P2-06 | Kết nối GitHub / GitLab / Linear thật | Token truy cập hoặc OAuth app thật | Cấu hình repo đích và kiểm tra tạo issue / pull request tự động | Chờ kiểm (P2) |
| `DAN-VERIFY-005` | P3-12 | Thông báo qua Telegram Bot / Discord Webhook | Bot token và Chat ID thật | Kích hoạt sự kiện và xác nhận nhận được tin nhắn thông báo trên ứng dụng | Chờ kiểm (P3) |
| `DAN-VERIFY-006` | P4-05 | Thực thi task trên máy chủ từ xa qua SSH | Máy chủ Linux có cấu hình SSH key | Khởi tạo worker SSH remote và kiểm tra quá trình clone / build / execute | Chờ kiểm (P4) |
