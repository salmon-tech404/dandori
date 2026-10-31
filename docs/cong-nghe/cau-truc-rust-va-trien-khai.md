# Cấu trúc dự án Rust và triển khai: nghiên cứu có nguồn

Ngày nghiên cứu: **2026-10-31**. Người làm: Claude (đọc trực tiếp tài liệu; các trang đã đọc ghi rõ bên dưới).
Câu hỏi của chủ dự án: (1) bản Rust production có chạy được trên Vercel, Render không; (2) cấu trúc thư mục và vận hành Rust sao cho hợp lý, tối ưu, có nguồn chuẩn.

## 0. Nguồn và độ tin cậy

| Mã | Nguồn | Loại | Ghi chú |
| :--- | :--- | :--- | :--- |
| **CB** | [Cargo Book: Package layout](https://doc.rust-lang.org/cargo/guide/project-layout.html), [Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html), [Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html) | Chính thức (dự án Rust) | Đã đọc cả ba trang. Chưa đọc trang `resolver` riêng |
| **RB** | [Rust Book chương 7](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html) (6 trang) | Chính thức | Đã đọc |
| **MK** | [matklad: Large Rust Workspaces](https://matklad.github.io/2021/08/22/large-rust-workspaces.html) | **Ý kiến** của tác giả rust-analyzer (ông tự nói "không phải chuẩn, chỉ là kinh nghiệm") | Đã đọc. Dựa trên một dự án (rust-analyzer, khoảng 200k dòng), không có số đo |
| **MS** | [Microsoft Pragmatic Rust Guidelines](https://microsoft.github.io/rust-guidelines/) | Hướng dẫn của một công ty | Chỉ đọc một phần nguyên văn (xem `02_QUY_UOC_RUST.md`) |
| **DK** | [Docker: hướng dẫn Rust](https://docs.docker.com/guides/rust/) | Chính thức (Docker) | Đã đọc |
| **CC** | [Luca Palmieri: cargo-chef](https://www.lpalmieri.com/posts/fast-rust-docker-builds/) | Ý kiến và đo đạc của tác giả, **năm 2020** | Đã đọc; phiên bản trong bài đã cũ |
| **TK** | [Tokio: Graceful Shutdown](https://tokio.rs/tokio/topics/shutdown), [ví dụ axum](https://github.com/tokio-rs/axum/tree/main/examples/graceful-shutdown) | Chính thức | Đã đọc |
| **12F** | [12factor: Config](https://12factor.net/config), [Logs](https://12factor.net/logs), [Disposability](https://12factor.net/disposability) | Phương pháp luận phổ biến (2017) | Đã đọc |
| **VC** | Vercel: [Rust runtime](https://vercel.com/docs/functions/runtimes/rust), [Giới hạn](https://vercel.com/docs/functions/limitations), [Runtimes](https://vercel.com/docs/functions/runtimes), [Functions](https://vercel.com/docs/functions) | Chính thức (Vercel) | Đã đọc. Hai trang phụ bị từ chối truy cập (Docker, WebSocket) nên **không** có kết luận về chúng |
| **RD** | Render: [Native runtimes](https://render.com/docs/native-runtimes), [Disks](https://render.com/docs/disks), [Free](https://render.com/docs/free), [Web services](https://render.com/docs/web-services), [Docker](https://render.com/docs/docker), [WebSocket](https://render.com/docs/websocket), [Rocket (Rust)](https://render.com/docs/deploy-rocket-rust) | Chính thức (Render) | Đã đọc |

Quy ước: **[CB/RB/TK/DK/VC/RD]** = tài liệu chính thức; **[MK/CC/12F]** = ý kiến hoặc kinh nghiệm; **[suy luận]** = Claude suy ra từ tài liệu, không có trong tài liệu.

---

## PHẦN 1. Có chạy được trên Vercel hoặc Render không

### 1.1 Vercel

| Mục | Tài liệu nói gì |
| :--- | :--- |
| Hỗ trợ Rust | Có, ở trạng thái **Beta** trên mọi gói (trang ghi ngày 2025-12-08, có thể đã đổi). Mỗi tệp trong thư mục `api/` là một "function", dùng crate `vercel_runtime` [VC Rust] |
| Mô hình chạy | Mỗi yêu cầu là **một lần gọi mới**; function **về 0 khi không có yêu cầu** [VC Functions] |
| Thời gian tối đa | Hobby: 300 giây. Pro/Enterprise: tối đa 800 giây (1800 giây là Beta, **không áp dụng cho Rust** theo trang liệt kê) [VC Giới hạn] |
| Ổ đĩa | **Chỉ đọc**, chỉ `/tmp` ghi được (tối đa 500 MB, là "chỗ nháp") [VC Runtimes] |
| Lưu dữ liệu giữa các yêu cầu | **Không có** trong Functions. Phải dùng dịch vụ lưu trữ riêng của Vercel [VC Runtimes] |
| Chạy chương trình con (như `git`, CLI) | **Tài liệu không nói.** Chưa xác nhận được |
| Chạy nhiều giờ | **Không**, trong một lần gọi. Việc dài phải dùng Workflows (chạy theo từng bước có thể tạm dừng) [VC Giới hạn, Workflows] |
| Container từ Dockerfile | Có nhắc đến một runtime "Container"; **chưa đọc chi tiết** nên không kết luận [VC Runtimes] |

### 1.2 Render

| Mục | Tài liệu nói gì |
| :--- | :--- |
| Hỗ trợ Rust | **Có**, là một runtime native (nền Debian 12). Ví dụ chính thức: build `cargo build --release`, chạy `cargo run --release` [RD Native, Rocket] |
| Docker | Có. Render tự build Dockerfile; hỗ trợ build nhiều giai đoạn [RD Docker] |
| Cổng mạng | Dịch vụ phải **nghe ở `0.0.0.0`**, nên dùng biến môi trường `PORT` (mặc định 10000). Chỉ một cổng HTTP ra công khai [RD Web services] |
| Ổ đĩa bền vững (persistent disk) | **Chỉ dịch vụ trả phí** (web service, private service, background worker). Chỉ **một bản chạy** có thể dùng đĩa; không mở rộng ngang; không dùng được lúc build. Có ảnh chụp tự động mỗi 24 giờ, giữ 7 ngày [RD Disks] |
| Gói miễn phí | **Ngủ sau 15 phút** không có yêu cầu (bật lại mất khoảng 1 phút). Hệ thống tệp tạm thời: **tệp SQLite cục bộ bị mất** khi khởi động lại. **Không có shell/SSH**. Giới hạn 750 giờ/tháng [RD Free] |
| Chạy lâu | Tài liệu **không** ghi giới hạn thời gian chạy của web service, nhưng instance bị thay thế định kỳ khi triển khai và bảo trì. Khi tắt: gửi **SIGTERM**, cho 30 giây (kéo dài tới 300 giây) [RD WebSocket] |
| Chạy chương trình con | **Suy luận**: các công cụ như `git`, `sqlite3` có sẵn lúc chạy [RD Native]; tài liệu không nói trực tiếp về việc sinh tiến trình con |
| Đăng nhập CLI bằng trình duyệt | **Tài liệu không đề cập.** Chưa xác nhận |

### 1.3 Kết luận cho Dandori

Dandori **khác** một web API thông thường ở bốn điểm, và chính chúng quyết định chỗ chạy:

| Dandori cần | Vercel Functions | Render |
| :--- | :--- | :--- |
| Một tiến trình **chạy liên tục** nhiều giờ (agent chạy lâu, hàng đợi qua đêm) | Không (tối đa 300-800 giây/lần gọi) | Có thể, trên dịch vụ trả phí (instance vẫn bị thay thế khi triển khai) |
| **Tệp SQLite và worktree git bền vững** trên đĩa | Không (đĩa chỉ đọc, `/tmp` là nháp) | Chỉ khi có persistent disk (trả phí, một bản chạy) |
| **Sinh chương trình con** (`git`, `claude`, `agy`) | Chưa xác nhận | Có vẻ được *(suy luận)* |
| CLI agent **đã đăng nhập sẵn** (đăng nhập bằng trình duyệt) | Không có cơ chế | Chưa xác nhận; bất tiện vì chỉ một cổng công khai và không có shell trên gói miễn phí |

- **Vercel Functions: không phù hợp** với kiến trúc Dandori hiện tại. Đây là kết luận từ chính các giới hạn trong tài liệu, không phải do Rust.
- **Render: về kỹ thuật làm được** (gói trả phí, Docker hoặc native, kèm persistent disk), nhưng vẫn là việc lớn: phải mở Dandori ra Internet, cài và đăng nhập từng CLI agent **trên máy chủ**, và giải quyết bảo mật. Đó chính là module `self-host` (P4-09, có cổng chủ dự án) trong kế hoạch.
- **Điều quan trọng nhất:** đây không phải vấn đề của ngôn ngữ Rust. Một web API không trạng thái viết bằng Rust chạy tốt trên Render. Dandori khó vì nó là **ứng dụng điều phối chạy lâu, giữ trạng thái và dùng đăng nhập cá nhân**, vốn hợp với máy của người dùng.
- Phần chưa xác nhận (sinh tiến trình con và đăng nhập CLI trên Render, Container/Daemons của Vercel, WebSocket của Vercel) cần thử thật nếu có ý định triển khai.

---

## PHẦN 2. Cấu trúc dự án Rust: nguồn nói gì và Dandori đang ở đâu

### 2.1 Những điều tài liệu chính thức khẳng định

| Điều khoản | Nguồn |
| :--- | :--- |
| Một **package** chứa tối đa **một** crate thư viện và **bao nhiêu crate chương trình** cũng được; `src/main.rs` và `src/lib.rs` là gốc mặc định; chương trình phụ để ở `src/bin/` | RB ch07-01; CB Layout |
| `tests/` là test tích hợp, `benches/`, `examples/` có chỗ riêng; tên chương trình viết kiểu `kebab-case`, module viết `snake_case` | CB Layout |
| Workspace chia sẻ **một `Cargo.lock`** và **một thư mục `target/`**; phần `[profile.*]` chỉ đọc ở file gốc | CB Workspaces, Profiles |
| **Virtual manifest** (file gốc chỉ có `[workspace]`, không có `[package]`) hợp với dự án nhiều gói tách riêng; khi đó **phải ghi rõ `resolver`**; ví dụ chính thức dùng `resolver = "3"` | CB Workspaces |
| Nên đặt các gói **phẳng** trong `crates/` và khai báo `members = ["crates/*"]` | CB Workspaces |
| `[workspace.package]`, `[workspace.dependencies]` (khai báo một lần, dùng `workspace = true`), `[workspace.lints]` | CB Workspaces (lints cần Rust 1.74+) |
| Chia module ra file: `src/foo.rs` (kiểu hiện hành) hoặc `src/foo/mod.rs` (kiểu cũ), **không trộn hai kiểu cho cùng một module** | RB ch07-05 |
| `mod` **không phải** là lệnh "include"; module mặc định riêng tư, phải `pub` mới thấy từ ngoài | RB ch07-02, 07-05 |
| Mặc định `release`: `opt-level = 3`, `debug = false`, `strip = "none"`, `lto = false`, `codegen-units = 16`, `panic = "unwind"` | CB Profiles |

### 2.2 Ý kiến đáng cân nhắc (không phải chuẩn chính thức)

| Ý kiến | Nguồn |
| :--- | :--- |
| Với dự án 10 nghìn đến 1 triệu dòng, dùng cấu trúc **phẳng** (không lồng crate trong crate); thư mục trùng tên crate, **không bỏ tiền tố**. Lý do: không gian tên crate của Cargo vốn phẳng; "cấu trúc cây dễ xuống cấp theo thời gian, phẳng thì không cần bảo trì" | MK |
| Gốc workspace là virtual manifest, tránh để `src/` ở gốc | MK |
| Dùng `version = "0.0.0"` cho crate nội bộ không xuất bản | MK |
| Viết tự động hóa bằng Rust (`cargo xtask`) thay cho `Makefile` hoặc script rời | MK |
| Microsoft: crate xếp phẳng, mọi crate là thành viên workspace, phụ thuộc ngoài khai báo ở `[workspace.dependencies]`, dùng `#[expect(..., reason)]` thay `#[allow]` | MS (qua bản tóm tắt, xem `02_QUY_UOC_RUST.md`) |

### 2.3 Đối chiếu cấu trúc thật của Dandori (đọc `Cargo.toml` ngày 2026-10-31)

| # | Điều đã đúng | Nguồn |
| :--- | :--- | :--- |
| ✅ | Gốc là virtual manifest, các gói phẳng trong `crates/`, thư mục trùng tên crate, giữ tiền tố `dandori-` | CB, MK |
| ✅ | `[workspace.package]`, `[workspace.dependencies]`, `[workspace.lints]` đều dùng; mỗi crate `workspace = true` | CB |
| ✅ | Có `tests/` là chỗ test tích hợp của từng crate (kế hoạch có rule `RS-TEST-002`) | CB Layout |

| # | Phát hiện cần xử lý | Vì sao | Mức |
| :--- | :--- | :--- | :--- |
| **A** | `members` đang **liệt kê từng crate**; tài liệu khuyên `members = ["crates/*"]` | Thiết kế yêu cầu "thêm một module chỉ cần một thư mục và một file khai báo, **không sửa lõi**" (tiêu chí PDF). Với danh sách tường minh, mỗi `mod-*` mới phải sửa `Cargo.toml` gốc | Nên sửa |
| **B** | **Khoảng trống trong kế hoạch:** `gen-modules` chỉ sinh `modules_generated.rs`, nhưng `dandori-server` **chưa khai báo** `mod-*` nào trong `[dependencies]` | Code Rust chỉ dùng được crate đã khai báo phụ thuộc. Vì vậy thêm module vẫn phải sửa `Cargo.toml` của server, trái tiêu chí "không sửa lõi" | **Quan trọng** |
| **C** | `resolver = "2"`, ví dụ chính thức ghi `"3"` | Trong virtual workspace phải ghi rõ; chưa đọc trang `resolver` nên chưa rõ khác biệt | Cần kiểm |
| **D** | **Chưa có `[profile.release]`** | Mặc định release giữ `strip = "none"` và `lto = false`. Chưa chọn cấu hình phát hành | Nên bổ sung trước P5 |
| **E** | Kế hoạch đặt test xuyên suốt ở `tests/e2e/**` **ngoài mọi crate** | Gốc là virtual manifest, **không có `[package]`**, nên `cargo test` không biên dịch `tests/` ở gốc. Test Rust phải nằm trong một crate. Chỉ kịch bản Playwright (Node) mới hợp ở gốc | Cần làm rõ |
| **F** | Chưa có rule về `foo.rs` hay `mod.rs` | RB: không trộn hai kiểu | Nhỏ |
| **G** | Kế hoạch **chưa nhắc SIGTERM** khi tắt chương trình | Môi trường chạy Linux/Render gửi SIGTERM; ví dụ chính thức xử lý cả Ctrl+C và SIGTERM [TK, 12F] | Cần cho P4-09 |

### 2.4 Đề xuất sửa (chưa áp dụng, chờ chủ dự án đồng ý)

**A và B (cùng một giải pháp):** thêm một crate sinh tự động `crates/dandori-modules` làm **cầu nối**: `gen-modules` ghi cả `Cargo.toml` (liệt kê mọi `mod-*` là phụ thuộc) lẫn `lib.rs` của crate này; `dandori-server` chỉ phụ thuộc đúng **một** crate `dandori-modules`. Kèm `members = ["crates/*"]`. Khi đó thêm module = thêm một thư mục + `module.json` + chạy `pnpm run gen:modules`, đúng tiêu chí gốc. Ghi chú: crate cầu nối cũng được thêm vào luật phụ thuộc ở `01_KIEN_TRUC.md` mục 3.

**D:** bổ sung vào `Cargo.toml` gốc khi tới giai đoạn đóng gói (cần **đo** trước khi chọn từng giá trị, vì `lto` và `codegen-units` đánh đổi thời gian build lấy tốc độ chạy [CB Profiles]):

```toml
[profile.release]
strip = true             # bỏ ký hiệu gỡ lỗi khỏi file chạy, file nhỏ hơn (Vibe Kanban cũng làm vậy)
lto = "thin"             # ứng cử viên; đo trước khi chốt
# panic: GIỮ MẶC ĐỊNH "unwind". Kế hoạch (RS-PANIC-003) dựa vào việc một worker panic
# chỉ làm dừng worker đó chứ không sập cả chương trình; "abort" sẽ phá điều này [suy luận từ kế hoạch]
```

**E:** test Rust xuyên suốt đặt trong một crate test riêng (ví dụ `crates/dandori-e2e/tests/`) hoặc trong `dandori-server/tests/`; `tests/e2e/` ở gốc chỉ dùng cho kịch bản Node/Playwright.

**G:** khi tắt, xử lý cả Ctrl+C lẫn SIGTERM; dùng `CancellationToken` để báo cho các task và `TaskTracker` để chờ chúng xong [TK]; đặt thời hạn tắt êm thấp hơn thời gian nhà cung cấp cho phép (Render: mặc định 30 giây) [RD WebSocket]. Đã phù hợp với rule `RS-ASYNC-003` của kế hoạch.

---

## PHẦN 3. Vận hành khi đưa lên máy chủ

### 3.1 Nguyên tắc chung (12-factor)

| Nguyên tắc | Nguồn | Dandori hiện tại |
| :--- | :--- | :--- |
| **Cấu hình qua biến môi trường** | 12F Config | Đã có: `DANDORI_DATA_DIR`. Nên mở rộng cho cổng, địa chỉ nghe khi tới P4-09 |
| **Nhật ký ghi ra stdout** dưới dạng luồng sự kiện, ứng dụng không tự lo nơi lưu | 12F Logs | Phù hợp: kế hoạch dùng `tracing` có cấu trúc, cấm `println!` |
| **Khởi động nhanh, tắt êm khi nhận SIGTERM** | 12F Disposability | Tắt êm đã có trong kế hoạch; **SIGTERM chưa được nhắc** (mục G) |

### 3.2 Đóng gói Docker (khi cần chạy trên Render hoặc máy chủ)

Theo hướng dẫn chính thức của Docker [DK]:
- **Build nhiều giai đoạn:** giai đoạn `build` có trình biên dịch, giai đoạn `final` chỉ chứa **file chạy**.
- Lệnh build: `cargo build --locked --release`.
- **Chạy bằng người dùng không phải root** (tài liệu gọi là "thực hành tốt khuyến nghị").
- Tăng tốc build bằng **bộ nhớ đệm BuildKit** (`--mount=type=cache`) cho `target/` và thư mục cargo.
- Lựa chọn thay thế: **cargo-chef** tách bước tải và biên dịch phụ thuộc thành một lớp riêng. Tác giả đo trên một dự án thương mại: khoảng 10 phút xuống khoảng 2 phút [CC, **số của tác giả, năm 2020**]. Docker chính thức không dùng cargo-chef.
- **Chưa nghiên cứu:** chọn ảnh nền cuối cùng (Debian slim, Alpine/musl, distroless). Docker hướng dẫn dùng Alpine/musl; bài cargo-chef chỉ dùng Debian slim. Lưu ý: kế hoạch có thư viện `reqwest` dùng `rustls`, nên **không** phụ thuộc OpenSSL, thuận lợi cho musl *(suy luận, cần thử)*.

### 3.3 Riêng với Dandori

Dandori không phải dịch vụ web thông thường. Khi tới `self-host` (P4-09), ngoài Docker còn phải xử lý: CLI agent cần cài và đăng nhập **bên trong** container hoặc máy chủ, thư mục dữ liệu phải nằm trên đĩa bền vững (Render: persistent disk gắn vào đường dẫn như `/app/...` với Docker [RD Disks]), và chỉ một bản chạy được dùng đĩa. Đó là lý do P4-09 là một **[CỔNG CHỦ]**, và ADR 0018 nên đọc lại các ghi chú ở tài liệu này.

---

## Việc còn mở

1. Đọc trang [Cargo: resolver](https://doc.rust-lang.org/cargo/reference/resolver.html) để quyết `resolver = "2"` hay `"3"` (mục C).
2. Quyết định có áp dụng đề xuất A+B (crate cầu nối `dandori-modules`), D, E, G không.
3. Nếu có ý định triển khai thật: thử trên Render (gói trả phí, Docker, persistent disk) để xác nhận các mục "chưa xác nhận" ở Phần 1.
