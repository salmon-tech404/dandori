# Thuật ngữ

Giải thích bằng lời thường. Thêm từ mới khi gặp.

| Thuật ngữ | Nghĩa dễ hiểu |
| :--- | :--- |
| **Biên dịch (compile)** | Biến code con người viết thành chương trình máy chạy được |
| **Bộ nối (linker)** | Bước cuối của biên dịch: ghép các mảnh đã dịch thành một file `.exe` hoàn chỉnh. Trên Windows Rust dùng `link.exe` của Microsoft |
| **MSVC** | Bộ công cụ biên dịch và nối của Microsoft, nằm trong Visual Studio Build Tools |
| **Toolchain** | Cả "bộ đồ nghề" để làm việc với một ngôn ngữ: trình biên dịch, bộ nối, công cụ phụ. Ví dụ `stable-x86_64-pc-windows-msvc` |
| **Kênh `stable` / `nightly`** | `stable` là bản Rust đã được thử kỹ, ổn định; `nightly` là bản thử nghiệm mỗi đêm, có tính năng mới nhưng dễ đổi |
| **Cargo** | Công cụ của Rust để build, test, tải thư viện và cài công cụ |
| **Crate** | Một "gói" code Rust: thư viện hoặc chương trình. Dandori có nhiều crate trong thư mục `crates/` |
| **Workspace** | Một dự án gồm nhiều crate/gói được quản lý chung |
| **Lockfile** (`Cargo.lock`, `pnpm-lock.yaml`) | File ghi chính xác phiên bản của mọi thư viện đang dùng, để ai build cũng ra cùng kết quả |
| **Lint / clippy** | Công cụ tự soát code để cảnh báo chỗ dễ sai. Clippy là bộ soát của Rust |
| **rustfmt** | Công cụ tự căn chỉnh định dạng code Rust cho thống nhất |
| **PATH** | Danh sách thư mục mà terminal tìm lệnh. Lệnh nằm ngoài PATH thì báo "not recognized" |
| **CLI** | Chương trình điều khiển bằng dòng lệnh, ví dụ `claude`, `git`, `cargo` |
| **ADR** | *Architecture Decision Record*: một trang ghi một quyết định lớn, gồm bối cảnh, các phương án đã cân nhắc, quyết định và hệ quả |
| **Worktree (git)** | Một thư mục làm việc riêng cho một nhánh, để làm song song không đè nhau |
| **Baseline** | Mốc xuất phát: commit đầu tiên trước khi agent làm gì. Mọi so sánh sau này dựa vào nó |
| **Commit** | Một bản lưu lịch sử trong git, kèm lời mô tả |
| **Module (Dandori)** | Một tính năng bật/tắt được. Dandori có lõi nhỏ và 43 module |
| **Slot / ô cắm** | Chỗ trên giao diện mà module cắm một khối của nó vào |
| **Agent** | Một "nhân viên" AI chạy bằng CLI, ví dụ Claude, Gemini |
