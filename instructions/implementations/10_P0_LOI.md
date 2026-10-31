# 10. Giai đoạn 0: Lõi và bảng Option (T01–T14)

Mục tiêu (PDF): dựng lõi cùng màn hình Option, để từ module đầu tiên đã có công tắc bật/tắt. **Cổng sang P1:** bật và tắt được một module mẫu; dữ liệu còn nguyên sau khi tắt; API của module tắt trả `module_disabled`.

Mỗi task có đủ: **Mục tiêu · Phụ thuộc · Chạm vào · Việc phải làm · Không làm · Tiêu chí nghiệm thu**. "Chạm vào" là phạm vi duy nhất được sửa (cộng báo cáo và `00_TRANG_THAI.md`). Đọc `00`, `01`, `02`, `03` trước.

---

## T01. Khung workspace, chuỗi công cụ, hồ sơ

- **Mục tiêu:** repo build được, có cấu hình chất lượng, có hồ sơ Kantanda, có ADR nền.
- **Phụ thuộc:** Bước 0 xong (Rust, git, baseline). Không có task trước.
- **Chạm vào:** `Cargo.toml`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `deny.toml`, `package.json`, `pnpm-workspace.yaml`, `.gitignore`, `.env.example`, `AGENTS.md`, `standards/profile.md`, `docs/adr/0001-dependencies.md`, `0002-rust-lints.md`, `0003-architecture-style.md`, `crates/*/Cargo.toml` + `src/lib.rs` (khung rỗng cho mọi crate lõi ở `01` mục 2, trừ `mod-*`), `crates/dandori-server/src/main.rs`, `frontend/app/**` (khung Vite+React+TS chạy được, một trang chào), `02_QUY_UOC_RUST.md` (chỉ mục "Đã chỉnh" theo `02` mục 6).
- **Việc phải làm:**
  1. Ghi vào `00_TRANG_THAI.md` mục Môi trường đầu ra: `rustc --version`, `cargo --version`, `rustup show`, `node -v`, `pnpm -v`, `git --version`, `cargo deny --version`, `sqlx --version`.
  2. Ghim phiên bản Rust ổn định đang cài vào `rust-toolchain.toml` (kênh `stable` + phiên bản cụ thể) với `clippy`, `rustfmt`; đặt `rust-version` và `edition` ở `Cargo.toml`.
  3. Tạo workspace theo `01` mục 2–3, `[workspace.dependencies]` (mọi phụ thuộc có phiên bản), `[workspace.lints]` theo `02` mục 3; mỗi crate `[lints] workspace = true`, `publish = false`.
  4. **Xác nhận mọi mục ở `02` mục 6** bằng cách chạy; ghi `docs/adr/0002-rust-lints.md` (lint nào tồn tại, nhóm nào, tên khóa clippy.toml đúng). Làm thử nghiệm RS-LINT-005: thêm một tên lint giả → `cargo clippy` phải đỏ → gỡ. Dán đầu ra.
  5. `docs/adr/0001-dependencies.md`: với từng thư viện ở `01` mục 12, ghi phiên bản chọn, kết quả `cargo deny check advisories`, và thay thế nếu có cảnh báo.
  6. `docs/adr/0003-architecture-style.md`: ghi kiểu kiến trúc theo `03` mục 1.
  7. `standards/profile.md` điền đủ theo `03` mục 2 (không để ngoặc).
  8. `.gitignore` theo `03` mục 7 và **test** `git check-ignore` cho các đường dẫn nêu ở đó.
  9. `AGENTS.md` gốc: điểm vào ngắn cho agent (đọc theo thứ tự `00`→`03`, lệnh `pnpm run verify`, điều cấm tóm tắt, Kantanda chỉ đọc), có bảng "Việc → file cần đọc".
  10. `crates/dandori-server`: `main.rs` chỉ khởi tạo `tracing` và thoát với thông báo "chưa có gì"; **chưa** có axum (T10).
- **Không làm:** không viết logic nghiệp vụ; không thêm thư viện ngoài danh sách; không tạo `mod-*`; chưa tạo `scripts/` (T02).
- **Tiêu chí nghiệm thu:**
  1. `cargo build --workspace --locked` và `cargo clippy --workspace --all-targets --locked -- -D warnings` qua (dán đầu ra).
  2. `cargo fmt --all --check` qua.
  3. Thử nghiệm RS-LINT-005 được dán, cho thấy `clippy` đỏ với tên lint giả.
  4. `pnpm install --frozen-lockfile` và build khung web qua.
  5. ADR 0001–0003 tồn tại, nội dung thật (không để trống).
  6. `standards/profile.md` không còn ký tự `[`…`]` hay `{{`.
  7. Test `git check-ignore` qua cả hai chiều (nên ignore / không được ignore).
  8. Không file nào ngoài "Chạm vào" bị đổi (`git diff --stat`).

## T01B. Chỉnh khung theo cấu trúc chuẩn (sửa sau khi kiểm T01)

- **Mục tiêu:** sửa các điểm Claude tìm thấy khi kiểm độc lập T01 và áp dụng **cấu trúc thư mục chuẩn** (ADR 0020), **trước khi** khóa cấu hình ở T02. Đây là task sửa, không thêm tính năng.
- **Phụ thuộc:** T01 (đã DONE; báo cáo kiểm: `reports/REVIEW_T01.md`). **Đọc `docs/adr/0020-cau-truc-ma-nguon.md` trước.**
- **Chạm vào:** `Cargo.toml`, `Cargo.lock`, `crates/dandori-server/**`, `crates/dandori-agent-adapters/**`, `crates/dandori-modules/**` (tạo mới), `package.json`, `pnpm-workspace.yaml`, `pnpm-lock.yaml`, `.gitignore`, `frontend/**` (đổi tên từ `web/**`), `standards/profile.md`, `docs/adr/0001-dependencies.md` và `docs/adr/0003-architecture-style.md` (**chỉ** đổi đường dẫn `web/` thành `frontend/`), `docs/adr/0002-rust-lints.md` (chỉ thêm mục bổ sung cho lint mới), `instructions/implementations/reports/T01B.md`, `instructions/implementations/00_TRANG_THAI.md` (chỉ dòng T01B), `instructions/implementations/02_QUY_UOC_RUST.md` (chỉ dòng "Đã chỉnh").
- **Việc phải làm:**
  1. **Đổi tên `web/` thành `frontend/`** (dùng lệnh di chuyển của hệ điều hành, **không** dùng git). Cập nhật mọi tham chiếu: `pnpm-workspace.yaml` (`frontend/*`), `.gitignore` (`/frontend/**/dist/`, `/frontend/**/node_modules/`), `standards/profile.md`, ADR 0001 và 0003. Chạy `pnpm install` một lần để `pnpm-lock.yaml` cập nhật khóa dự án, rồi kiểm `pnpm install --frozen-lockfile` xanh.
  2. **`package.json` gốc:** thêm `"packageManager": "pnpm@12.14.0"` và `"engines": { "node": ">=24" }`. Dán đầu ra `pnpm -v` và `node -v`. (Lý do: pnpm 12 tự đổi phiên bản theo trường `packageManager` của từng dự án; cần ghim để mọi người dùng chung một bản.)
  3. **`Cargo.toml` gốc:** đổi `members` thành **`["crates/*"]`**. Với `resolver`: **đọc trang chính thức** https://doc.rust-lang.org/cargo/reference/resolver.html và Cargo Book (Workspaces), rồi đặt `resolver = "3"` nếu tài liệu xác nhận phù hợp với `edition = "2024"` và `rust-version`; **dán trích dẫn ngắn (tối đa 15 từ)** làm bằng chứng. Thêm `[profile.release]` chỉ với `strip = true` và chú thích: "lto, codegen-units: chốt sau khi đo ở P5-01; panic giữ unwind (RS-PANIC-003)". **Không** thêm gì khác vào profile.
  4. **Lint `mod_module_files = "deny"`** (RS-ARCH-008): thêm vào `[workspace.lints.clippy]`, chạy clippy để **xác nhận tên lint có tồn tại và thuộc nhóm nào**; dán đầu ra. Nếu lint không tồn tại: không thêm, ghi vào ADR 0002 mục bổ sung kèm đầu ra.
  5. **Crate cầu nối `crates/dandori-modules`** (khung, chưa có module nào): `Cargo.toml` (phụ thuộc `dandori-types` và `dandori-module-api`; dòng đầu ghi "TỆP SINH TỰ ĐỘNG bởi scripts/gen-modules.mjs (T05); không sửa tay"), `src/lib.rs` (`//!`, `#![warn(missing_docs)]`, một hằng `VERSION` như các crate khác). `dandori-server` thêm `dandori-modules` vào `[dependencies]`.
  6. **`dandori-server`:** thêm `[[bin]] name = "dandori" path = "src/main.rs"`; tách `src/lib.rs` (hàm `pub fn run() -> anyhow::Result<()>`: khởi tạo `tracing` và ghi dòng "chưa có gì (khung)") và `src/main.rs` **mỏng** (gọi `run`). Chạy `cargo run -p dandori-server` và dán đầu ra; xác nhận có file `target\debug\dandori.exe`.
  7. **`dandori-agent-adapters/src/lib.rs`:** bỏ hàm `init()` (do agent tự thêm, **không có trong thiết kế**); đưa về khung tối giản như các crate lõi khác (`//!`, `#![warn(missing_docs)]`, hằng `VERSION`).
  8. **`frontend/app/src/App.tsx`:** thay các giá trị cứng (`2rem`, `1.5rem`, mã màu `#...`, v.v.) bằng biến trong `tokens.css` (thêm token còn thiếu, ví dụ `--color-on-primary`). Ghi trong báo cáo: bộ token hiện tại chỉ là **tạm**; T12 sẽ thay bằng bộ ở `04_GIAO_DIEN.md` mục 5.
  9. **`.gitignore`:** thêm `/release/`.
  10. **`standards/profile.md`:** sửa đường dẫn `frontend/app`; sửa dòng "Presubmit: GitHub Actions / Hook" cho đúng thực tế (chưa có CI; presubmit = chạy `pnpm run verify` trên bản sao sạch, có thể thêm hook cục bộ sau). Giữ nguyên các phần còn lại.
  11. Ghi vào `02_QUY_UOC_RUST.md` các dòng "Đã chỉnh" nếu tên lint hoặc khóa cấu hình khác với mô tả.
- **Không làm:** không viết logic nghiệp vụ; không tạo `scripts/` (T02); không thêm thư viện; không đổi nội dung `instructions/**` ngoài các ngoại lệ ở mục "Chạm vào" (Claude đã cập nhật kế hoạch); không dùng git để ghi.
- **Tiêu chí nghiệm thu:**
  1. `cargo fmt --all --check`, `cargo build --workspace --locked`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`, `cargo deny check` đều qua; dán đầu ra.
  2. `cargo metadata --format-version 1 --no-deps` liệt kê **13** crate (12 cũ + `dandori-modules`); dán số đếm bằng lệnh.
  3. **Chứng minh glob:** tạo tạm thư mục `crates/zz-temp/` với một `Cargo.toml` và `src/lib.rs` hợp lệ; `cargo metadata` thấy nó mà **không sửa `Cargo.toml` gốc**; rồi **xóa** thư mục tạm và xác nhận `cargo metadata` quay về 13 crate. Dán cả hai lần.
  4. `cargo run -p dandori-server` chạy được; file chạy tên `dandori.exe`.
  5. Sau khi đổi tên: `pnpm install --frozen-lockfile` và `pnpm --filter app build` qua; thư mục `web/` **không còn tồn tại**; lệnh tìm chuỗi `web/app` và `web/` trong `Cargo.toml`, `package.json`, `pnpm-workspace.yaml`, `.gitignore`, `standards/`, `docs/adr/` **không ra kết quả** (dán lệnh và đầu ra).
  6. `package.json` có `packageManager` và `engines` (dán nội dung).
  7. Lint `mod_module_files`: có bằng chứng tồn tại, hoặc đã ghi lý do bỏ ở ADR 0002.
  8. `App.tsx` không còn mã màu `#...` hay giá trị `rem` cứng (dán kết quả tìm kiếm).
  9. `dandori-agent-adapters` không còn hàm `init`.
  10. QC2 trên **bản sao sạch** chạy lại toàn bộ mục 1 và 5; dán đầu ra.

## T02. Cổng `verify` và các script kiểm

- **Mục tiêu:** một lệnh `pnpm run verify` chạy mọi kiểm tra ở `03` mục 5; mỗi script kiểm có test xanh/đỏ riêng.
- **Phụ thuộc:** T01B.
- **Chạm vào:** `scripts/**`, `tests/tooling/**`, `package.json` (scripts), `frontend/tooling/kantanda-eslint.config.mjs`, `frontend/app/eslint.config.js`, `frontend/app/vitest.config.ts`, `crates/dandori-types/**` và `crates/dandori-server/src/bin/generate_types.rs` (chỉ khung sinh kiểu), `shared/types.ts`, `docs/manual-test-checklist.md` (tạo khung).
- **Việc phải làm:**
  1. `scripts/verify.mjs`: chạy các bước theo thứ tự `00_QUY_TAC_CHUNG` mục 4, dừng ở lỗi đầu tiên, in rõ bước nào đỏ, mã thoát khác 0 khi đỏ; chạy được Windows và Linux; mỗi bước có timeout.
  2. `scripts/check-architecture.mjs` (RS-ARCH-001/002/004, RS-ERR-002, RS-DEP-002): đọc `cargo metadata --format-version 1 --locked`, so với bảng luật `01` mục 3 (đặt thành **dữ liệu** trong file JSON, không rải trong code), so `Cargo.toml` với danh sách ở ADR 0001.
  3. `scripts/check-lints.mjs` (RS-LINT-002/004, RS-DEP-004, RS-ERR-001): xem `02`.
  4. `scripts/check-sql.mjs` (RS-SEC-005): từ chối `format!`/ghép chuỗi trong lời gọi `sqlx::query*`.
  5. `scripts/scan-secrets.mjs` (RS-SEC-003): quét mẫu khóa API/token/khóa riêng trong mọi tệp được git theo dõi; danh sách cho phép chỉ gồm `tests/fixtures/`; **không** dùng mẫu quá rộng gây báo nhầm trên tệp khóa băm của lockfile (lỗi đã gặp ở dự án trước).
  6. `scripts/gen-modules.mjs` chỉ cần **khung** (đọc `crates/mod-*/module.json`, báo "0 module") và chế độ `--check`; bản đầy đủ ở T05.
  7. `generate_types` binary và `shared/types.ts` (RS-TS-001) với chế độ `--check`; một kiểu mẫu để chứng minh vòng sinh/kiểm.
  8. `frontend/tooling/kantanda-eslint.config.mjs`: bản sao của `D:\10-kantanda\tooling\eslint.config.mjs` kèm dòng đầu ghi nguồn và mã commit Kantanda (`git -C D:\10-kantanda rev-parse --short HEAD`); `frontend/app/eslint.config.js` dùng nó với `styles: []`. Máy phải đăng ký plugin `typescript-eslint` cho các rule nó nhắc (kiểm bằng chạy).
  9. `tests/tooling/**`: **mỗi script có ít nhất một ca đỏ** (fixture vi phạm làm script thất bại đúng thông điệp) và một ca xanh. Ví dụ: fixture có cạnh `mod-a → mod-b` làm `check-architecture` đỏ; `#![allow(...)]` làm `check-lints` đỏ; `format!` trong `sqlx::query` làm `check-sql` đỏ; chuỗi giống khóa làm `scan-secrets` đỏ.
  10. Kiểm `verify` trên worktree sạch.
- **Không làm:** không viết mã nghiệp vụ; không nới `Cargo.toml`/`clippy.toml` của T01.
- **Tiêu chí nghiệm thu:**
  1. `pnpm run verify` xanh trên cây hiện tại và trên **worktree sạch** (dán đầu ra cả hai, kèm thời lượng).
  2. Mỗi fixture đỏ trong `tests/tooling/` làm đúng script tương ứng thất bại (dán đầu ra một ca cho mỗi script).
  3. Chạy `verify` khi cố ý đổi một dòng trong `shared/types.ts` bằng tay làm bước types đỏ (dán đầu ra), rồi hoàn tác.
  4. Không script nào dùng cú pháp chỉ chạy trên một hệ điều hành.
  5. Sau khi chủ dự án **duyệt và commit** T02, chủ dự án ghi mã commit đó làm **mốc khóa cấu hình** vào `00_TRANG_THAI.md` (`00_QUY_TAC_CHUNG` mục 10). Agent không tự ghi mốc này.

## T03. Cơ sở dữ liệu lõi (`dandori-db`)

- **Mục tiêu:** pool SQLite, migration lõi, repository của lõi, bốn bảng tính năng.
- **Phụ thuộc:** T02.
- **Chạm vào:** `crates/dandori-db/**`, `crates/dandori-types/**` (ID newtype, kiểu hàng), `.sqlx/**`, `shared/types.ts` (sinh).
- **Việc phải làm:**
  1. Migration tiến duy nhất theo `01` mục 6 (bảng lõi + `feature_module`, `feature_setting`, `feature_pack`, `feature_audit`). Khóa ngoại bật; chế độ nhật ký phù hợp cho SQLite cục bộ (ghi rõ lựa chọn trong ADR hoặc chú thích). Bảng `agent_account` có đủ các cột ở `01` mục 6.2 (`status`, `enabled`, `priority`, `exhausted_until`, `last_checked_at`).
  2. Hàm tạo pool từ đường dẫn tệp **hoặc** `:memory:` (cho test); chạy migration khi mở.
  3. Repository có kiểu cho `user`, `session`, `workspace`, `workspace_member`, `project`, `task`, `run`, `run_event`, `agent_account`, `secret` (chỉ bản ghi mã hóa, không logic mã hóa) và bốn bảng tính năng. Mọi truy vấn dùng macro sqlx kiểm lúc biên dịch; `.sqlx/` cập nhật.
  4. ID newtype (`UserId`, `TaskId`, ...) trong `dandori-types` (RS-TYPE-001), derive ts-rs.
  5. Mọi truy vấn dữ liệu theo workspace/project nhận phạm vi sở hữu làm tham số **bắt buộc** (khó gọi sai).
- **Không làm:** không bảng ngoài danh sách; không logic registry (T06); không dữ liệu mẫu.
- **Tiêu chí nghiệm thu:**
  1. Test: migration chạy từ DB rỗng; chạy lại không lỗi.
  2. Test: mỗi repository có thêm/đọc/sửa/xóa.
  3. Test sở hữu: người dùng ở workspace A **không** đọc/sửa được task của workspace B (RS-DB-002).
  4. Test: khóa ngoại thật sự ngăn xóa cha còn con.
  5. `cargo sqlx prepare --check` qua; `.sqlx/` đã commit.
  6. `verify` xanh trên worktree sạch.

## T04. Bus sự kiện (`dandori-events`)

- **Mục tiêu:** bus trong tiến trình, có kiểu, người nghe chậm không làm nghẽn người phát.
- **Phụ thuộc:** T03 (kiểu), T02.
- **Chạm vào:** `crates/dandori-events/**`, `crates/dandori-types/**` (kiểu sự kiện), `shared/types.ts`.
- **Việc phải làm:** `Envelope`, `EventKind` (danh sách ở `01` mục 7), phạm vi, `publish`, `subscribe(filter)`, hàng đợi có giới hạn cho mỗi người nghe; khi tràn thì đếm "mất sự kiện" (có số đo đọc được) và **không chặn**; hủy đăng ký khi bỏ handle; đồng hồ trừu tượng cho test (RS-TEST-004).
- **Không làm:** không lưu sự kiện ra DB (module khác làm); không thêm loại sự kiện ngoài `01` mục 7.
- **Tiêu chí nghiệm thu:**
  1. Test: nhiều người nghe cùng nhận đủ sự kiện đúng thứ tự phát.
  2. Test: một người nghe cố tình chậm **không** làm `publish` chậm hơn ngưỡng đặt; bộ đếm mất sự kiện tăng.
  3. Test: lọc theo phạm vi workspace/project hoạt động.
  4. Test: bỏ handle thì dừng nhận; không rò task.
  5. Test: không kiểu sự kiện nào có trường kiểu chuỗi để chứa bí mật nguyên văn (kiểm bằng cách nào đó hợp lý, ví dụ payload chỉ nhận kiểu đã định, không `serde_json::Value` tự do cho sự kiện lõi).
  6. `verify` xanh trên worktree sạch.

## T05. Hợp đồng module (`dandori-module-api`) và `gen-modules`

- **Mục tiêu:** trait `Module`, `Manifest`, `ModuleContext`; script sinh danh sách module; chu trình phụ thuộc bị chặn **khi build**.
- **Phụ thuộc:** T03, T04.
- **Chạm vào:** `crates/dandori-module-api/**`, `scripts/gen-modules.mjs`, `crates/dandori-modules/Cargo.toml` và `crates/dandori-modules/src/lib.rs` (sinh; crate cầu nối, xem `01` mục 3), `frontend/app/src/core/features_generated.ts` (sinh), `tests/tooling/gen-modules/**`, `crates/mod-sample-hello/**` (module mẫu để thử cổng P0; xem T14).
- **Việc phải làm:**
  1. `Manifest` đọc/kiểm `module.json` theo `01` mục 4.1 (kiểu có kiểm tra, từ chối trường lạ, kiểm schema `settings`).
  2. Trait `Module` và `ModuleContext` theo `01` mục 4.2: tay cầm hẹp, **không** lộ pool thô của bảng lõi.
  3. `scripts/gen-modules.mjs` đầy đủ: id khớp tên thư mục; `requires` trỏ tới module có thật; **phát hiện chu trình kể cả bắc cầu** và in đường đi chu trình; `core` hợp lệ; sinh `crates/dandori-modules/Cargo.toml` (mỗi `mod-*` là một phụ thuộc dạng `path`) và `crates/dandori-modules/src/lib.rs`; `--check` so file sinh với file đang có. **Đọc `ui.contributes`** (`04_GIAO_DIEN.md` mục 3.1): kiểm `slot` thuộc danh mục slot (đặt thành **dữ liệu** JSON, không rải trong code), `id` không trùng trong module; sinh thêm `frontend/app/src/core/features_generated.ts` (ánh xạ slot → danh sách ô cắm nạp lười).
  4. Module mẫu `mod-sample-hello`: `module.json` (nhóm `safety` hay nhóm phù hợp, `risk: low`), một route `GET /ping`, một migration tạo bảng `m_sample_hello_note` (quy tắc đặt tên: tiền tố `m_`, rồi `id` module với gạch ngang đổi thành gạch dưới, rồi `_` và tên bảng; ghi quy tắc này vào chú thích của `dandori-module-api`), một worker đếm nhịp, một bộ nghe sự kiện. Mục đích: thử cổng P0. **Chỉ được đưa vào bản build khi cờ `sample-modules` bật** (cờ cargo cộng thêm, chỉ cho dev/test; đây không phải bật/tắt module, RS-ARCH-003).
- **Không làm:** không registry (T06/T07); không module thật.
- **Tiêu chí nghiệm thu:**
  1. Test của `Manifest`: hợp lệ, thiếu trường, trường lạ, `requires` rỗng, schema `settings` sai.
  2. Fixture hai module cần nhau: `gen-modules` **thất bại**, thông điệp nêu đường đi chu trình (ví dụ `a -> b -> a`). Fixture chu trình bắc cầu 3 module cũng thất bại.
  3. Fixture `requires` tới module không tồn tại: thất bại. **Thêm một thư mục `mod-zz` hợp lệ rồi chạy `gen-modules`: `dandori-modules/Cargo.toml` tự có thêm `mod-zz` mà `Cargo.toml` gốc và `Cargo.toml` của `dandori-server` KHÔNG phải sửa** (chứng minh tiêu chí "không sửa lõi"). Fixture `ui.contributes` với `slot` không có trong danh mục: thất bại.
  4. `gen-modules --check` xanh khi file sinh khớp; đỏ khi sửa tay.
  5. `check-architecture` xanh với `mod-sample-hello` (chỉ phụ thuộc `types` và `module-api`); và **đỏ** với fixture cho `mod-a → mod-b`.
  6. `verify` xanh trên worktree sạch.

## T06. Registry: trạng thái hiệu lực, cấu hình, audit, gói

- **Mục tiêu:** logic thuần của Feature Registry: tính trạng thái hiệu lực theo phạm vi/khóa; đọc/ghi `feature_setting`; ghi `feature_audit`; gói mẫu.
- **Phụ thuộc:** T05.
- **Chạm vào:** `crates/dandori-registry/**`, `crates/dandori-db/**` (thêm truy vấn nếu cần), `crates/dandori-types/**`.
- **Việc phải làm:**
  1. Nạp danh mục từ crate `dandori-modules` (sinh tự động) vào `feature_module` (upsert theo phiên bản).
  2. Hàm `effective_state(module, workspace, project)` đúng 4 bước ở `01` mục 5; **không lưu** kết quả.
  3. `plan_enable(module, scope)` trả `{ will_enable_with: [...], needs_risk_ack: bool, risk_note }` (phụ thuộc bắc cầu, thứ tự topo); `plan_disable(module, scope)` trả `{ will_break: [...] }` (module đang bật cần nó, bắc cầu).
  4. Quyền đổi theo cấp (chủ hệ thống, quản trị viên workspace, chủ dự án); `locked` ở cấp trên làm cấp dưới **không đổi được**; trả lỗi có kiểu.
  5. Mọi thay đổi ghi `feature_audit` (module, phạm vi, hành động, người, lý do, thời điểm), **kể cả khi module `audit-log` tắt** (không phụ thuộc module đó). Hỗ trợ hoàn tác thay đổi gần nhất (ghi một hàng audit loại `undo`).
  6. Gói mẫu `feature_pack` (Solo nhẹ, Nhóm nhỏ, Đầy đủ) nạp từ dữ liệu; áp gói = một `plan_enable` cho cả bộ.
  7. Module `core: true` không tắt được (lỗi có kiểu).
- **Không làm:** không khởi động/dừng worker hay route (T07); không HTTP (T10+).
- **Tiêu chí nghiệm thu:**
  1. **Bảng test trạng thái hiệu lực** có tối thiểu: không có giá trị → tắt; chỉ hệ thống bật → bật; workspace tắt ghi đè hệ thống bật; dự án bật ghi đè workspace tắt; hệ thống `locked` bật → dự án tắt **bị từ chối**; workspace `locked` → dự án không đổi được; hệ thống không `locked` nhưng workspace `locked` tắt → dự án không bật được. Mỗi ca có tên và khẳng định kết quả.
  2. Test `plan_enable` với phụ thuộc bắc cầu 3 tầng; `plan_disable` liệt kê đủ module bị mất chức năng.
  3. Test: người dùng không đủ quyền bị từ chối (3 cấp).
  4. Test: audit có đủ hàng sau chuỗi bật/tắt/đổi cấu hình/hoàn tác; audit vẫn ghi khi `audit-log` tắt.
  5. Test: hoàn tác trả đúng trạng thái trước.
  6. Test: không thể tắt module `core`.
  7. `verify` xanh trên worktree sạch.

## T07. Registry: vòng đời bật/tắt thật

- **Mục tiêu:** bật/tắt module thật sự thay đổi hệ thống chạy: migration, worker, route, bộ nghe, sự kiện. "Tắt là tắt thật."
- **Phụ thuộc:** T06.
- **Chạm vào:** `crates/dandori-registry/**`, `crates/dandori-module-api/**`.
- **Việc phải làm:**
  1. **Bật:** (xác nhận người dùng đã được hiển thị `plan_enable` ở tầng trên) áp dụng theo thứ tự topo: chạy migration của module (bảng `m_<id>_*`), đăng ký bộ nghe, khởi động worker nền (với `CancellationToken` riêng), mở cổng route; phát `module.enabled`. Lỗi giữa chừng thì **hoàn tác phần đã làm** (không để nửa bật).
  2. **Tắt:** dừng worker (hủy token, chờ có timeout, ghi nhận nếu quá hạn), hủy lịch đang chờ của module, gỡ bộ nghe, đóng cổng route; **giữ dữ liệu và cấu hình**; phát `module.disabled`.
  3. **Cổng route:** middleware bọc mọi route dưới `/api/v1/m/<id>/`: module tắt tại phạm vi yêu cầu → `403 module_disabled` với thông điệp tiếng Việt "module đang tắt" (dạng lỗi chuẩn `01` mục 10).
  4. Phạm vi: worker nền chạy theo **hệ thống/workspace**; ghi chú rõ trong ADR hay chú thích cách xử lý module bật ở một số dự án nhưng tắt ở dự án khác (đề xuất: worker chạy nếu **bất kỳ** phạm vi nào bật; mỗi yêu cầu/mỗi sự kiện vẫn kiểm trạng thái hiệu lực của phạm vi của nó).
  5. Khi khởi động ứng dụng: khôi phục theo `feature_setting` hiện có (module đang bật thì khởi động lại worker).
  6. Khai báo `requires_worker_restart`: đổi cấu hình của module đó thì báo "cần khởi động lại worker" (và thực hiện khi người dùng xác nhận).
- **Không làm:** không HTTP quản trị (T13 lo giao diện, T10/T11 lo server); không module thật.
- **Tiêu chí nghiệm thu (dùng `mod-sample-hello`):**
  1. Test: bật → migration chạy (bảng tồn tại), worker đếm nhịp, route `GET /ping` trả 200, nhận được `module.enabled`.
  2. Test: tắt → worker **dừng trong thời gian giới hạn** (đo; không còn nhịp sau đó), route trả `403 module_disabled`, bộ nghe không còn được gọi, nhận `module.disabled`.
  3. Test: **dữ liệu còn nguyên**: ghi một dòng vào `m_sample_hello_note` khi bật, tắt, đọc lại đúng dòng đó; bật lại thấy dòng cũ.
  4. Test: tắt module có module khác đang cần → `plan_disable` liệt kê; tắt cả chuỗi dừng đúng thứ tự ngược.
  5. Test: migration của module lỗi giữa chừng → không ở trạng thái "bật" và không để worker mồ côi.
  6. Test khởi động lại tiến trình: module đang bật được khôi phục.
  7. Test RS-ASYNC-004: không rò task sau tắt (đếm task còn sống).
  8. `verify` xanh trên worktree sạch.

## T08. Kho khóa mã hóa (`dandori-vault`)

- **Mục tiêu:** lưu bí mật mã hóa trong DB, khóa chủ trong kho hệ điều hành.
- **Phụ thuộc:** T03.
- **Chạm vào:** `crates/dandori-vault/**`, `crates/dandori-db/**` (truy vấn bảng `secret`), `docs/adr/0004-vault.md`, `docs/manual-test-checklist.md` (thêm DAN-VERIFY-001).
- **Việc phải làm:**
  1. Trait `MasterKeyStore` với hai hiện thực: hệ điều hành (qua `keyring`, Windows Credential Manager) và **bộ nhớ** (chỉ dùng cho test, bật bằng cờ tường minh).
  2. Mã hóa AEAD (khóa 256 bit), **nonce duy nhất** mỗi bản ghi; `zeroize` bộ đệm; kiểu bí mật `SecretString`/`SecretBox` với `Debug` che.
  3. API: `put(label, secret) -> SecretId`, `reveal(id) -> secret` (chỉ trả cho mã nội bộ qua `ModuleContext`/adapter, **không** có route HTTP trả giá trị), `delete(id)`, `list()` chỉ trả nhãn.
  4. Khi kho hệ điều hành không dùng được: **lỗi rõ ràng**, không rơi về lưu rõ hay khóa cố định.
  5. ADR 0004: mô hình mối đe dọa ngắn (ai đọc được gì), lựa chọn thuật toán và thư viện, và giới hạn (ví dụ: người có quyền đăng nhập Windows của người dùng đọc được khóa chủ).
- **Không làm:** không xoay khóa (ghi vào "Đề xuất"); không UI.
- **Tiêu chí nghiệm thu:**
  1. Test: lưu rồi đọc lại đúng giá trị.
  2. Test: sửa một byte của dữ liệu mã hóa → `reveal` thất bại (phát hiện giả mạo).
  3. Test: hai lần `put` cùng giá trị cho nonce khác nhau.
  4. Test: **quét byte của tệp DB** sau `put` không tìm thấy giá trị gốc.
  5. Test: `format!("{:?}", secret)` và `Display` (nếu có) không chứa giá trị.
  6. Test: không có kho hệ điều hành → lỗi, không tạo bản ghi.
  7. `DAN-VERIFY-001` (kho Windows thật) ghi vào checklist là **CHỜ NGƯỜI DÙNG KIỂM**.
  8. `verify` xanh trên worktree sạch.

## T09. Chạy tiến trình, cổng agent, `fake-agent`

- **Mục tiêu:** nơi duy nhất tạo tiến trình con; trait `AgentAdapter`; binary giả CLI cho test.
- **Phụ thuộc:** T02.
- **Chạm vào:** `crates/dandori-process/**`, `crates/dandori-agent-port/**`, `crates/fake-agent/**`, `crates/dandori-agent-adapters/**` (chỉ `FakeAdapter`), `clippy.toml` (**chỉ thêm** `#[expect]` ở đúng điểm tạo tiến trình; không đổi `disallowed-methods`).
- **Việc phải làm:**
  1. `dandori-process`: `ProcessSpec` (chương trình, mảng tham số, thư mục làm việc, biến môi trường **cho phép**, HOME riêng, timeout tổng, timeout im lặng, giới hạn dòng); `spawn` trả luồng sự kiện (dòng stdout/stderr, thoát). `kill_on_drop(true)`. Hủy bằng token. **Giết cả cây tiến trình** khi quá hạn/hủy: trên Windows qua `taskkill /T /F /PID` (gọi **qua cùng điểm tạo tiến trình duy nhất**), trên Unix qua nhóm tiến trình. Mọi tiến trình khác trong repo bị `clippy` cấm (RS-SEC-001).
  2. `dandori-agent-port`: trait `AgentAdapter` (khởi động lần chạy với thư mục làm việc, prompt, account, vai trò; trả luồng sự kiện chuẩn hóa: `Text`, `ToolCall`, `QuotaExhausted`, `Finished`, `Error`), kiểu `AgentEvent`, `AgentAccountRef`. Đây là **port**; chưa có adapter thật.
  3. `fake-agent`: binary đọc kịch bản (JSON qua tham số hoặc biến môi trường) để: in các dòng, ngủ, thoát với mã X, **sinh một tiến trình cháu**, treo mãi, xả đầu ra cực nhiều, in ra "hết hạn mức", ghi một tệp vào thư mục làm việc. Dùng cho mọi test sau này.
  4. `FakeAdapter` trong `dandori-agent-adapters` chạy `fake-agent` qua `dandori-process`.
- **Không làm:** không adapter Claude/Gemini/Codex (P1-01); không đọc cấu hình CLI thật.
- **Tiêu chí nghiệm thu:**
  1. Test: quá timeout tổng → tiến trình **và tiến trình cháu** bị giết (kiểm bằng `fake-agent` sinh cháu).
  2. Test: im lặng quá timeout im lặng → bị giết.
  3. Test: xả đầu ra lớn: bộ nhớ bị chặn trên (giới hạn dòng/tổng), không tràn.
  4. Test: biến môi trường **không** nằm trong danh sách cho phép (đặt một biến bí mật giả) không tới tiến trình con.
  5. Test: hủy bằng token dừng tiến trình nhanh.
  6. Test: `FakeAdapter` phát chuỗi sự kiện đúng kịch bản, gồm `QuotaExhausted` và `Error`.
  7. `clippy` xanh và có test/kiểm chứng việc `Command::new` ở crate khác bị chặn (dùng fixture hoặc thử nghiệm thủ công được dán).
  8. `verify` xanh trên worktree sạch.

## T10. Máy chủ axum, lỗi, phiên, module `auth-login`

- **Mục tiêu:** khung HTTP; lỗi một dạng; đăng nhập cục bộ.
- **Phụ thuộc:** T07, T08.
- **Chạm vào:** `crates/dandori-server/**`, `crates/mod-auth-login/**`, `crates/dandori-registry/**` (chỉ nối cổng route), `shared/types.ts`.
- **Việc phải làm:**
  1. `dandori-server`: cấu hình (bind **127.0.0.1**, cổng, thư mục dữ liệu qua `directories` hoặc `DANDORI_DATA_DIR`), khởi tạo `tracing`, kết nối DB, vault, registry, bus; router gốc; `request_id` mỗi yêu cầu; `ApiError` một dạng (RS-ERR-003); `GET /api/v1/health`.
  2. Kiểm `Origin`/`Host` (RS-SEC-006), cookie phiên `HttpOnly; SameSite=Strict`.
  3. `mod-auth-login` (`core: true`): lần chạy đầu tạo **chủ hệ thống** (chỉ khi chưa có người dùng); chủ hệ thống tạo người dùng khác; đăng nhập/đăng xuất/`GET /api/v1/me`; mật khẩu băm **argon2**; phiên lưu băm token; hạn 8 giờ gia hạn khi hoạt động; **giới hạn thử sai** (khóa tạm theo tài khoản và theo nguồn; có bộ đếm bền); thông điệp lỗi **không phân biệt** sai tên với sai mật khẩu; so sánh thời gian không rò thông tin (không có nhánh nhanh khi tên không tồn tại).
  4. Mọi route (trừ health, đăng nhập, khởi tạo chủ lần đầu) đòi phiên hợp lệ.
  5. Cổng route của module (T07) được gắn vào router.
- **Không làm:** không CRUD workspace/task (T11); không đăng nhập bên thứ ba; không refresh token; không JWT.
- **Tiêu chí nghiệm thu:**
  1. Test tích hợp (máy chủ thật trên cổng ngẫu nhiên): khởi tạo chủ lần đầu; lần hai bị từ chối.
  2. Test: sai mật khẩu và tên không tồn tại cho **cùng** phản hồi (mã và thân).
  3. Test: sau N lần sai bị khóa tạm; lần đúng trong lúc khóa vẫn bị từ chối; hết khóa đăng nhập được.
  4. Test: yêu cầu có `Origin` lạ bị từ chối; cookie có đủ cờ.
  5. Test: gây lỗi nội bộ → phản hồi không chứa đường dẫn, `panicked at`, hay thông điệp SQL.
  6. Test: không route nào ngoài danh sách cho phép truy cập được khi chưa đăng nhập (liệt kê route bằng một test duyệt router).
  7. Test: mật khẩu không xuất hiện trong log (bắt đầu ra `tracing` trong test).
  8. Cấu hình mặc định **không** bind `0.0.0.0` (test cấu hình).
  8b. **Tắt êm (RS-OPS-001):** gửi tín hiệu hủy (Ctrl+C giả lập qua `CancellationToken`; SIGTERM thật khi chạy trên Linux) → máy chủ ngừng nhận yêu cầu mới, chờ yêu cầu đang chạy, rồi thoát trong thời hạn; không còn task nền (đếm).
  9. `verify` xanh trên worktree sạch.

## T11. Workspace, dự án, task (API lõi)

- **Mục tiêu:** miền lõi và API: workspace, thành viên, dự án, task, vòng đời trạng thái task.
- **Phụ thuộc:** T10.
- **Chạm vào:** `crates/dandori-server/**`, `crates/dandori-db/**`, `crates/dandori-types/**`, `shared/types.ts`.
- **Việc phải làm:**
  1. API CRUD: workspace, thành viên (vai trò `admin`/`member`), dự án (tên, đường dẫn repo cục bộ, nhánh mặc định; kiểm đường dẫn nằm hợp lệ, là repo git), task (tiêu đề, mô tả, trạng thái, lead/worker để trống ở lõi).
  2. Trạng thái task: `backlog`, `running`, `needs_human`, `in_review`, `ready_to_merge`, `done`, `failed`, `cancelled` (khớp cột Kanban của mockup: Backlog, Đang chạy, Cần bạn, Đang review, Sẵn sàng merge). Bảng chuyển trạng thái hợp lệ ở **một chỗ**; chuyển sai bị từ chối. Mỗi chuyển phát `task.status_changed`.
  3. Mọi API lọc theo thành viên của workspace (BE-DB-001, SEC-BOLA-002); đầu vào kiểm chặt (BE-VAL-001/002); phân trang cho danh sách.
  4. `GET /api/v1/features/effective?project=<id>` (từ registry) và API quản trị registry cho T13: `plan_enable`, `apply`, `plan_disable`, `undo`, danh sách module, gói, audit, cấu hình module; đều kiểm quyền theo cấp.
- **Không làm:** không gán agent (P1-02); không chạy task (P1-08); không Kanban UI.
- **Tiêu chí nghiệm thu:**
  1. Test: người dùng A không thấy/sửa/xóa dữ liệu workspace B qua bất kỳ route nào (liệt kê mọi route có `:id`).
  2. Test: đường dẫn repo không phải repo git hoặc không tồn tại bị từ chối; đường dẫn dạng `..` bị từ chối.
  3. Test: bảng chuyển trạng thái, gồm ít nhất 3 chuyển hợp lệ và 3 chuyển bị từ chối.
  4. Test: trường lạ trong thân yêu cầu bị từ chối (`deny_unknown_fields`).
  5. Test: API registry kiểm quyền 3 cấp (khớp T06).
  6. `verify` xanh trên worktree sạch.

## T12. Kiểu chia sẻ và khung giao diện (`frontend/app`)

- **Mục tiêu:** frontend có khung ứng dụng (thanh bên 7 màn hình), đăng nhập, hệ thống ô cắm (slot) sinh từ registry, i18n, token thiết kế. **Đọc `04_GIAO_DIEN.md` và xem ảnh `ui-reference/` trước.**
- **Phụ thuộc:** T11.
- **Chạm vào:** `frontend/app/**`, `shared/types.ts` (sinh), `standards/profile.md` (điền `{{touch_target}}`), `frontend/app/src/ui/tokens.css`.
- **Việc phải làm:**
  1. `src/core/`: client API có kiểu (chỉ dùng `shared/types.ts`), kiểm dữ liệu vào bằng `zod` ở biên, xử lý lỗi `{error:{code,message,request_id}}`, `react-query`, router, bố cục (thanh trên, bộ chọn dự án, trạng thái server), màn hình đăng nhập và khởi tạo lần đầu.
  2. **Khung và ô cắm** theo `04_GIAO_DIEN.md` mục 2–3: thanh bên trái (logo, bộ chọn dự án, 7 màn hình cố định, ô ghi chú đáy), thanh tiêu đề (chỉ nhãn "Server: đang chạy"; **không** nhãn "Dữ liệu minh họa"), vùng chứa `<Slot name=... context=.../>` gọi các ô cắm của module **đang bật** (từ `features/effective` và `features_generated.ts`), sắp theo `order`, mỗi ô cắm trong ranh giới lỗi. Màn hình không còn ô cắm nào → ẩn mục menu; truy cập thẳng URL của nó hiện "Tính năng này đang tắt" (có liên kết tới Cài đặt nếu đủ quyền). **Board khi `kanban` tắt** hiện danh sách task đơn giản do lõi dựng (không để trống). Màn hình đăng nhập/khởi tạo lần đầu dựng theo token (mockup không có).
  3. `src/ui/tokens.css` (bộ token do T01 dựng chỉ là **tạm**, khác mockup; thay hoàn toàn): **đúng bộ token ở `04` mục 5** (màu sáng/tối, cỡ chữ, bo góc, khoảng cách), chế độ sáng/tối; phông DM Sans và JetBrains Mono **tự chứa** (lấy từ HTML gốc hoặc nguồn chính thức, kèm giấy phép OFL; thiếu tệp thì dùng dự phòng và ghi lại); cấm giá trị ngoài thang (kiểm bằng script `check-tokens` thêm vào `verify`).
  4. i18n tiếng Việt: mọi chuỗi giao diện trong tệp ngôn ngữ; script `check-i18n` kiểm không còn chuỗi cứng trong JSX và không khóa thừa.
  5. Thành phần dùng chung: nút, ô nhập "nhãn | điều khiển", hộp thoại xác nhận, tooltip "?", thẻ trạng thái, bảng; đều có trạng thái rỗng/đang tải/lỗi, thao tác bằng bàn phím, vùng chạm ≥ 44px, icon Lucide.
  6. Điền `{{touch_target}}` thật vào `standards/profile.md`.
- **Không làm:** chưa có màn hình Option (T13); không màn hình module nào (mỗi module tự làm ở task của nó).
- **Tiêu chí nghiệm thu:**
  1. `tsc --noEmit`, ESLint (cấu hình Kantanda), `vitest` xanh; không `any`.
  2. Test: ô cắm của module **tắt** không render và của module **bật** render đúng vùng, sắp theo `order` (dữ liệu `effective` và `features_generated` giả); màn hình không còn ô cắm → mục menu ẩn.
  3. Test: truy cập thẳng URL màn hình không còn ô cắm hiện "đang tắt"; một ô cắm ném lỗi chỉ làm hỏng thẻ của nó (ranh giới lỗi), không làm hỏng màn hình; Board khi `kanban` tắt vẫn hiện danh sách task của lõi.
  4. Test: lỗi API hiển thị thông điệp an toàn, không hiển thị `request_id` thô cho người dùng ngoài một mục "chi tiết kỹ thuật" thu gọn.
  5. Test hợp phần cho mỗi thành phần dùng chung: có trạng thái rỗng/lỗi, thao tác bàn phím.
  6. `check-tokens` và `check-i18n` có ca đỏ trong `tests/tooling/` (chỉ **thêm**, không nới).
  7. `verify` xanh trên worktree sạch.

## T13. Màn hình Setting > Tính năng (bảng Option)

- **Mục tiêu:** màn hình bật/tắt module theo đúng mô tả của PDF, **bố cục theo mockup** (`ui-reference/07-cai-dat-tinh-nang.png`, `04_GIAO_DIEN.md` mục 4 phần Cài đặt): thẻ Gói mẫu, thẻ nhóm có chip "n/m đang bật", hàng module (công tắc, tên, mô tả, dòng "Cần bật trước: …", chip công sức bên phải), module lõi công tắc mờ vô hiệu. PDF thêm: phạm vi, tìm kiếm, bộ lọc, nhãn rủi ro, bánh răng, hộp thoại, hoàn tác.
- **Phụ thuộc:** T12.
- **Chạm vào:** `frontend/app/src/features/settings-features/**` (đây là màn hình của **lõi**, không phải module), i18n, `docs/manual-test-checklist.md`.
- **Việc phải làm (theo PDF, "Bố cục" và "Hành vi khi bấm công tắc"):**
  1. Trên cùng: bộ chọn phạm vi (Hệ thống, Workspace, Dự án), ô tìm kiếm, bộ lọc "Đang bật"/"Đang tắt".
  2. Hàng **gói mẫu** (Solo nhẹ, Nhóm nhỏ, Đầy đủ): bấm áp cả bộ (qua `plan_enable`, có hộp xác nhận liệt kê).
  3. 8 nhóm thu gọn được, mỗi nhóm hiện "đang bật n/m".
  4. Mỗi module một hàng: công tắc, tên, mô tả một dòng, nhãn công sức và rủi ro, biểu tượng bánh răng mở cấu hình riêng (sinh từ schema `settings` của module; **chỉ sáng khi module đang bật**).
  5. Bật module cần module khác → hộp thoại "Sẽ bật kèm: A, B" (Xác nhận/Hủy). Tắt module đang được cần → "Tắt sẽ làm mất chức năng: X, Y" (tắt cả chuỗi hoặc hủy). Bật module rủi ro → một dòng giải thích rủi ro (`risk_note`) và yêu cầu xác nhận. Module bị khóa ở cấp cao hơn → công tắc mờ kèm "Do quản trị viên workspace quy định".
  6. Mỗi thay đổi hiện trong "Nhật ký thay đổi tính năng" và có nút **Hoàn tác thay đổi gần nhất**.
  7. Mọi thay đổi áp dụng ngay, không khởi động lại, trừ module khai báo `requires_worker_restart` (hiện "cần khởi động lại worker").
  8. Module `core` hiện công tắc **bật và vô hiệu**, kèm giải thích trong tooltip "?".
- **Không làm:** không thêm gói/nhóm ngoài PDF; không dữ liệu minh họa của mockup.
- **Tiêu chí nghiệm thu:**
  1. Test hợp phần cho từng hành vi ở mục 5 (mỗi hành vi một test có tên rõ).
  2. Test: áp gói "Nhóm nhỏ" hiển thị đúng danh sách bật kèm theo PDF.
  3. Test: thay đổi cấu hình bị chặn nếu sai schema; bánh răng bị vô hiệu khi module tắt.
  4. Test: khóa ở cấp trên làm công tắc mờ và không gọi API.
  5. **E2E trình duyệt** (Playwright) với máy chủ thật + `mod-sample-hello` (cờ `sample-modules`): bật → ping 200; tắt → thấy lỗi "module đang tắt"; dữ liệu còn. Chụp ảnh màn hình vào `reports/T13-screens/` (**không** thêm cơ chế vượt xác thực để chụp; đăng nhập thật bằng người dùng test).
  6. Thao tác hoàn toàn bằng bàn phím; kiểm trình đọc màn hình bằng nhãn (`aria`).
  7. `DAN-VERIFY-002`: chủ dự án xem giao diện thật so với mockup (ghi vào checklist).
  8. `verify` xanh trên worktree sạch.

## T14. Cổng P0

- **Mục tiêu:** chứng minh cổng sang P1 bằng một kiểm thử xuyên suốt; không thêm tính năng.
- **Phụ thuộc:** T13.
- **Chạm vào:** `crates/dandori-e2e/**` (**tạo crate này ở T14**: chỉ có `Cargo.toml` và `tests/p0_gate.rs`; các cổng sau thêm tệp vào đó; không cần sửa `Cargo.toml` gốc nhờ `members = ["crates/*"]`), `frontend/app/e2e/**` (kịch bản trình duyệt, nếu có), `docs/manual-test-checklist.md`, báo cáo.
- **Việc phải làm:** một kịch bản tự động từ DB rỗng: khởi tạo chủ → tạo workspace/dự án → bật `sample-hello` qua API quản trị → ping → ghi dữ liệu → tắt → ping bị `module_disabled` → dữ liệu còn → khởi động lại tiến trình → trạng thái khôi phục → bật lại → dữ liệu còn. Kèm kiểm: sau tắt **không còn** task nền của module (đếm), không còn mục menu của module (qua `features/effective`).
- **Không làm:** không sửa mã nguồn lõi để cho kịch bản qua (phát hiện lỗi thì ghi `BLOCKED` kèm mô tả và sửa ở task phù hợp).
- **Tiêu chí nghiệm thu:**
  1. Kịch bản xanh, dán đầu ra nguyên văn.
  2. Kịch bản **đỏ khi cố ý làm hỏng** một bước (ví dụ để worker không dừng) — chứng minh nó thực sự kiểm tra (dán bằng chứng, rồi hoàn tác).
  3. `verify` xanh trên worktree sạch.
  4. Cập nhật `docs/manual-test-checklist.md` với mục kiểm tay P0 (kho khóa thật, giao diện).
  5. Ghi vào `00_TRANG_THAI.md`: "Cổng P0: đạt (tự động)". Chờ Claude kiểm độc lập trước khi sang P1.
