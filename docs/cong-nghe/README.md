# Bảng công nghệ

Cập nhật lần cuối: **2026-10-30**. Mỗi dòng trả lời bốn câu: *là gì · để làm gì · vì sao chọn · đã loại gì*.

Trạng thái: **Đã kiểm** = đã chạy lệnh và thấy kết quả · **Dự kiến** = có trong kế hoạch, chưa cài · **Chưa cài** = cần mà máy chưa có.

## Nền tảng đã có trên máy

| Công nghệ | Để làm gì | Phiên bản | Trạng thái | Vì sao chọn | Đã loại / không chọn |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Rust** (kênh `stable`, bộ nối MSVC) | Viết phần backend (lõi, máy chủ, các module) | `rustc 1.99.1` | Đã kiểm | Bạn chọn stack giống Vibe Kanban (quyết định D-02). Rust cho chương trình nhanh, ít lỗi bộ nhớ | **Nightly:** Vibe Kanban dùng vì vài tùy chọn của `rustfmt` chỉ có ở nightly; ta không dùng các tùy chọn đó nên ở kênh ổn định cho đỡ rủi ro (D-07). **Bộ nối GNU** (không cần Visual Studio): MSVC là cấu hình mặc định và ít gặp lỗi lạ hơn với thư viện có mã C *(nhận định chung, chưa kiểm lại với tài liệu chính thức)* |
| **Visual Studio Build Tools** (C++) | Cung cấp bộ nối `link.exe` và thư viện Windows cho Rust, và trình biên dịch C cho các thư viện có mã C (như SQLite) | _(đã cài)_ | Đã kiểm: build thử chương trình mẫu nối và chạy ra `Hello, world!` | Rust trên Windows cần nó để tạo file `.exe` | Cài cả Visual Studio đầy đủ (nặng hơn, không cần) |
| **cargo** | Quản lý dự án Rust: build, test, tải thư viện, cài công cụ | `cargo 1.99.1` | Đã kiểm | Đi kèm Rust, là công cụ chuẩn | — |
| **rustup** | Quản lý các phiên bản Rust | toolchain `stable-x86_64-pc-windows-msvc` | Đã kiểm | Cách cài Rust chính thức | — |
| **clippy** | "Người soát lỗi": cảnh báo code Rust dễ sai hoặc không chuẩn | `clippy 0.1.99` | Đã kiểm | Là cách kiểm chuẩn code Rust; kế hoạch yêu cầu chạy với `-D warnings` (xem `02_QUY_UOC_RUST.md`) | — |
| **rustfmt** | Tự động căn chỉnh định dạng code | `rustfmt 1.10.0-stable` | Đã kiểm | Mọi người dùng chung một định dạng, khỏi tranh cãi | Tùy chọn chỉ có ở nightly (xem dòng Rust) |
| **Node.js** | Chạy các script kiểm tra và phần frontend | `v24.19.0` | Đã kiểm | Cần cho frontend và `scripts/*.mjs` | — |
| **pnpm** | Quản lý thư viện JavaScript | `12.14.0` (toàn cục; mỗi dự án ghim bằng `packageManager`) | Đã kiểm | Dùng workspace nhiều gói; Vibe Kanban cũng dùng (D-13) | `npm` (làm được nhưng ta theo hướng pnpm) |
| **Git** | Lưu lịch sử thay đổi | `2.55.0.windows.5` | Đã kiểm | Chuẩn của ngành | — |
| **CLI agent `claude`, `agy`** | Các "nhân viên" AI mà Dandori điều phối | _(có trên máy)_ | Đã kiểm là có mặt (chưa chạy thử) | Dandori chỉ dùng CLI chính thức đã đăng nhập (D-06) | Dùng token OAuth bên ngoài CLI (có thể vi phạm điều khoản nhà cung cấp) |

## Công cụ Rust cài thêm (đã cài 2026-10-30)

| Công nghệ | Để làm gì | Phiên bản | Trạng thái | Vì sao cần | Đã loại / không chọn |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **cargo-deny** | Kiểm thư viện phụ thuộc: lỗ hổng, giấy phép, nguồn lạ | `0.20.2` | Đã kiểm | Rule `RS-DEP-001`; T01 phải chạy nó | **cargo-audit** chỉ kiểm lỗ hổng; `cargo-deny` kiểm rộng hơn *(theo hướng dẫn cộng đồng, chưa đối chiếu tài liệu chính thức)* |
| **sqlx-cli** | Tạo "bộ nhớ đệm" truy vấn để build không cần mở cơ sở dữ liệu | `0.9.0` | Đã kiểm | Rule `RS-DB-001`; T01 ghi phiên bản của nó | — |
| **cargo-nextest** | Chạy test nhanh và dễ đọc hơn | `0.9.150` | Đã kiểm | Tùy chọn, giúp tiện | `cargo test` có sẵn vẫn dùng được; kế hoạch không bắt buộc nextest |

## Chưa cài

| Công nghệ | Để làm gì | Trạng thái | Ghi chú |
| :--- | :--- | :--- | :--- |
| **Codex CLI** | Nhà cung cấp AI thứ ba để kiểm Dandori | Chưa cài | Chỉ cần nếu muốn kiểm với OpenAI; có thể bỏ qua giai đoạn đầu |

## Công nghệ dự kiến cho chính sản phẩm Dandori (chưa cài, đã chọn trong kế hoạch)

Nguồn: `instructions/implementations/00_QUYET_DINH.md`. Mã trong ngoặc là mã quyết định ở đó.

| Công nghệ | Để làm gì | Vì sao chọn | Đã loại |
| :--- | :--- | :--- | :--- |
| **axum + tokio** (D-09) | Máy chủ web và chạy nhiều việc cùng lúc | Phổ biến, cùng họ với Vibe Kanban | — |
| **SQLite + sqlx** (D-08) | Lưu dữ liệu cục bộ, kiểm truy vấn lúc biên dịch | Chạy trên máy người dùng, không cần dịch vụ riêng | Máy chủ cơ sở dữ liệu riêng (nặng cho ứng dụng cục bộ) |
| **React + Vite + TypeScript** (D-14) | Giao diện web | Ứng dụng cục bộ, không cần dựng trang phía máy chủ | **Next.js** (không cần SSR) |
| **SSE** (D-10) | Đẩy cập nhật thời gian thực lên giao diện | Đơn giản hơn, tự kết nối lại | **WebSocket** (làm lại nếu phần điện thoại cần hai chiều) |
| **Lệnh `git` có sẵn** (D-11) | Tạo nhánh/worktree cho từng task | Hành vi giống git bạn đang dùng, tránh lỗi build thư viện trên Windows | **Thư viện `git2`** |
| **ts-rs** (D-12) | Tự sinh kiểu TypeScript từ kiểu Rust, tránh viết hai lần | Cùng cách Vibe Kanban làm | Viết tay hai bản (dễ lệch) |

Khi một dòng đổi từ "dự kiến" sang "đã cài", **chuyển nó lên bảng đầu** và ghi phiên bản thật.
