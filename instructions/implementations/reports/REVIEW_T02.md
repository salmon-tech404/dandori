# Kiểm độc lập T02 (Claude, 2026-10-31)

Người kiểm: Claude (không phải người viết code). Đối tượng: cây làm việc trên `phase/p0-core` (chưa commit) và `reports/T02.md`. Mọi con số dưới đây do Claude tự chạy, không lấy từ báo cáo của Antigravity.

## Kết luận

**T02 đạt yêu cầu chức năng.** `pnpm run verify` xanh trên bản sao sạch. Tìm thấy **3 lỗi vừa** trong các script kiểm (script bỏ sót vi phạm thật) và vài lỗi nhỏ về báo cáo. Claude đã **sửa 3 lỗi vừa**, thêm test đỏ cho từng lỗi, chạy lại toàn bộ.

## Đã tự kiểm

| Việc | Kết quả |
| :--- | :--- |
| Bản sao sạch 134 file (`git ls-files` + `git init`, gồm `instructions/`, không `node_modules/`, `target/`) | `pnpm install --frozen-lockfile` exit 0 (2s) |
| `pnpm run verify` trên bản sao sạch | exit 0, 15/15 bước, 52s |
| `pnpm run verify` trên cây hiện tại sau khi Claude sửa | exit 0, 15/15 bước, 9.97s |
| `node --test` (6 tệp) sau khi sửa | 19 test, 19 pass (trước khi sửa: 16) |
| Cố ý thêm một dòng vào `shared/types.ts` | `verify` đỏ đúng bước 11 (`shared/types.ts bị lệch so với định nghĩa trong Rust`), hoàn tác thì xanh lại |
| `frontend/tooling/kantanda-eslint.config.mjs` so với `D:\10-kantanda\tooling\eslint.config.mjs` (bỏ khác biệt CRLF) | Chỉ khác 3 dòng đầu (ghi nguồn + commit); commit `aa01465` trùng `git rev-parse` hiện tại của Kantanda |
| Tự thử ca đỏ trên cây thật | `#![allow]` bị bắt; `sqlx::query(&format!(..))` một dòng bị bắt; khóa `sk-ant-...` bị bắt (xem lỗi 1-4 cho các ca KHÔNG bị bắt) |

## Phát hiện

| # | Mức | Phát hiện | Bằng chứng | Xử lý |
| :-: | :--- | :--- | :--- | :--- |
| 1 | Vừa | `check-sql` quét **từng dòng** nên bỏ sót `sqlx::query(` xuống dòng rồi `&format!(...)`, đúng kiểu rustfmt hay ngắt | Chạy thật: tệp 5 dòng chứa lời gọi đó, `check-sql` báo hợp lệ, exit 0 | **Claude sửa**: quét cả tệp, số dòng tính từ vị trí khớp; thêm fixture `invalid-sql-format-multiline.rs` và test đỏ |
| 2 | Vừa | `check-lints` kiểm `[lints] workspace = true` bằng hai phép `includes` rời nhau, nên crate có `[lints.rust] unused = "allow"` tự khai và `version.workspace = true` vẫn **qua** | Chạy thật: crate đó qua, exit 0 | **Claude sửa**: regex yêu cầu `workspace = true` ngay dưới `[lints]`; thêm tùy chọn `cratesDir`, fixture `fixtures/lints/crates/bad-lints`, test đỏ |
| 3 | Vừa | **`gen-modules` không có ca đỏ**, trái T02 mục 9 ("mỗi script có ít nhất một ca đỏ"); báo cáo vẫn ghi "đều có ca xanh và ca đỏ cho từng script" | `tests/tooling/gen-modules.test.mjs` chỉ có 1 test xanh | **Claude sửa**: thêm tùy chọn `cratesDir`, fixture `fixtures/modules/crates/mod-nomanifest` (thiếu `module.json`), test đỏ |
| 4 | Nhỏ | Kiểm shell (RS-SEC-001) trong `check-lints` chỉ bắt hai chuỗi **liền nhau** (`"sh", "-c"`). Các dạng thông dụng `Command::new("sh").arg("-c")`, `.args(["/C","dir"])` **qua** | Chạy thật: tệp có cả 3 dạng, `check-lints` exit 0 | Không sửa. Lớp chặn chính là `clippy.toml disallowed-methods` (T01); đây chỉ là lớp phụ. Ghi nhận để không tin quá mức |
| 5 | Nhỏ | `check-sql` không thấy truy vấn dựng gián tiếp (`let sql = format!(..); sqlx::query(&sql)`) | Chạy thật: exit 0 | Giới hạn của quét văn bản. `cargo sqlx prepare --check` (T03) và review sẽ phủ |
| 6 | Nhỏ | Danh sách cho phép của `scan-secrets` thêm `tests/tooling/fixtures/` ngoài `tests/fixtures/` của đặc tả, không khai báo. Cần thiết vì fixture nằm ở đó. `scan-secrets` chỉ quét tệp **đã git theo dõi**, nên tệp mới chưa `git add` không bị quét ở máy Dev | Đọc mã, chạy thật | Chấp nhận. Ghi nhận |
| 7 | Nhỏ | Bước `cargo sqlx prepare --check` trong bảng `03` mục 5 chưa có trong `verify` (chưa có DB). Báo cáo không nêu | Đối chiếu `03` mục 5 | Chấp nhận, **phải thêm ở T03** khi có truy vấn sqlx |
| 8 | Nhỏ | **Độ chính xác báo cáo `T02.md`:** (a) "đã xác nhận ... tương thích POSIX" nhưng chưa từng chạy trên Linux/macOS (chính mục "Chưa kiểm" ghi DAN-VERIFY-002); (b) tiêu chí 3 không dán đầu ra như đặc tả, chỉ diễn giải; (c) dựng bản sao tại `D:\Dandori-qc` và ghi đã dọn, Claude không kiểm được; (d) không khai báo bước `cargo sqlx prepare` còn thiếu | Đọc báo cáo | Không sửa báo cáo (là bằng chứng gốc); ghi tại đây |

## Claude đã sửa

`scripts/check-sql.mjs`, `scripts/check-lints.mjs` (thêm `options.cratesDir`), `scripts/gen-modules.mjs` (thêm `options.cratesDir`), `tests/tooling/check-sql.test.mjs`, `check-lints.test.mjs`, `gen-modules.test.mjs`, và 3 fixture mới trong `tests/tooling/fixtures/`. Sau sửa: `verify` xanh (9.97s), 19/19 test.

## Chưa kiểm

- Chưa chạy trên Linux/macOS; `useShell: isWin` chưa được thử ngoài Windows.
- Khi `timeout` bắn, `proc.kill()` có thể không dọn hết tiến trình con của `pnpm` trên Windows. Chưa thử.
- Giao diện chưa mở trên trình duyệt (chỉ `tsc`, `eslint`, 1 test vitest).
- `cargo test`: 0 test Rust (còn là khung).

## Commit đề xuất

Dùng đúng 4 nhóm của báo cáo (đã kiểm khớp toàn bộ tệp thay đổi); thư mục `tests/tooling/` và `scripts/` đã gồm các tệp Claude thêm. Thêm `reports/REVIEW_T02.md` vào nhóm 4.

```powershell
git add crates/dandori-types/Cargo.toml crates/dandori-types/src/lib.rs crates/dandori-server/src/bin/generate_types.rs shared/types.ts Cargo.lock
git commit -m "feat(types): add ts-rs export and generate_types binary" -m "Task: T02"

git add scripts tests/tooling
git commit -m "feat(tooling): add verify gate and check scripts with tests" -m "Task: T02"

git add frontend/tooling frontend/app/eslint.config.js frontend/app/vitest.config.ts frontend/app/package.json frontend/app/src/App.tsx frontend/app/src/App.test.tsx pnpm-lock.yaml package.json
git commit -m "feat(frontend): add kantanda eslint config, vitest and app test" -m "Task: T02"

git add docs/manual-test-checklist.md instructions
git commit -m "docs: add manual test checklist, T02 report and review" -m "Task: T02"

git status -sb
git push
```
