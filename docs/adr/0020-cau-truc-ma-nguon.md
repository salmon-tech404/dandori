# ADR 0020: Cấu trúc thư mục mã nguồn của Dandori

- **Trạng thái:** Đã chấp nhận
- **Ngày:** 2026-10-31
- **Người quyết định:** Chủ dự án (giao Claude thiết kế "cấu trúc chuẩn nhất có thể")
- **Soạn bởi:** Claude. Nguồn tham khảo và độ tin cậy: xem `docs/cong-nghe/cau-truc-rust-va-trien-khai.md`.

## Bối cảnh

Task T01 dựng khung workspace. Khi kiểm độc lập và đối chiếu với tài liệu chính thức (Cargo Book, Rust Book), tìm thấy các chỗ chưa chuẩn hoặc mâu thuẫn với tiêu chí thiết kế (PDF): "thêm một module chỉ cần một thư mục và một file khai báo, không sửa lõi". Chủ dự án cũng thấy tên thư mục `web/` (tên cũ do T01 tạo theo kế hoạch ban đầu) khó hiểu.

## Quyết định

### 1. Cây thư mục đích

```
D:\19-dandori\
├── Cargo.toml                  Virtual manifest: members = ["crates/*"], resolver ghi rõ,
│                               [workspace.package|lints|dependencies], [profile.release]
├── Cargo.lock                  (commit)
├── rust-toolchain.toml  rustfmt.toml  clippy.toml  deny.toml
├── package.json  pnpm-workspace.yaml  pnpm-lock.yaml    (ghim packageManager, engines)
├── .gitignore  .env.example  AGENTS.md
│
├── crates/                     PHẲNG: mọi crate Rust là anh em
│   ├── dandori-types/ db/ events/ vault/ process/ agent-port/ agent-adapters/ git/
│   │   module-api/ registry/    (lõi; tên thư mục = tên crate, giữ tiền tố dandori-)
│   ├── dandori-modules/        SINH TỰ ĐỘNG: crate cầu nối gom mọi mod-*
│   ├── dandori-server/         Điểm ghép nối; chương trình tên `dandori`
│   ├── dandori-e2e/            (từ T14) test xuyên suốt viết bằng Rust, chỉ có tests/
│   ├── fake-agent/             chương trình giả, chỉ cho test
│   └── mod-<id>/               43 module, mỗi module một crate
│
├── frontend/                   pnpm workspace: phần giao diện
│   ├── app/                    Vite + React + TypeScript
│   │   ├── src/{core,features,ui,i18n}/
│   │   ├── e2e/                kịch bản Playwright
│   │   └── public/             phông tự chứa, v.v.
│   └── tooling/                bản sao cấu hình ESLint của Kantanda
│
├── shared/types.ts             SINH bởi ts-rs (không sửa tay)
├── scripts/                    Node .mjs: verify, check-*, gen-modules
├── tests/tooling/              test của chính các script (node --test)
├── standards/profile.md        hồ sơ Kantanda
├── docs/                       sổ tay + adr/ + manual-test-checklist.md
├── deploy/                     (P4-09, P5) Dockerfile, tệp dịch vụ mẫu
├── release/                    (bị ignore) đầu ra đóng gói
└── instructions/               kế hoạch (chỉ đọc)
```

### 2. Quy tắc từng chỗ

| # | Quy tắc | Nguồn | Vì sao |
| :--- | :--- | :--- | :--- |
| 1 | Gốc là **virtual manifest** (không có `[package]`); mọi crate nằm phẳng trong `crates/`; thư mục trùng tên crate | Cargo Book (Workspaces); matklad (ý kiến) | Tránh `src/` ở gốc; liệt kê crate bằng `ls crates` |
| 2 | `members = ["crates/*"]` (glob), không liệt kê từng crate | Cargo Book (Workspaces) | Thêm module không phải sửa file gốc |
| 3 | `resolver` **ghi rõ** ở gốc (đặt `"3"` theo edition 2024, Antigravity xác nhận từ trang chính thức) | Cargo Book (virtual workspace phải ghi rõ) | Virtual manifest không có `edition` để suy ra |
| 4 | Crate `dandori-modules` **sinh tự động** gom mọi `mod-*`; `dandori-server` chỉ phụ thuộc đúng crate này | Thiết kế (xem `docs/cong-nghe/cau-truc-rust-va-trien-khai.md` mục 2.3 B) | Rust chỉ dùng được crate đã khai báo phụ thuộc; không có cầu nối thì thêm module vẫn phải sửa `Cargo.toml` của server |
| 5 | Chương trình chính tên **`dandori`** (`[[bin]]` trong `dandori-server`) | Quyết định W-03 | Hiện đang tên `dandori-server` |
| 6 | Crate chương trình: `main.rs` **mỏng**, logic nằm trong `lib.rs` | Cargo Book (Layout) | Test được, không phải chạy cả chương trình |
| 7 | Chia module theo kiểu `foo.rs` (+ thư mục `foo/` cho con), **không** `mod.rs`, không trộn hai kiểu | Rust Book ch07-05 | Chuẩn hiện hành; trộn là lỗi biên dịch |
| 8 | Test tích hợp của từng crate ở `crates/<tên>/tests/`; test xuyên suốt Rust ở `crates/dandori-e2e/tests/`; kịch bản trình duyệt ở `frontend/app/e2e/`. Thư mục `tests/` ở **gốc** chỉ dành cho test của script Node | Cargo Book (virtual manifest không có package nên không biên dịch `tests/` ở gốc) | `cargo test` phải chạy được test |
| 9 | Tự động hóa (verify, check-*, gen-modules) viết bằng **Node** ở `scripts/`, chưa dùng `cargo xtask` | Ngược ý kiến matklad (viết bằng Rust) | Máy đã cần Node cho giao diện, script chạy chung Windows/Linux; ghi nhận đây là đánh đổi có chủ ý, có thể xét lại |
| 10 | Ghim `packageManager` (pnpm) và `engines` (Node) trong `package.json` | Thực tế: pnpm 12 tự đổi phiên bản theo `packageManager` của từng dự án | Cùng một lệnh `pnpm -v` báo khác nhau tùy thư mục đứng; cần ghim để mọi người dùng chung |
| 11 | `[profile.*]` chỉ khai báo ở gốc; `[profile.release]` hiện chỉ đặt `strip = true`; `lto` và `codegen-units` chốt sau khi **đo** ở P5-01; `panic` giữ mặc định `unwind` | Cargo Book (Profiles); RS-PANIC-003 | `abort` sẽ làm một worker panic sập cả chương trình |
| 12 | Giao diện đặt trong **`frontend/`** (không phải `web/` như kế hoạch ban đầu) | Quyết định của chủ dự án (rõ nghĩa hơn); không rule kỹ thuật nào ép tên | Người đọc lần đầu hiểu ngay |

### 3. Cấu trúc bên trong một crate

```
crates/<tên>/                       (crate thư viện lõi)
├── Cargo.toml                      [lints] workspace = true, publish = false
├── src/
│   ├── lib.rs                      //! tài liệu crate, #![warn(missing_docs)], khai báo mod, pub use
│   ├── error.rs                    enum lỗi (thiserror)
│   └── <khu_vực>.rs [+ <khu_vực>/] theo kiểu foo.rs
└── tests/                          test tích hợp dùng API công khai

crates/dandori-server/              (chương trình)
├── src/main.rs                     mỏng: cấu hình, gọi dandori_server::run()
└── src/lib.rs                      logic, axum, tắt êm

crates/mod-<id>/                    (module)
├── Cargo.toml  module.json
├── migrations/                     SQL chỉ tiến
├── src/{lib.rs, routes.rs, workers.rs, events.rs, model.rs}
└── tests/

frontend/app/src/
├── main.tsx
├── core/                           khung ứng dụng, client API, vùng chứa ô cắm, features_generated.ts
├── features/<module-id>/           index.ts, components/, hooks/, api/, types.ts
├── ui/                             tokens.css và thành phần dùng chung
└── i18n/                           vi.json
```

### 4. Vận hành (dữ liệu và tiến trình)

- **Thư mục dữ liệu** (`DANDORI_DATA_DIR`, mặc định theo hệ điều hành): `db/dandori.db`, `accounts/<id>/home/`, `worktrees/<project>/<task>/`, `backups/`. **Không** nằm trong repo.
- **Cấu hình** qua biến môi trường `DANDORI_*` (12-factor). **Nhật ký** ghi ra stdout bằng `tracing`, không tự ghi tệp.
- **Tắt êm:** xử lý cả Ctrl+C và SIGTERM; `CancellationToken` báo cho các task, `TaskTracker` chờ chúng xong; thời hạn tắt thấp hơn thời gian nhà cung cấp cho phép.
- Mặc định nghe `127.0.0.1`.

## Đã cân nhắc và không chọn

| Phương án | Vì sao không chọn |
| :--- | :--- |
| Giữ `members` liệt kê tay | Mỗi module mới phải sửa `Cargo.toml` gốc, trái tiêu chí thiết kế |
| Để `dandori-server` phụ thuộc trực tiếp từng `mod-*` | Mỗi module mới phải sửa `Cargo.toml` của server |
| Một crate lớn thay vì nhiều crate nhỏ | Mất khả năng để trình biên dịch ép "module không gọi module" |
| Thêm thư mục `backend/` bọc `crates/` | `crates/` là quy ước của Cargo; thêm lớp bọc không có lợi ích kỹ thuật |
| Đặt `tests/e2e/` ở gốc cho test Rust | Gốc là virtual manifest nên `cargo test` không biên dịch nó |
| `cargo xtask` cho tự động hóa | Xem quy tắc 9; xét lại nếu số script Rust-hóa được tăng |

## Hệ quả

- **Tốt:** thêm một module = thêm một thư mục + `module.json` + chạy `gen-modules`; không sửa lõi. Cấu trúc khớp tài liệu chính thức của Cargo.
- **Đánh đổi:** có thêm một crate sinh tự động (`dandori-modules`) cần được `verify` kiểm không lệch (`gen-modules --check`). Tên `frontend/` khác tên `web/` của kế hoạch ban đầu nên phải đổi tên thư mục và sửa tham chiếu (làm ở T01B).
- **Xét lại khi:** số crate vượt tầm vài trăm, hoặc build quá chậm (cân nhắc gộp nhóm module); hoặc cần nhiều ứng dụng giao diện (khi đó `frontend/` có thêm thư mục con).

## Nguồn

Cargo Book (Package layout, Workspaces, Profiles), Rust Book ch07, matklad "Large Rust Workspaces" (ý kiến), Tokio "Graceful Shutdown", 12-factor. Liên kết và mức độ đã đọc: `docs/cong-nghe/cau-truc-rust-va-trien-khai.md`.
