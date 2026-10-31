# Kiểm độc lập T01 (Claude, 2026-10-31)

Người kiểm: Claude (vai: kiểm độc lập, không phải người viết code). Đối tượng: toàn bộ thay đổi của T01 trong cây làm việc (chưa commit) và `reports/T01.md`.

## Kết luận

**T01 chạy thật và đạt phần lớn tiêu chí. Không phát hiện lỗi chặn.** Có một số điểm cần sửa, và cấu trúc thư mục được chốt lại cho chuẩn hơn; tất cả được giao cho Antigravity ở **T01B** trước khi sang T02.

## Đã tự kiểm (không dựa vào báo cáo)

Chạy trên **bản sao sạch** (62 file, không có `node_modules/`, `target/`, `instructions/`), `git init` riêng trong bản sao. Mã thoát và thời gian:

| Lệnh | Mã thoát | Thời gian |
| :--- | :---: | ---: |
| `pnpm install --frozen-lockfile` | 0 | 2s |
| `pnpm --filter app build` | 0 | 2s |
| `cargo fmt --all --check` | 0 | 0s |
| `cargo build --workspace --locked` | 0 | 24s |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 | 15s |
| `cargo test --workspace --locked` | 0 | 2s (0 test, vì chưa có logic) |
| `cargo deny check` | 0 | 13s (`advisories ok, bans ok, licenses ok, sources ok`) |

Đối chiếu tiêu chí T01 (8 mục): các mục 1, 2, 4 được xác nhận bằng bảng trên; mục 3 (RS-LINT-005) có bằng chứng trong ADR 0002 (lint giả làm clippy lỗi `E0602`); mục 5 và 6 đọc trực tiếp (ADR có nội dung thật; `standards/profile.md` không còn ngoặc mẫu); mục 7 (`git check-ignore`): đã kiểm `web/app/dist`, `*.tsbuildinfo`, `node_modules`, `target` bị ignore và `Cargo.lock`, `pnpm-lock.yaml` không bị ignore; mục 8 (phạm vi): các file ngoài danh sách là ghi chú sổ tay của chủ dự án, hợp lệ.

Điểm tốt đáng ghi nhận: tự thêm `unknown_lints = "deny"` sau khi phát hiện `-D warnings` không bắt lint lạ (có bằng chứng, ghi vào "Đã chỉnh"); lint giả được thử thật; không có `unsafe`, `unwrap`, `println!`; không có liên kết hay phông tải từ ngoài trong giao diện.

## Phát hiện

| # | Mức | Phát hiện | Bằng chứng | Xử lý |
| :-: | :--- | :--- | :--- | :--- |
| 1 | Trung bình | ADR 0001 ghi "Sạch, không có advisory" cho cả 27 thư viện, nhưng **10 thư viện chưa từng được `cargo deny` kiểm** (không nằm trong `Cargo.lock`) | `Cargo.lock` không có `futures, axum, tower, tower-http, ts-rs, argon2, directories, reqwest, croner, tempfile` | **Đã xử lý:** Claude dựng dự án thử ngoài repo, chạy `cargo deny check advisories` với `deny.toml` của dự án: `advisories ok`. Ghi vào ADR 0001 mục 5. Lưu ý: "đang được duy trì tích cực" vẫn chưa có bằng chứng |
| 2 | Trung bình | `members` liệt kê từng crate; mỗi module mới phải sửa `Cargo.toml` gốc, **trái tiêu chí PDF "thêm module không sửa lõi"** | `Cargo.toml` dòng `members = [...]` | T01B mục 3, ADR 0020 |
| 3 | Trung bình | **Khoảng trống trong kế hoạch của Claude:** `dandori-server` chưa khai báo `mod-*`; `gen-modules` chỉ sinh `modules_generated.rs`, nên thêm module vẫn phải sửa `Cargo.toml` của server | `crates/dandori-server/Cargo.toml` không có `mod-` nào | Thiết kế lại: crate sinh tự động `dandori-modules` (ADR 0020). Kế hoạch T05 đã sửa; T01B mục 5 tạo khung |
| 4 | Thấp | Chương trình chính tên `dandori-server`, kế hoạch (W-03) muốn `dandori`; `main.rs` chứa logic thay vì mỏng | `crates/dandori-server/Cargo.toml` không có `[[bin]]` | T01B mục 6 |
| 5 | Thấp | `dandori-agent-adapters` có hàm `pub fn init() -> &'static str` do agent tự thêm, không có trong thiết kế | `crates/dandori-agent-adapters/src/lib.rs` | T01B mục 7 |
| 6 | Thấp | `App.tsx` dùng giá trị cứng (`2rem`, `#ffffff`, v.v.) thay vì token; bộ token của T01 (`tokens.css`) dùng bảng màu chung chung, **khác** mockup (`04_GIAO_DIEN.md` mục 5) | `frontend/app/src/App.tsx`, `tokens.css` | T01B mục 8 (sửa tạm); T12 thay bộ token (đã ghi vào T12) |
| 7 | Thấp | Chưa ghim `packageManager` và `engines` | `package.json` | T01B mục 2 |
| 8 | Thấp | `profile.md` ghi "Presubmit: GitHub Actions" trong khi dự án chưa có CI | `standards/profile.md` mục 8 | T01B mục 10 |
| 9 | Thấp | `.gitignore` thiếu `/release/` | `.gitignore` | T01B mục 9 |
| 10 | Thấp | `resolver = "2"` trong khi ví dụ chính thức ghi `"3"`; chưa có `[profile.release]` | `Cargo.toml` | T01B mục 3 (Antigravity đọc trang chính thức để quyết); ADR 0020 |
| 11 | Thông tin | Kế hoạch (của Claude) đặt test xuyên suốt ở `tests/e2e/` ngoài mọi crate; gốc là virtual manifest nên `cargo test` không chạy | Cargo Book (Layout, Workspaces) | Đã sửa kế hoạch: `crates/dandori-e2e/tests/`; tạo ở T14 |
| 12 | Thông tin | Kế hoạch chưa nhắc SIGTERM khi tắt | Tokio, 12-factor | Đã thêm RS-OPS-001, tiêu chí T10 8b |

### Nghi ngờ đã rút lại

Lúc đầu tôi nghi Antigravity đã nâng pnpm từ `10.13.1` lên `12.14.0`. **Sai.** Gói pnpm toàn cục `12.14.0` được cài từ 2026-10-17, trước T01. Con số `10.13.1` trong ghi chú sổ tay của tôi đến từ việc tôi chạy lệnh khi đang đứng trong thư mục Vibe Kanban (nơi `packageManager: pnpm@10.13.1`; pnpm 12 tự chuyển phiên bản theo dự án). Đã sửa sổ tay. Bài học: Dandori phải ghim `packageManager` (T01B mục 2).

## Chưa kiểm

- Mã nguồn chưa có logic nên chưa kiểm được hành vi nào (0 test).
- Tính "duy trì tích cực" của các thư viện (mục 1).
- Tên lint `mod_module_files` (T01B xác nhận bằng cách chạy).
- Nội dung `pnpm-lock.yaml` chi tiết; chỉ kiểm `--frozen-lockfile` qua.

## Đề xuất cho chủ dự án

Duyệt T01 (có kèm T01B ngay sau), rồi **commit T01 trước khi giao T01B** để `git diff` của T01B chỉ hiện đúng phần sửa. Gợi ý commit T01 nằm trong `reports/T01.md` mục "Đề xuất commit".
