# 00. Bảng trạng thái

Agent chỉ được sửa **dòng của task đang làm** (cột Trạng thái và Báo cáo; cột **Commit do chủ dự án điền** sau khi duyệt, vì agent không commit). Trạng thái: `⏳ Todo` · `🔄 Đang làm` · `✅ DONE` · `🟡 DONE_WITH_GAPS` · `⛔ BLOCKED` · `⤼ SKIPPED`. Ghi thời gian bằng lệnh `date`, không đoán.

## Môi trường (T01 điền; chủ dự án điền phần Bước 0)

| Mục | Giá trị |
| :--- | :--- |
| `rustc --version` | _(điền)_ |
| `cargo --version` | _(điền)_ |
| `rustup show` | _(điền)_ |
| `node -v` / `pnpm -v` / `git --version` | _(điền)_ |
| `cargo deny --version` / `sqlx --version` | _(điền)_ |
| CLI có trên máy | `claude`: có · `agy`: có · `codex`: **chưa cài** (đã kiểm 2026-10-30, kế hoạch viết) |
| Kantanda | `D:\10-kantanda`, nhánh `main`, commit `aa01465` khi viết kế hoạch |

## Mốc quan trọng

| Mục | Giá trị |
| :--- | :--- |
| Baseline (Bước 0c) | _(chủ dự án tự commit rồi điền mã)_ |
| Mốc khóa cấu hình (sau khi chủ dự án duyệt và commit T01, T02) | _(chủ dự án điền mã commit)_ |

## Cổng chủ dự án (`00_QUY_TAC_CHUNG` mục 9)

Chủ dự án (hoặc Claude thay mặt, theo lời chủ dự án) ghi `ĐỒNG Ý <ngày>` sau khi đọc ADR tương ứng. Chưa có dòng này thì chế độ tự động **dừng** trước task.

| Task | ADR | Trạng thái |
| :--- | :--- | :--- |
| P4-01 chế độ không giám sát | 0013 | chưa |
| P4-02 chế độ đánh thức | 0014 | chưa |
| P4-05 `ssh-remote` | 0015 | chưa |
| P4-06 `sandbox` | 0016 | chưa |
| P4-09 `self-host` | 0018 | chưa |
| P2-07 tự merge | 0007 | chưa |

## Kiểm độc lập của Claude (mỗi cổng giai đoạn)

| Giai đoạn | Báo cáo | Kết quả |
| :--- | :--- | :--- |
| P0 | `reports/REVIEW_P0.md` | chưa |
| P1 | `reports/REVIEW_P1.md` | chưa |
| P2 | `reports/REVIEW_P2.md` | chưa |
| P3 | `reports/REVIEW_P3.md` | chưa |
| P4 | `reports/REVIEW_P4.md` | chưa |
| P5 | `reports/REVIEW_P5.md` | chưa |

## Task (63)

Cột Phụ thuộc: task phải `DONE` (hoặc `DONE_WITH_GAPS` chấp nhận được) trước. `[CỔNG]` = cổng chủ dự án. **Cổng GĐ** = điểm dừng bắt buộc chờ Claude kiểm.

### Giai đoạn 0: Lõi và Option (`10_P0_LOI.md`)

| # | Task | Phụ thuộc | Trạng thái | Commit (chủ dự án) | Báo cáo |
| :-: | :--- | :--- | :--- | :--- | :--- |
| T01 | Khung workspace, chuỗi công cụ, hồ sơ | Bước 0 | ⏳ Todo | | |
| T02 | Cổng `verify` và các script kiểm | T01 | ⏳ Todo | | |
| T03 | Cơ sở dữ liệu lõi | T02 | ⏳ Todo | | |
| T04 | Bus sự kiện | T03 | ⏳ Todo | | |
| T05 | Hợp đồng module và `gen-modules` | T03, T04 | ⏳ Todo | | |
| T06 | Registry: trạng thái hiệu lực, audit, gói | T05 | ⏳ Todo | | |
| T07 | Registry: vòng đời bật/tắt thật | T06 | ⏳ Todo | | |
| T08 | Kho khóa mã hóa | T03 | ⏳ Todo | | |
| T09 | Chạy tiến trình, cổng agent, `fake-agent` | T02 | ⏳ Todo | | |
| T10 | Máy chủ axum, lỗi, phiên, `auth-login` | T07, T08 | ⏳ Todo | | |
| T11 | Workspace, dự án, task (API lõi) | T10 | ⏳ Todo | | |
| T12 | Kiểu chia sẻ và khung giao diện | T11 | ⏳ Todo | | |
| T13 | Setting > Tính năng (bảng Option) | T12 | ⏳ Todo | | |
| T14 | **Cổng P0** | T09, T13 | ⏳ Todo | | |

### Giai đoạn 1: chạy được một task (`11_P1_CHAY_MOT_TASK.md`)

| # | Task | Phụ thuộc | Trạng thái | Commit (chủ dự án) | Báo cáo |
| :-: | :--- | :--- | :--- | :--- | :--- |
| P1-01 | Lõi chạy task, điểm mở rộng | T14 | ⏳ Todo | | |
| P1-02 | `subscription-connect` + adapter CLI | P1-01 | ⏳ Todo | | |
| P1-03 | `lead-worker` | P1-02 | ⏳ Todo | | |
| P1-04 | `worktree` + `dandori-git` | P1-01 | ⏳ Todo | | |
| P1-05 | `kanban` | T14 | ⏳ Todo | | |
| P1-06 | `live-status` | P1-01 | ⏳ Todo | | |
| P1-07 | `prompt-library` | P1-03 | ⏳ Todo | | |
| P1-08 | `tech-profile` | P1-07 | ⏳ Todo | | |
| P1-09 | **Cổng P1** | P1-02…P1-08 | ⏳ Todo | | |

### Giai đoạn 2: vòng làm việc (`12_P2_VONG_LAM_VIEC.md`)

| # | Task | Phụ thuộc | Trạng thái | Commit (chủ dự án) | Báo cáo |
| :-: | :--- | :--- | :--- | :--- | :--- |
| P2-01 | `run-history` | P1-09 | ⏳ Todo | | |
| P2-02 | `diff-preview` | P1-04 | ⏳ Todo | | |
| P2-03 | `approval-gate` + `TransitionGuard` | P1-09 | ⏳ Todo | | |
| P2-04 | `test-agent` | P1-09 | ⏳ Todo | | |
| P2-05 | `doc-agent` | P1-09 | ⏳ Todo | | |
| P2-06 | `integrations` + `GitHostProvider` | T14 | ⏳ Todo | | |
| P2-07 | `pr-merge` (tự merge `[CỔNG]`) | P1-04, P2-06 (nên P2-03) | ⏳ Todo | | |
| P2-08 | `multi-account` | P1-02 | ⏳ Todo | | |
| P2-09 | `quota-tracking` | P1-02 | ⏳ Todo | | |
| P2-10 | `auto-switch` | P2-08, P2-09 | ⏳ Todo | | |
| P2-11 | `api-key-fallback` | T14 | ⏳ Todo | | |
| P2-12 | `cost-tracking` | P1-06 | ⏳ Todo | | |
| P2-13 | **Cổng P2** | P2-01…P2-12 | ⏳ Todo | | |

### Giai đoạn 3: tự động và tri thức (`13_P3_TU_DONG_TRI_THUC.md`)

| # | Task | Phụ thuộc | Trạng thái | Commit (chủ dự án) | Báo cáo |
| :-: | :--- | :--- | :--- | :--- | :--- |
| P3-01 | `calendar` | P1-02, P2-13 | ⏳ Todo | | |
| P3-02 | `md-jobs` | P3-01 | ⏳ Todo | | |
| P3-03 | `quiet-hours` (+ mở rộng guard) | P3-01 (nên P2-03) | ⏳ Todo | | |
| P3-04 | `overnight-queue` | P3-01 | ⏳ Todo | | |
| P3-05 | `reports` | P1-05, P2-01 | ⏳ Todo | | |
| P3-06 | `voice-report` | P3-05 | ⏳ Todo | | |
| P3-07 | `eta` | P1-06, P2-01 | ⏳ Todo | | |
| P3-08 | `skills-md` | P1-07 | ⏳ Todo | | |
| P3-09 | `instruction-sync` + `WorkdirDecorator` | P1-08, P3-08 | ⏳ Todo | | |
| P3-10 | `shared-memory` | P1-07 | ⏳ Todo | | |
| P3-11 | `auto-lessons` | P3-10 | ⏳ Todo | | |
| P3-12 | `notifications` | T14 | ⏳ Todo | | |
| P3-13 | `mobile` | P2-03, P1-06 | ⏳ Todo | | |
| P3-14 | `audit-log` | T14 | ⏳ Todo | | |
| P3-15 | **Cổng P3** | P3-01…P3-14 | ⏳ Todo | | |

### Giai đoạn 4: nặng và rủi ro (`14_P4_NANG_RUI_RO.md`)

| # | Task | Phụ thuộc | Trạng thái | Commit (chủ dự án) | Báo cáo |
| :-: | :--- | :--- | :--- | :--- | :--- |
| P4-01 | `pm-decompose` (+ dữ liệu cha-con lõi) | P2-13 | ⏳ Todo | | |
| P4-02 | `agent-messaging` | P2-13 | ⏳ Todo | | |
| P4-03 | `multi-repo` | P1-04 | ⏳ Todo | | |
| P4-04 | `agent-tree` | P1-06, P4-01 | ⏳ Todo | | |
| P4-05 | `ssh-remote` `[CỔNG]` | P1-04 | ⏳ Todo | | |
| P4-06 | `sandbox` `[CỔNG]` | P1-04 | ⏳ Todo | | |
| P4-07 | `knowledge-graph` | P3-10 | ⏳ Todo | | |
| P4-08 | `staging-buffer` | P1-04, P2-02, P2-03 | ⏳ Todo | | |
| P4-09 | `self-host` `[CỔNG]` | T14 | ⏳ Todo | | |
| P4-10 | **Cổng P4** | P4-01…P4-09 | ⏳ Todo | | |

### Phát hành (`15_P5_PHAT_HANH.md`)

| # | Task | Phụ thuộc | Trạng thái | Commit (chủ dự án) | Báo cáo |
| :-: | :--- | :--- | :--- | :--- | :--- |
| P5-01 | Đóng gói và chạy thử bản phát hành | P4-10 | ⏳ Todo | | |
| P5-02 | Rà soát cuối và tổng kết | P5-01 | ⏳ Todo | | |

## Mục chờ người dùng kiểm (`DAN-VERIFY-nnn`)

Danh sách đầy đủ ở `docs/manual-test-checklist.md` (T02 tạo khung, các task bổ sung). Mã đã dùng trong kế hoạch: 001 kho khóa Windows thật (T08) · 002 giao diện so mockup (T13) · 003 CLI thật một lời nhắc, 004 cô lập HOME thật, 005 một task thật (P1) · 006 bài kiểm một tuần (P2-13) · 007 GitHub/GitLab/Linear thật · 008 PR thật · 009 khóa API thật · 010 thông báo thật · 011 qua đêm thật · 012 giọng đọc · 013 duyệt từ điện thoại · 014 PM thật phân rã · 015 SSH thật · 016 thoát sandbox thử · 017 tự host thật · 018 cài bản phát hành.

## Nhật ký

- _(Chủ dự án và Claude ghi sự kiện quan trọng kèm thời gian từ lệnh `date`.)_
