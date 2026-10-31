# AGENTS.md — Hướng dẫn dành cho AI Agent trong Dandori

Chào mừng agent đến với dự án **Dandori** (Trung tâm điều phối agent code chạy trên máy người dùng).
Mọi agent tham gia dự án **bắt buộc tuân thủ** các quy tắc dưới đây.

---

## 1. Thứ tự đọc tài liệu bắt buộc

Trước khi viết bất kỳ dòng mã nào, bạn phải đọc theo đúng thứ tự:
1. `instructions/implementations/00_README.md`: Cách dùng kế hoạch, mục tiêu, cổng giai đoạn.
2. `instructions/implementations/00_QUY_TAC_CHUNG.md`: Vai trò Leader/Dev/QC1/QC2, điều cấm, quy trình báo cáo, worktree sạch.
3. `instructions/implementations/00_QUYET_DINH.md`: Các quyết định kiến trúc đã chốt (D-01 đến D-21).
4. `instructions/implementations/01_KIEN_TRUC.md`: Kiến trúc hệ thống, luật phụ thuộc giữa các crate, hợp đồng module.
5. `instructions/implementations/02_QUY_UOC_RUST.md`: Quy chuẩn mã nguồn Rust (RS-*, tier T1/T2/T3, lints, clippy).
6. `instructions/implementations/03_ANH_XA_KANTANDA.md`: Quy ước TypeScript/React, ánh xạ Kantanda và hồ sơ dự án.
7. `instructions/implementations/04_GIAO_DIEN.md` (kèm thư mục `ui-reference/`): Đọc trước mọi task có liên quan đến giao diện.
8. File giai đoạn tương ứng (`10_P0_LOI.md` -> `15_P5_PHAT_HANH.md`) để đọc chi tiết task được giao.

---

## 2. Bảng "Việc cần làm → File cần đọc"

| Công việc | Tài liệu chính cần đọc |
| :--- | :--- |
| Tạo hoặc sửa crate Rust | `01_KIEN_TRUC.md`, `02_QUY_UOC_RUST.md` |
| Tạo module mới (`crates/mod-<id>/`) | `01_KIEN_TRUC.md` mục 4 & 9, `04_GIAO_DIEN.md` mục 3 |
| Viết mã giao diện TypeScript / React | `03_ANH_XA_KANTANDA.md`, `04_GIAO_DIEN.md`, `ui-reference/` |
| Thao tác cơ sở dữ liệu SQLite / migration | `01_KIEN_TRUC.md` mục 6, `02_QUY_UOC_RUST.md` mục RS-DB-* |
| Tạo tiến trình con / gọi lệnh hệ thống | `01_KIEN_TRUC.md` mục 8, `02_QUY_UOC_RUST.md` mục RS-SEC-001/002 |
| Đối chiếu quy chuẩn Kantanda | `standards/profile.md`, `D:\10-kantanda\AGENTS.md` (chỉ đọc) |
| Kiểm tra chất lượng và báo cáo | `00_QUY_TAC_CHUNG.md` mục 3 & 4, `00_MAU_BAO_CAO.md` |

---

## 3. Điều cấm tuyệt đối (Tóm tắt)

1. **Tuyệt đối không tắt máy, khởi động lại hay chuyển sang chế độ ngủ** (`shutdown`, `Stop-Computer`, `Restart-Computer`).
2. **Agent không ghi vào git:** Không `git add`, `git commit`, `git push`, `git checkout`, `git reset`, `git clean`, `git stash`, `git worktree`. Chỉ dùng git để đọc (`git status`, `git diff`, `git log`, `git check-ignore`). Chủ dự án tự duyệt và commit.
3. **Không sửa thư mục ngoài phạm vi:** Không sửa `instructions/**` (ngoại trừ ghi báo cáo tại `reports/<ID>.md`, cập nhật dòng task trong `00_TRANG_THAI.md`, và ghi quyết định tự động). Không sửa thư mục bên ngoài `D:\19-dandori\`.
4. **Kantanda (`D:\10-kantanda`) và Vibe Kanban (`D:\16-agent-os\vibe-kanban`) là CHỈ ĐỌC:** Không sao chép mã nguồn hay sửa đổi nội dung.
5. **Không dùng `unsafe`, không `unwrap()`/`expect()` ngoài mã test;** không `panic!`, `println!`, `dbg!` trong mã chạy thật; không `any`, `@ts-ignore` không lý do.
6. **Không thêm thư viện (crate / npm) ngoài danh sách được duyệt** trong `01_KIEN_TRUC.md` mục 12 nếu chưa có ADR và phê duyệt.
7. **Không bịa số liệu hay kết quả:** Luôn chạy lệnh thật và dán đầu ra nguyên văn vào báo cáo.
8. **Không chạm vào dịch vụ / CLI thật khi chạy test:** Sử dụng môi trường giả lập (`fake-agent`, mock HTTP, database tạm).

---

## 4. Lệnh kiểm tra chất lượng (Verification Gate)

Cổng kiểm tra duy nhất của dự án là:
```powershell
pnpm run verify
```
Lệnh này tự động thực thi chuỗi kiểm tra chất lượng:
1. `cargo fmt --all --check`
2. `cargo clippy --workspace --all-targets --locked -- -D warnings`
3. `cargo test --workspace --locked`
4. `cargo deny check`
5. Các script kiểm tra kiến trúc, lints, SQL, và rò rỉ bí mật
6. Kiểm tra TypeScript (`tsc --noEmit`), ESLint, Vitest cho web app.
