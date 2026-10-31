# Sổ tay Kiểm thử Thủ công (Manual Test Checklist)

Tài liệu này tổng hợp toàn bộ các hạng mục kiểm thử cần sự can thiệp hoặc tài nguyên thật của người dùng (tài khoản đăng nhập CLI thật, dịch vụ đám mây, thông báo qua kênh chat, máy chủ từ xa SSH, thiết bị di động, kiểm tra giao diện trực tiếp trên trình duyệt).

Mỗi hạng mục được gán mã định danh duy nhất theo định dạng `DAN-VERIFY-nnn`.

---

## Danh mục Hạng mục Chờ Người Dùng Kiểm

| Mã | Task liên quan | Hạng mục cần kiểm thử | Điều kiện tiên quyết | Hướng dẫn kiểm thử thủ công | Trạng thái |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `DAN-VERIFY-001` | T08 | Kho khóa Windows Credential Manager thật | Chạy ứng dụng trên máy Windows thật | Gọi `OsKeyStore::new().get_master_key()`, mở Windows Credential Manager (`Control Panel > User Accounts > Credential Manager > Generic Credentials`), xác nhận tồn tại mục `dandori-vault` / `master-key` | Chờ kiểm (CHỜ NGƯỜI DÙNG KIỂM) |
| `DAN-VERIFY-002` | T13 | Chủ dự án xem giao diện thật so với mockup | Chạy app và đăng nhập tài khoản kiểm thử | Mở màn hình Setting > Tính năng, so sánh bố cục, công tắc, chip công sức/rủi ro với `ui-reference/07-cai-dat-tinh-nang.png` | Chờ kiểm (P0) |
| `DAN-VERIFY-003` | P1-02 | Kết nối subscription CLI agent thật (Claude / Gemini / Codex) | CLI đã đăng nhập tài khoản trả phí trên máy | Chạy lệnh tương tác với CLI thật, xác nhận lấy được hạn mức và thông tin model | Chờ kiểm (P1) |
| `DAN-VERIFY-004` | P2-06 | Kết nối GitHub / GitLab / Linear thật | Token truy cập hoặc OAuth app thật | Cấu hình repo đích và kiểm tra tạo issue / pull request tự động | Chờ kiểm (P2) |
| `DAN-VERIFY-005` | P3-12 | Thông báo qua Telegram Bot / Discord Webhook | Bot token và Chat ID thật | Kích hoạt sự kiện và xác nhận nhận được tin nhắn thông báo trên ứng dụng | Chờ kiểm (P3) |
| `DAN-VERIFY-006` | P4-05 | Thực thi task trên máy chủ từ xa qua SSH | Máy chủ Linux có cấu hình SSH key | Khởi tạo worker SSH remote và kiểm tra quá trình clone / build / execute | Chờ kiểm (P4) |
| `DAN-VERIFY-007` | T14 | Khởi động máy chủ thật và kiểm thử thủ công toàn bộ luồng P0 (Tạo chủ -> Workspace -> Dự án -> Bật/tắt module sample-hello -> Bảo toàn dữ liệu) | Chạy binary dandori trên máy thật và trình duyệt | Khởi chạy máy chủ, thực hiện chuỗi thao tác từ giao diện người dùng, xác nhận bảng `m_sample_hello_note` lưu giữ dữ liệu khi module tắt và bật lại | Chờ kiểm (P0) |
