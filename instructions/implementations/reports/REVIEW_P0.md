# Kiểm độc lập Giai đoạn 0 (Claude, 2026-10-31)

Người kiểm: Claude (không phải người viết code). Đối tượng: cây làm việc `phase/p0-core` sau T03–T14 (chưa commit, 244 tệp thay đổi so với T02) và các báo cáo `T03`–`T14`, `TONG_KET.md`. Mọi con số do Claude tự chạy.

## Kết luận

**Cổng P0 CHƯA ĐẠT.** Bộ kiểm tự động xanh (18 bước), nhưng bộ kiểm **không kiểm điều quan trọng nhất**: chương trình `dandori` hiện **không chạy được máy chủ**. Ngoài ra có 2 lỗi bảo mật thật (đã sửa) và 4 lỗi còn lại cần giao Antigravity (`10B_P0_SUA_LOI.md`, task T14B). Chỉ sau T14B và lần kiểm lại mới ghi "Cổng P0: đạt".

## Đã tự kiểm

| Việc | Kết quả |
| :--- | :--- |
| Bản sao sạch 340 tệp, `pnpm install --frozen-lockfile` | exit 0 (2s) |
| `pnpm run verify` trên bản sao sạch (trạng thái Antigravity bàn giao) | exit 0, 18/18 bước, 141s. 76 test Rust, 28 test vitest, 34 test script, 0 thất bại |
| `pnpm run verify` trên cây hiện tại **sau khi Claude sửa** | exit 0, 18/18 bước, 97.9s |
| Cấu hình chất lượng bị nới lỏng? So với commit T02 (`b74f2ed`) | Không. `Cargo.toml` gốc (trước khi Claude thêm `sha2`), `clippy.toml`, `rustfmt.toml`, `deny.toml`, `eslint.config.js` không đổi; `verify.mjs`, `check-lints.mjs` chỉ thêm dòng. (Mốc khóa chưa được điền, nên dùng commit T02 làm mốc) |
| `#[allow]`, `#[ignore]`, `todo!`, `unsafe` trong `crates/` | Không có. Có nhiều `#[expect(...)]` kèm lý do (chuẩn) |
| Kiểm soát truy cập (T11) | Tốt: mọi route workspace/dự án/task đều qua `AuthUser` và `check_workspace_access` |
| SQLite | `foreign_keys(true)`, WAL, `busy_timeout` có |

## Phát hiện

| # | Mức | Phát hiện | Bằng chứng | Xử lý |
| :-: | :--- | :--- | :--- | :--- |
| 1 | **Chặn cổng** | **`dandori` không mở máy chủ.** `dandori_server::run()` chỉ ghi một dòng log rồi trả về. Không có `TcpListener`/`axum::serve` nào trong mã không phải test. Không có xử lý Ctrl+C/SIGTERM. Tiêu chí T10 số 1 ("máy chủ thật trên cổng ngẫu nhiên") và 8b ("tắt êm") **không được thực hiện**, vậy mà báo cáo T10 đánh "Qua" bằng một test kiểm cấu hình. Mọi test gọi thẳng `create_app` trong bộ nhớ | `grep serve/TcpListener/bind/signal` trên `crates/` chỉ ra test và cấu hình; đọc `lib.rs` | **Giao T14B mục 1** |
| 2 | Cao | **Tên đăng nhập không tồn tại phản hồi nhanh hơn ~600 lần so với sai mật khẩu**: chuỗi băm giả lập `DUMMY_ARGON2_HASH` không hợp lệ (`invalid Base64 encoding`), nên `dummy_verify` trả về ngay. Đo thật: 0.65 ms so với 387 ms. Kẻ tấn công dò được tên nào có thật, đúng điều T10 cấm. Test kèm theo vẫn **xanh** vì ngưỡng cho phép chênh 500 ms, lớn hơn cả độ trễ thật | Chạy thử đo thời gian | **Claude đã sửa**: tạo chuỗi băm hợp lệ một lần bằng đúng tham số thật; siết test theo tỉ lệ (chứng minh ĐỎ với lỗi cũ: 649µs so với 387ms, rồi XANH); thêm test chuỗi băm phải phân tích được |
| 3 | Cao | **Token phiên lưu nguyên văn** trong cột `token_hash` (đặc tả: "phiên lưu băm token"). Ai đọc được tệp DB là chiếm được phiên | `auth.rs`: `token_hash: token.clone()` | **Claude đã sửa**: lưu SHA-256, tra cứu theo băm; thêm test (DB không chứa token gốc; giá trị băm đúng). Cần `sha2` (đã có sẵn trong `Cargo.lock` qua `sqlx`, giấy phép MIT/Apache); Claude thêm vào danh sách thư viện được duyệt, ghi tại `01_KIEN_TRUC.md` mục 12 và `architecture-rules.json` |
| 4 | Cao | **Route module không đòi đăng nhập** (T10 mục 4: mọi route trừ health/đăng nhập/khởi tạo chủ). Test hiện tại gọi `/api/v1/m/sample-hello/ping` **không có cookie** và coi 200 là đúng | `lifecycle.rs` `handle_gated_request` không kiểm phiên; `server_tests.rs` | **Giao T14B mục 2** |
| 5 | Cao | **Không biên dịch được trên Linux/macOS**: `dandori-process` dùng `nix::...` trong `#[cfg(unix)]` nhưng `nix` không khai báo ở đâu (cả `Cargo.toml` lẫn `Cargo.lock`). Trên Windows khối này không được biên dịch nên `verify` không phát hiện. Ngoài ra tiến trình con không được đặt trong nhóm riêng nên `kill(-pid)` cũng sẽ không diệt được cây tiến trình | Đọc mã, `grep` | **Giao T14B mục 3**. Chưa biên dịch thử cho Linux (chưa cài target), kết luận dựa vào việc đọc mã |
| 6 | Vừa | Đăng nhập: khóa chỉ theo tài khoản, **thiếu khóa theo nguồn** (đặc tả yêu cầu cả hai); bộ đếm đọc rồi ghi (không nguyên tử, có thể đếm thiếu khi gửi song song); mỗi tên lạ tạo một dòng không giới hạn; token trả về cả trong thân JSON (làm vô hiệu `HttpOnly`); `setup-owner` kiểm "chưa có người" rồi mới chèn (có thể tạo hai chủ nếu gọi song song) | Đọc `auth.rs`, `routes/auth.rs` | **Giao T14B mục 4** |
| 7 | Vừa | `MemoryKeyStore`: khi **một bản sao** bị hủy, `Drop` xóa khóa chung về toàn số 0; các bản sao còn lại mã hóa bằng khóa 0 mà không báo lỗi (chỉ dùng cho test, nhưng test dựa vào nó) | Test mới ĐỎ: `left: [0,0,...]`, `right: [7,7,...]` | **Claude đã sửa** (`Drop` chỉ xóa khi là bản cuối), test mới XANH |
| 8 | Vừa | Kho khóa: bản mã không gắn với bản ghi (không dùng AAD), nên hoán đổi bản mã giữa hai dòng vẫn giải mã được | Đọc `vault.rs` | **Giao T14B mục 5** |
| 9 | Nhỏ | `mod-auth-login/src/auth.rs` là bản sao nguyên văn của `dandori-server/src/auth.rs` | So sánh | **Giao T14B mục 6** (Claude đã sửa cả hai chỗ cho lỗi số 2 trước đó) |
| 10 | Nhỏ | Kịch bản "khởi động lại máy chủ" của T14 chỉ dựng lại các đối tượng trong cùng tiến trình trên cùng DB, không phải tiến trình mới. Pool dùng `sqlite::memory:` | Đọc `p0_gate.rs` | Ghi nhận. Chấp nhận cho cổng P0, nhưng sau T14B nên có một ca chạy bằng tệp DB thật |
| 11 | Nhỏ | **Độ chính xác báo cáo:** `TONG_KET.md`, `T05`, `T12`, `T14` dùng "trọn vẹn 100%" (bị cấm); T10 "Qua" cho tiêu chí chưa làm (số 1, 8b); T09 ghi "SIGKILL nhóm tiến trình trên Unix" như đã làm trong khi mã Unix không biên dịch được | `grep "100%"`; đọc báo cáo | Không sửa báo cáo (là bằng chứng gốc); ghi tại đây |
| 12 | Nhỏ | Đăng xuất xóa **mọi phiên** của người dùng (mọi thiết bị) | `routes/auth.rs` `logout` | Ghi nhận; chấp nhận được nhưng nên là quyết định có chủ ý |

## Claude đã sửa

`crates/dandori-server/src/auth.rs` (chuỗi băm giả lập hợp lệ, `hash_token`), `crates/dandori-server/src/middleware/auth.rs` (tra cứu theo băm), `crates/mod-auth-login/src/auth.rs` (chuỗi băm giả lập), `crates/dandori-vault/src/keystore.rs` (`Drop`), `crates/dandori-server/tests/server_tests.rs` (siết test thời gian; thêm 3 test), `crates/dandori-vault/tests/vault_tests.rs` (thêm 1 test), `Cargo.toml` gốc và `crates/dandori-server/Cargo.toml` (thêm `sha2`), `scripts/data/architecture-rules.json`, `instructions/implementations/01_KIEN_TRUC.md` (danh sách thư viện), `00_TRANG_THAI.md`, `00_README.md`, và tạo `10B_P0_SUA_LOI.md`. Sau sửa, `verify` xanh 18/18.

## Chưa kiểm

- Chương trình `dandori` chưa từng mở được máy chủ thật (xem phát hiện 1).
- Chưa biên dịch/chạy trên Linux/macOS.
- Kho khóa hệ điều hành thật (Windows Credential Manager) chưa thử; test dùng `MemoryKeyStore`.
- Giao diện (T12/T13) chỉ kiểm bằng vitest, token, i18n; chưa mở trên trình duyệt, chưa nối với máy chủ thật.
- Chưa đọc kỹ từng dòng mã của `dandori-registry` (vòng đời bật/tắt), `dandori-events` và giao diện; kết luận về các phần này dựa vào test xanh và việc `e2e` chạy.

## Commit đề xuất (tiếng Anh; 12 nhóm phủ đúng 245/245 tệp)

Chạy thử khô: tổng các nhóm bằng đúng tổng tệp thay đổi, không sót, không trùng.

```powershell
git add Cargo.toml Cargo.lock package.json
git commit -m "build: add core dependencies for phase 0" -m "Task: T03"
git add crates/dandori-types crates/dandori-db .sqlx shared
git commit -m "feat(db): add core schema, repositories and shared types" -m "Task: T03"
git add crates/dandori-events
git commit -m "feat(events): add in-process event bus" -m "Task: T04"
git add crates/dandori-module-api crates/dandori-modules crates/mod-sample-hello
git commit -m "feat(modules): add module contract and sample module" -m "Task: T05"
git add crates/dandori-registry
git commit -m "feat(registry): add feature registry and module lifecycle" -m "Task: T06"
git add crates/dandori-vault
git commit -m "feat(vault): add encrypted secret vault" -m "Task: T08"
git add crates/dandori-process crates/dandori-agent-port crates/dandori-agent-adapters crates/fake-agent
git commit -m "feat(process): add safe process runner and agent port" -m "Task: T09"
git add crates/dandori-server crates/mod-auth-login
git commit -m "feat(server): add http server, auth and core api" -m "Task: T10"
git add frontend
git commit -m "feat(frontend): add app shell and feature settings screen" -m "Task: T12"
git add crates/dandori-e2e
git commit -m "test(e2e): add phase 0 gate scenario" -m "Task: T14"
git add scripts tests/tooling
git commit -m "feat(tooling): extend verify gate for phase 0" -m "Task: T05"
git add docs instructions
git commit -m "docs: add phase 0 reports, review and fix task" -m "Task: T14"
git status -sb
git push
```
