# ADR 0001: Danh mục thư viện phụ thuộc được duyệt (Approved Dependencies)

- **Trạng thái:** Đã chốt (Accepted)
- **Ngày:** 2026-10-30
- **Tác giả:** Dandori Team (theo 01_KIEN_TRUC.md mục 12 và quy chuẩn RS-DEP-001/002)

## 1. Bối cảnh (Context)
Dandori kiểm soát chặt chẽ chuỗi cung ứng phần mềm và thư viện bên ngoài theo quy tắc RS-DEP-001 và RS-DEP-002:
- Mọi thư viện phải thuộc danh sách được duyệt.
- Phiên bản được khai báo tập trung tại `[workspace.dependencies]` trong `Cargo.toml` gốc.
- Không sử dụng các thư viện có lỗ hổng bảo mật (advisories), thiếu bảo trì (unmaintained), hoặc tiềm ẩn nguy cơ âm thầm panic/unsound.
- Giấy phép phải tuân thủ danh mục cho phép (MIT, Apache-2.0, BSD, ISC, Zlib, CC0).

## 2. Danh mục thư viện Rust

Tất cả crate đã được kiểm tra tính tương thích với Rust 1.99.1 (Edition 2024), cấu hình khóa trong `Cargo.lock` và kiểm định qua `cargo deny check advisories`.

| Tên Crate | Phiên bản | Giấy phép | Mục đích sử dụng | Đánh giá bảo mật / RustSec |
| :--- | :--- | :--- | :--- | :--- |
| `tokio` | 1.43 | MIT | Async runtime đa luồng cho backend | Sạch, không có advisory |
| `tokio-util` | 0.7 | MIT | Tiện ích CancellationToken cho worker | Sạch, không có advisory |
| `futures` | 0.3 | MIT / Apache-2.0 | Tiện ích xử lý async stream | Sạch, không có advisory |
| `axum` | 0.8 | MIT | Framework HTTP server cục bộ | Sạch, phiên bản mới nhất |
| `tower` | 0.5 | MIT | Abstraction tầng middleware | Sạch |
| `tower-http` | 0.6 | MIT | Middleware CORS, tracing request | Sạch |
| `serde` | 1.0 | MIT / Apache-2.0 | Framework tuần tự hóa dữ liệu | Sạch |
| `serde_json` | 1.0 | MIT / Apache-2.0 | Xử lý JSON cho API và config | Sạch |
| `sqlx` | 0.8 | MIT / Apache-2.0 | Driver SQLite async, kiểm tra truy vấn biên dịch | Sạch |
| `thiserror` | 2.0 | MIT / Apache-2.0 | Định nghĩa enum lỗi có kiểu cho crate thư viện | Sạch |
| `anyhow` | 1.0 | MIT / Apache-2.0 | Xử lý lỗi cấp ứng dụng (chỉ dandori-server) | Sạch |
| `tracing` | 0.1 | MIT | Ghi nhật ký có cấu trúc | Sạch |
| `tracing-subscriber` | 0.3 | MIT | Định dạng và lọc đầu ra log | Sạch |
| `uuid` | 1.13 | MIT / Apache-2.0 | Định danh duy nhất (v4) | Sạch |
| `chrono` | 0.4 | MIT / Apache-2.0 | Thời gian thực và timestamp UTC | Sạch |
| `ts-rs` | 10.1 | MIT | Tự động sinh kiểu TypeScript từ struct Rust | Sạch |
| `argon2` | 0.5 | MIT / Apache-2.0 | Băm mật khẩu người dùng cục bộ | Sạch |
| `rand` | 0.8 | MIT / Apache-2.0 | Sinh số ngẫu nhiên an toàn | Sạch |
| `secrecy` | 0.10 | MIT / Apache-2.0 | Che dấu bí mật trong bộ nhớ và log | Sạch |
| `zeroize` | 1.8 | MIT / Apache-2.0 | Xóa sạch vùng nhớ nhạy cảm khi drop | Sạch |
| `aes-gcm` | 0.10 | MIT / Apache-2.0 | Mã hóa AEAD cho kho khóa cục bộ | Sạch |
| `keyring` | 3.6 | MIT / Apache-2.0 | Lưu trữ khóa chủ qua OS Credential Manager | Sạch |
| `directories` | 6.0 | MIT / Apache-2.0 | Xác định thư mục dữ liệu chuẩn của hệ điều hành | Sạch |
| `url` | 2.5 | MIT / Apache-2.0 | Phân tích và kiểm tra URL hợp lệ | Sạch |
| `reqwest` | 0.12 | MIT / Apache-2.0 | HTTP client (dùng rustls, không dùng native-tls) | Sạch |
| `croner` | 2.1 | MIT / Apache-2.0 | Phân tích biểu thức cron cho lịch chạy | Sạch |
| `tempfile` | 3.17 | MIT / Apache-2.0 | Thư mục tạm độc lập cho unit/integration test | Sạch (dev-dependency) |

## 3. Danh mục gói npm Frontend (`frontend/app`)

| Tên Package | Phiên bản | Giấy phép | Mục đích sử dụng |
| :--- | :--- | :--- | :--- |
| `react` / `react-dom` | ^19.0.0 | MIT | Thư viện UI React |
| `react-router-dom` | ^7.2.0 | MIT | Điều hướng client-side SPA |
| `@tanstack/react-query` | ^5.66.0 | MIT | Quản lý server state và cache API |
| `zustand` | ^5.0.3 | MIT | Quản lý UI state toàn cục |
| `zod` | ^3.24.2 | MIT | Kiểm tra tính hợp lệ dữ liệu ở biên |
| `tailwindcss` | ^4.0.7 | MIT | Hệ thống utility CSS |
| `lucide-react` | ^1.16.0 | ISC | Bộ biểu tượng SVG nhất quán |
| `typescript` | ^5.7.3 | Apache-2.0 | Trình biên dịch TypeScript (strict mode) |
| `vite` | ^6.1.0 | MIT | Build tool và dev server siêu tốc |
| `@vitejs/plugin-react` | ^4.3.4 | MIT | Plugin React cho Vite |
| `vitest` | ^3.0.5 | MIT | Framework kiểm thử unit test phía frontend |
| `@testing-library/react` | ^16.2.0 | MIT | Tiện ích kiểm thử component React |
| `jsdom` | ^26.0.0 | MIT | Môi trường DOM giả lập cho Vitest |

## 4. Kết luận
Tất cả các thư viện lựa chọn đều đang được duy trì tích cực, có giấy phép nguồn mở tương thích thương mại an toàn và không phát hiện lỗ hổng bảo mật nào trong cơ sở dữ liệu RustSec.

## 5. Xác minh bổ sung (Claude, 2026-10-31)

Khi kiểm T01, tôi thấy mục 2 ghi "Sạch, không có advisory" cho cả 27 thư viện, nhưng **10 thư viện chưa từng được `cargo deny` kiểm** vì chưa crate nào dùng chúng nên chúng không nằm trong `Cargo.lock`: `futures`, `axum`, `tower`, `tower-http`, `ts-rs`, `argon2`, `directories`, `reqwest`, `croner`, `tempfile`. Nghĩa là lúc T01 phát biểu đó chưa có bằng chứng cho 10 thư viện này.

Để bù, tôi dựng một dự án thử bên ngoài repo, khai báo đúng 10 thư viện với đúng phiên bản yêu cầu, rồi chạy `cargo deny check advisories` với đúng `deny.toml` của dự án. **Kết quả: `advisories ok`.** Phiên bản được khóa lúc kiểm: `futures 0.3.34`, `axum 0.8.9`, `tower 0.5.3`, `tower-http 0.6.11`, `ts-rs 10.1.0`, `argon2 0.5.3`, `directories 6.0.0`, `reqwest 0.12.28`, `croner 2.2.0`, `tempfile 3.27.0`.

Ghi chú: `cargo` báo có bản chính mới hơn cho `reqwest` (0.13.5), `tower-http` (0.7.1) và `ts-rs` (12.0.1). Danh mục hiện dùng các bản cũ hơn; đây là **lựa chọn có thể xét lại**, chưa phải lỗi. Phát biểu "đang được duy trì tích cực" trong mục 4 vẫn **chưa có bằng chứng** (chưa kiểm ngày phát hành gần nhất của từng thư viện); chỉ có bằng chứng về lỗ hổng đã công bố.
