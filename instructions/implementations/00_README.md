# 00. Bộ kế hoạch Dandori: cách dùng

> Viết ngày 2026-10-30 bởi Claude (vai: lập kế hoạch và kiểm tra độc lập). Antigravity (Leader + Dev + QC1 + QC2) thực hiện. Chủ dự án kiểm những gì agent không làm được (CLI thật đã đăng nhập, giao diện, tài khoản thật).

## Mục tiêu

Dandori là một **trung tâm điều phối agent code chạy trên máy người dùng**: người dùng giao việc, một agent PM chia task, các agent worker (Claude, Gemini, Codex...) làm trong nhánh git riêng, và mọi thứ hiện trên một dashboard. Mỗi tính năng là một **module bật/tắt được** trong màn hình Setting > Tính năng.

Phạm vi: **lõi nhỏ luôn bật + 43 module** trong 8 nhóm (xem `01_KIEN_TRUC.md` mục 9). Stack: **Rust (backend) + TypeScript/React (frontend)**, cùng kiểu với `D:\16-agent-os\vibe-kanban` (chỉ để tham khảo ý tưởng, không sao chép mã, xem `00_QUYET_DINH.md` D-03).

Nguồn thiết kế gốc của chủ dự án (chỉ đọc):
- `../system-ideas/Thiết kế hệ thống code đa Agent module và bảng Option.pdf`
- `../system-ideas/Agent Hub – Giao diện chính.html` (mockup giao diện; đã được chụp và giải mã vào `ui-reference/`, xem `04_GIAO_DIEN.md`)

## Thứ tự đọc

| File | Nội dung |
| :--- | :--- |
| `../../AGENTS.md` | Điểm vào cho agent (T01 tạo) |
| `00_QUY_TAC_CHUNG.md` | Luật chung, vai Leader/Dev/QC1/QC2, điều cấm, báo cáo, worktree sạch, chế độ tự động, khóa cấu hình |
| `00_QUYET_DINH.md` | Quyết định đã chốt, giả định, việc chờ chủ dự án |
| `01_KIEN_TRUC.md` | Kiến trúc, cây thư mục, crate, registry, mô hình dữ liệu, sự kiện, adapter, kho khóa, API, danh mục 43 module |
| `02_QUY_UOC_RUST.md` | **Chuẩn code Rust** (rule ID, tier T1/T2/T3, nguồn, công cụ kiểm) |
| `03_ANH_XA_KANTANDA.md` | Quy ước TypeScript/frontend và ánh xạ Kantanda, hồ sơ dự án |
| `04_GIAO_DIEN.md` + `ui-reference/` | **Giao diện:** bố cục theo mockup, hệ thống ô cắm (slot) cho module, token thiết kế, ảnh tham chiếu 7 màn hình. Đọc trước mọi task có giao diện |
| `10_P0_LOI.md` | Giai đoạn 0: lõi + Option (T01–T14) |
| `11_P1_CHAY_MOT_TASK.md` | Giai đoạn 1: chạy được một task (P1-01…P1-09) |
| `12_P2_VONG_LAM_VIEC.md` | Giai đoạn 2: vòng làm việc đầy đủ (P2-01…P2-13) |
| `13_P3_TU_DONG_TRI_THUC.md` | Giai đoạn 3: tự động và tri thức (P3-01…P3-15) |
| `14_P4_NANG_RUI_RO.md` | Giai đoạn 4: nặng và rủi ro (P4-01…P4-10) |
| `15_P5_PHAT_HANH.md` | Phát hành (P5-01…P5-02) |
| `00_MAU_BAO_CAO.md`, `00_TRANG_THAI.md` | Mẫu báo cáo, bảng tiến độ |
| `00_PROMPT_CHAY_DEM.md` | Prompt dán một lần để chạy tự động nhiều giờ |

## Bước 0: chuẩn bị (chủ dự án, một lần, theo thứ tự)

Các việc này **agent không làm được** (cần quyền quản trị, cài đặt có giao diện, hoặc đăng nhập).

**0a. Cài chuỗi công cụ Rust (điều kiện chặn).** Khi kế hoạch được viết (2026-10-30), máy **chưa có** `rustc`/`cargo`/`rustup` và chưa thấy công cụ build C++ của Visual Studio. Cần:
1. Cài *Visual Studio Build Tools* với thành phần "Desktop development with C++" (để Rust build được trên Windows).
2. Cài `rustup` từ trang chính thức của Rust, chọn toolchain `stable`.
3. Mở PowerShell mới và chạy: `rustc --version`, `cargo --version`, `rustup component add clippy rustfmt`.
4. Cài công cụ kiểm: `cargo install --locked cargo-deny cargo-nextest sqlx-cli` (có thể mất 10–20 phút).
5. Dán đầu ra các lệnh vào `00_TRANG_THAI.md` (mục "Môi trường").

**0b. Các CLI agent.** Máy hiện có `claude` và `agy`; **chưa có `codex`**. Module "Kết nối subscription" cần ít nhất một CLI đã đăng nhập thật để chủ dự án kiểm cuối. Cài Codex CLI nếu muốn kiểm nhà cung cấp OpenAI. Các test tự động **không** dùng CLI thật (dùng CLI giả, xem `00_QUY_TAC_CHUNG.md` mục 5).

**0c. Commit baseline (chủ dự án tự làm).** Repo git đã được khởi tạo ở `D:\19-dandori`. **Chủ dự án tự commit và tự review**; agent không bao giờ commit hay `git add` (D-21). Chạy trong PowerShell:

```powershell
cd D:\19-dandori
git branch -m main
git add instructions docs
git commit -m "chore: baseline instructions before dandori build"
git rev-parse --short HEAD
```

Điền mã commit in ra vào `00_TRANG_THAI.md` (dòng Baseline). Có thể nhờ Claude giải thích các lệnh. Baseline là **mốc xuất phát** để so sánh mọi thay đổi sau này.

**0d. Các quyết định còn mở** trong `00_QUYET_DINH.md` mục "Chờ chủ dự án". Có mặc định an toàn, nhưng nên xem trước khi chạy.

## Cách giao một task cho Antigravity

Dán đoạn sau, thay `<ID>`:

```text
Bạn là Leader của một đội gồm Dev, QC1, QC2 (xem instructions/implementations/00_QUY_TAC_CHUNG.md).
Đọc theo thứ tự: AGENTS.md, instructions/implementations/00_QUY_TAC_CHUNG.md, 01_KIEN_TRUC.md,
02_QUY_UOC_RUST.md, 03_ANH_XA_KANTANDA.md, 04_GIAO_DIEN.md (và xem ảnh ui-reference/ nếu task có giao diện), rồi task <ID> trong file giai đoạn tương ứng.
Chỉ làm task <ID>. Không sửa file ngoài phạm vi của task.
KHÔNG commit, KHÔNG git add, KHÔNG push: chỉ dùng git để ĐỌC (status, diff, log). Chủ dự án tự commit.
Kết thúc bằng báo cáo đúng mẫu 00_MAU_BAO_CAO.md tại instructions/implementations/reports/<ID>.md (có mục Đề xuất commit), rồi DỪNG chờ chủ dự án duyệt.
Không tuyên bố đã kiểm với CLI/tài khoản thật. Không bịa số liệu; dán đầu ra lệnh nguyên văn.
```

Giao theo thứ tự trong `00_TRANG_THAI.md` (cột Phụ thuộc). Task cùng giai đoạn không phụ thuộc nhau có thể chạy song song **chỉ khi** chúng không sửa cùng file (mỗi task ghi rõ "Chạm vào").

## Nhịp làm việc của chủ dự án (mỗi task)

1. Giao task cho Antigravity (đoạn prompt trên). Khi nó dừng, mở `reports/<ID>.md`.
2. **Đừng chỉ tin dòng "DONE".** Chạy `git status` và `git diff` để xem thay đổi thật; hoặc nhờ Claude "kiểm <ID>" để đối chiếu báo cáo với code và chạy lại `verify` trên bản sao sạch.
3. Nếu ổn: **tự commit** theo mục "Đề xuất commit" trong báo cáo (danh sách file và thông điệp). Ghi mã commit vào cột Commit của `00_TRANG_THAI.md`.
4. Nếu chưa ổn: nhắn lại Antigravity những chỗ cần sửa, lặp lại bước 1.
5. Sau T02 được duyệt: ghi mã commit làm **mốc khóa cấu hình**.

Vì agent không commit nên **mỗi task nên được duyệt và commit trước khi giao task kế**. Nhờ vậy `git diff` luôn chỉ hiện thay đổi của đúng một task.

## Sau mỗi giai đoạn: Claude kiểm tra độc lập

Chủ dự án nhờ Claude: "kiểm giai đoạn N". Claude sẽ: chạy lại `scripts/verify` trên bản sao sạch, đối chiếu báo cáo với code thật, lấy mẫu vài mục để thử, kiểm các đường dẫn được bảo vệ và phạm vi commit, kiểm cổng của giai đoạn. Báo cáo kiểm lưu ở `reports/REVIEW_P<N>.md`.

## Cổng giữa các giai đoạn (lấy từ PDF thiết kế)

| Giai đoạn | Cổng để sang giai đoạn sau |
| :--- | :--- |
| **P0 Lõi** | Bật và tắt được một module mẫu; dữ liệu còn nguyên sau khi tắt; API của module tắt trả lỗi "module đang tắt" |
| **P1 Chạy được một task** | Giao một task cho worker, xem agent chạy và nhận kết quả trên nhánh riêng (với CLI giả trong test; CLI thật do chủ dự án kiểm) |
| **P2 Vòng làm việc** | Một dự án thật chạy trọn một tuần chỉ bằng hệ thống (chủ dự án kiểm, agent không làm được) |
| **P3 Tự động và tri thức** | Hệ thống chạy qua đêm và để lại báo cáo buổi sáng |
| **P4 Nặng và rủi ro** | Từng module qua kiểm thử riêng trước khi cho bật ở chế độ thường |

Cổng P2 và P3 cần người dùng thật nên agent **chỉ chuẩn bị** (kịch bản kiểm trong `docs/manual-test-checklist.md`), không tuyên bố đã qua.

## Thứ tự chạy tự động (tuần tự)

`T01 → T02 → … → T14 → P1-01 → … → P1-09 → P2-01 → … → P2-13 → P3-01 → … → P3-15 → P4-01 → … → P4-10 → P5-01 → P5-02`

Thứ tự trong từng file giai đoạn đã được sắp theo phụ thuộc. Giai đoạn 4 gồm module rủi ro cao: mỗi module có **cổng chủ dự án** riêng (`00_QUY_TAC_CHUNG.md` mục 9) và không được chạy tự động khi chưa có xác nhận.
