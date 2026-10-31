# 02. Chuẩn code Rust của Dandori

Phạm vi: mọi mã Rust trong `crates/` và `scripts` Rust (nếu có). Mã TypeScript theo `03_ANH_XA_KANTANDA.md`.

Kantanda (`D:\10-kantanda`) **chưa có rule Rust**; file này là chuẩn Rust của riêng Dandori. Nó có thể được đưa ngược vào Kantanda như một module `rust` sau khi dùng thử (D-17). Mỗi rule có: **ID** (tiền tố `RS-`), **tier** theo định nghĩa của Kantanda (T1 = công cụ bắt được, chặn commit; T2 = cần review, gợi ý; T3 = hướng dẫn, không chặn), **nguồn**, **cách kiểm**.

## 1. Nguồn và độ tin cậy

Chuẩn này dựa trên các nguồn dưới đây. Cột "Đã đọc" cho biết Claude đã đọc đến đâu khi viết (2026-10-30), để không ai tin hơn mức có thể.

| Mã | Nguồn | Loại | Đã đọc |
| :--- | :--- | :--- | :--- |
| **API** | [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) (checklist `C-*`) | Chính thức (dự án Rust) | Đọc danh sách 48 mục checklist; chưa đọc phần giải thích từng mục |
| **CLP** | [Clippy docs](https://doc.rust-lang.org/clippy/) và [Cargo reference, mục `[lints]`](https://doc.rust-lang.org/cargo/reference/manifest.html#the-lints-section) | Chính thức | Nhóm lint và mặc định; phát biểu "không bật cả nhóm `restriction`"; cấu hình `[lints]` (Cargo tôn trọng từ 1.74). **Chưa xác nhận** nhóm của `unwrap_used`, `panic`, `todo`, `print_stdout`, `indexing_slicing` (xem mục 6) |
| **MS** | [Microsoft Pragmatic Rust Guidelines](https://microsoft.github.io/rust-guidelines/) (`M-*`, bản dành cho agent) | Hướng dẫn của một công ty, công khai | Chỉ ~29% nguyên văn (các mục AI, Application, Correctness, Documentation, FFI, Macros); phần Performance, Project, Universal, Libraries chỉ qua **bản tóm tắt do công cụ sinh**, ID có thể sai |
| **TOK** | [Tokio tutorial: shared state](https://tokio.rs/tokio/tutorial/shared-state) và tài liệu Tokio | Chính thức (dự án Tokio) | Chỉ trang shared state. Phần hủy bỏ/tắt êm lấy từ hướng dẫn cộng đồng |
| **COM** | Đồng thuận cộng đồng (thiserror cho thư viện, anyhow cho ứng dụng; cargo-deny/cargo-audit/RustSec) | Thứ cấp (blog, trang hướng dẫn) | Chưa đọc tài liệu chính thức của các công cụ; cú pháp cấu hình phải kiểm bằng `--help` |
| **VK** | `D:\16-agent-os\vibe-kanban` (`package.json`, `Cargo.toml`, `rustfmt.toml`) | Thực tế một dự án cùng loại | Đã đọc các tệp cấu hình; chỉ để tham khảo thực hành (D-03) |
| **DAN** | Quyết định của Dandori | Nội bộ | — |

> **Quy tắc đọc bảng dưới:** rule mang nguồn MS chưa-đọc-nguyên-văn hoặc COM là **đề xuất tốt, chưa phải chân lý**. Nếu Dev thấy một rule mâu thuẫn với tài liệu gốc, ghi vào "Đề xuất" của báo cáo; không tự bỏ rule.

## 2. Rule

### 2.1 Định dạng, lint, công cụ

| ID | Rule | Tier | Nguồn | Cách kiểm |
| :--- | :--- | :---: | :--- | :--- |
| RS-FMT-001 | Mã được định dạng bằng `rustfmt` mặc định, edition 2024 hoặc mới nhất ổn định. **Chỉ dùng tùy chọn rustfmt chạy được trên `stable`** (không `group_imports`/`imports_granularity` của nightly) | T1 | API, VK (VK dùng nightly, ta không theo) | `cargo fmt --all --check` trong `verify` |
| RS-LINT-001 | Không cảnh báo clippy nào được bỏ qua: chạy với `-D warnings` trên mọi target | T1 | CLP, VK (VK cũng chạy `cargo clippy --workspace --all-targets -- -D warnings`) | `cargo clippy --workspace --all-targets -- -D warnings` |
| RS-LINT-002 | Mức lint khai báo **một nơi**: `[workspace.lints]` ở `Cargo.toml` gốc; mỗi crate có `[lints] workspace = true`. Không đặt `#![allow/warn/deny(...)]` rải rác ở đầu file | T1 | CLP/Cargo reference | `scripts/check-lints.mjs`: mọi `Cargo.toml` crate phải có `lints.workspace = true`; không file `.rs` nào có `#![allow(` |
| RS-LINT-003 | Ngoại lệ lint dùng `#[expect(<lint>, reason = "...")]`, **không** `#[allow]`; mọi attribute lint phải có `reason` | T1 | MS `M-LINT-OVERRIDE-EXPECT` (tóm tắt), CLP (`allow_attributes_without_reason`: nhóm restriction, mặc định allow, đã xác nhận) | `clippy::allow_attributes_without_reason = "deny"` + `clippy::allow_attributes = "deny"` (T01 xác nhận tên lint) |
| RS-LINT-004 | Không bật cả nhóm `clippy::restriction` hay `clippy::pedantic`. Chọn **từng lint** (danh sách ở mục 3) | T1 | CLP ("không nên bật cả restriction"; pedantic: nên chọn lọc) | `check-lints.mjs` từ chối `restriction =`/`pedantic =` trong `[workspace.lints]` |
| RS-LINT-005 | Tên lint trong cấu hình phải **đúng**: lint không tồn tại làm `verify` đỏ, không bị lặng lẽ bỏ qua | T1 | DAN | T01 chứng minh bằng thử nghiệm (thêm một tên lint giả, `verify` phải đỏ) rồi gỡ |
| RS-LINT-006 | Hàm không dài hơn `{{max_fn_lines}}` dòng (đặt **50** cho Rust ở `standards/profile.md`) | T1 | Kantanda `CODE-FUN-002`; CLP (`too_many_lines`, nhóm pedantic, chọn từng lint) | `clippy::too_many_lines` + `too-many-lines-threshold` trong `clippy.toml` |

### 2.2 An toàn bộ nhớ và bất định

| ID | Rule | Tier | Nguồn | Cách kiểm |
| :--- | :--- | :---: | :--- | :--- |
| RS-UNSAFE-001 | **Cấm `unsafe`** trong toàn workspace. Cần `unsafe` (ví dụ FFI) thì ADR + hỏi chủ dự án; khi được duyệt, mỗi khối có `// SAFETY:` và chạy Miri | T1 | MS `M-UNSAFE`, `M-UNSAFE-IMPLIES-UB`, `M-UNSOUND` (nguyên văn); Cargo reference (ví dụ `unsafe_code = "forbid"`) | `[workspace.lints.rust] unsafe_code = "forbid"` |
| RS-PANIC-001 | **Panic chỉ cho lỗi lập trình** (vi phạm bất biến nội bộ), không dùng cho lỗi đầu vào, I/O, mạng, dữ liệu người dùng. Panic có thông điệp rõ kèm giá trị liên quan | T2 | MS `M-PANIC-IS-STOP`, `M-PANIC-ON-BUG`, `M-PANIC-MESSAGE` (nguyên văn) | Review; RS-PANIC-002 chặn các đường dễ panic |
| RS-PANIC-002 | Trong mã chạy thật (không phải test) cấm: `unwrap()`, `expect()`, `panic!`, `todo!`, `unimplemented!`, `dbg!`. Trong test cho phép `unwrap`/`expect`/`panic`/chỉ mục | T1 | CLP (các lint restriction chọn từng cái); MS `M-LOG-NOT-PRINT` | `clippy::unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `dbg_macro` = `deny`; `clippy.toml`: `allow-unwrap-in-tests`, `allow-expect-in-tests`, `allow-panic-in-tests` (T01 xác nhận tên khóa; clippy báo lỗi với khóa lạ) |
| RS-PANIC-003 | Không `catch_unwind` để "tiếp tục chạy". Panic ở worker nền → worker dừng, registry ghi `worker.failed` và có thể khởi động lại có kiểm soát | T2 | MS `M-PANIC-CONTINUATION` (nguyên văn) | Review |
| RS-PANIC-004 | Cắt/chỉ mục trên dữ liệu **từ ngoài vào** (đầu ra CLI, JSON, đường dẫn) dùng `get(..)`/`split_at_checked`, không `a[i]`/`a[x..y]` | T1 | CLP (`indexing_slicing`, restriction; T01 xác nhận) | `clippy::indexing_slicing = "warn"` trong crate xử lý dữ liệu ngoài; có thể `#[expect]` có lý do |

### 2.3 Lỗi

| ID | Rule | Tier | Nguồn | Cách kiểm |
| :--- | :--- | :---: | :--- | :--- |
| RS-ERR-001 | Crate thư viện trả **kiểu lỗi có kiểu** (`thiserror`), có ý nghĩa, implement `Error + Display + Send + Sync`. **Không** `Box<dyn Error>`/`anyhow` trong crate thư viện | T2 | API `C-GOOD-ERR`, `C-SEND-SYNC`; COM (thiserror cho thư viện) | Review + `check-lints` cấm `anyhow` trong `Cargo.toml` của crate ngoài `dandori-server` |
| RS-ERR-002 | `anyhow` **chỉ** ở `dandori-server` (ứng dụng) và **một loại lỗi ứng dụng duy nhất** | T1 | MS `M-APP-ERROR` (nguyên văn); COM | `check-architecture.mjs` |
| RS-ERR-003 | Lỗi nội bộ không rò ra API: một kiểu `ApiError` duy nhất ánh xạ lỗi nội bộ → `{code, message an toàn, request_id}`; không stack trace, không đường dẫn tuyệt đối | T1 | Kantanda `BE-ERR-002` | Test: gây lỗi nội bộ, khẳng định phản hồi không chứa đường dẫn/`panicked at` |
| RS-ERR-004 | Không nuốt lỗi: cấm `let _ = <Result>`; dùng `?`, hoặc xử lý/ghi log có chủ ý | T1 | CLP (`let_underscore_must_use`, restriction, T01 xác nhận) | lint tương ứng `= "deny"` |
| RS-ERR-005 | Lỗi crate thư viện **không** làm lộ kiểu của thư viện bên thứ ba trong API công khai khi tránh được | T3 | MS `M-DONT-LEAK-TYPES` (tóm tắt) | Review |

> Ghi chú lệch chuẩn: MS `M-ERRORS-CANONICAL-STRUCTS` (tóm tắt) khuyến lỗi dạng struct theo tình huống. Các crate của Dandori là **nội bộ** (`publish = false`), nên enum lỗi theo crate là chấp nhận được. Ghi nhận ở đây, không phải bỏ sót.

### 2.4 Async và đồng thời

| ID | Rule | Tier | Nguồn | Cách kiểm |
| :--- | :--- | :---: | :--- | :--- |
| RS-ASYNC-001 | **Không giữ khóa qua `.await`**: không giữ `std::sync::Mutex`/`RwLock` guard hay `RefCell` borrow qua `.await`. Cần giữ qua await thì dùng `tokio::sync::Mutex`, hoặc tốt hơn: một task sở hữu trạng thái và nhận tin nhắn | T1 | TOK (đã đọc: hai cách — mutex hoặc task sở hữu trạng thái; guard `std` không `Send`); CLP (`await_holding_lock`, `await_holding_refcell_ref`: nhóm `suspicious`, warn, **đã xác nhận**) | `clippy::await_holding_lock`, `await_holding_refcell_ref` = `deny` |
| RS-ASYNC-002 | **Không chặn luồng async:** cấm `std::thread::sleep`, I/O tệp đồng bộ nặng, tính toán CPU dài trong hàm async. Dùng `tokio::time::sleep`, `tokio::fs`, hoặc `tokio::task::spawn_blocking` | T1/T2 | TOK; MS `M-YIELD-POINTS` (tóm tắt) | `clippy.toml` `disallowed-methods` cấm `std::thread::sleep`, `std::fs::*` trong crate async (T01 xác nhận cú pháp); phần còn lại T2 review |
| RS-ASYNC-003 | **Mọi task sinh ra có chủ**: giữ `JoinHandle`/`JoinSet`, có `CancellationToken`; tắt êm có timeout; cấm `tokio::spawn` "bắn rồi quên" cho việc quan trọng | T2 | COM (CancellationToken, JoinSet, timeout khi tắt); TOK chưa đọc phần này | Review + test tắt êm ở T05/T14 |
| RS-ASYNC-004 | **Worker nền tôn trọng hủy**: vòng lặp `select!` có nhánh `token.cancelled()`; hủy rồi thì không còn tiến trình/lịch của module | T1 | PDF ("tắt là tắt thật"); COM | Test mỗi module có worker: tắt module → worker thoát trong thời gian giới hạn |
| RS-ASYNC-005 | **Hủy an toàn:** mọi `.await` có thể bị drop. Không để trạng thái nửa vời; việc dọn dẹp cần async không đặt trong `Drop` (dùng task dọn riêng hoặc bước dọn tường minh) | T3 | COM; TOK chưa đọc phần này | Review |
| RS-ASYNC-006 | Future hot lớn: không giữ biến lớn qua `.await`; `Box::pin` khi cần | T3 | MS `M-ASYNC-STACK-SIZE` (tóm tắt); CLP (`large_futures`: pedantic, chưa xác nhận hoàn toàn) | Review |
| RS-ASYNC-007 | Không `let _ = tokio::spawn(...)`/bỏ future không `.await` | T1 | CLP (`let_underscore_future`: suspicious, warn — qua tóm tắt, chưa nguyên văn) | `clippy::let_underscore_future = "deny"` |
| RS-ASYNC-008 | Hàm async của thư viện trả future `Send` (để chạy được trong `tokio::spawn`) | T1 | API `C-SEND-SYNC`; MS `M-TYPES-SEND` (tóm tắt) | Biên dịch: `tokio::spawn` đòi `Send`; test biên dịch trong `tests/` |

### 2.5 Tiến trình con, đường dẫn, bí mật

| ID | Rule | Tier | Nguồn | Cách kiểm |
| :--- | :--- | :---: | :--- | :--- |
| RS-SEC-001 | **`Command::new` chỉ được gọi trong `dandori-process`.** Không qua shell: không `sh -c`, `bash -c`, `cmd /C`, `powershell -Command` với chuỗi ghép từ đầu vào; tham số là mảng | T1 | DAN; Kantanda `SEC-*` (injection) | `clippy.toml` `disallowed-methods` cho `std::process::Command::new` và `tokio::process::Command::new`; `dandori-process` dùng `#[expect(clippy::disallowed_methods, reason = ...)]` ở đúng một chỗ; `check-lints.mjs` cấm chuỗi `"-c"`/`"/C"` đứng cạnh tên shell |
| RS-SEC-002 | Tiến trình con luôn: timeout tổng + timeout im lặng, `kill_on_drop(true)`, giới hạn kích thước dòng đọc, danh sách biến môi trường cho phép, HOME riêng theo account | T2 | DAN; TOK (`kill_on_drop` thuộc API `tokio::process`, chưa đọc tài liệu) | Test với `fake-agent`: treo, spam đầu ra, chết đột ngột, không thoát |
| RS-SEC-003 | Bí mật dùng kiểu `secrecy::SecretString`/`SecretBox`; `Debug`/`Display` đã che; **không** `format!("{secret}")`, không log, không đưa vào lỗi/sự kiện/tham số dòng lệnh | T1 | MS `M-PUBLIC-DEBUG`, `M-LOG-STRUCTURED` ("che dữ liệu nhạy cảm", tóm tắt); Kantanda `SEC-LOG-001` | Test: `format!("{:?}", secret)` không chứa giá trị; `scripts/scan-secrets.mjs` quét repo; test sự kiện/log không chứa giá trị |
| RS-SEC-004 | Đường dẫn từ đầu vào: chuẩn hóa và kiểm **nằm trong** thư mục cho phép (chống `..`, liên kết, ký tự đặc biệt của Windows) trước khi dùng | T2 | MS `M-STRONG-TYPES` (dùng `PathBuf`, tóm tắt); Kantanda `SEC-UPL-*` | Test có tệp `..\..` và junction/symlink |
| RS-SEC-005 | SQL **chỉ** qua tham số ràng buộc của sqlx (`query!`/`query_as!`/`bind`); cấm ghép chuỗi `format!` vào SQL | T1 | Kantanda `BE-DB-001`/SQL injection; DAN | `scripts/check-sql.mjs` (từ chối `format!`/ghép chuỗi trong lời gọi `sqlx::query*`); `cargo sqlx prepare --check` |
| RS-SEC-006 | Server **chỉ nghe `127.0.0.1`** theo mặc định; kiểm `Origin`/`Host`; mở ra ngoài là module `self-host` | T1 | DAN | Test tích hợp: yêu cầu có `Origin` lạ bị từ chối; cấu hình mặc định không bind `0.0.0.0` |
| RS-SEC-007 | HTTP ra ngoài chỉ tới tên miền trong danh sách cho phép; chặn loopback/mạng nội bộ/metadata cloud | T1 | Kantanda `SEC-SSRF-001` | Test với máy chủ giả |

### 2.6 Kiểu, API nội bộ, tài liệu

| ID | Rule | Tier | Nguồn | Cách kiểm |
| :--- | :--- | :---: | :--- | :--- |
| RS-TYPE-001 | ID là **newtype** (`TaskId`, `RunId`, `ModuleId`...), không truyền `String`/`Uuid` trần giữa các hàm công khai | T2 | API `C-NEWTYPE`; MS `M-STRONG-TYPES-GUARD` (tóm tắt) | Review |
| RS-TYPE-002 | Tham số "cờ bool/Option khó hiểu" thay bằng kiểu có tên | T3 | API `C-CUSTOM-TYPE` | Review |
| RS-TYPE-003 | Struct công khai của crate lõi có **trường riêng tư**; kiểu DTO API thì được `pub` | T2 | API `C-STRUCT-PRIVATE` | Review |
| RS-TYPE-004 | Builder khi một kiểu có ≥4 tham số khởi tạo; setter không lỗi, kiểm ở `build()` | T3 | API `C-BUILDER`; MS `M-INIT-BUILDER`, `M-BUILD-RESULT` (tóm tắt) | Review |
| RS-API-001 | Mọi kiểu công khai implement `Debug` (kiểu chứa bí mật dùng `Debug` tự viết đã che) | T1 | API `C-DEBUG`; MS `M-PUBLIC-DEBUG` | `[workspace.lints.rust] missing_debug_implementations = "warn"` (+ `-D warnings`) |
| RS-API-002 | Đặt tên theo RFC 430 (`snake_case` hàm, `UpperCamelCase` kiểu, `SCREAMING_SNAKE_CASE` hằng); tiền tố chuyển đổi `as_`/`to_`/`into_`; getter không có `get_` | T1 (casing) / T3 | API `C-CASE`, `C-CONV`, `C-GETTER` | rustc cảnh báo casing mặc định (`non_snake_case`...) → `-D warnings` |
| RS-API-003 | Tên ngắn, **không** hậu tố mơ hồ `Manager`, `Service`, `Factory`, `Handler` khi không có nghĩa riêng | T3 | MS `M-WEASEL-WORDS`, `M-SHORT-NAMES` (tóm tắt) | Review |
| RS-DOC-001 | Mục công khai của crate lõi có doc comment: câu đầu một dòng; mục `# Errors`, `# Panics`, `# Safety` khi áp dụng | T2 | API `C-FAILURE`; MS `M-CANONICAL-DOCS`, `M-FIRST-DOC-SENTENCE` (nguyên văn) | `#![warn(missing_docs)]` ở **mọi crate thư viện** (T01B đã bật; kèm `-D warnings` nên mục công khai nào thiếu tài liệu sẽ làm `verify` đỏ); Review phần còn lại |
| RS-DOC-002 | Doc comment mô tả **hành vi hiện tại**, không kể quá trình thiết kế/tác giả AI | T3 | MS `M-NO-META-DESIGN-DOCUMENTATION` (nguyên văn) | Review |
| RS-DOC-003 | Hằng "magic" có tên và chú thích; không số/chuỗi lạ rải rác | T3 | MS `M-DOCUMENTED-MAGIC` (tóm tắt) | Review |
| RS-DOC-004 | Mỗi crate có **`description`** trong `Cargo.toml`; câu đầu của `//!` là một dòng ngắn | T1 | API `C-METADATA`; MS `M-FIRST-DOC-SENTENCE` (nguyên văn) | `check-lints.mjs` kiểm `description` |
| RS-DOC-005 | `cargo doc --workspace --no-deps` với `RUSTDOCFLAGS="-D warnings"` phải qua (không liên kết gãy) | T1 | API `C-CRATE-DOC`; quy ước Cargo | bước trong `verify` (T02) |

### 2.7 Nhật ký

| ID | Rule | Tier | Nguồn | Cách kiểm |
| :--- | :--- | :---: | :--- | :--- |
| RS-LOG-001 | Mã chạy thật dùng **`tracing`** có cấu trúc (sự kiện có tên, trường rõ ràng); cấm `println!`/`eprintln!`/`dbg!` | T1 | MS `M-LOG-NOT-PRINT`, `M-LOG-STRUCTURED` (nguyên văn/tóm tắt) | `clippy::print_stdout`, `print_stderr` = `deny` (T01 xác nhận); `allow-print-in-tests` |
| RS-LOG-002 | Không log trong vòng lặp nóng; log mức `debug`/`trace` cho chi tiết | T3 | MS `M-LOG-OVERHEAD` (tóm tắt) | Review |

### 2.8 Kiến trúc, crate, phụ thuộc

| ID | Rule | Tier | Nguồn | Cách kiểm |
| :--- | :--- | :---: | :--- | :--- |
| RS-ARCH-001 | Luật phụ thuộc crate ở `01` mục 3 | T1 | Kantanda `ARCH-DDD-005` (hexagonal), `ARCH-MOD-002`; DAN | `scripts/check-architecture.mjs` đọc `cargo metadata`, dừng khi có cạnh cấm; kèm `tests/tooling/` có ca xanh/đỏ |
| RS-ARCH-002 | **Module không phụ thuộc module.** `mod-*` chỉ phụ thuộc `dandori-types` + `dandori-module-api` | T1 | PDF ("module không gọi nhau trực tiếp"); `ARCH-MOD-002` | cùng script |
| RS-ARCH-003 | Bật/tắt module là **trạng thái chạy**, không dùng cargo feature để bật/tắt module. Cargo feature (nếu có) chỉ mang tính cộng thêm | T2 | MS `M-FEATURES-ADDITIVE` (tóm tắt); D-18 | Review |
| RS-ARCH-004 | Crate xếp **phẳng** dưới `crates/`; gốc là virtual manifest với **`members = ["crates/*"]`** và `resolver` ghi rõ; mọi crate là thành viên workspace; phụ thuộc ngoài khai báo ở `[workspace.dependencies]` với phiên bản, crate dùng `dep = { workspace = true }`; `[profile.*]` chỉ ở gốc | T1 | CB (Workspaces, Profiles), MS `M-CRATES-FLAT-FOLDER`, `M-CRATES-IN-WORKSPACE`, `M-CARGO-WORKSPACE` (tóm tắt) | `check-architecture.mjs`; `cargo deny` cấm `*` |
| RS-ARCH-005 | Không `pub use x::*` (glob re-export) ở API công khai; không `use x::*` trừ trong `#[cfg(test)]` | T1 | MS `M-NO-GLOB-REEXPORTS`, `M-NO-PRELUDE` (tóm tắt); CLP (`enum_glob_use`: có trong ví dụ cấu hình của tài liệu; `wildcard_imports`: chưa xác nhận) | `clippy::enum_glob_use`, `wildcard_imports` = `deny` (T01 xác nhận) |
| RS-ARCH-006 | Macro là **phương án cuối**; ưu tiên hàm/generic/trait | T3 | MS `M-MACRO-LAST-RESORT`, `M-EXAMPLE-OVER-PROC` (nguyên văn) | Review |
| RS-ARCH-007 | Mỗi crate có một mục đích; tách khi một phần dùng độc lập được. Không đặt logic nghiệp vụ trong `dandori-server` ngoài việc ghép nối | T2 | MS `M-SMALLER-CRATES` (tóm tắt); Kantanda `ARCH-LAY-001` | Review |
| RS-ARCH-008 | Chia module theo kiểu **`foo.rs`** (+ thư mục `foo/` cho module con), **không** dùng `mod.rs`; không trộn hai kiểu | T1 | RB ch07-05 (trộn hai kiểu cho cùng một module là lỗi biên dịch) | `clippy::mod_module_files = "deny"` (T01B xác nhận tên lint bằng cách chạy) |
| RS-ARCH-009 | Crate chương trình: `main.rs` **mỏng** (cấu hình, gọi `lib.rs`), logic nằm trong `lib.rs` để test được. Test tích hợp ở `tests/` của chính crate; test xuyên suốt Rust ở `crates/dandori-e2e/tests/`; thư mục `tests/` ở gốc chỉ cho test script Node | T2 | CB (Layout: gốc virtual manifest không có package nên không biên dịch `tests/` ở gốc); MK (ý kiến) | Review; `check-architecture` cấm `tests/*.rs` ở gốc |
| RS-ARCH-010 | Crate `dandori-modules` do `gen-modules` sinh; **không sửa tay**; `dandori-server` chỉ phụ thuộc crate này, không phụ thuộc trực tiếp `mod-*` | T1 | ADR 0020 | `gen-modules --check`; `check-architecture.mjs` |
| RS-ARCH-011 | **Tên crate và thư mục mô tả mục đích**; **cấm** tên chung chung `utils`, `common`, `helpers`, `misc`, `base`, `lib`; nhóm theo tiền tố `dandori-*`, `mod-*` | T1 | MK (ý kiến), MS `M-WEASEL-WORDS`, `M-SHORT-NAMES` (tóm tắt) | `check-architecture.mjs` có danh sách tên bị cấm |
| RS-ARCH-012 | Có `README.md` ở gốc và ở `crates/`, `frontend/`, `scripts/`, `docs/` | T1 | ADR 0020 mục 5 | `check-architecture.mjs` kiểm tồn tại |

### 2.9 Phụ thuộc và chuỗi cung ứng

| ID | Rule | Tier | Nguồn | Cách kiểm |
| :--- | :--- | :---: | :--- | :--- |
| RS-DEP-001 | `cargo deny check` (advisories, licenses, bans, sources) phải qua: cấm lỗ hổng/unmaintained/unsound đã biết, giấy phép ngoài danh sách cho phép, registry/git lạ, phiên bản `*` | T1 | COM (cargo-deny, RustSec); Kantanda `SWE-DEP-001` | `cargo deny check` trong `verify`; `deny.toml` viết từ `cargo deny init` và hướng dẫn của dự án (kiểm bằng `--help`) |
| RS-DEP-002 | Chỉ dùng thư viện trong danh sách ở `01` mục 12. Thêm = ADR + chủ dự án | T1 | DAN | `check-architecture.mjs` so `Cargo.toml` với `docs/adr/0001-dependencies.md` |
| RS-DEP-003 | `Cargo.lock` và `pnpm-lock.yaml` được commit; build dùng `--locked`/`--frozen-lockfile` | T1 | DAN | `verify` dùng cờ đó |
| RS-DEP-004 | Ghim phiên bản Rust ổn định trong `rust-toolchain.toml`; đặt `rust-version` (MSRV) trong `Cargo.toml` | T1 | MS `M-MSRV`, `M-LATEST-EDITION` (tóm tắt) | `check-lints.mjs` kiểm có đủ |

### 2.10 Kiểm thử

| ID | Rule | Tier | Nguồn | Cách kiểm |
| :--- | :--- | :---: | :--- | :--- |
| RS-TEST-001 | `cargo test --workspace` phải xanh; test không đụng dữ liệu thật (CLI thật, HOME thật, mạng, kho khóa hệ điều hành) | T1 | `00_QUY_TAC_CHUNG` mục 5 | `verify`; P1-09 thêm kiểm "thư mục cấu hình của CLI thật không đổi" nếu khả thi |
| RS-TEST-002 | Test tích hợp API công khai của crate nằm ở `tests/`; test đơn vị nằm cạnh mã (`#[cfg(test)]`) | T3 | MS `M-INTEGRATION-TESTS` (tóm tắt) | Review |
| RS-TEST-003 | Test **có ý nghĩa**: kiểm tính chất quan sát được, không lặp lại hằng/thực thi như code | T3 | MS `M-TAUTOLOGICAL-TESTS` (nguyên văn); Kantanda `TEST-ANT-001` | Review (QC1) |
| RS-TEST-004 | I/O, đồng hồ, ngẫu nhiên **có thể thay thế được** (trait hoặc tham số) để test không phụ thuộc thời gian thật | T2 | MS `M-MOCKABLE-SYSCALLS` (tóm tắt) | Review; test dùng `tokio::time::pause` hoặc đồng hồ giả |
| RS-TEST-005 | Mỗi lỗi sửa được có test tái hiện trước (đỏ) rồi mới sửa | T3 | Kantanda (TDD) | Review |
| RS-TEST-006 | Mỗi module có worker: test "tắt module dừng worker, API trả `module_disabled`, dữ liệu còn" | T1 | PDF ("tắt là tắt thật") | `cargo test` |

### 2.11 Dữ liệu và kiểu chia sẻ

| ID | Rule | Tier | Nguồn | Cách kiểm |
| :--- | :--- | :---: | :--- | :--- |
| RS-DB-001 | Migration **chỉ tiến**, bảng của module có tiền tố `m_<id>_`; truy vấn dùng sqlx macro với cache offline `.sqlx/` được commit | T1 | D-08; sqlx | `cargo sqlx prepare --check` trong `verify` (T01 xác nhận lệnh bằng `--help`) |
| RS-DB-002 | Bảng chủ sở hữu theo `project`/`workspace`: mọi truy vấn dữ liệu của người dùng lọc theo phạm vi sở hữu | T1/T2 | Kantanda `BE-DB-001`, `SEC-BOLA-002` | Test: người dùng A không đọc/sửa được dữ liệu workspace của B |
| RS-TS-001 | Mọi kiểu API dùng ở frontend được sinh bằng ts-rs vào `shared/types.ts`; file sinh ra không lệch | T1 | D-12; VK (`generate-types:check` đã xác nhận có ở `package.json`) | `cargo run --bin generate_types -- --check` (tên binary do T02 đặt) |

### 2.12 Vận hành

| ID | Rule | Tier | Nguồn | Cách kiểm |
| :--- | :--- | :---: | :--- | :--- |
| RS-OPS-001 | **Tắt êm** khi nhận **Ctrl+C và SIGTERM**: `CancellationToken` báo mọi task, `TaskTracker` chờ chúng xong, có thời hạn; máy chủ axum dùng `with_graceful_shutdown`. Nếu việc nghe tín hiệu lỗi vẫn phải tắt | T1 | TK (tokio.rs/tokio/topics/shutdown; ví dụ chính thức axum); 12F Disposability | Test: gửi hủy → chương trình thoát trong thời hạn, không còn task (T07, T10); SIGTERM test trên Linux hoặc giả lập (P4-09) |
| RS-OPS-002 | Cấu hình qua **biến môi trường `DANDORI_*`**; nhật ký ra **stdout** bằng `tracing`; ứng dụng không tự định tuyến hay lưu nhật ký | T2 | 12F Config, Logs | Review; test cấu hình mặc định |
| RS-OPS-003 | `[profile.release]` chỉ ở `Cargo.toml` gốc. Hiện đặt `strip = true`; **không** đặt `lto`/`codegen-units` cho tới khi **đo** ở P5-01; `panic` giữ **`unwind`** (RS-PANIC-003: panic ở một worker không được làm sập cả chương trình) | T2 | CB (Profiles); DAN | Review; `check-lints.mjs` kiểm không có `panic = "abort"` |

## 3. Cấu hình mẫu (T01 tạo, T02 khóa)

Đây là **hình dạng**, Dev kiểm từng tên lint/khóa bằng cách chạy công cụ (rule RS-LINT-005). Chỗ nào công cụ báo không tồn tại: ghi lại và dùng tên đúng, không bỏ lint.

```toml
# Cargo.toml (gốc)
[workspace.lints.rust]
unsafe_code = "forbid"
missing_debug_implementations = "warn"

[workspace.lints.clippy]
# nhóm mặc định, không bật cả restriction/pedantic (RS-LINT-004)
all = { level = "warn", priority = -1 }
# từng lint được chọn (nhóm theo tài liệu Clippy; T01 xác nhận)
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
unimplemented = "deny"
dbg_macro = "deny"
print_stdout = "deny"
print_stderr = "deny"
await_holding_lock = "deny"
await_holding_refcell_ref = "deny"
let_underscore_future = "deny"
allow_attributes_without_reason = "deny"
allow_attributes = "deny"
enum_glob_use = "deny"
wildcard_imports = "deny"
too_many_lines = "warn"
indexing_slicing = "warn"
mod_module_files = "deny"        # RS-ARCH-008; T01B xác nhận tên lint
```

```toml
# clippy.toml (khóa phải khớp; clippy lỗi với khóa lạ, đó là cách tự kiểm)
too-many-lines-threshold = 50
allow-unwrap-in-tests = true
allow-expect-in-tests = true
allow-panic-in-tests = true
allow-print-in-tests = true
allow-indexing-slicing-in-tests = true
disallowed-methods = [
  { path = "std::process::Command::new", reason = "chỉ dandori-process được tạo tiến trình" },
  { path = "tokio::process::Command::new", reason = "chỉ dandori-process được tạo tiến trình" },
  { path = "std::thread::sleep", reason = "dùng tokio::time::sleep" },
]
```

```toml
# rustfmt.toml: chỉ tùy chọn ổn định
edition = "2024"
```

`deny.toml`: bốn phần `advisories`, `licenses` (danh sách cho phép giấy phép phổ biến, ví dụ MIT, Apache-2.0, BSD-3-Clause, ISC, Unicode, Zlib; **không** liệt kê giấy phép copyleft mạnh nếu chưa hỏi chủ dự án), `bans` (`wildcards = "deny"`), `sources` (registry và git lạ: `deny`). T01 sinh từ `cargo deny init` và hướng dẫn chính thức của cargo-deny.

## 4. Ví dụ ✗/✓

**RS-PANIC-002 / RS-ERR-001 (không unwrap trong mã chạy thật, lỗi có kiểu)**

```rust
// ✗
pub fn read_manifest(path: &Path) -> Manifest {
    let text = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(&text).unwrap()
}

// ✓
#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("không đọc được file khai báo {path}")]
    Read { path: PathBuf, #[source] source: std::io::Error },
    #[error("file khai báo {path} không hợp lệ")]
    Parse { path: PathBuf, #[source] source: serde_json::Error },
}

pub fn read_manifest(path: &Path) -> Result<Manifest, ManifestError> {
    let text = std::fs::read_to_string(path)
        .map_err(|source| ManifestError::Read { path: path.to_owned(), source })?;
    serde_json::from_str(&text)
        .map_err(|source| ManifestError::Parse { path: path.to_owned(), source })
}
```

**RS-ASYNC-001 (không giữ khóa qua await)**

```rust
// ✗  guard std giữ qua .await
let mut guard = state.lock().unwrap();
guard.count += 1;
send_event(&guard.snapshot()).await;

// ✓  lấy dữ liệu cần dùng, thả khóa, rồi mới await
let snapshot = {
    let mut guard = state.lock().map_err(|_| StateError::Poisoned)?;
    guard.count += 1;
    guard.snapshot()
};
send_event(&snapshot).await;
```

**RS-SEC-001 (không shell, tham số là mảng; chỉ trong `dandori-process`)**

```rust
// ✗
Command::new("sh").arg("-c").arg(format!("git checkout {branch}"));

// ✓  (trong dandori-process, có #[expect(clippy::disallowed_methods, reason = "điểm tạo tiến trình duy nhất")])
spawn(ProcessSpec::new("git").args(["checkout", "--", branch.as_str()]).timeout(GIT_TIMEOUT));
```

**RS-SEC-003 (bí mật)**

```rust
// ✗
tracing::info!("dùng khóa {}", api_key);
// ✓
tracing::info!(account = %account.label, "dùng khóa API đã lưu"); // không có giá trị khóa
```

## 5. Quan hệ với Kantanda

Rule **không phụ thuộc ngôn ngữ** của Kantanda vẫn áp dụng cho Rust về nguyên tắc: `BE-VAL-001/002` (kiểm đầu vào ở biên, từ chối trường lạ: dùng `serde` với `#[serde(deny_unknown_fields)]`), `BE-ERR-002`, `SEC-AUTH-*`, `SEC-SSRF-001`, `SEC-LOG-001`, `SWE-DEP-001`, `GIT-*`, `TEST-*`, `ARCH-DDD-002` (adapter cô lập dịch vụ ngoài: `dandori-agent-adapters`). **Công cụ kiểm của Kantanda (ESLint, Semgrep) chỉ cho TypeScript**, nên Rust dùng công cụ ở file này. Ánh xạ chi tiết và hồ sơ: `03_ANH_XA_KANTANDA.md`.

## 6. Việc T01 phải xác nhận bằng cách chạy (không dựa vào ký ức của Claude)

1. Mỗi lint ở mục 3 tồn tại và thuộc nhóm nào: chạy `cargo clippy` và ghi đầu ra vào `docs/adr/0002-rust-lints.md`. Các lint Claude **chưa xác nhận** được từ tài liệu: `unwrap_used`, `panic`, `todo`, `unimplemented`, `print_stdout`, `print_stderr`, `indexing_slicing`, `let_underscore_must_use`, `wildcard_imports`, `allow_attributes`, `too_many_lines`, `large_futures`.
2. Các khóa của `clippy.toml` (`allow-*-in-tests`, `too-many-lines-threshold`, `disallowed-methods`).
3. Cú pháp `deny.toml` và các lệnh `cargo sqlx prepare --check`, `cargo deny check`.
4. Thử nghiệm RS-LINT-005 (tên lint giả làm `verify` đỏ).
5. Phiên bản Rust ổn định đang cài (`rustc --version`) để ghim, và `rust-version`.

Chỗ nào kết quả khác mô tả ở đây: **kết quả chạy thật thắng**; sửa file này bằng một dòng "Đã chỉnh" kèm lý do (đây là ngoại lệ của điều cấm sửa `instructions/**`, chỉ với file này và chỉ ở T01).

- **Đã chỉnh (2026-10-30, T01):** Bổ sung `[workspace.lints.rust] unknown_lints = "deny"` vào `Cargo.toml`. Lý do: `rustc` mặc định đặt `unknown_lints` là cảnh báo và bỏ qua `-D warnings`; cần bật tường minh thành `deny` để thực thi RS-LINT-005 khi có lint giả.
- **Đã chỉnh (2026-10-30, T01):** Thêm `private = { ignore = true }` vào `[licenses]` và `allow-wildcard-paths = true` vào `[bans]` trong `deny.toml`. Lý do: Crate nội bộ có `publish = false` không cần khai báo license, và path dependency của workspace không bị tính là wildcard. Bỏ `version = 2` ở mục `[bans]` và `[sources]` do `cargo-deny 0.20.2` không hỗ trợ khóa này.
- **Đã chỉnh (2026-10-31, T01B):** Bổ sung `clippy::mod_module_files = "deny"` vào `[workspace.lints.clippy]`. Xác nhận lint thuộc nhóm `clippy::restriction` (mặc định `allow`), hoạt động chính xác với mức `deny` trên Rust 1.99.1 để thực thi RS-ARCH-008.
