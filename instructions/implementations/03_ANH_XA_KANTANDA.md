# 03. Ánh xạ Kantanda, quy ước TypeScript và hồ sơ dự án

Kantanda ở `D:\10-kantanda` (nhánh `main`, đã hợp nhất PR #1, commit `aa01465` khi viết kế hoạch). **Chỉ đọc.** Dev đọc rule bằng bảng "Task → file" trong `D:\10-kantanda\AGENTS.md`; không đọc cả cây.

Nguyên tắc: Kantanda là lớp **chung**; Dandori khai báo giá trị và ngoại lệ ở `standards/profile.md`. Chuẩn Rust ở `02_QUY_UOC_RUST.md`.

## 1. Kiểu kiến trúc của Dandori (khai báo cho cơ chế `applies_to` của Kantanda)

Theo `D:\10-kantanda\rules\architecture\styles.md`, khai báo **theo gói**:

| Gói / đường dẫn | Partitioning | Dependency style | Khác |
| :--- | :--- | :--- | :--- |
| `crates/` (Rust) | domain-partitioned (mỗi module một crate) | hexagonal (port do lõi sở hữu: `dandori-module-api`, `dandori-agent-port`; adapter cắm ở `dandori-server`) | `domain-model` không áp dụng (CRUD là chính) |
| `frontend/app/` (TypeScript) | domain-partitioned | — | `feature-slice` (`src/features/<module-id>/`) |

Hệ quả:

- **Áp dụng:** `ARCH-DDD-002` (adapter cô lập dịch vụ ngoài), `ARCH-DDD-005` (hexagonal; **không có công cụ cho Rust**, nên được kiểm bằng `scripts/check-architecture.mjs` = RS-ARCH-001/002), `ARCH-MOD-002` (không deep-import; áp cho TS bằng ESLint của Kantanda, cho Rust bằng RS-ARCH-002), `ARCH-MONO-001`, `FE-ARCH-002`, `ARCH-ADR-001`.
- **Không áp dụng:** `ARCH-LAY-001`/`ARCH-SINK-001` (không khai báo `layered`), `FE-ARCH-001` (Next.js App Router; Dandori dùng Vite, D-14), `ARCH-DDD-003` (không `domain-model`). Không áp dụng **không phải** vi phạm.
- Kiểu kiến trúc được chọn bằng ADR đầu tiên (`docs/adr/0003-architecture-style.md`, T01).

## 2. Hồ sơ Kantanda: `standards/profile.md` (T01 tạo từ `D:\10-kantanda\setup\profile-template.md`)

| Mục | Giá trị |
| :--- | :--- |
| Stack | Rust (phiên bản ghim ở `rust-toolchain.toml`), axum, tokio, sqlx + SQLite; React + Vite + TypeScript; pnpm |
| Module Kantanda | base bắt buộc. `rag-pgvector`: **tắt**. `python-ml`: **tắt** |
| `{{ownership_key}}` | `workspace_id` (và `project_id` cho dữ liệu cấp dự án) |
| `{{max_fn_lines}}` | 40 cho TypeScript; **50 cho Rust** (RS-LINT-006) |
| `{{max_depth}}` | 3 |
| `{{touch_target}}` | **44** (vùng chạm hiệu dụng; nút có thể nhìn cao 36px như mockup nhưng vùng bấm mở rộng ≥ 44px, xem `04_GIAO_DIEN.md` mục 5) |
| `{{max_upload}}` | 5MB (chỉ cho nhập Skill/Markdown ở P3-08, P3-02) |
| `{{access_ttl}}` | phiên 8 giờ, gia hạn khi hoạt động; không có refresh token (ứng dụng cục bộ) |
| `{{refresh_ttl}}` | không áp dụng |
| `{{adr_path}}` | `docs/adr` (không nằm trong `.gitignore`) |
| Verify | `pnpm run verify` |
| Overrides | **Không có** lúc đầu. Mọi override cần: rule ID, đổi gì, vì sao, **ngày hết hạn**, người duyệt (Kantanda mới) |
| Extensions | `RS-*` (chuẩn Rust) và `DAN-*` (rule riêng Dandori nếu có) |

Điền giá trị thật, không để ngoặc. Tham số nào Kantanda đòi mà chưa có giá trị: dừng hỏi.

## 3. Rule Kantanda áp dụng cho từng phần Dandori

| Việc trong Dandori | Rule Kantanda | Cách áp dụng |
| :--- | :--- | :--- |
| API HTTP của lõi và module | `BE-API-001/002`, `BE-VAL-001/002`, `BE-ERR-001/002` | Kiểm đầu vào ở biên, từ chối trường lạ; lỗi một dạng, không rò nội bộ (RS-ERR-003) |
| Truy vấn dữ liệu | `BE-DB-001`, `SEC-BOLA-002`, `BE-DB-002` | Lọc theo `workspace_id`/`project_id`; test A không đọc được B (RS-DB-002) |
| Đăng nhập, phiên | `SEC-AUTH-001/002/003`, `SEC-JWT-001` (chỉ nếu dùng JWT; Dandori dùng cookie phiên nên phần JWT không áp dụng) | P0 T10 |
| Gọi dịch vụ ngoài (GitHub, Linear, Telegram...) | `SEC-SSRF-001`, `ARCH-DDD-002` | Danh sách tên miền cho phép; adapter riêng cho mỗi dịch vụ |
| Nhập tệp (Skill, Markdown job) | `SEC-UPL-001/002` | Kiểm kích thước, loại, đường dẫn |
| Nhật ký | `SEC-LOG-001` | Che bí mật (RS-SEC-003, RS-LOG-001) |
| Giới hạn tần suất | `SEC-RAT-001` (Non-Negotiable) | Giới hạn đăng nhập sai và route nhạy cảm (T10, T08); **Kantanda chưa có công cụ kiểm cho rule này**, nên QC1 kiểm bằng test hành vi |
| Phụ thuộc | `SWE-DEP-001` (Non-Negotiable) | RS-DEP-001 |
| Nội dung do AI sinh khi hiển thị | `frontend/security.md` (`SEC-AI-002`) | Không `dangerouslySetInnerHTML` với đầu ra agent; làm sạch Markdown |
| Chống prompt injection | `backend/security.md` (xem mục prompt injection) | Đầu ra/đầu vào agent là **không đáng tin**; không để nội dung tệp của repo người dùng quyết định hành động đặc quyền (module PM, messaging, memory) |
| Frontend | `frontend/ui.md`, `typescript.md`, `architecture.md`, `testing.md` | Mục 4 |
| Kiểm thử | `common/testing.md`, `backend/testing.md` | Đồ giả cho thứ ngoài tiến trình; không mock lớp nội bộ (`TEST-MCK-001`) |
| Commit | `common/git.md` (`GIT-CMT-001`, `CD-GAT-001`) | Mục 6 |
| Tài liệu | `guide/writing.md` | README, ADR, hướng dẫn |

**Lưu ý trung thực về Kantanda:** theo `D:\10-kantanda\rules\README.md` (khi viết kế hoạch), chỉ khoảng một nửa rule T1 có công cụ kiểm trong `tooling/`; `BE-VAL-001`, `SEC-RAT-001`, `SWE-DEP-001` chưa có. Với các rule đó QC1 phải kiểm bằng test hoặc review có bằng chứng, **không** coi "ESLint xanh" là đã tuân thủ.

## 4. Quy ước TypeScript và giao diện

- **TypeScript:** `strict: true`; không `any`/`as any`/`@ts-ignore` không lý do (`FE-TS-001`); `eqeqeq`; hàm ≤ 40 dòng, lồng ≤ 3 (`CODE-FUN-002`).
- **Kiểu API:** chỉ từ `shared/types.ts` (RS-TS-001). Dữ liệu từ ngoài (SSE, API) được kiểm bằng `zod` ở biên trước khi dùng.
- **Gọi mạng:** chỉ trong `features/<id>/api/` (hàm có kiểu); không `fetch`/`axios` trong JSX (`FE-API-001`).
- **Cấu trúc:** `src/features/<module-id>/` chứa `components/`, `hooks/`, `api/`, `types.ts` (`FE-ARCH-002`). Không import sâu vào ruột feature khác; chỉ qua `index.ts` công khai (`ARCH-MOD-002`).
- **Trạng thái:** dữ liệu máy chủ qua `@tanstack/react-query`; trạng thái UI toàn cục qua `zustand`; trạng thái lọc/phân trang trong URL (`FE-STA-002`).
- **Giao diện:** bố cục, ô cắm (slot) và token theo `04_GIAO_DIEN.md`. Mọi màn hình có trạng thái rỗng/đang tải/lỗi (`UI-STA-001`); vùng chạm hiệu dụng ≥ 44px (`UI-LAW-002`); biểu tượng SVG (Lucide), **không emoji làm icon** (`UI-GEN-001`); truy cập được bằng bàn phím và nhãn cho trình đọc màn hình (`UI-A11Y-001`).
- **Văn bản:** tiếng Việt qua i18n; không tiêu đề viết hoa toàn bộ; chú thích dài trong tooltip "?"; ô nhập dạng "nhãn | điều khiển".
- **ESLint:** `frontend/` dùng bản **sao** `D:\10-kantanda\tooling\eslint.config.mjs` đặt ở `frontend/tooling/kantanda-eslint.config.mjs` (có dòng đầu ghi nguồn và mã commit Kantanda), gọi với `styles: []` (TypeScript của Dandori không khai báo `hexagonal`). Sao thay vì import đường dẫn tuyệt đối để worktree sạch vẫn chạy. Cập nhật bản sao là một task riêng.
- **Semgrep:** `D:\10-kantanda\tooling\semgrep.yml` có thể chạy nếu máy có Semgrep; không bắt buộc cho `verify`. Việc kiểm tương đương cho Rust là RS-SEC-005 và các script `check-*`.

## 5. Chuẩn kiểm ở `verify` (bảng ghép)

| Bước | Tầng | Rule chính |
| :--- | :--- | :--- |
| `cargo fmt --check` | Rust | RS-FMT-001 |
| `cargo clippy ... -D warnings` | Rust | RS-LINT-*, RS-PANIC-002, RS-ASYNC-001/007, RS-API-001/002, RS-LOG-001, RS-ARCH-005, RS-SEC-001 |
| `cargo test --workspace` | Rust | RS-TEST-*, RS-ERR-003, RS-SEC-003/006/007, RS-ASYNC-004 |
| `cargo deny check` | Rust | RS-DEP-001 |
| `scripts/check-architecture.mjs` | Rust | RS-ARCH-001/002/004, RS-ERR-002, RS-DEP-002 |
| `scripts/check-lints.mjs` | Rust | RS-LINT-002/004, RS-DEP-004, RS-ERR-001 (anyhow) |
| `scripts/check-sql.mjs` + `cargo sqlx prepare --check` | Rust | RS-SEC-005, RS-DB-001 |
| `generate-types --check` | Rust↔TS | RS-TS-001 |
| `scripts/scan-secrets.mjs` | Cả hai | RS-SEC-003, điều cấm 12 |
| `scripts/gen-modules.mjs --check` | Cả hai | Hợp đồng module (`01` mục 4.1) |
| `tsc --noEmit`, `eslint`, `vitest` | TypeScript | `FE-*`, `UI-*`, `TEST-*` của Kantanda |

Tên script cụ thể do T02 đặt; bảng này là **yêu cầu**, T02 phải có đủ.

## 6. Commit (chủ dự án tự làm, quyết định D-21)

**Agent không commit, không `git add`, không push.** Chủ dự án tự commit sau khi duyệt từng task. Để việc đó nhanh và đúng chuẩn Kantanda (`GIT-CMT-001`), mỗi báo cáo có mục **Đề xuất commit** gồm: (1) danh sách đường dẫn cụ thể để `git add`, (2) thông điệp commit theo mẫu dưới, (3) một dòng nhắc commit nên tách nếu task chạm hai mục đích khác nhau.

Mẫu thông điệp (Conventional Commits; một thay đổi một mục đích):

```text
<type>(<phạm-vi>): <mô tả ngắn, chữ thường, không dấu chấm cuối>

<thân tùy chọn: vì sao, không phải cái gì>

Task: <ID task>
```

`type`: `feat`, `fix`, `test`, `refactor`, `docs`, `chore`, `build`. Ví dụ: `feat(registry): tính trạng thái hiệu lực theo phạm vi`.

- Không đưa tên "Claude" hay "Antigravity" hay dòng "Co-Authored-By" của agent vào thông điệp (chủ dự án là tác giả).
- Nên `git add <đường dẫn cụ thể>` thay vì `git add -A`, để không vô tình quét tệp ngoài phạm vi.
- Duyệt xong mới commit, và commit **trước** khi giao task kế để `git diff` luôn chỉ hiện đúng một task.

## 7. `.gitignore` (T01 tạo; yêu cầu tối thiểu)

```gitignore
# build
/target/
/frontend/**/dist/
/frontend/**/node_modules/
/node_modules/
/release/
# dữ liệu và bí mật
.env
.env.*
!.env.example
*.db
*.db-journal
*.sqlite*
/.dandori-data/
# công cụ
.vscode/
.idea/
Thumbs.db
.DS_Store
# báo cáo tạm của QC (KHÔNG ignore instructions/implementations/reports/)
/tmp-qc/
```

Quy tắc: thư mục neo bằng dấu `/` đầu dòng; **không** dùng mẫu chung chung (ví dụ `*.json`, `test*`) có thể che nhầm tệp thật; cho phép tệp fixture chứa "bí mật giả" (đặt trong `tests/fixtures/` và thêm vào danh sách cho phép của `scan-secrets.mjs`). T01 viết test `git check-ignore` cho các đường dẫn quan trọng (phải ignore: `target/`, `.env`; **không** ignore: `Cargo.lock`, `pnpm-lock.yaml`, `.sqlx/`, `shared/types.ts`, `instructions/implementations/reports/`, `docs/adr/`).

## 8. Kantanda tìm thấy gì khi Dandori dùng (để ghi lại)

Nếu Dev thấy một rule Kantanda sai, mơ hồ hoặc không hợp với Rust: **không sửa Kantanda** (chỉ đọc). Ghi vào mục "Đề xuất" của báo cáo kèm rule ID và lý do; Claude sẽ đưa vào Kantanda sau.
