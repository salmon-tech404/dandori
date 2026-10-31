# 01. Kiến trúc Dandori

Nguồn gốc: PDF thiết kế của chủ dự án (ghi chú "PDF" bên dưới) cộng các quyết định ở `00_QUYET_DINH.md`. Task nào mâu thuẫn với file này: dừng và hỏi, không tự sửa kiến trúc.

## 1. Nguyên tắc (từ PDF, bắt buộc)

1. **Lõi nhỏ, luôn bật.** Lõi gồm: tài khoản (đăng nhập), workspace/dự án, task, adapter agent, bus sự kiện, feature registry, kho khóa mã hóa.
2. **Module không gọi nhau trực tiếp.** Chúng chỉ nói chuyện qua **sự kiện** và **API của lõi**. Tắt một module không làm vỡ module khác. Điều này được **trình biên dịch và script kiểm tra** ép (mục 3), không dựa vào kỷ luật.
3. **Phụ thuộc được khai báo, không đoán.** Bật A cần B: bảng Option bật kèm B sau khi hỏi. Tắt B: hiện danh sách module sẽ mất chức năng. Chu trình phụ thuộc bị chặn **khi build**.
4. **Tắt là tắt thật:** dừng tiến trình nền, hủy lịch đang chờ, ẩn menu và màn hình, API trả lỗi "module đang tắt". **Dữ liệu cũ được giữ.**
5. **Mặc định an toàn:** module mới cài ở trạng thái tắt. Module rủi ro (SSH, sandbox, tự merge, tự host) luôn cần xác nhận khi bật.

Ba tiêu chí đạt (PDF): (a) người mới chạy được task đầu chỉ với lõi + một subscription + một task + một worker; (b) tắt module thì không còn tiến trình nền, bảng dữ liệu hay mục menu nào của nó hoạt động; (c) thêm module mới chỉ cần **một thư mục và một file khai báo**, không sửa lõi.

## 2. Cây thư mục

Chốt bởi **ADR 0020** (`docs/adr/0020-cau-truc-ma-nguon.md`), dựa trên Cargo Book và Rust Book; nguồn và độ tin cậy ở `docs/cong-nghe/cau-truc-rust-va-trien-khai.md`.

```
D:\19-dandori\
├── Cargo.toml                     virtual manifest: members = ["crates/*"], resolver ghi rõ,
│                                  [workspace.package|lints|dependencies], [profile.release]
├── Cargo.lock
├── rust-toolchain.toml  rustfmt.toml  clippy.toml  deny.toml
├── package.json  pnpm-workspace.yaml  pnpm-lock.yaml     (ghim packageManager và engines)
├── .gitignore  .env.example  AGENTS.md
├── standards/profile.md           hồ sơ Kantanda của Dandori
├── shared/types.ts                SINH bởi ts-rs, kiểm bằng generate-types --check
├── crates/                        PHẲNG: mọi crate là anh em, không lồng nhau
│   ├── dandori-types/             kiểu dùng chung, ID newtype, sự kiện, derive ts-rs
│   ├── dandori-db/                pool sqlx, migration lõi, repository của lõi
│   ├── dandori-events/            bus sự kiện
│   ├── dandori-module-api/        trait Module, manifest, ModuleContext (thứ DUY NHẤT module được phụ thuộc)
│   ├── dandori-registry/          feature registry
│   ├── dandori-vault/             kho khóa mã hóa
│   ├── dandori-process/           chạy tiến trình con an toàn (không shell, timeout, kill_on_drop)
│   ├── dandori-agent-port/        trait AgentAdapter + kiểu dữ liệu (port)
│   ├── dandori-agent-adapters/    adapter Claude/Gemini/Codex + (adapter giả cho test)
│   ├── dandori-git/               bọc lệnh `git` (worktree, diff, branch)
│   ├── dandori-modules/           SINH TỰ ĐỘNG bởi gen-modules: gom mọi mod-* (Cargo.toml + lib.rs)
│   ├── dandori-server/            axum, ghép nối (composition root); chương trình tên `dandori`
│   ├── dandori-e2e/               (từ T14) test xuyên suốt viết bằng Rust; chỉ có tests/
│   ├── fake-agent/                chương trình giả CLI agent, chỉ dùng cho test
│   └── mod-<id>/                  MỖI MODULE MỘT CRATE: module.json + migrations/ + src/ + tests/
├── frontend/                      pnpm workspace: phần giao diện
│   ├── app/                       Vite + React + TS; src/{core,features,ui,i18n}/; e2e/ (Playwright)
│   └── tooling/                   bản sao cấu hình ESLint của Kantanda
├── scripts/                       Node .mjs: verify, check-*, gen-modules (chạy chung Windows/Linux)
├── tests/tooling/                 test của chính các script (node --test); KHÔNG chứa test Rust
├── docs/                          sổ tay, adr/, manual-test-checklist.md
├── deploy/                        (P4-09, P5) Dockerfile, tệp dịch vụ mẫu
├── release/                       (bị ignore) đầu ra đóng gói
└── instructions/                  kế hoạch và tài liệu thiết kế (CHỈ ĐỌC)
```

Crate lõi **không** có tiền tố `mod-`. Tên crate dùng `dandori-` hoặc `mod-`. Quy tắc phẳng theo Cargo Book và M-CRATES-FLAT-FOLDER (Microsoft Pragmatic Rust Guidelines).

### 2.1 Cấu trúc bên trong một crate

```
crates/<tên>/                      (thư viện lõi)
├── Cargo.toml                     [lints] workspace = true; publish = false
├── src/lib.rs                     //! tài liệu crate; #![warn(missing_docs)]; khai báo mod; pub use
├── src/error.rs                   enum lỗi (thiserror)
├── src/<khu_vực>.rs [+ <khu_vực>/]   kiểu foo.rs, KHÔNG dùng mod.rs
└── tests/                         test tích hợp dùng API công khai

crates/dandori-server/             (chương trình)
├── src/main.rs                    MỎNG: cấu hình và gọi dandori_server::run()
└── src/lib.rs                     logic (axum, tắt êm)

crates/mod-<id>/                   (module)
├── Cargo.toml  module.json        migrations/ (SQL chỉ tiến)
├── src/{lib.rs, routes.rs, workers.rs, events.rs, model.rs}
└── tests/

frontend/app/src/
├── main.tsx
├── core/                          khung ứng dụng, client API, vùng chứa ô cắm, features_generated.ts
├── features/<module-id>/          index.ts, components/, hooks/, api/, types.ts
├── ui/                            tokens.css và thành phần dùng chung
└── i18n/                          vi.json
```

## 3. Luật phụ thuộc giữa crate (kiểm tự động bằng `scripts/check-architecture.mjs` đọc `cargo metadata`)

Mũi tên "→" nghĩa là "được phụ thuộc vào". Mọi cạnh khác **bị cấm** và `verify` đỏ.

| Crate | Được phụ thuộc vào |
| :--- | :--- |
| `dandori-types` | (không crate nội bộ nào) |
| `dandori-events` | `dandori-types` |
| `dandori-db` | `dandori-types` |
| `dandori-vault` | `dandori-types` |
| `dandori-process` | `dandori-types` |
| `dandori-git` | `dandori-types`, `dandori-process` |
| `dandori-agent-port` | `dandori-types` |
| `dandori-agent-adapters` | `dandori-types`, `dandori-process`, `dandori-agent-port` |
| `dandori-module-api` | `dandori-types`, `dandori-events`, `dandori-db`, `dandori-vault`, `dandori-agent-port`, `dandori-git` |
| `dandori-registry` | `dandori-types`, `dandori-events`, `dandori-db`, `dandori-module-api` |
| `mod-<id>` | **chỉ** `dandori-types`, `dandori-module-api` (và thư viện ngoài được duyệt) |
| `dandori-modules` | `dandori-types`, `dandori-module-api`, và **mọi `mod-*`** (crate sinh tự động; nơi DUY NHẤT ngoài `dandori-server` biết các module) |
| `dandori-server` | mọi crate lõi ở trên **và `dandori-modules`** (composition root). **Không** phụ thuộc trực tiếp `mod-*` |
| `dandori-e2e` | mọi crate (chỉ cho test; **không crate nào phụ thuộc vào nó**) |
| `fake-agent` | không crate nội bộ nào |

Hệ quả: `mod-A` **không thể** `use mod_b::...` (script sẽ thấy cạnh `mod-a → mod-b` và dừng). Hai module muốn phối hợp thì phát/nhận sự kiện hoặc gọi API lõi qua `ModuleContext`. Đây là kiểu **hexagonal** của Kantanda (`ARCH-DDD-005`): port (`dandori-agent-port`, `dandori-module-api`) do lõi sở hữu, adapter (`dandori-agent-adapters`, `dandori-git`, `dandori-vault`) cắm từ ngoài vào ở `dandori-server`.

**Vì sao cần `dandori-modules`:** Rust chỉ dùng được crate đã khai báo trong `[dependencies]`. Nếu `dandori-server` phụ thuộc trực tiếp từng `mod-*` thì mỗi module mới lại phải sửa `Cargo.toml` của server, trái tiêu chí "thêm module không sửa lõi". Vì vậy `gen-modules` ghi cả `Cargo.toml` lẫn `src/lib.rs` của `dandori-modules`; server chỉ phụ thuộc đúng crate này.

## 4. Hợp đồng module

### 4.1 File khai báo `crates/mod-<id>/module.json` (theo PDF)

```json
{
  "id": "cost-tracking",
  "group": "dashboard",
  "name": "Token và chi phí theo task",
  "description": "Một dòng mô tả hiện trong bảng Option",
  "version": "0.1.0",
  "requires": ["live-status"],
  "scopes": ["workspace", "project"],
  "effort": "medium",
  "risk": "low",
  "risk_note": "chỉ có khi risk != low",
  "core": false,
  "requires_worker_restart": false,
  "settings": { "budgetPerTask": { "type": "number", "default": null } },
  "ui": { "contributes": [ { "id": "cost-card", "slot": "board.card.badge", "label": "Chi phí", "order": 40 } ] }
}
```

- `id`: chữ thường, số, gạch ngang; **trùng tên thư mục** `mod-<id>`.
- `group`: một trong `accounts`, `orchestration`, `runtime`, `dashboard`, `scheduling`, `knowledge`, `integrations`, `safety`.
- `effort`: `light|medium|heavy`; `risk`: `low|medium|high`. `risk != low` thì bật phải có xác nhận kèm `risk_note`.
- `core: true` chỉ dành cho module không được tắt (hiện chỉ `auth-login`).
- `settings`: JSON Schema đơn giản (`type`, `default`, `enum?`, `min?`, `max?`). Cấu hình lưu ở `feature_setting.config`, được kiểm theo schema này.
- `ui.contributes`: danh sách **ô cắm giao diện** mà module đóng góp (`id`, `slot`, `label`, `order`); mã slot hợp lệ ở `04_GIAO_DIEN.md` mục 3.3. Module không có giao diện thì để mảng rỗng.
- `scripts/gen-modules.mjs` đọc mọi `crates/mod-*/module.json`, kiểm hợp lệ (id khớp thư mục, `requires` trỏ tới module có thật, **không có chu trình** kể cả bắc cầu, `ui.contributes[].slot` thuộc danh mục slot), rồi sinh **`crates/dandori-modules/Cargo.toml` và `crates/dandori-modules/src/lib.rs`** (crate cầu nối, mục 3) **và** `frontend/app/src/core/features_generated.ts` (ánh xạ slot → ô cắm nạp lười). Thêm một module = thêm một thư mục + `module.json` + chạy `pnpm run gen:modules`. **Không sửa lõi** (nhờ `members = ["crates/*"]` và crate cầu nối). `verify` kiểm file sinh ra không lệch (`--check`).

### 4.2 Trait Rust (đặt trong `dandori-module-api`)

Hình dạng (Dev chi tiết hóa, giữ nguyên ý nghĩa; tên có thể chỉnh nếu ghi lý do):

```rust
pub trait Module: Send + Sync {
    fn manifest(&self) -> &Manifest;
    /// Migration SQL của module (đường dẫn trong crate, kèm tiền tố bảng `m_<id>_`).
    fn migrations(&self) -> &'static [Migration];
    /// Đăng ký route dưới /api/v1/m/<id>/ ; registry bọc bằng cổng "module đang tắt".
    fn routes(&self, ctx: ModuleContext) -> Router;
    /// Worker nền: được khởi động khi bật, dừng khi tắt. Phải tôn trọng CancellationToken.
    fn workers(&self, ctx: ModuleContext) -> Vec<WorkerSpec>;
    /// Đăng ký bộ nghe sự kiện.
    fn subscriptions(&self) -> Vec<EventSubscription>;
}
```

`ModuleContext` là **tay cầm hẹp** do lõi trao: bus sự kiện (phát/nghe), kho dữ liệu của chính module (chỉ bảng `m_<id>_*` và các repository lõi **chỉ đọc**/qua API), cấu hình hiệu lực của module, kho khóa (chỉ qua API, không lộ giá trị), `AgentAdapter` factory, `dandori-git`. Module **không** nhận pool `SqlitePool` thô của bảng lõi.

### 4.3 Khi bật một module (PDF)

1. Registry tính các module phụ thuộc (bắc cầu), hiện danh sách "sẽ bật kèm", chờ xác nhận.
2. Chạy migration của module nếu có.
3. Đăng ký route, menu, màn hình, worker nền.
4. Phát sự kiện `module.enabled`.

### 4.4 Khi tắt một module (PDF)

1. Registry tìm module đang cần nó; hiện cảnh báo, chỉ tắt tiếp khi xác nhận (có thể tắt cả chuỗi).
2. Dừng worker nền, hủy lịch đang chờ của module.
3. Gỡ route/menu/màn hình; API trả `module_disabled`.
4. **Giữ nguyên dữ liệu và cấu hình**; phát `module.disabled`.

Mọi thay đổi trạng thái đều ghi `feature_audit`, **kể cả khi module Nhật ký kiểm toán đang tắt** (PDF).

### 4.5 Điểm mở rộng của lõi (cách DUY NHẤT để module cắm hành vi vào lõi)

Module không gọi nhau, nhưng đôi khi cần thay đổi cách lõi làm việc. Lõi sở hữu một danh sách **nhỏ, đóng** các điểm mở rộng (trait trong `dandori-module-api`); module đăng ký khi bật, registry gỡ khi tắt. Mỗi điểm mở rộng thêm sau P1 đều cần ADR và Claude kiểm riêng. Module cũng được **hỏi lõi** trạng thái hiệu lực của module khác (một API chỉ-đọc) khi thật cần, nhưng không import nhau.

| Điểm mở rộng | Việc | Thêm ở | Bên cung cấp |
| :--- | :--- | :--- | :--- |
| `WorkdirProvider` | Chuẩn bị/giải phóng thư mục làm việc của một task | P1-01 | `worktree` |
| `PromptContributor` | Thêm phần vào prompt của một lần chạy | P1-01 | `prompt-library`, `tech-profile`, `skills-md`, `shared-memory`, `agent-messaging`, `knowledge-graph` |
| `TransitionGuard` | Cho phép/từ chối/đòi duyệt một lần chuyển trạng thái task (biết nguồn kích hoạt; có thể kèm `retry_after`) | P2-03 (mở rộng nguồn ở P3-03) | `approval-gate`, `quiet-hours`, `pm-decompose` |
| `GitHostProvider` | Tạo/đọc/merge PR/MR ở máy chủ git | P2-06 | `integrations` |
| `WorkdirDecorator` | Ghi tệp hướng dẫn sinh ra vào gốc worktree trước khi chạy agent | P3-09 | `instruction-sync` |
| `ExecutionTarget` | Nơi chạy tiến trình agent (cục bộ, SSH, sandbox) | P4-05 | `ssh-remote`, `sandbox` |

Chọn tài khoản agent **không** phải điểm mở rộng: lõi chọn theo các cột `enabled`, `priority`, `exhausted_until`, `status` của `agent_account`; các module `multi-account`, `quota-tracking`, `auto-switch` chỉ ghi các cột đó và gọi `start_run` (có tham số loại trừ tài khoản).

## 5. Trạng thái hiệu lực của module tại một dự án (PDF, theo thứ tự)

1. Nếu cấp hệ thống hoặc workspace đặt `locked`: dùng giá trị của cấp đó.
2. Nếu không: giá trị ở cấp dự án nếu có.
3. Nếu không: cấp workspace, rồi cấp hệ thống.
4. Nếu không có giá trị nào: **tắt**.

Trạng thái hiệu lực được **tính** từ `feature_setting`, không lưu thẳng. Quyền đổi: chủ hệ thống đổi ở cấp hệ thống; quản trị viên workspace ở workspace; chủ dự án ở dự án.

## 6. Mô hình dữ liệu

### 6.1 Bốn bảng tính năng (PDF)

| Bảng | Khóa chính | Nội dung |
| :--- | :--- | :--- |
| `feature_module` | `id` (id module) | Danh mục nạp từ file khai báo: nhóm, tên, requires, scopes, risk, effort, phiên bản |
| `feature_setting` | (`module`, `scope_kind`, `scope_id`) | `enabled`, `locked`, `config` (JSON theo schema), `changed_by`, `changed_at` |
| `feature_pack` | `id` | Tên gói và danh sách module của gói |
| `feature_audit` | `id` (sự kiện) | module, phạm vi, hành động (bật/tắt/đổi cấu hình/hoàn tác), người thực hiện, lý do, thời điểm |

Gói mẫu (PDF; gợi ý chỉnh được): **Solo nhẹ** (đăng nhập, kết nối subscription, chọn Lead/Worker, worktree, Kanban, xem thời gian thực, thư viện prompt, hồ sơ kỹ thuật); **Nhóm nhỏ** (Solo nhẹ + nhiều tài khoản, theo dõi hạn mức, tự chuyển tài khoản, agent test, agent tài liệu, diff, tạo PR, nối GitHub/GitLab/Linear, cổng duyệt, calendar, chi phí, thông báo); **Đầy đủ** (cả 43).

### 6.2 Bảng lõi (do T03 tạo; **chỉ những bảng này**, mọi bảng khác thuộc module và có tiền tố `m_<id>_`)

| Bảng | Mục đích |
| :--- | :--- |
| `user` | id, tên đăng nhập, mật khẩu băm (argon2), vai trò hệ thống, thời điểm tạo |
| `session` | id, user, hạn dùng, băm của token |
| `workspace`, `workspace_member` | workspace và vai trò (`admin`, `member`) |
| `project` | id, workspace, tên, đường dẫn repo, nhánh mặc định |
| `task` | id, project, tiêu đề, mô tả, trạng thái, lead_agent, worker_agent, `workdir_path` (nullable), thời điểm |
| `run` | id, task, agent, account, trạng thái, bắt đầu, kết thúc, mã thoát (lõi ghi; module "Lịch sử" đọc và làm giàu) |
| `run_event` | id, run, thứ tự, loại, dữ liệu (dòng log/sự kiện của lần chạy) |
| `secret` | id, nhãn, nonce, dữ liệu mã hóa (giá trị **không bao giờ** ở dạng rõ trong DB) |
| `agent_account` | id, nhà cung cấp, nhãn, thư mục HOME riêng, loại đăng nhập (`cli` hoặc `api_key`), tham chiếu `secret` nếu là api_key, `status` (`not_installed`/`not_logged_in`/`ready`/`session_expired`/`unknown`), `enabled`, `priority` (số nhỏ = ưu tiên cao), `exhausted_until` (nullable), `last_checked_at` |

Giai đoạn sau **thêm vào lõi** đúng những thứ sau và không gì khác: `task.parent_task_id`, `run.parent_run_id`, bảng `task_dependency` (P4-01), bảng `project_repo` (P4-03). Bất kỳ bảng lõi nào khác cần ADR.

Migration chỉ tiến (forward-only) trong `crates/dandori-db/migrations/` và `crates/mod-<id>/migrations/`. Cache truy vấn offline `.sqlx/` được commit; `verify` kiểm không lệch.

## 7. Bus sự kiện

Một bus trong tiến trình (`tokio::sync::broadcast` hoặc tương đương đã duyệt). Mọi sự kiện là một kiểu có phiên bản trong `dandori-types`:

```text
Envelope { id: Uuid, at: timestamp, kind: EventKind, scope: { workspace?, project?, task?, run? }, payload }
```

Loại sự kiện lõi (module có thể thêm loại **của riêng mình** khai báo trong `dandori-types` qua task có phạm vi rõ): `module.enabled`, `module.disabled`, `task.created`, `task.status_changed`, `task.assigned`, `run.started`, `run.output`, `run.finished`, `run.failed`, `approval.requested`, `approval.decided`, `account.quota_updated`, `notification.requested`, `audit.recorded`, `worker.failed`.

- Người nghe chậm không được làm nghẽn người phát: dùng hàng đợi có giới hạn, khi tràn thì **ghi nhận mất sự kiện** (đếm và log), không chặn.
- Sự kiện **không** chứa bí mật.
- Bộ nghe của module chỉ chạy khi module đang bật.

## 8. Adapter agent và chạy tiến trình

- `dandori-agent-port` định nghĩa `AgentAdapter` (khởi động một lần chạy với: thư mục làm việc, prompt, account, vai trò; trả luồng sự kiện chuẩn hóa: văn bản, gọi công cụ, hết hạn mức, kết thúc, lỗi) và kiểu dữ liệu quanh nó. Đây là **port**; `dandori-agent-adapters` là **adapter** (Claude, Gemini, Codex, và adapter giả cho test). Thêm một nhà cung cấp AI = thêm một adapter, không sửa module nào.
- `dandori-process` là nơi **duy nhất** được `Command::new`. Quy tắc: không qua shell; tham số là mảng; `kill_on_drop(true)`; luôn có timeout tổng và timeout im lặng; đọc stdout/stderr bằng luồng có giới hạn kích thước dòng; đặt biến môi trường theo danh sách cho phép (không kế thừa toàn bộ môi trường); đặt `HOME` (hoặc tương đương) riêng cho từng account; không bao giờ đưa bí mật qua tham số dòng lệnh (dùng biến môi trường hoặc stdin).
- Mọi CLI chạy dưới **account của nó** với thư mục HOME riêng, nên đăng nhập các account không đè nhau (theo mockup). Hệ thống **không** đọc nội dung thư mục đăng nhập của CLI.
- **CLI thật** (`claude`, `agy`, `codex`): tham số lấy từ `--help` của chính CLI (điều cấm 11). Chỉ P1-02 đụng tới việc này; phần còn lại dùng `AgentAdapter` trừu tượng và `fake-agent`.

## 9. Danh mục 43 module

Công sức/rủi ro từ PDF (`công sức`: Nhẹ/Vừa/Nặng). "Task" là mã trong các file giai đoạn. `core` nghĩa là không tắt được.

| # | id | Nhóm | Module | Cần bật trước | Công sức | Task |
| :-: | :--- | :--- | :--- | :--- | :--- | :--- |
| 1 | `auth-login` | accounts | Đăng ký / đăng nhập hệ thống (core) | lõi | Nhẹ | T10 |
| 2 | `subscription-connect` | accounts | Kết nối subscription qua CLI chính thức | auth-login | Vừa | P1-02 |
| 3 | `multi-account` | accounts | Nhiều tài khoản cùng một nhà cung cấp | subscription-connect | Vừa | P2-08 |
| 4 | `quota-tracking` | accounts | Theo dõi hạn mức từng tài khoản | subscription-connect | Vừa | P2-09 |
| 5 | `auto-switch` | accounts | Tự chuyển tài khoản khi hết hạn mức | multi-account, quota-tracking | Vừa | P2-10 |
| 6 | `api-key-fallback` | accounts | API key trả theo token làm dự phòng | auth-login (+ kho khóa lõi) | Nhẹ | P2-11 |
| 7 | `lead-worker` | orchestration | Chọn Lead (PM) và Worker cho từng task | subscription-connect | Vừa | P1-03 |
| 8 | `doc-agent` | orchestration | Agent viết tài liệu tự chạy khi task xong | lead-worker | Vừa | P2-05 |
| 9 | `test-agent` | orchestration | Agent test chuyên trách | lead-worker | Vừa | P2-04 |
| 10 | `pm-decompose` | orchestration | PM tự phân rã mục tiêu thành đồ thị task | lead-worker | Nặng | P4-01 |
| 11 | `approval-gate` | orchestration | Cổng duyệt của con người | lõi | Vừa | P2-03 |
| 12 | `agent-messaging` | orchestration | Kênh nhắn tin giữa các agent | lead-worker | Nặng | P4-02 |
| 13 | `worktree` | runtime | Mỗi task một git worktree và branch riêng | lõi | Vừa | P1-04 |
| 14 | `pr-merge` | runtime | Tạo PR với mô tả do AI viết, merge khi đạt điều kiện | worktree, integrations | Vừa | P2-07 |
| 15 | `ssh-remote` | runtime | Chạy trên máy từ xa qua SSH | worktree | Nặng | P4-05 |
| 16 | `self-host` | runtime | Tự host, chạy 24/7 trên server | lõi (cách triển khai) | Vừa | P4-09 |
| 17 | `multi-repo` | runtime | Workspace nhiều repo | worktree | Vừa | P4-03 |
| 18 | `kanban` | dashboard | Bảng task dạng Kanban theo trạng thái | lõi | Vừa | P1-05 |
| 19 | `live-status` | dashboard | Xem agent đang làm gì theo thời gian thực | lõi (bus sự kiện) | Vừa | P1-06 |
| 20 | `eta` | dashboard | Dự tính thời điểm xong của task | live-status, run-history | Nặng | P3-07 |
| 21 | `reports` | dashboard | Báo cáo tổng kết ngày và tuần | kanban, run-history | Vừa | P3-05 |
| 22 | `voice-report` | dashboard | Đọc báo cáo bằng giọng nói | reports | Nhẹ | P3-06 |
| 23 | `cost-tracking` | dashboard | Token và chi phí theo task, theo agent | live-status | Vừa | P2-12 |
| 24 | `diff-preview` | dashboard | Xem diff trong giao diện và mở bản xem trước | worktree | Vừa | P2-02 |
| 25 | `run-history` | dashboard | Xem lại lịch sử từng lần chạy | lõi | Vừa | P2-01 |
| 26 | `agent-tree` | dashboard | Cây quan hệ PM và agent con | lead-worker, live-status | Nhẹ | P4-04 |
| 27 | `calendar` | scheduling | Calendar để lên lịch task định kỳ | subscription-connect | Vừa | P3-01 |
| 28 | `md-jobs` | scheduling | Job viết bằng file Markdown | calendar | Nhẹ | P3-02 |
| 29 | `quiet-hours` | scheduling | Khung giờ cấm chạy | calendar | Nhẹ | P3-03 |
| 30 | `overnight-queue` | scheduling | Hàng đợi chạy qua đêm | calendar | Vừa | P3-04 |
| 31 | `prompt-library` | knowledge | Thư viện prompt gắn theo agent và theo skill | lead-worker | Vừa | P1-07 |
| 32 | `tech-profile` | knowledge | Hồ sơ kỹ thuật dự án | prompt-library | Vừa | P1-08 |
| 33 | `skills-md` | knowledge | Skill theo chuẩn SKILL.md | prompt-library | Vừa | P3-08 |
| 34 | `instruction-sync` | knowledge | Đồng bộ file hướng dẫn từ một nguồn | tech-profile, skills-md | Vừa | P3-09 |
| 35 | `shared-memory` | knowledge | Bộ nhớ dùng chung giữa các agent | prompt-library | Nặng | P3-10 |
| 36 | `auto-lessons` | knowledge | Tự ghi bài học sau mỗi task | shared-memory | Nhẹ | P3-11 |
| 37 | `knowledge-graph` | knowledge | Đồ thị tri thức của code | shared-memory | Nặng | P4-07 |
| 38 | `integrations` | integrations | Nối GitHub, GitLab, Linear | auth-login | Vừa | P2-06 |
| 39 | `notifications` | integrations | Thông báo qua Telegram, Discord, Slack | lõi (bus sự kiện) | Vừa | P3-12 |
| 40 | `mobile` | integrations | Duyệt và theo dõi từ điện thoại | auth-login, approval-gate, live-status | Vừa | P3-13 |
| 41 | `audit-log` | safety | Nhật ký kiểm toán | lõi (bus sự kiện) | Nhẹ | P3-14 |
| 42 | `sandbox` | safety | Sandbox cho task rủi ro | worktree | Nặng | P4-06 |
| 43 | `staging-buffer` | safety | Vùng đệm duyệt trước khi áp dụng | worktree, approval-gate, diff-preview | Vừa | P4-08 |

Nhóm: 6 + 6 + 5 + 9 + 4 + 7 + 3 + 3 = **43**. `[CỔNG CHỦ]` (đọc `00_QUY_TAC_CHUNG.md` mục 9): `ssh-remote`, `self-host`, `sandbox`, tự-merge trong `pr-merge`, `pm-decompose` khi chạy không giám sát, `agent-messaging` khi có quyền ghi.

> Khớp thứ tự khóa của PDF: một số "Cần bật trước" trong PDF ghi tên tính năng, ở đây ghi `id`. `pr-merge` cần `integrations` để nói chuyện với GitHub/GitLab. Nếu một task phát hiện phụ thuộc thiếu/thừa so với bảng này, ghi vào "Đề xuất", không tự đổi.

## 10. Quy ước API

- Tiền tố `/api/v1`. Route của module nằm dưới `/api/v1/m/<module-id>/...`. Registry gắn **cổng** vào mọi route đó: module tắt ở phạm vi của yêu cầu thì trả `403` với mã lỗi `module_disabled` và thông điệp tiếng Việt "module đang tắt".
- Mọi lỗi cùng một dạng: `{ "error": { "code": "<snake_case>", "message": "<an toàn, không lộ nội bộ>", "request_id": "<uuid>" } }`. Không bao giờ trả stack trace hay đường dẫn tuyệt đối của máy (Kantanda `BE-ERR-002`).
- Xác thực: cookie phiên `HttpOnly; SameSite=Strict`, kiểm `Origin`/`Host` với danh sách cho phép cục bộ; mọi route (trừ đăng nhập/đăng ký lần đầu, kiểm sức khỏe) đòi phiên hợp lệ. Server nghe **127.0.0.1**.
- Dữ liệu vào được **kiểm ở biên** bằng kiểu có kiểm tra (Kantanda `BE-VAL-001/002`: schema chặt, từ chối trường lạ).
- Thời gian thực: do module `live-status` cung cấp tại `GET /api/v1/m/live-status/stream` (SSE), lọc theo phạm vi và quyền người dùng, chỉ trả loại sự kiện cho phép (P1-06). Module khác cần "làm mới" thì dùng truy vấn thường, không phụ thuộc luồng này.
- `GET /api/v1/features/effective?project=<id>` trả trạng thái hiệu lực của mọi module (để frontend dựng menu).

## 11. Frontend

Chi tiết bố cục, hệ thống ô cắm, token và ảnh tham chiếu: **`04_GIAO_DIEN.md`** (bắt buộc đọc với task giao diện).

- `frontend/app`: Vite + React + TypeScript (strict). Cấu trúc **theo tính năng** `src/features/<module-id>/` (components, hooks, api, types colocated; `index.ts` xuất `contributions`); `src/core/` cho khung ứng dụng, vùng chứa ô cắm và client API; `src/ui/` cho thành phần dùng chung và token. Kantanda `FE-ARCH-002`, `FE-API-001` áp dụng.
- **Menu theo 7 màn hình cố định** (thanh bên trái), mỗi màn hình dựng từ các **ô cắm** do module đang bật đóng góp (`ui.contributes` trong `module.json`). Module tắt thì ô cắm của nó biến mất; màn hình không còn ô cắm nào thì ẩn mục menu. Trạng thái bật/tắt lấy từ `GET /api/v1/features/effective`.
- Kiểu dữ liệu API lấy từ `shared/types.ts` (sinh bởi ts-rs). **Cấm** viết tay kiểu trùng với kiểu Rust.
- Token thiết kế (màu sáng/tối, cỡ chữ, khoảng cách, bo góc, chuyển động) khai báo một nơi (`frontend/app/src/ui/tokens.css`) theo `04` mục 5; cấm giá trị ngoài thang (Kantanda `UI-*`). Phông tự chứa, không tải từ mạng.
- Chuỗi hiển thị tiếng Việt qua tệp i18n (không viết cứng trong JSX). Không chữ in hoa toàn bộ cho tiêu đề. Chú thích dài đặt trong tooltip "?".

## 12. Thư viện được duyệt

Thêm thư viện ngoài danh sách = ADR + hỏi chủ dự án (điều cấm 6). **Danh sách này là ứng viên, chưa được xác minh còn bảo trì hay không.** T01 phải chạy `cargo deny check advisories` và ghi vào `docs/adr/0001-dependencies.md` tình trạng từng crate (bản mới nhất, thông báo RustSec). Crate nào có cảnh báo "unmaintained/unsound" thì đề xuất thay thế, không dùng bừa.

**Rust:** `tokio`, `tokio-util` (CancellationToken), `futures`, `axum`, `tower`, `tower-http`, `serde`, `serde_json`, `sqlx` (sqlite, macros, migrate, runtime-tokio), `thiserror` (crate thư viện), `anyhow` (chỉ `dandori-server`), `tracing`, `tracing-subscriber`, `uuid`, `chrono`, `ts-rs`, `argon2`, `rand`, `secrecy`, `zeroize`, `aes-gcm`, `keyring`, `directories`, `url`, `reqwest` (rustls, không native-tls), `croner` (biểu thức cron), `tempfile` (dev), `wiremock` hoặc tương đương (dev, máy chủ HTTP giả), `proptest` (dev, tùy chọn). Thư viện YAML cho front matter Markdown ở P3-02: Dev chọn crate còn bảo trì, ghi ADR.

**npm:** `react`, `react-dom`, `react-router-dom`, `@tanstack/react-query`, `zustand`, `zod`, `tailwindcss`, `lucide-react`, `typescript`, `vite`, `@vitejs/plugin-react`, `eslint`, `typescript-eslint`, `vitest`, `@testing-library/react`, `jsdom`, `playwright` (từ P1-08 cho E2E giao diện).

## 13. Dữ liệu và thư mục chạy

- Thư mục dữ liệu (`DANDORI_DATA_DIR`; mặc định theo hệ điều hành qua `directories`, W-03). **Không** nằm trong repo. Bố cục:

```
<data>/
├── db/dandori.db                  SQLite
├── accounts/<account-id>/home/    HOME riêng của từng account CLI
├── worktrees/<project>/<task>/    git worktree của task
└── backups/
```

- Worktree của task không nằm trong cây repo của dự án người dùng.
- **Nhật ký không ghi tệp:** ghi ra stdout bằng `tracing` (12-factor, nguồn: 12factor.net/logs).
- Backup/di trú dữ liệu: ngoài phạm vi, trừ khi một task nói rõ.

## 14. Bảo mật cốt lõi (chung cho mọi task)

1. Không bí mật nào ở dạng rõ trong DB, log, sự kiện, báo cáo, tham số dòng lệnh. Kiểu bí mật là `secrecy::SecretString`/`SecretBox` với `Debug` đã che; có test.
2. Kho khóa: khóa chủ nằm trong kho hệ điều hành (Windows Credential Manager qua `keyring`); dữ liệu trong bảng `secret` mã hóa bằng AEAD với nonce duy nhất; `zeroize` bộ đệm. Nếu kho hệ điều hành không có (CI), dùng kho giả **chỉ** khi cờ test bật; mặc định thất bại rõ ràng, không tự rơi về lưu rõ.
3. Mọi tiến trình con qua `dandori-process` (mục 8). Mọi đường dẫn từ đầu vào được chuẩn hóa và kiểm nằm trong thư mục cho phép (chống đi ra ngoài bằng `..`/liên kết).
4. HTTP ra ngoài (GitHub, Linear, Telegram...): chỉ tới tên miền trong danh sách cho phép của từng tích hợp, chặn địa chỉ mạng nội bộ/loopback/metadata (Kantanda `SEC-SSRF-001`).
5. Đăng nhập: giới hạn thử sai, không phân biệt "sai tên" và "sai mật khẩu" trong thông điệp, phiên có hạn (Kantanda `SEC-AUTH-*`).
6. Nhật ký: `tracing` có cấu trúc, che bí mật (Kantanda `SEC-LOG-001`); không `println!`.

## 15. Vận hành

Nguồn: Tokio "Graceful Shutdown", ví dụ chính thức của axum, 12-factor (xem `docs/cong-nghe/cau-truc-rust-va-trien-khai.md` Phần 3).

- **Cấu hình** qua biến môi trường `DANDORI_*` (đã có `DANDORI_DATA_DIR`; thêm địa chỉ và cổng nghe). Mặc định nghe `127.0.0.1` (RS-SEC-006).
- **Tắt êm:** xử lý **cả Ctrl+C lẫn SIGTERM** (Linux, Render và hầu hết hệ quản lý tiến trình gửi SIGTERM). `CancellationToken` báo cho mọi task, `TaskTracker` chờ chúng xong, có thời hạn tắt; axum dùng `with_graceful_shutdown`. Rule `RS-OPS-001`.
- **Khởi động nhanh:** không làm việc nặng trước khi nghe cổng (12-factor "Disposability").
- **Đóng gói và triển khai** (Docker nhiều giai đoạn, người dùng không phải root, cache BuildKit): làm ở P4-09 và P5-01; ghi chú nguồn ở tài liệu trên.
