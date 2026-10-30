# Sổ lệnh hay dùng

Mỗi lệnh: **làm gì · khi nào dùng · kết quả trông thế nào**. Ví dụ kết quả lấy từ máy này ngày 2026-10-30.

## Kiểm công cụ đã cài (đây là nhóm lệnh `--version`)

Lệnh có đuôi `--version` chỉ **hỏi công cụ tự giới thiệu**, không thay đổi gì. Dùng để trả lời hai câu: *"đã cài chưa?"* và *"phiên bản nào?"*. Nếu báo "not recognized" nghĩa là chưa cài hoặc terminal chưa nhận PATH.

| Lệnh | Hỏi cái gì | Ví dụ kết quả |
| :--- | :--- | :--- |
| `rustc --version` | Trình biên dịch Rust (biến code Rust thành chương trình) | `rustc 1.99.1 (58cf8f9c3 2026-10-13)` |
| `cargo --version` | Cargo, công cụ quản lý dự án Rust | `cargo 1.99.1 (5f94df478 2026-08-27)` |
| `rustup show active-toolchain` | Bộ công cụ Rust đang dùng | `stable-x86_64-pc-windows-msvc (default)` |
| `cargo clippy --version` | Clippy, công cụ soát lỗi | `clippy 0.1.99 (58cf8f9c35 2026-10-13)` |
| `cargo fmt --version` | rustfmt, công cụ căn chỉnh định dạng | `rustfmt 1.10.0-stable (...)` |
| `node -v` | Node.js | `v24.19.0` |
| `pnpm -v` | pnpm | `10.13.1` |
| `git --version` | Git | `git version 2.55.0.windows.5` |

Đọc số phiên bản: `1.99.1` là *chính.phụ.sửa-lỗi*. Số trong ngoặc là mã bản dựng và ngày phát hành, hữu ích khi báo lỗi.

## Làm việc với Rust

| Lệnh | Làm gì | Khi nào |
| :--- | :--- | :--- |
| `cargo new <tên>` | Tạo dự án Rust mới | Bắt đầu một chương trình nhỏ để thử |
| `cargo build` | Biên dịch và nối thành chương trình | Kiểm code có biên dịch được không |
| `cargo run` | Build rồi chạy luôn | Thử nhanh |
| `cargo test` | Chạy các bài kiểm tra tự động | Sau khi sửa code |
| `cargo clippy` | Soát lỗi và lời khuyên | Trước khi coi như xong |
| `cargo fmt` | Tự căn chỉnh định dạng | Trước khi coi như xong |
| `cargo install --locked <công cụ>` | Cài một công cụ dòng lệnh viết bằng Rust | Cài `cargo-deny`, `sqlx-cli`... Cờ `--locked` = dùng đúng phiên bản phụ thuộc đã khóa, kết quả lặp lại được |
| `rustup show` | Xem các bản Rust đã cài | Khi nghi ngờ đang dùng nhầm bản |

## Ba công cụ cài thêm (sau khi cài xong; chi tiết ở `cai-dat.md` Bước 3)

Các lệnh này **chưa chạy thử** trên máy vì công cụ chưa cài; cách dùng nêu theo kiến thức chung, kiểm bằng `--help` khi cần.

| Lệnh | Làm gì | Khi nào |
| :--- | :--- | :--- |
| `cargo install --list` | Liệt kê các công cụ đã cài bằng `cargo install` | Kiểm lệnh cài đã thành công chưa |
| `cargo deny check` | Kiểm thư viện phụ thuộc: lỗ hổng, giấy phép, nguồn lạ | Trong `verify`; task T01 |
| `cargo nextest run` | Chạy test nhanh, dễ đọc (thay `cargo test`) | Tùy chọn |
| `cargo sqlx prepare` | Tạo bộ nhớ đệm truy vấn vào thư mục `.sqlx/` | Sau khi sửa câu truy vấn SQL |
| `cargo sqlx prepare --check` | Kiểm bộ nhớ đệm có lỗi thời so với code không | Trong `verify` |

## Làm việc với git (chỉ đọc, an toàn)

| Lệnh | Làm gì |
| :--- | :--- |
| `git status` | Cho biết file nào đã đổi, file nào mới |
| `git diff` | Xem chính xác dòng nào đã đổi trong file đang theo dõi |
| `git log --oneline` | Xem danh sách commit rút gọn |
| `git rev-parse --short HEAD` | In mã ngắn của commit hiện tại (dùng để ghi mốc) |

## Làm việc với JavaScript

| Lệnh | Làm gì |
| :--- | :--- |
| `pnpm install --frozen-lockfile` | Cài thư viện **đúng như file khóa**, không tự nâng phiên bản. Dùng khi kiểm "cài mới có chạy không" |
| `pnpm run verify` | Chạy toàn bộ kiểm tra của dự án (sẽ có sau task T02) |

## Khi gặp lỗi "not recognized" (không nhận ra lệnh)

| Lệnh | Làm gì |
| :--- | :--- |
| `Get-Command cargo` (PowerShell) | Cho biết `cargo` nằm ở đâu; không ra gì nghĩa là terminal chưa thấy nó |
| `where cargo` (cmd) | Tương tự cho Command Prompt |
| `$env:Path` (PowerShell) | In danh sách thư mục terminal đang tìm lệnh (PATH) |

Chi tiết một ca thật: `../su-co/2026-10-30-cargo-khong-nhan-ra.md`.

> Lưu ý hai loại cửa sổ: **PowerShell** (dấu nhắc `PS D:\...>`) và **Command Prompt** (dấu nhắc `C:\...>`) dùng cú pháp khác nhau cho biến môi trường. Lệnh `$env:Path = ...` chỉ chạy được trong PowerShell.
