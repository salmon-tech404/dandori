# Dandori

Dandori là trung tâm điều phối agent code chạy cục bộ trên máy người dùng, tích hợp cơ chế bật/tắt module linh hoạt và đảm bảo an toàn dữ liệu. Hệ thống phối hợp backend viết bằng Rust hiệu năng cao với frontend web hiện đại bằng React và TypeScript.

## Cách chạy

- **Chạy backend (máy chủ Dandori):**
  ```powershell
  cargo run -p dandori-server
  ```
- **Xây dựng frontend (giao diện web):**
  ```powershell
  pnpm --filter app build
  ```

## Bản đồ thư mục

Cấu trúc thư mục mã nguồn được chuẩn hóa theo [ADR 0020](docs/adr/0020-cau-truc-ma-nguon.md):

- `crates/`: Chứa toàn bộ các crate Rust thành phần (lõi, module, adapter, server) theo cấu trúc phẳng.
- `frontend/`: Không gian làm việc pnpm cho giao diện web (Vite + React + TypeScript) và tooling liên quan.
- `docs/`: Tài liệu kiến trúc, sổ tay kỹ thuật, quy trình kiểm thử thủ công và các bản ghi quyết định kiến trúc (ADR).
- `standards/`: Hồ sơ quy chuẩn và tiêu chuẩn kỹ thuật Kantanda áp dụng cho dự án.
- `instructions/`: Kế hoạch thực hiện từng giai đoạn và nhiệm vụ phát triển (chỉ đọc).
- `scripts/`: Bộ kịch bản kiểm tra chất lượng, xác minh kiến trúc và sinh mã (từ T02).
- `tests/`: Test của các script kiểm tra (từ T02). Test Rust nằm trong từng crate (`crates/<tên>/tests/`) và `crates/dandori-e2e` (từ T14).
- `shared/`: Chứa các định nghĩa kiểu dữ liệu dùng chung sinh tự động giữa Rust và TypeScript (từ T02).
- `deploy/`: Cấu hình Dockerfile và dịch vụ mẫu triển khai (từ P4-09, P5).
- `release/`: Thư mục chứa gói phát hành sản phẩm (được gitignore).

## Đọc gì tiếp theo

- [Tổng quan các crate Rust](crates/README.md): Bảng phân tầng trách nhiệm của các crate trong workspace.
- [Hướng dẫn giao diện frontend](frontend/README.md): Cấu trúc thư mục và cách phát triển frontend.
- [Tài liệu kỹ thuật và kiến trúc](docs/README.md): Tổng hợp các tài liệu phân tích kỹ thuật và quyết định kiến trúc.
- [Kế hoạch triển khai Dandori](instructions/implementations/00_README.md): Lộ trình, quy tắc và trạng thái các task thực hiện.
