# 00. Bảng trạng thái

Agent chỉ được sửa **dòng của task đang làm** (cột Trạng thái và Báo cáo; cột **Commit do chủ dự án điền** sau khi duyệt, vì agent không commit). Trạng thái: `⏳ Todo` · `🔄 Đang làm` · `✅ DONE` · `🟡 DONE_WITH_GAPS` · `⛔ BLOCKED` · `⤼ SKIPPED`. Ghi thời gian bằng lệnh `date`, không đoán.

## Môi trường (T01 điền; chủ dự án điền phần Bước 0)

| Mục | Giá trị |
| :--- | :--- |
| `rustc --version` | `rustc 1.99.1 (58cf8f9c3 2026-10-13)` |
| `cargo --version` | `cargo 1.99.1 (5f94df478 2026-08-27)` |
| `rustup show` | `stable-x86_64-pc-windows-msvc (active, default), host x86_64-pc-windows-msvc` |
| `node -v` / `pnpm -v` / `git --version` | `v24.19.0` / `12.14.0` / `git version 2.55.0.windows.5` |
| `cargo deny --version` / `sqlx --version` | `cargo-deny 0.20.2` / `sqlx-cli 0.9.0` |
| CLI có trên máy | `claude`: có · `agy`: có · `codex`: **chưa cài** (đã kiểm 2026-10-30, kế hoạch viết) |
| Kantanda | `D:\10-kantanda`, nhánh `main`, commit `aa01465` khi viết kế hoạch |

## Mốc quan trọng

| Mục | Giá trị |
| :--- | :--- |
| Baseline (Bước 0c) | `f1532bc` ("first commit", nhánh `main`, đã đẩy lên GitHub; kiểm 2026-10-30) |
| Mốc khóa cấu hình (sau khi chủ dự án duyệt và commit T01, T02) | _(chủ dự án điền mã commit)_ |

## Nhánh theo giai đoạn (`docs/quy-trinh-git.md`)

| Giai đoạn | Nhánh | Trạng thái |
| :--- | :--- | :--- |
| P0 | `phase/p0-core` | đang làm |
| P1 | `phase/p1-run-one-task` | chưa tạo (tạo từ `main` sau khi gộp P0) |
| P2 | `phase/p2-full-workflow` | chưa tạo |
| P3 | `phase/p3-automation-knowledge` | đã tạo (chưa commit) |
| P4 | `phase/p4-heavy-risky` | chưa tạo |
| P5 | `phase/p5-release` | chưa tạo |

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
| P0 | `reports/REVIEW_P0.md` | **ĐẠT** sau kiểm lần 2 (Claude, 2026-10-31, kèm các sửa của Claude); còn mở: biên dịch Linux/macOS (DAN-VERIFY-008) |
| P1 | `reports/REVIEW_P1.md` | **ĐẠT (tự động)** sau kiểm độc lập của Claude 2026-11-01 kèm 15 sửa lỗi; **`DAN-VERIFY-003/004` ĐÃ ĐẠT với Claude (chủ dự án chạy thật 2026-11-01)**; `agy` chưa chạy thật |
| P2 | `reports/REVIEW_P2.md` | **ĐẠT (tự động)** sau kiểm độc lập của Claude 2026-11-02 kèm 13 sửa lỗi (vault dùng kho khóa hệ điều hành, token tích hợp vào vault, SSRF, phân quyền). **Còn mở:** P2-07 `pr-merge` chờ chủ dự án đọc ADR 0007; bài kiểm một tuần; kết nối thật GitHub/GitLab/Linear |
| P3 | `reports/REVIEW_P3.md` | **ĐẠT (tự động)** sau kiểm độc lập của Claude 2026-11-02 kèm 14 sửa lỗi (duyệt bộ nhớ chỉ admin và lọc theo workspace, nhật ký kiểm toán admin-only và ghi sự kiện thật, Origin theo ranh giới tên miền, bom giải nén, symlink, bộ che bí mật). **Còn mở:** nhật ký chưa ghi đăng nhập/kho khóa/ghép thiết bị; duyệt từ điện thoại chưa đi qua approval-gate; bài kiểm qua đêm thật (`DAN-VERIFY-022`); Telegram/Discord/Slack/VPN thật |
| P4 | `reports/REVIEW_P4.md` | chưa |
| P5 | `reports/REVIEW_P5.md` | chưa |

## Task (65)

Cột Phụ thuộc: task phải `DONE` (hoặc `DONE_WITH_GAPS` chấp nhận được) trước. `[CỔNG]` = cổng chủ dự án. **Cổng GĐ** = điểm dừng bắt buộc chờ Claude kiểm.

### Giai đoạn 0: Lõi và Option (`10_P0_LOI.md`)

| # | Task | Phụ thuộc | Trạng thái | Commit (chủ dự án) | Báo cáo |
| :-: | :--- | :--- | :--- | :--- | :--- |
| T01 | Khung workspace, chuỗi công cụ, hồ sơ | Bước 0 | ✅ DONE | | `reports/T01.md` |
| T01B | Chỉnh khung theo cấu trúc chuẩn (sau kiểm T01) | T01 | ✅ DONE | | `reports/T01B.md` |
| T02 | Cổng `verify` và các script kiểm | T01B | ✅ DONE | | `reports/T02.md` |
| T03 | Cơ sở dữ liệu lõi | T02 | ✅ DONE | | `reports/T03.md` |
| T04 | Bus sự kiện | T03 | ✅ DONE | | `reports/T04.md` |
| T05 | Hợp đồng module và `gen-modules` | T03, T04 | ✅ DONE | | `reports/T05.md` |
| T06 | Registry: trạng thái hiệu lực, audit, gói | T05 | ✅ DONE | | `reports/T06.md` |
| T07 | Registry: vòng đời bật/tắt thật | T06 | ✅ DONE | | `reports/T07.md` |
| T08 | Kho khóa mã hóa | T03 | ✅ DONE | | `reports/T08.md` |
| T09 | Chạy tiến trình, cổng agent, `fake-agent` | T02 | ✅ DONE | | `reports/T09.md` |
| T10 | Máy chủ axum, lỗi, phiên, `auth-login` | T07, T08 | ✅ DONE | | `reports/T10.md` |
| T11 | Workspace, dự án, task (API lõi) | T10 | ✅ DONE | | `reports/T11.md` |
| T12 | Kiểu chia sẻ và khung giao diện | T11 | ✅ DONE | | `reports/T12.md` |
| T13 | Setting > Tính năng (bảng Option) | T12 | ✅ DONE | | `reports/T13.md` |
| T14 | **Cổng P0** | T09, T13 | 🟡 DONE_WITH_GAPS (Claude: chưa đạt, xem T14B) | | `reports/T14.md` |
| T14B | Sửa lỗi tìm thấy khi kiểm P0 (`10B_P0_SUA_LOI.md`) | T14 | ✅ DONE | | `reports/T14B.md` |

### Giai đoạn 1: chạy được một task (`11_P1_CHAY_MOT_TASK.md`)

| # | Task | Phụ thuộc | Trạng thái | Commit (chủ dự án) | Báo cáo |
| :-: | :--- | :--- | :--- | :--- | :--- |
| P1-01 | Lõi chạy task, điểm mở rộng | T14B | ✅ DONE | | `reports/P1-01.md` |
| P1-02 | `subscription-connect` + adapter CLI | P1-01 | ✅ DONE | | `reports/P1-02.md` |
| P1-03 | `lead-worker` | P1-02 | ✅ DONE | | `reports/P1-03.md` |
| P1-04 | `worktree` + `dandori-git` | P1-01 | ✅ DONE | | `reports/P1-04.md` |
| P1-05 | `kanban` | T14 | ✅ DONE | | `reports/P1-05.md` |
| P1-06 | `live-status` | P1-01 | ✅ DONE | | `reports/P1-06.md` |
| P1-07 | `prompt-library` | P1-03 | ✅ DONE | | `reports/P1-07.md` |
| P1-08 | `tech-profile` | P1-07 | ✅ DONE | | `reports/P1-08.md` |
| P1-09 | **Cổng P1** | P1-02…P1-08 | ✅ DONE | | `reports/P1-09.md` |

### Giai đoạn 2: vòng làm việc (`12_P2_VONG_LAM_VIEC.md`)

| # | Task | Phụ thuộc | Trạng thái | Commit (chủ dự án) | Báo cáo |
| :-: | :--- | :--- | :--- | :--- | :--- |
| P2-01 | `run-history` | P1-09 | ✅ DONE | | `reports/P2-01.md` |
| P2-02 | `diff-preview` | P1-04 | ✅ DONE | | `reports/P2-02.md` |
| P2-03 | `approval-gate` + `TransitionGuard` | P1-09 | ✅ DONE | | `reports/P2-03.md` |
| P2-04 | `test-agent` | P1-09 | ✅ DONE | | `reports/P2-04.md` |
| P2-05 | `doc-agent` | P1-09 | ✅ DONE | 2026-11-01 | Agent tài liệu tự chạy khi task ready_to_merge, anti-loop, cờ cảnh báo tệp ngoài mẫu |
| P2-06 | `integrations` + `GitHostProvider` | T14 | ✅ DONE | | `reports/P2-06.md` |
| P2-07 | `pr-merge` (tự merge `[CỔNG]`) | P1-04, P2-06 (nên P2-03) | ⏳ SKIPPED_AWAITING_OWNER | | `docs/adr/0007-auto-merge.md` |
| P2-08 | `multi-account` | P1-02 | ✅ DONE | | `reports/P2-08.md` |
| P2-09 | `quota-tracking` | P1-02 | ✅ DONE | | `reports/P2-09.md` |
| P2-10 | `auto-switch` | P2-08, P2-09 | ✅ DONE | | `reports/P2-10.md` |
| P2-11 | `api-key-fallback` | T14 | ✅ DONE | | `reports/P2-11.md` |
| P2-12 | `cost-tracking` | P1-06 | ✅ DONE | | `reports/P2-12.md` |
| P2-13 | **Cổng P2** | P2-01…P2-12 | 🟡 DONE_WITH_GAPS (phần tự động đạt; bài kiểm một tuần: CHỜ NGƯỜI DÙNG KIỂM) | | `reports/P2-13.md` |

### Giai đoạn 3: tự động và tri thức (`13_P3_TU_DONG_TRI_THUC.md`)

| # | Task | Phụ thuộc | Trạng thái | Commit (chủ dự án) | Báo cáo |
| :-: | :--- | :--- | :--- | :--- | :--- |
| P3-01 | `calendar` | P1-02, P2-13 | ✅ DONE | | reports/P3-01.md |
| P3-02 | `md-jobs` | P3-01 | ✅ DONE | | reports/P3-02.md |
| P3-03 | `quiet-hours` (+ mở rộng guard) | P3-01 (nên P2-03) | ✅ DONE | | reports/P3-03.md |
| P3-04 | `overnight-queue` | P3-01 | ✅ DONE | | reports/P3-04.md |
| P3-05 | `reports` | P1-05, P2-01 | ✅ DONE | | reports/P3-05.md |
| P3-06 | `voice-report` | P3-05 | ✅ DONE | | reports/P3-06.md |
| P3-07 | `eta` | P1-06, P2-01 | ✅ DONE | | reports/P3-07.md |
| P3-08 | `skills-md` | P1-07 | ✅ DONE | | reports/P3-08.md |
| P3-09 | `instruction-sync` + `WorkdirDecorator` | P1-08, P3-08 | ✅ DONE | | reports/P3-09.md |
| P3-10 | `shared-memory` | P1-07 | ✅ DONE | | reports/P3-10.md |
| P3-11 | `auto-lessons` | P3-10 | ✅ DONE | | reports/P3-11.md |
| P3-12 | `notifications` | T14 | ✅ DONE | | reports/P3-12.md |
| P3-13 | `mobile` | P2-03, P1-06 | ✅ DONE | | reports/P3-13.md |
| P3-14 | `audit-log` | T14 | 🟡 DONE_WITH_GAPS | 2026-11-02 | Chuỗi băm SHA-256, phát hiện can thiệp, router bất biến, UI kiểm tra. Sau kiểm độc lập: admin-only, người ghi lấy từ phiên, tự ghi bật/tắt module, phê duyệt, hạn mức. **Chưa ghi** đăng nhập, kho khóa, ghép thiết bị, push/PR/merge (lõi chưa phát sự kiện) |
| P3-15 | **Cổng P3** | P3-01…P3-14 | 🟡 DONE_WITH_GAPS | 2026-11-02 | Kịch bản E2E qua đêm đạt, bảo vệ giờ cấm, tóm tắt sáng, audit log hợp lệ; bài kiểm qua đêm thật: CHỜ NGƯỜI DÙNG KIỂM |

### Đợt nối giao diện (`16_NOI_GIAO_DIEN.md`): làm sau P3, trước P4

Lý do: ngày 2026-11-02 mở giao diện thật lần đầu thì vỏ ứng dụng chưa nối với máy chủ (dữ liệu mẫu viết cứng, đăng nhập giả, menu ẩn). Nghiệm thu bằng trình duyệt thật so với mockup. Tài khoản đăng nhập cố định qua `.env` (backend đã xong).

| # | Task | Phụ thuộc | Trạng thái | Commit (chủ dự án) | Báo cáo |
| :-: | :--- | :--- | :--- | :--- | :--- |
| UI-00 | Siết kiểm i18n + trợ thủ chụp trình duyệt | P3 | ✅ DONE | | [UI-00.md](reports/UI-00.md) |
| UI-01 | Đăng nhập thật | UI-00 | ✅ DONE | | [UI-01.md](reports/UI-01.md) |
| UI-02 | Chọn/tạo workspace và dự án thật | UI-01 | ✅ DONE | | [UI-02.md](reports/UI-02.md) |
| UI-03 | Danh sách module thật, Cài đặt hoạt động, menu luôn đủ 7 mục | UI-02 | ✅ DONE | | [UI-03.md](reports/UI-03.md) |
| UI-04 | Bảng task thật (Kanban) và Giao task | UI-03 | ✅ DONE | | [UI-04.md](reports/UI-04.md) |
| UI-05 | Giao việc cho agent và theo dõi chạy | UI-04 | ✅ DONE | | [UI-05.md](reports/UI-05.md) |
| UI-06 | Các màn module còn lại với dữ liệu thật | UI-03 | 🟡 DONE_WITH_GAPS (giao diện các màn module chưa theo mockup, xem REVIEW_UI) | | [UI-06.md](reports/UI-06.md) |
| UI-07 | Kiểm thử trình duyệt thật trong `verify`, đối chiếu mockup | UI-01…UI-06 | 🟡 DONE_WITH_GAPS (giao diện các màn module chưa theo mockup, xem REVIEW_UI) | | [UI-07.md](reports/UI-07.md) |
| UI-08 | **Cổng nối giao diện** | UI-00…UI-07 | ✅ DONE | | [TONG_KET_UI.md](TONG_KET_UI.md) |

### Đợt thành phần dùng chung và làm đẹp theo mockup (`17_UI_THANH_PHAN_DUNG_CHUNG.md`): sau đợt nối giao diện, trước P4

Lý do: chủ dự án dùng thử thật 2026-11-03; Claude đo bằng trình duyệt thật: Tailwind chưa nối (351 chỗ class vô tác dụng, ~20 hộp thoại không phủ màn hình), nút "++", dropdown không đóng khi bấm ra ngoài, chuyển trang giật (CLS 0,51), tái sử dụng kém (175 nút thô). Không nối Tailwind: dùng bộ thành phần chung + token.

| # | Task | Phụ thuộc | Trạng thái | Commit (chủ dự án) | Báo cáo |
| :-: | :--- | :--- | :--- | :--- | :--- |
| UI-09 | Bộ thành phần dùng chung + trang xem thử | UI-08 | ⏳ Todo | | |
| UI-10 | Modal + Dropdown dùng chung, thay toàn bộ (~24 hộp thoại, dropdown không đóng) | UI-09 | ⏳ Todo | | |
| UI-11 | Nút/ô nhập dùng chung (175/70/22/8), bỏ "++", kiểm tra tự động | UI-09 | ⏳ Todo | | |
| UI-12 | Gỡ class Tailwind (351 chỗ), gỡ dependency | UI-09 | ⏳ Todo | | |
| UI-13 | Chuyển trang mượt (CLS < 0,1) | UI-09 | ⏳ Todo | | |
| UI-14 | Màn Agent và tài khoản theo mockup | UI-10…UI-12 | ⏳ Todo | | |
| UI-15 | Màn Dashboard và Lịch theo mockup | UI-10…UI-12 | ⏳ Todo | | |
| UI-16 | Màn Prompt và tri thức, Tích hợp theo mockup | UI-10…UI-12 | ⏳ Todo | | |
| UI-17 | **Cổng đợt thành phần dùng chung** | UI-09…UI-16 | ⏳ Todo | | |

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

## Backlog ý tưởng và nợ kỹ thuật (chưa giao việc)

Ghi nhận ngày 2026-11-01; chi tiết và nguồn ở `docs/y-tuong/bo-nho-va-tim-ma-nguon.md`.

| Mã | Việc | Gắn vào |
| :-: | :--- | :--- |
| B-01 | `code-map`: bản đồ repo tree-sitter + gợi ý `tệp:dòng` từ test hỏng | đầu P3 |
| B-02 | `context-budget`: lời nhắc xếp phần ổn định lên đầu, mô hình/effort theo vai trò, trần chi phí | sau P2-12 |
| B-03 | "Bản đồ não": tóm tắt sau mỗi lần chạy, mục lục, chọn nút liên quan | P3-10, P3-11, P4-07 |
| B-04 | Đo chi phí thật bằng `-ExtraArgs` trong script thử | trước B-01/B-02 |
| B-05 | Chạy thật `agy`, sửa bộ đọc đầu ra | trước khi dùng `agy` |
| B-06 | Giới hạn chạy đồng thời có thể bị vượt khi yêu cầu sát nhau | khi rảnh |
| B-07 | Dọn cảnh báo `ts-rs` khi build | khi rảnh |
| B-08 | Biên dịch và kiểm Linux/macOS | trước P5 |
| B-09 | Mở giao diện trên trình duyệt thật | trước P5 |
| B-10 | Che khóa riêng nhiều dòng trong đầu ra agent | P3-10 (✅ Đã giải quyết) |
