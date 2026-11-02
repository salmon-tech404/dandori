# Tổng kết Giai đoạn 2: Vòng làm việc đầy đủ (P2-01…P2-13)

- **Người thực hiện:** Autonomous Engineering Team (Leader, Dev, QC1, QC2)
- **Thời gian hoàn thành:** 2026-11-01
- **Trạng thái cổng P2:** 🟡 DONE_WITH_GAPS (Phần tự động đạt; bài kiểm một tuần: CHỜ NGƯỜI DÙNG KIỂM; P2-07 chờ duyệt ADR)

---

## 1. Bảng Trạng thái Chi tiết Các Task Giai đoạn 2 (P2-01 ➔ P2-13)

| Task | Tên Task / Module | Phụ thuộc | Trạng thái | Lý do cụ thể & Kết quả nghiệm thu |
| :---: | :--- | :--- | :---: | :--- |
| **P2-01** | `run-history` | P1-09 | ✅ DONE | Lưu trữ lịch sử lần chạy, phát lại log phân trang, lọc đa tiêu chí, gắn nhãn/ghi chú, xuất Markdown/JSON đã che bí mật, worker dọn dẹp theo hạn lưu giữ không xóa run tham chiếu. |
| **P2-02** | `diff-preview` | P1-04 | ✅ DONE | So sánh diff tệp (thêm/sửa/xóa/đổi tên), hiển thị an toàn không chèn HTML thô, khởi chạy máy chủ xem trước trên localhost (127.0.0.1) qua mảng tham số và dừng an toàn. |
| **P2-03** | `approval-gate` | P1-09 | ✅ DONE | Điểm mở rộng `TransitionGuard` của lõi (ADR 0005). Chặn chuyển trạng thái khi chưa duyệt, phân tách trách nhiệm (người yêu cầu không được tự phê duyệt), tích hợp máy trạng thái nhiệm vụ. |
| **P2-04** | `test-agent` | P1-09 | ✅ DONE | Tự động kích hoạt agent kiểm thử sau khi worker kết thúc. Phân tích kết quả test, chuyển trạng thái nhiệm vụ sang `needs_human` nếu test đỏ hoặc `in_review` nếu test xanh. Anti-loop bảo vệ. |
| **P2-05** | `doc-agent` | P1-09 | ✅ DONE | Tự động chạy agent cập nhật tài liệu khi nhiệm vụ tới trạng thái `ready_to_merge`. Tự nhận diện thay đổi kiến trúc/API, giới hạn sửa tệp doc, gắn cảnh báo nếu sửa tệp ngoài mẫu. |
| **P2-06** | `integrations` | T14 | ✅ DONE | Điểm mở rộng `GitHostProvider` của lõi (ADR 0006). Hỗ trợ GitHub, GitLab, Linear qua mock adapter chuẩn; quản lý token trong Két bảo mật (Vault); BOLA cách ly workspace. |
| **P2-07** | `pr-merge` (Tự merge `[CỔNG]`) | P1-04, P2-06 | ⏳ SKIPPED_AWAITING_OWNER | Đã lập ADR `docs/adr/0007-auto-merge.md` chi tiết mối đe dọa, biện pháp giảm thiểu và mặc định TẮT. Không viết mã tính năng khi chưa có "ĐỒNG Ý" từ chủ dự án theo quy định. |
| **P2-08** | `multi-account` | P1-02 | ✅ DONE | Quản lý nhiều tài khoản agent CLI cùng hoặc khác nhà cung cấp. Cách ly HOME riêng biệt (`accounts/<id>/home`), kiểm tra trạng thái đăng nhập ngầm, sắp xếp ưu tiên theo độ ưu tiên (`priority`). |
| **P2-09** | `quota-tracking` | P1-02 | ✅ DONE | Theo dõi hạn mức sử dụng subscription/quota cho từng tài khoản. Đánh dấu `exhausted_until`, tự động chuyển sang trạng thái sẵn sàng khi hết thời gian chờ, cảnh báo sắp cạn hạn mức. |
| **P2-10** | `auto-switch` | P2-08, P2-09 | ✅ DONE | Tự động chuyển đổi tài khoản dự phòng khi nhận sự kiện `RunFailed(QuotaExhausted)`. Cơ chế chống vòng lặp (`max_switches`), ngắt chuyển và đưa về `needs_human` khi hết tài khoản khả dụng. |
| **P2-11** | `api-key-fallback` | T14 | ✅ DONE | Lưu trữ API key trả phí trong Két bảo mật (Vault) dạng mã hóa AES-256-GCM. Bắt buộc đặt trần chi tiêu (`spending_limit`), chính sách chỉ kích hoạt khi CLI hết quota, che giấu khóa masked key. |
| **P2-12** | `cost-tracking` | P1-06 | ✅ DONE | Đọc số token thực tế từ CLI agent trong `run_event` (không ước đoán bịa đặt). Tính toán chi phí qua bảng giá người dùng tự cấu hình. Giám sát ngân sách nhiệm vụ và cảnh báo vượt ngưỡng đúng 1 lần. |
| **P2-13** | **Cổng P2 (P2 Gate)** | P2-01…P2-12 | 🟡 DONE_WITH_GAPS | Kịch bản tích hợp E2E tự động (`p2_gate.rs`) xanh 100%; kiểm thử đột biến QC1 đỏ và hoàn tác; probe HTTP thành công trên `dandori.exe`; bài kiểm 7 ngày chuẩn bị sẵn sàng chờ người dùng kiểm. |

---

## 2. Danh sách Quyết định Tự động đã Ghi nhận trong Giai đoạn 2

Các quyết định thiết kế và cài đặt tự động được cập nhật liên tục tại `instructions/implementations/QUYET_DINH_TU_DONG.md`:
- **Quyết định 45 (P2-01):** Phân trang dựa trên con trỏ (Cursor-based Pagination) và che giấu bí mật nhiều tầng trong `mod-run-history`.
- **Quyết định 46 (P2-01):** Cơ chế bảo toàn dữ liệu tham chiếu (Referential Safety) trong worker dọn dẹp `mod-run-history`.
- **Quyết định 47 (P2-02):** Giới hạn kích thước diff, thoát HTML an toàn và cô lập đường dẫn tệp trong worktree.
- **Quyết định 48 (P2-02):** Ranh giới an toàn cho tính năng xem trước (Preview Server): tham số mảng, localhost và cổng người dùng chọn.
- **Quyết định 49 (P2-03):** Điểm mở rộng `TransitionGuard` của lõi và cơ chế đăng ký linh hoạt cho module (ADR 0005).
- **Quyết định 50 (P2-03):** Tách biệt trách nhiệm phê duyệt (Disallow Self-Approval) và chuyển đổi an toàn sang `needs_human`.
- **Quyết định 51 (P2-04):** Điều phối tự động Test Agent sau lượt chạy và cơ chế ngắt vòng lặp (Anti-loop Guard).
- **Quyết định 52 (P2-04):** BOLA cách ly workspace và kiểm soát chặt chẽ tham số thực thi của Test Agent.
- **Quyết định 53 (P2-05):** Điều phối tự động Doc Agent tại trạng thái `ready_to_merge` và kiểm soát phạm vi tệp tài liệu.
- **Quyết định 54 (P2-05):** Giới hạn số lần chạy tối đa (`max_doc_runs`) và cảnh báo khi sửa tệp ngoài mẫu.
- **Quyết định 55 (P2-06):** Điểm mở rộng `GitHostProvider` của lõi và kiến trúc đa nền tảng GitHub/GitLab/Linear (ADR 0006).
- **Quyết định 56 (P2-06):** Bảo mật Access Token và Webhook Secret trong Két bảo mật (Vault) và cách ly BOLA workspace.
- **Quyết định 57 (P2-07):** Xử lý ngoại lệ Cổng Chủ Task P2-07: lập hồ sơ ADR 0007, hoãn viết mã chức năng và đánh dấu `SKIPPED_AWAITING_OWNER`.
- **Quyết định 58 (P2-08):** Kiến trúc cô lập thư mục HOME cho từng tài khoản agent (`accounts/<id>/home`) và bảo vệ biến môi trường.
- **Quyết định 59 (P2-08):** Thuật toán lựa chọn tài khoản theo thứ tự ưu tiên (`priority`) kết hợp kiểm tra trạng thái hoạt động thực tế.
- **Quyết định 60 (P2-09):** Theo dõi chu kỳ hạn mức và cơ chế tự động giải phóng tài khoản khi hết thời gian chờ (`exhausted_until`).
- **Quyết định 61 (P2-09):** Cảnh báo sắp cạn hạn mức (`warning_threshold_percent`) và cập nhật trạng thái tự động qua sự kiện.
- **Quyết định 62 (P2-10):** Tự động chuyển tài khoản thay thế khi tài khoản kiệt hạn mức (`RunFailed` với `QuotaExhausted`).
- **Quyết định 63 (P2-10):** Giới hạn số lần chuyển tối đa (`max_switches`), ngắt vòng lặp chuyển đổi (Loop Breaking) và chuyển trạng thái `needs_human`.
- **Quyết định 64 (P2-11):** Quản lý và bảo vệ khóa API dự phòng trong Két bảo mật (Vault) và bắt buộc trần chi tiêu (`spending_limit`).
- **Quyết định 65 (P2-11):** Chính sách điều phối Fallback Policy trong `RunService` và kiểm soát quyền truy cập BOLA.
- **Quyết định 66 (P2-12):** Theo dõi token thực tế và chi phí linh hoạt qua bảng giá cấu hình trong module `cost-tracking`.
- **Quyết định 67 (P2-12):** Giám sát ngân sách nhiệm vụ (`budgetPerTask`) và cảnh báo vượt ngân sách một lần duy nhất per crossing.
- **Quyết định 68 (P2-13):** Kiểm thử tích hợp toàn diện Cổng Giai đoạn 2 (P2 Gate) và Phân tách trách nhiệm duyệt.

---

## 3. Các Hạng mục Chờ Người Dùng Thật Kiểm Nghiệm (Manual Checklist)

| Mã kiểm thử | Task liên quan | Nội dung cần kiểm thử thực tế | Điều kiện kiểm thử | Ghi chú & Trạng thái |
| :---: | :---: | :--- | :--- | :--- |
| `DAN-VERIFY-006` | P2-06 | Kết nối GitHub / GitLab / Linear thật | Token truy cập OAuth thật | Cấu hình repo đích và kiểm tra tạo issue / pull request tự động. Trạng thái: Chờ kiểm (P2). |
| `DAN-VERIFY-011` | P2-11 | Dự phòng API key thật khi CLI hết quota | API key Anthropic/OpenAI có credit thật | Bật dự phòng API key, gán trần chi tiêu nhỏ ($0.5), chạy task khi CLI cạn quota, xác nhận task hoàn thành và usage cộng dồn chính xác. Trạng thái: Chờ kiểm (P2). |
| `DAN-VERIFY-012` | P2-12 | Theo dõi token thật và bảng giá chi phí từ CLI chính thức (Claude / Codex / Agy) | Chạy task bằng CLI agent thật có trả về usage metadata | Chạy task thật, kiểm tra tab Chi phí ghi nhận đúng số token từ CLI, cấu hình bảng giá mô hình và xác nhận tổng chi phí tính đúng; nếu CLI không trả về token thì hiển thị "Không có số liệu", không ước đoán bịa đặt. Trạng thái: Chờ kiểm (P2). |
| **Bài kiểm một tuần** | P2-13 | Chạy một dự án thật trong 7 ngày liên tục chỉ bằng hệ thống Dandori | Tối thiểu 2 tài khoản CLI thật | Theo dõi biểu mẫu 7 ngày trong `docs/manual-test-checklist.md` mục P2-13. Ghi nhận số task, số lần đổi tài khoản, lỗi gặp. Trạng thái: CHỜ NGƯỜI DÙNG KIỂM. |

---

## 4. Danh mục Các Thành phần Chưa Thể Kiểm tra Tự động (Unverified Items)

1. **CLI Agent thật & Hạn mức Nhà cung cấp thật:** Các bài kiểm tra tự động sử dụng `fake-agent` và fixture JSONL chuẩn (`claude_stream_real.jsonl`) để tuân thủ quy tắc không tiêu tốn quota và chi phí thật. Các tài khoản Claude Code, Agy CLI, Codex CLI thật cần người dùng đăng nhập kiểm nghiệm qua `DAN-VERIFY-003`, `DAN-VERIFY-004`, `DAN-VERIFY-011`, `DAN-VERIFY-012`.
2. **Khối mã chuyên biệt cho hệ điều hành UNIX (`#[cfg(unix)]`):** Trong `dandori-process` (xử lý process group `setpgid`, tín hiệu POSIX `SIGTERM`/`SIGKILL`), mã nguồn chỉ được kiểm thử trên Windows host trong môi trường hiện tại; việc cross-compile Linux đã được chuẩn bị theo dõi tại `DAN-VERIFY-010`.
3. **Cổng kiểm tra Giai đoạn 3 (P3 Gate):** Toàn bộ các task thuộc Giai đoạn 3 (P3-01 đến P3-12) **tuyệt đối chưa được bắt đầu**, tuân thủ nghiêm ngặt ranh giới dừng sau Task P2-13.

---

## 5. Đầu ra Nguyên văn Cổng Kiểm tra Chất lượng QC2 trên Bản sao Sạch (`D:\Dandori-qc`)

```text
Scope: all 2 workspace projects
✓ Lockfile passes supply-chain policies (verified 1d ago)
Lockfile is up to date, resolution step is skipped
Packages are hard linked from the content-addressable store to the virtual store.
  Content-addressable store is at: D:\.pnpm-store\v11
  Virtual store is at:             node_modules/.pnpm
Packages: +252
++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
Progress: resolved 252, reused 252, downloaded 0, added 252, done
.../esbuild@0.25.12/node_modules/esbuild postinstall$ node install.js
.../esbuild@0.25.12/node_modules/esbuild postinstall: Done
Done in 1.6s using pnpm v12.14.0
$ node scripts/verify.mjs
===============================================================
  DANDORI VERIFICATION GATE
===============================================================

[1/18] Bắt đầu: 1. Rust Formatting (cargo fmt)...
[1/18] ✓ THÀNH CÔNG: 1. Rust Formatting (cargo fmt) (2.37s)

[2/18] Bắt đầu: 2. Rust Clippy (cargo clippy -D warnings)...
[2/18] ✓ THÀNH CÔNG: 2. Rust Clippy (cargo clippy -D warnings) (19.46s)

[3/18] Bắt đầu: 3. Rust Unit & Integration Tests (cargo test)...
[3/18] ✓ THÀNH CÔNG: 3. Rust Unit & Integration Tests (cargo test) (165.73s)

[4/18] Bắt đầu: 4. Cargo Deny Check (licenses, bans, sources)...
[4/18] ✓ THÀNH CÔNG: 4. Cargo Deny Check (licenses, bans, sources) (1.80s)

[5/18] Bắt đầu: 5. Security & Secrets Scan...
[5/18] ✓ THÀNH CÔNG: 5. Security & Secrets Scan (0.64s)

[6/18] Bắt đầu: 6. SQL Linter...
[6/18] ✓ THÀNH CÔNG: 6. SQL Linter (0.24s)

[7/18] Bắt đầu: 7. SQLx Query Metadata Verification...
[7/18] ✓ THÀNH CÔNG: 7. SQLx Query Metadata Verification (20.37s)

[8/18] Bắt đầu: 8. Architecture Boundary Linter...
[8/18] ✓ THÀNH CÔNG: 8. Architecture Boundary Linter (0.35s)

[9/18] Bắt đầu: 9. Rust Code Lints & Safety...
[9/18] ✓ THÀNH CÔNG: 9. Rust Code Lints & Safety (0.24s)

[10/18] Bắt đầu: 10. Design Tokens Sync & Hardcoded Colors...
[10/18] ✓ THÀNH CÔNG: 10. Design Tokens Sync & Hardcoded Colors (0.05s)

[11/18] Bắt đầu: 11. Module Registry Sync Check...
[11/18] ✓ THÀNH CÔNG: 11. Module Registry Sync Check (0.07s)

[12/18] Bắt đầu: 12. Type Definitions Sync Check...
[12/18] ✓ THÀNH CÔNG: 12. Type Definitions Sync Check (20.91s)

[13/18] Bắt đầu: 13. I18n Translation & Raw JSX Text Check...
[13/18] ✓ THÀNH CÔNG: 13. I18n Translation & Raw JSX Text Check (0.22s)

[14/18] Bắt đầu: 14. TypeScript Compilation (tsc)...
[14/18] ✓ THÀNH CÔNG: 14. TypeScript Compilation (tsc) (6.28s)

[15/18] Bắt đầu: 15. Vite Production Build...
[15/18] ✓ THÀNH CÔNG: 15. Vite Production Build (5.49s)

[16/18] Bắt đầu: 16. ESLint Frontend...
[16/18] ✓ THÀNH CÔNG: 16. ESLint Frontend (6.54s)

[17/18] Bắt đầu: 17. Frontend Unit Tests (vitest run)...
 Test Files  21 passed (21)
      Tests  109 passed (109)
   Duration  5.08s
[17/18] ✓ THÀNH CÔNG: 14. Frontend Unit Tests (vitest run) (5.83s)

[18/18] Bắt đầu: 18. Tooling Scripts Tests (node --test)...
ℹ tests 34
ℹ suites 0
ℹ pass 34
ℹ fail 0
ℹ cancelled 0
ℹ skipped 0
ℹ todo 0
ℹ duration_ms 13465.7961
[18/18] ✓ THÀNH CÔNG: 15. Tooling Scripts Tests (node --test) (13.52s)

===============================================================
  ✓ CỔNG KIỂM TRA VERIFY XANH (Toàn bộ 18 bước qua trong 266.29s)
===============================================================
Cleaning D:\Dandori-qc...
Mirroring D:\19-dandori to D:\Dandori-qc...
Running pnpm install in clean copy...
Running pnpm run verify in clean copy...
QC2 clean copy verification SUCCESS!
```

---

## 6. Trạng thái Cây Làm việc Git Hiện tại (`git status --porcelain`)

```text
 M .env.example
 M Cargo.lock
 M crates/dandori-agent-adapters/src/fake.rs
 M crates/dandori-agent-adapters/src/lib.rs
 M crates/dandori-agent-port/src/lib.rs
 M crates/dandori-db/src/repo/agent_account.rs
 M crates/dandori-db/src/repo/run.rs
 M crates/dandori-db/src/repo/task.rs
 M crates/dandori-e2e/Cargo.toml
 M crates/dandori-git/Cargo.toml
 M crates/dandori-git/src/lib.rs
 M crates/dandori-module-api/Cargo.toml
 M crates/dandori-module-api/src/context.rs
 M crates/dandori-module-api/src/lib.rs
 M crates/dandori-module-api/src/module.rs
 M crates/dandori-modules/Cargo.toml
 M crates/dandori-modules/src/lib.rs
 M crates/dandori-process/src/executor.rs
 M crates/dandori-process/src/lib.rs
 M crates/dandori-process/tests/process_tests.rs
 M crates/dandori-registry/src/error.rs
 M crates/dandori-registry/src/lifecycle.rs
 M crates/dandori-registry/src/registry.rs
 M crates/dandori-server/Cargo.toml
 M crates/dandori-server/src/app.rs
 M crates/dandori-server/src/config.rs
 M crates/dandori-server/src/error.rs
 M crates/dandori-server/src/lib.rs
 M crates/dandori-server/src/main.rs
 M crates/dandori-server/src/routes.rs
 M crates/dandori-server/src/routes/projects.rs
 M crates/dandori-server/src/routes/tasks.rs
 M crates/dandori-server/src/state.rs
 M crates/dandori-server/tests/core_api_tests.rs
 M crates/dandori-server/tests/server_tests.rs
 M crates/dandori-vault/src/keystore.rs
 M crates/fake-agent/src/main.rs
 M deny.toml
 M docs/manual-test-checklist.md
 M frontend/app/src/core/features_generated.ts
 M frontend/app/src/core/i18n.ts
 M instructions/implementations/00_README.md
 M instructions/implementations/00_TRANG_THAI.md
 M instructions/implementations/01_KIEN_TRUC.md
 M instructions/implementations/QUYET_DINH_TU_DONG.md
 M scripts/data/architecture-rules.json
 M scripts/gen-modules.mjs
 M scripts/scan-secrets.mjs
 M scripts/verify.mjs
 M tests/tooling/gen-modules.test.mjs
?? TONG_KET.md
?? crates/dandori-agent-adapters/src/agy.rs
?? crates/dandori-agent-adapters/src/claude.rs
?? crates/dandori-agent-adapters/src/codex.rs
?? crates/dandori-agent-adapters/src/discovery.rs
?? crates/dandori-agent-adapters/tests/cli_adapter_tests.rs
?? crates/dandori-agent-adapters/tests/fixtures/
?? crates/dandori-e2e/tests/p1_gate.rs
?? crates/dandori-e2e/tests/p2_gate.rs
?? crates/dandori-git/src/client.rs
?? crates/dandori-git/src/diff.rs
?? crates/dandori-git/src/error.rs
?? crates/dandori-git/src/info.rs
?? crates/dandori-git/src/sanitize.rs
?? crates/dandori-git/tests/
?? crates/dandori-module-api/src/access.rs
?? crates/dandori-module-api/src/extension.rs
?? crates/dandori-module-api/src/login_probe.rs
?? crates/dandori-module-api/src/secrets.rs
?? crates/dandori-module-api/tests/login_probe_tests.rs
?? crates/dandori-module-api/tests/secrets_tests.rs
?? crates/dandori-process/src/resolve.rs
?? crates/dandori-process/tests/resolve_tests.rs
?? crates/dandori-server/src/routes/runs.rs
?? crates/dandori-server/src/run_service.rs
?? crates/dandori-server/tests/p1_01_run_pipeline_tests.rs
?? crates/dandori-server/tests/p1_02_subscription_connect_tests.rs
?? crates/dandori-server/tests/p1_03_lead_worker_tests.rs
?? crates/dandori-server/tests/p1_04_worktree_tests.rs
?? crates/dandori-server/tests/p1_05_kanban_tests.rs
?? crates/dandori-server/tests/p1_06_live_status_tests.rs
?? crates/dandori-server/tests/p1_07_prompt_library_tests.rs
?? crates/dandori-server/tests/p1_08_tech_profile_tests.rs
?? crates/dandori-server/tests/p2_01_run_history_tests.rs
?? crates/dandori-server/tests/p2_02_diff_preview_tests.rs
?? crates/dandori-server/tests/p2_03_approval_gate_tests.rs
?? crates/dandori-server/tests/p2_04_test_agent_tests.rs
?? crates/dandori-server/tests/p2_05_doc_agent_tests.rs
?? crates/dandori-server/tests/p2_06_integrations_tests.rs
?? crates/dandori-server/tests/p2_08_multi_account_tests.rs
?? crates/dandori-server/tests/p2_09_quota_tracking_tests.rs
?? crates/dandori-server/tests/p2_10_auto_switch_tests.rs
?? crates/dandori-server/tests/p2_11_api_key_fallback_tests.rs
?? crates/dandori-server/tests/p2_12_cost_tracking_tests.rs
?? crates/mod-api-key-fallback/
?? crates/mod-approval-gate/
?? crates/mod-auto-switch/
?? crates/mod-cost-tracking/
?? crates/mod-diff-preview/
?? crates/mod-doc-agent/
?? crates/mod-integrations/
?? crates/mod-kanban/
?? crates/mod-lead-worker/
?? crates/mod-live-status/
?? crates/mod-multi-account/
?? crates/mod-prompt-library/
?? crates/mod-quota-tracking/
?? crates/mod-run-history/
?? crates/mod-subscription-connect/
?? crates/mod-tech-profile/
?? crates/mod-test-agent/
?? crates/mod-worktree/
?? docs/adr/0005-transition-guard.md
?? docs/adr/0006-git-host-provider.md
?? docs/adr/0007-auto-merge.md
?? docs/agents/
?? docs/y-tuong/
?? frontend/app/src/features/api-key-fallback/
?? frontend/app/src/features/approval-gate/
?? frontend/app/src/features/auto-switch/
?? frontend/app/src/features/cost-tracking/
?? frontend/app/src/features/diff-preview/
?? frontend/app/src/features/doc-agent/
?? frontend/app/src/features/integrations/
?? frontend/app/src/features/kanban/
?? frontend/app/src/features/lead-worker/
?? frontend/app/src/features/live-status/
?? frontend/app/src/features/multi-account/
?? frontend/app/src/features/prompt-library/
?? frontend/app/src/features/quota-tracking/
?? frontend/app/src/features/run-history/
?? frontend/app/src/features/subscription-connect/
?? frontend/app/src/features/tech-profile/
?? frontend/app/src/features/test-agent/
?? frontend/app/src/features/worktree/
?? instructions/implementations/00_BAI_HOC_P1.md
?? instructions/implementations/TONG_KET_P2.md
?? instructions/implementations/reports/P1-01.md
?? instructions/implementations/reports/P1-02.md
?? instructions/implementations/reports/P1-03.md
?? instructions/implementations/reports/P1-04.md
?? instructions/implementations/reports/P1-05.md
?? instructions/implementations/reports/P1-06.md
?? instructions/implementations/reports/P1-07.md
?? instructions/implementations/reports/P1-08.md
?? instructions/implementations/reports/P1-09.md
?? instructions/implementations/reports/P2-01.md
?? instructions/implementations/reports/P2-02.md
?? instructions/implementations/reports/P2-03.md
?? instructions/implementations/reports/P2-04.md
?? instructions/implementations/reports/P2-05.md
?? instructions/implementations/reports/P2-06.md
?? instructions/implementations/reports/P2-08.md
?? instructions/implementations/reports/P2-09.md
?? instructions/implementations/reports/P2-10.md
?? instructions/implementations/reports/P2-11.md
?? instructions/implementations/reports/P2-12.md
?? instructions/implementations/reports/P2-13.md
?? instructions/implementations/reports/REVIEW_P1.md
?? scratch/
```

*(Lưu ý: Không thực hiện bất kỳ lệnh ghi git nào như `git add` hay `git commit`. Chủ dự án sẽ tự duyệt và commit).*
