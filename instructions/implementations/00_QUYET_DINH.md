# 00. Nhật ký quyết định

Quyết định đã chốt. Đổi được **trước khi giao việc**; đổi sau thì phải sửa các file liên quan (ghi vào đây).

## Chủ dự án đã chốt (2026-10-30)

| ID | Quyết định | Ghi chú |
| :--- | :--- | :--- |
| D-01 | Phạm vi: **lõi + cả 43 module** | PDF thiết kế liệt kê 43 module (41 chọn ban đầu + 2 bổ sung: đọc báo cáo bằng giọng nói, vùng đệm duyệt trước). Tin nhắn đầu của chủ dự án nêu 12 tính năng; sau đó chọn cả 43. |
| D-02 | Stack **giống Vibe Kanban**: backend **Rust**, frontend **TypeScript/React**. Claude đặt chuẩn code Rust | Chuẩn ở `02_QUY_UOC_RUST.md`, dựa trên tài liệu chính thống (nguồn ghi từng rule). |
| D-04 | Hệ thống **chạy trên máy người dùng** (server cục bộ), dùng đăng nhập CLI sẵn có của máy | Module "Tự host 24/7" là phần mở rộng sau (P4), không đổi mặc định. |
| D-05 | **Kho khóa mã hóa nằm trong lõi**, không phải module | Module "API key dự phòng" bắt buộc cần nó. |

## Claude đặt (đổi được nếu chủ dự án không đồng ý)

| ID | Quyết định | Lý do |
| :--- | :--- | :--- |
| D-03 | `D:\16-agent-os\vibe-kanban` chỉ để **đọc lấy ý tưởng**, **không sao chép mã**. Nó dùng giấy phép Apache-2.0 | Tránh nghĩa vụ giấy phép (giữ thông báo, ghi chú thay đổi) và tránh kéo theo mã không hiểu rõ. Nếu cần chép một đoạn, phải hỏi chủ dự án và thêm `NOTICE`. |
| D-06 | **Chỉ dùng CLI chính thức đã đăng nhập** của nhà cung cấp; không dùng token OAuth ngoài CLI | Theo PDF thiết kế: dùng token ngoài CLI có thể vi phạm điều khoản nhà cung cấp. Chủ dự án chưa phản đối. |
| D-07 | Rust **`stable`**, ghim đúng phiên bản đang cài trên máy ở T01 (ghi vào `rust-toolchain.toml`). **Không dùng nightly** | Vibe Kanban dùng nightly vì `rustfmt.toml` của nó có tùy chọn chỉ nightly (`group_imports`, `imports_granularity`). Ta không dùng tùy chọn đó để tránh phụ thuộc nightly. |
| D-08 | Cơ sở dữ liệu **SQLite** qua **sqlx** (truy vấn kiểm tra lúc biên dịch, cache offline `.sqlx/`) | Chạy cục bộ không cần dịch vụ; sqlx cho kiểm tra kiểu. |
| D-09 | Máy chủ HTTP **axum** trên **tokio** | Phổ biến, cùng họ với Vibe Kanban. |
| D-10 | Thời gian thực dùng **SSE** (Server-Sent Events) một chiều server→trình duyệt; hành động của người dùng là HTTP thường | Đơn giản hơn WebSocket, tự kết nối lại. Đổi sang WebSocket nếu P3-13 (di động) cần hai chiều. |
| D-11 | Thao tác git bằng **lệnh `git` có sẵn** (gọi bằng mảng tham số, không qua shell), không dùng thư viện `git2` | Tránh lỗi build libgit2 trên Windows; hành vi giống git người dùng đang dùng. |
| D-12 | Chia sẻ kiểu Rust → TypeScript bằng **ts-rs**, file sinh ra commit vào `shared/types.ts`, kiểm bằng `generate-types --check` | Cùng cách Vibe Kanban đang làm. |
| D-13 | Monorepo: **cargo workspace** (thư mục `crates/`, phẳng) + **pnpm workspace** (thư mục `frontend/`) | Theo M-CRATES-FLAT-FOLDER của Microsoft Pragmatic Rust Guidelines. |
| D-14 | Frontend: **React + Vite + TypeScript**, không Next.js | Ứng dụng cục bộ, không cần SSR. Hệ quả: rule `FE-ARCH-001` (Next App Router) của Kantanda **không áp dụng**. |
| D-15 | Hai module PDF không xếp vào giai đoạn nào được xếp: **Đọc báo cáo bằng giọng nói → P3** (sau Báo cáo ngày/tuần), **Vùng đệm duyệt trước → P4** (sau Diff, Cổng duyệt, Worktree) | PDF chỉ liệt kê 41 module trong các giai đoạn. |
| D-16 | Đăng nhập: tài khoản cục bộ (mật khẩu băm argon2, phiên bằng cookie `HttpOnly`); người dùng đầu tiên là **chủ hệ thống** | PDF xếp "Đăng ký / đăng nhập" vào lõi. |
| D-17 | Kantanda ở `D:\10-kantanda` được **đọc** làm nguồn rule; Dandori khai báo hồ sơ riêng ở `standards/profile.md` (T01 tạo). Kantanda **chưa có rule Rust**; chuẩn Rust của Dandori ở `02` và có thể đưa ngược vào Kantanda sau | Xem `03_ANH_XA_KANTANDA.md`. |
| D-18 | Mọi module **biên dịch sẵn** trong một binary; bật/tắt là **trạng thái chạy**, không phải cargo feature | Theo PDF: tắt là dừng tiến trình, ẩn menu, từ chối API; dữ liệu giữ nguyên. |
| D-19 | Giao diện theo mockup: **thanh bên trái 7 màn hình**; module đưa giao diện vào qua **hệ thống ô cắm (slot)** khai báo trong `module.json` (`ui.contributes`), không phải một route cho mỗi module | Mockup có menu theo màn hình, còn tính năng theo module (xem `04_GIAO_DIEN.md` mục 3). PDF nói "mục menu" của module nên đây là diễn giải của Claude |
| D-20 | Vùng chạm hiệu dụng 44px (`{{touch_target}}` = 44) trong khi nút nhìn cao 36px như mockup | Dung hòa Kantanda `UI-LAW-002` với mockup |
| D-21 | **Chủ dự án tự commit và tự review; agent không ghi vào git** (không `git add`, `commit`, `push`, `worktree`, nhánh...), chỉ dùng git để đọc. QC2 kiểm trên **bản sao sạch** dựng từ danh sách file của git thay cho `git worktree`; mỗi báo cáo có mục "Đề xuất commit"; mặc định dừng sau mỗi task | Quyết định của chủ dự án 2026-10-30. Hệ quả: mỗi task nên được duyệt và commit trước khi giao task kế để `git diff` chỉ hiện một task; mốc khóa cấu hình là commit do chủ dự án tạo; chạy tự động nhiều task chỉ khi chủ dự án nói rõ "chạy liền" |
| D-22 | **Cấu trúc mã nguồn chuẩn** (ADR 0020): giao diện ở **`frontend/`** (thay `web/`); `members = ["crates/*"]`; crate sinh tự động **`dandori-modules`** làm cầu nối (server không phụ thuộc trực tiếp `mod-*`); `resolver` ghi rõ; chương trình tên **`dandori`**; `main.rs` mỏng + `lib.rs`; module kiểu `foo.rs`; test xuyên suốt Rust ở `crates/dandori-e2e`; ghim `packageManager` pnpm và `engines` Node | Kiểm T01 + nghiên cứu Cargo Book/Rust Book (`docs/cong-nghe/cau-truc-rust-va-trien-khai.md`). Chủ dự án giao Claude chốt "cấu trúc chuẩn nhất có thể" |
| D-23 | **Vận hành** (RS-OPS-*): tắt êm bắt cả Ctrl+C và SIGTERM; cấu hình qua biến môi trường; nhật ký ra stdout; `[profile.release]` hiện chỉ `strip = true`, `lto`/`codegen-units` chốt sau khi đo (P5-01); `panic` giữ `unwind` | 12-factor, Tokio, Cargo Book; lý do `unwind` ở RS-PANIC-003 |
| D-24 | Chủ dự án chọn **Rust** cho backend (không đổi sang Node/Python/Next.js) và **không clone Antigravity**; chỉ học ý tưởng từ tài liệu công khai. Mọi ý tưởng đưa vào danh sách ý tưởng, làm sau cổng P1 | Quyết định 2026-10-31; lý do: Antigravity là phần mềm có bản quyền, khác loại sản phẩm với Dandori |

## Giả định CHƯA KIỂM CHỨNG (agent phải kiểm hoặc đưa vào checklist của chủ dự án)

| ID | Giả định | Cách xử lý |
| :--- | :--- | :--- |
| A-01 | Mỗi CLI agent hỗ trợ chạy không tương tác (headless) và cho phép đặt thư mục HOME riêng cho từng account | P1-02 phải đọc `--help` thật của từng CLI và ghi kết quả; chỗ nào không chắc thì ghi "CHỜ NGƯỜI DÙNG KIỂM" |
| A-02 | Nhà cung cấp trả được số liệu hạn mức | PDF đã nêu rủi ro: có nhà không trả. P2-09 chỉ **ước lượng từ lịch sử dùng** khi không có số liệu, và nói rõ trên giao diện |
| A-03 | Dự tính thời điểm xong của task có thể làm tốt | PDF: chưa có mẫu mã nguồn mở tốt. P3-07 làm bản đơn giản dựa trên lịch sử, hiển thị khoảng và độ tin cậy, không cam kết chính xác |
| A-04 | Sandbox khả thi trên Windows | Chưa biết. P4-06 mở đầu bằng spike và ADR; có thể chọn "chỉ hỗ trợ Linux/WSL" |
| A-05 | Đồ thị tri thức và PM phân rã đồ thị task làm được ở mức dùng được | P4-01, P4-07 làm theo từng bước có cổng |

## Chờ chủ dự án (mặc định an toàn ghi kèm)

| ID | Việc | Mặc định nếu chưa trả lời |
| :--- | :--- | :--- |
| W-01 | Cài Rust và công cụ build C++ (Bước 0a) | **Chặn**: không bắt đầu T01 khi chưa có |
| W-02 | Có theo bố cục mockup `Agent Hub – Giao diện chính.html` không; và vị trí giao diện của 5 module mockup không có (`agent-messaging`, `voice-report`, `staging-buffer`, đăng nhập, `knowledge-graph` tab riêng) | Theo mockup về bố cục và nhãn tiếng Việt; dữ liệu minh họa không đưa vào sản phẩm; vị trí 5 module đó theo đề xuất ở `04_GIAO_DIEN.md` mục 3.3 |
| W-03 | Tên binary và thư mục dữ liệu | **Đã chốt (D-22):** binary `dandori`; dữ liệu ở thư mục dữ liệu ứng dụng của hệ điều hành (qua crate `directories`), không phải trong repo |
| W-04 | Tài khoản dịch vụ ngoài để kiểm cuối (Telegram bot, Discord/Slack webhook, GitHub, Linear) | Test tự động dùng máy chủ giả; việc kiểm thật nằm trong checklist thủ công |
| W-05 | Máy SSH để kiểm P4-05 | Test dùng sshd cục bộ hoặc giả; kiểm thật do chủ dự án |
| W-06 | Có mở mã nguồn không (ảnh hưởng giấy phép và README) | Chưa chọn giấy phép; không thêm file LICENSE |
