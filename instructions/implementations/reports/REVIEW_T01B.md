# Kiểm độc lập T01B (Claude, 2026-10-31)

Người kiểm: Claude (không phải người viết code). Đối tượng: toàn bộ thay đổi của T01B trong cây làm việc (chưa commit) và `reports/T01B.md`. Mọi con số dưới đây do Claude **tự chạy lại** trên bản sao sạch, không lấy từ báo cáo của Antigravity.

## Kết luận

**T01B đạt yêu cầu về chức năng.** Mọi tiêu chí kỹ thuật được xác nhận độc lập. Tìm thấy **2 lỗi vừa** và vài lỗi nhỏ về độ chính xác của tài liệu/báo cáo; Claude đã **sửa trực tiếp** các lỗi này (danh sách ở mục "Claude đã sửa") và kiểm lại toàn bộ. Sau sửa, cây làm việc sẵn sàng để commit.

## Đã tự kiểm (bản sao sạch, 71 file, không có `node_modules/`, `target/`, `instructions/`)

Lần chạy cuối, sau khi Claude sửa:

| Lệnh | Mã thoát |
| :--- | :---: |
| `pnpm install --frozen-lockfile` | 0 |
| `pnpm --filter app build` | 0 |
| `cargo fmt --all --check` | 0 |
| `cargo build --workspace --locked` | 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 |
| `cargo test --workspace --locked` | 0 (0 test sẵn có, vì chưa có logic) |
| `cargo deny check` | 0 (`advisories ok, bans ok, licenses ok, sources ok`) |
| `cargo doc --workspace --no-deps` với `RUSTDOCFLAGS=-D warnings` | 0 |
| `cargo run -p dandori-server` | 0; in `INFO dandori_server: dandori: chưa có gì (khung)` |

Đối chiếu từng tiêu chí T01B:

| # | Tiêu chí | Kết quả kiểm độc lập |
| :-: | :--- | :--- |
| 1 | Chuỗi cargo/pnpm qua | Qua (bảng trên) |
| 2 | 13 crate | Qua: `cargo metadata --no-deps` ra 13, đúng tên |
| 3 | Chứng minh glob | Qua: thêm `crates/zz-temp` → 14 crate, `Cargo.toml` gốc **không đổi** (so băm), xóa → 13 |
| 4 | Chương trình tên `dandori` | Qua: có `target\debug\dandori.exe`, **không** có `dandori-server.exe` |
| 5 | `web/` hết, không còn tham chiếu | Qua: `web/` không có trong bản sao; chuỗi `web/` chỉ còn trong tài liệu nói về việc đổi tên |
| 6 | `packageManager` và `engines` | Qua: `pnpm@12.14.0`, `node >=24` |
| 7 | Lint `mod_module_files` | Qua, **đã tự thử**: crate dùng `src/foo/mod.rs` bị clippy chặn (`mod.rs files are not allowed`), crate dùng `src/foo.rs` (đối chứng) qua |
| 8 | `App.tsx` không còn giá trị cứng | Qua: không còn `#...` hay `rem`; dùng biến token. (Vẫn còn `1px` và `fontWeight` số, nhỏ) |
| 9 | Hết hàm `init` | Qua |
| 10 | QC2 bản sao sạch | Qua (chính lần chạy này) |
| 11 | README tồn tại, bản đồ khớp | Tồn tại; **nội dung có lỗi** (mục Phát hiện 1, 3, 4), đã sửa |
| 12 | Mọi crate có `description` | Qua: 13/13, khớp dòng đầu `//!` |
| 13 | `cargo doc -D warnings` | Qua |

Điểm tốt: `resolver = "3"` đúng (xem lưu ý trích dẫn bên dưới); `mod_module_files = "deny"` hoạt động thật; `[profile.release]` chỉ có `strip = true` và giữ `panic` mặc định đúng như đã quy định; đã sửa lỗi chính tả "Cở sở"; `pnpm-lock.yaml` thêm 30 gói là do pnpm 12 ghi chính nó vào mục `packageManagerDependencies` (gói `@pnpm/exe.*`), hệ quả bình thường của trường `packageManager`, không phải thư viện lạ.

## Phát hiện

| # | Mức | Phát hiện | Bằng chứng | Xử lý |
| :-: | :--- | :--- | :--- | :--- |
| 1 | Vừa | **Bảng "Phụ thuộc chính" trong `crates/README.md` sai ở 6 trên 14 dòng**: `dandori-vault`, `dandori-process`, `dandori-agent-port`, `dandori-git`, `dandori-module-api`, `fake-agent` | So với `cargo metadata`: ví dụ README ghi `dandori-vault` phụ thuộc `types, db` nhưng thật chỉ `types`; `dandori-agent-port` ghi `types, events, process` nhưng thật chỉ `types`; `fake-agent` ghi phụ thuộc `types` nhưng thật không phụ thuộc gì; `module-api` thiếu `vault, agent-port, git` | **Claude đã viết lại** bảng; cột phụ thuộc lấy từ `cargo metadata`; cột nhiệm vụ lấy nguyên văn `description` |
| 2 | Vừa | **`dandori_server::run()` panic khi gọi lần thứ hai**, trong khi tài liệu hàm nói "trả về lỗi" | Chạy thật: `Unable to install global subscriber: SetGlobalDefaultError(...)`, test `FAILED` | **Claude đã sửa**: `.init()` thành `.try_init().map_err(...)?`; test lại: lần 1 `Ok`, lần 2 `Err`, không panic |
| 3 | Nhỏ | `frontend/README.md` liệt kê `core/`, `features/`, `i18n/`, `e2e/`, `public/`, `tooling/` như **đang có** (chưa có thư mục nào), và ghi thêm "tiếng Anh" cho i18n và "WebSocket" (kế hoạch chỉ có tiếng Việt và SSE) | `find frontend -type f` | **Claude đã sửa**: đánh dấu *(chưa có)* kèm task sẽ tạo; bỏ chi tiết không có trong thiết kế |
| 4 | Nhỏ | `README.md` gốc ghi `tests/` chứa "kiểm thử hệ thống", mâu thuẫn ADR 0020 (test hệ thống ở `crates/dandori-e2e`) | `README.md` | **Claude đã sửa** |
| 5 | Nhỏ | **Thay đổi ngoài phạm vi không khai báo:** thêm `#![warn(missing_docs)]` vào 8 `lib.rs` (`agent-port, db, events, git, process, registry, vault` và `agent-adapters`); "Chạm vào" chỉ cho phép `dandori-db/src/lib.rs` (sửa lỗi chính tả) và `agent-adapters` | `git diff crates/*/src/lib.rs` | Vô hại và hợp với hướng của RS-DOC-001. **Chấp nhận** và cập nhật RS-DOC-001 thành "mọi crate thư viện". Antigravity không nêu mục này trong "Quyết định tự đưa ra" |
| 6 | Nhỏ | **Độ chính xác của báo cáo `T01B.md`:** (a) trích dẫn Cargo Book "The "3" resolver is available since Rust 1.84... In edition 2024, resolver 3 is the default." **không có nguyên văn trên trang**. Nội dung đúng (trang ghi `"3"` là mặc định của `edition = "2024"`, yêu cầu Rust 1.84+) nhưng đã trình bày như một câu trích; (b) dẫn quy tắc không tồn tại (`RS-UI-*`) và sai mã quyết định (`D-25` cho yêu cầu `description`; đúng là D-22/ADR 0020 mục 5); (c) dùng cụm "xanh 100%" mà `00_QUY_TAC_CHUNG` mục 3 cấm; (d) ghi "đã ghi vào ADR 0002 mục 4" nhưng thực tế chỉ thêm một dòng vào bảng ở mục 2, **không dán đầu ra** thử nghiệm lint như task yêu cầu; (e) mục "Chưa kiểm chứng" ghi "Không có" trong khi giao diện chưa được chạy trên trình duyệt; (f) "Đề xuất commit" dùng cú pháp riêng của PowerShell (`2>$null`) và gộp thành một commit mơ hồ | Đối chiếu trực tiếp (trang Cargo Book; `git diff docs/adr/0002...`) | Không sửa báo cáo (là bằng chứng gốc); ghi tại đây. Commit đề xuất đã thay bằng bản chia nhỏ (cuối báo cáo) |
| 7 | Thông tin | Báo cáo T01B không nêu: `package.json` và `pnpm-lock.yaml` thay đổi do `packageManager` | xem trên | Đã giải thích ở trên |

## Claude đã sửa (để chủ dự án biết chính xác ai đổi gì)

`crates/README.md`, `frontend/README.md`, `README.md`, `crates/dandori-server/src/lib.rs` (3 dòng, `try_init`), `instructions/implementations/02_QUY_UOC_RUST.md` (RS-DOC-001), `docs/cong-nghe/cau-truc-rust.md` (một dòng lỗi thời). Sau các sửa này đã chạy lại toàn bộ mục "Đã tự kiểm" (xanh).

## Chưa kiểm

- Giao diện chưa chạy trên trình duyệt: chỉ kiểm `build` thành công, chưa `dev` hay xem trang.
- Mới kiểm trên Windows; build Linux/macOS chưa thử.
- `cargo test` chưa có test nào (0 test), nên chưa kiểm hành vi ngoài `run()`.
- Tính "đang được duy trì" của các thư viện (đã nêu ở `REVIEW_T01.md`).

## Commit đề xuất (tiếng Anh, tách theo chủ đề)

```powershell
cd D:\19-dandori

git add -A -- web frontend/app package.json pnpm-workspace.yaml pnpm-lock.yaml .gitignore standards/profile.md
git commit -m "chore(frontend): rename web to frontend and use design tokens" -m "Task: T01B"

git add Cargo.toml Cargo.lock crates/dandori-* crates/fake-agent
git commit -m "refactor(workspace): add modules bridge crate and thin server main" -m "Task: T01B"

git add README.md crates/README.md frontend/README.md
git commit -m "docs: add root, crates and frontend readmes" -m "Task: T01B"

git add docs instructions
git commit -m "docs: add T01B report and independent review" -m "Task: T01B"

git status -sb
git push
```
