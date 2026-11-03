# TỔNG KẾT ĐỢT NỐI GIAO DIỆN (UI-00…UI-08)

- **Giai đoạn:** Đợt nối giao diện (UI Wiring Phase - `16_NOI_GIAO_DIEN.md`)
- **Ngày hoàn thành:** 2026-11-03
- **Thực hiện bởi:** Leader AI Agent & Team (Dev, QC1, QC2)
- **Nhánh thực hiện:** `phase/ui-wire` (Không thay đổi nhánh, Git ở chế độ chỉ đọc)
- **Chuẩn nghiệm thu:** Trình duyệt thật (Chrome qua CDP), kết nối máy chủ Dandori thật, đối chiếu mockup `instructions/implementations/ui-reference/*.png`.

---

## 1. Bảng trạng thái từng task trong Đợt nối giao diện

| Mã Task | Tên Task | Phụ thuộc | Trạng thái chuẩn | Báo cáo chi tiết |
| :---: | :--- | :--- | :---: | :--- |
| **UI-00** | Siết kiểm i18n + trợ thủ chụp trình duyệt CDP (`ui-shots.mjs`) | P3 | **DONE** | [UI-00.md](reports/UI-00.md) |
| **UI-01** | Đăng nhập thật qua HttpOnly Cookie, loại bỏ màn Setup Owner | UI-00 | **DONE** | [UI-01.md](reports/UI-01.md) |
| **UI-02** | Chọn và tạo workspace, dự án thật (xóa "Dandori Core" tĩnh) | UI-01 | **DONE** | [UI-02.md](reports/UI-02.md) |
| **UI-03** | Feature Registry động, Cài đặt hoạt động, menu đủ 7 mục | UI-02 | **DONE** | [UI-03.md](reports/UI-03.md) |
| **UI-04** | Bảng Kanban 5 cột theo mockup, nạp task mẫu qua API thật | UI-03 | **DONE** | [UI-04.md](reports/UI-04.md) |
| **UI-05** | Giao việc cho agent, nhật ký trực tiếp thuần văn bản, hủy run | UI-04 | **DONE** | [UI-05.md](reports/UI-05.md) |
| **UI-06** | Các màn module còn lại với dữ liệu thật & cấu trúc Tabbed Slot | UI-03 | **DONE** | [UI-06.md](reports/UI-06.md) |
| **UI-07** | Kiểm thử trình duyệt thật trong `verify` (bước 16), đối chiếu mockup | UI-01…UI-06 | **DONE** | [UI-07.md](reports/UI-07.md) |
| **UI-08** | Cổng đợt nối giao diện & Dọn dung lượng an toàn | UI-00…UI-07 | **DONE** | [TONG_KET_UI.md](TONG_KET_UI.md) |

---

## 2. Trạng thái Git nguyên văn (`git status --porcelain`)

```
 M crates/dandori-server/src/routes/features.rs
 M crates/dandori-server/tests/core_api_tests.rs
 M docs/manual-test-checklist.md
 M frontend/app/src/App.test.tsx
 M frontend/app/src/App.tsx
 M frontend/app/src/core/AppLayout.tsx
 M frontend/app/src/core/AuthScreen.tsx
 M frontend/app/src/core/Slot.tsx
 M frontend/app/src/core/api.ts
 M frontend/app/src/core/i18n.ts
 M frontend/app/src/core/useFeatures.tsx
 M frontend/app/src/features/api-key-fallback/AddApiKeyModal.tsx
 M frontend/app/src/features/api-key-fallback/ApiKeyFallbackCard.tsx
 M frontend/app/src/features/audit-log/AuditLogPanel.tsx
 M frontend/app/src/features/auto-lessons/AutoLessonsPanel.tsx
 M frontend/app/src/features/auto-switch/AutoSwitchConfigCard.tsx
 M frontend/app/src/features/calendar/ScheduleFormFields.tsx
 M frontend/app/src/features/cost-tracking/PricingModal.tsx
 M frontend/app/src/features/eta/EtaTaskSection.tsx
 M frontend/app/src/features/instruction-sync/SyncForm.tsx
 M frontend/app/src/features/instruction-sync/types_generated.ts
 M frontend/app/src/features/integrations/components/IntegrationsSettings.tsx
 M frontend/app/src/features/kanban/KanbanBoard.tsx
 M frontend/app/src/features/kanban/KanbanCard.tsx
 M frontend/app/src/features/kanban/KanbanHeaderChip.tsx
 M frontend/app/src/features/kanban/TaskDetailPanel.tsx
 M frontend/app/src/features/kanban/useKanban.ts
 M frontend/app/src/features/md-jobs/JobDetailModal.tsx
 M frontend/app/src/features/multi-account/AccountGroupCard.tsx
 M frontend/app/src/features/notifications/ChannelFormModal.tsx
 M frontend/app/src/features/notifications/NotificationsPanel.tsx
 M frontend/app/src/features/prompt-library/PromptEditorModal.tsx
 M frontend/app/src/features/prompt-library/PromptLibraryTab.tsx
 M frontend/app/src/features/prompt-library/VersionHistoryModal.tsx
 M frontend/app/src/features/quiet-hours/QuietWindowFormModal.tsx
 M frontend/app/src/features/run-history/components/NoteEditorModal.tsx
 M frontend/app/src/features/run-history/components/RunHistoryFilter.tsx
 M frontend/app/src/features/sample-hello/index.tsx
 M frontend/app/src/features/settings-features/AuditLogSection.tsx
 M frontend/app/src/features/settings-features/SettingsFeaturesScreen.tsx
 M frontend/app/src/features/settings-features/useFeatureManagement.ts
 M frontend/app/src/features/shared-memory/types_generated.ts
 M frontend/app/src/features/skills-md/ImportSkillDialog.tsx
 M frontend/app/src/features/skills-md/SkillDetailDialog.tsx
 M frontend/app/src/features/skills-md/SkillsPanel.tsx
 M frontend/app/src/features/subscription-connect/AddAccountModal.tsx
 M frontend/app/src/features/tech-profile/TechProfileTab.tsx
 M frontend/app/src/features/voice-report/VoiceReportControl.tsx
 M frontend/app/vite.config.ts
 M instructions/implementations/00_TRANG_THAI.md
 M instructions/implementations/16_NOI_GIAO_DIEN.md
 M instructions/implementations/QUYET_DINH_TU_DONG.md
 M scripts/check-i18n.mjs
 M scripts/verify.mjs
 M tests/tooling/check-i18n.test.mjs
?? frontend/app/src/core/ProjectModals.tsx
?? frontend/app/src/core/ProjectSelector.tsx
?? frontend/app/src/core/project.test.tsx
?? frontend/app/src/core/project.tsx
?? frontend/app/src/core/session.test.tsx
?? frontend/app/src/core/session.tsx
?? frontend/app/src/core/storage-security.test.ts
?? frontend/app/src/features/kanban/CreateTaskModal.tsx
?? frontend/app/src/features/kanban/TaskDispatchCard.tsx
?? frontend/app/src/features/kanban/TaskLiveRunSection.tsx
?? frontend/app/src/features/kanban/kanban-wiring.test.tsx
?? frontend/app/src/features/kanban/sampleTasks.ts
?? frontend/app/src/features/kanban/task-dispatch.test.tsx
?? frontend/app/src/features/screens-wiring.test.tsx
?? frontend/app/src/features/settings-features/features-wiring.test.tsx
?? instructions/implementations/TONG_KET_UI.md
?? instructions/implementations/reports/UI-00.md
?? instructions/implementations/reports/UI-01.md
?? instructions/implementations/reports/UI-02.md
?? instructions/implementations/reports/UI-03.md
?? instructions/implementations/reports/UI-04.md
?? instructions/implementations/reports/UI-05.md
?? instructions/implementations/reports/UI-06.md
?? instructions/implementations/reports/UI-07.md
?? instructions/implementations/reports/ui/
?? scripts/sync-qc.mjs
?? scripts/ui-shots.mjs
?? tests/tooling/fixtures/i18n/alt-hardcoded.tsx
?? tests/tooling/fixtures/i18n/aria-label-hardcoded.tsx
?? tests/tooling/fixtures/i18n/clean-i18n.tsx
?? tests/tooling/fixtures/i18n/label-hardcoded.tsx
?? tests/tooling/fixtures/i18n/option-hardcoded.tsx
?? tests/tooling/fixtures/i18n/placeholder-hardcoded.tsx
?? tests/tooling/fixtures/i18n/title-hardcoded.tsx
?? tests/tooling/ui-shots.test.mjs
```

---

## 3. Đầu ra nguyên văn cổng kiểm tra `verify` trên bản sao sạch (QC2: `D:\dandori-qc`)

```
$ node scripts/verify.mjs
===============================================================
  DANDORI VERIFICATION GATE
===============================================================

[1/19] Bắt đầu: 1. Rust Formatting (cargo fmt)...
[1/19] ✓ THÀNH CÔNG: 1. Rust Formatting (cargo fmt) (6.08s)

[2/19] Bắt đầu: 2. Rust Clippy (cargo clippy -D warnings)...
[2/19] ✓ THÀNH CÔNG: 2. Rust Clippy (cargo clippy -D warnings) (13.67s)

[3/19] Bắt đầu: 3. Rust Workspace Tests (cargo test)...
[3/19] ✓ THÀNH CÔNG: 3. Rust Workspace Tests (cargo test) (137.95s)

[4/19] Bắt đầu: 4. Cargo Deny Check (cargo deny check)...
[4/19] ✓ THÀNH CÔNG: 4. Cargo Deny Check (cargo deny check) (1.49s)

[5/19] Bắt đầu: 5. Cargo Machete (Unused Dependencies)...
[5/19] ✓ THÀNH CÔNG: 5. Cargo Machete (Unused Dependencies) (0.64s)

[6/19] Bắt đầu: 6. Check Architecture (scripts/check-architecture.mjs)...
[6/19] ✓ THÀNH CÔNG: 6. Check Architecture (scripts/check-architecture.mjs) (0.13s)

[7/19] Bắt đầu: 7. Check Lints & MSRV (scripts/check-lints.mjs)...
[7/19] ✓ THÀNH CÔNG: 7. Check Lints & MSRV (scripts/check-lints.mjs) (0.33s)

[8/19] Bắt đầu: 8. Check SQL Safety (scripts/check-sql.mjs)...
[8/19] ✓ THÀNH CÔNG: 8. Check SQL Safety (scripts/check-sql.mjs) (0.24s)

[9/19] Bắt đầu: 8b. Check SQLx Query Cache (cargo sqlx prepare --check)...
[9/19] ✓ THÀNH CÔNG: 8b. Check SQLx Query Cache (cargo sqlx prepare --check) (18.66s)

[10/19] Bắt đầu: 9. Scan Secrets (scripts/scan-secrets.mjs)...
[10/19] ✓ THÀNH CÔNG: 9. Scan Secrets (scripts/scan-secrets.mjs) (0.44s)

[11/19] Bắt đầu: 10. Check Modules Sync (scripts/gen-modules.mjs --check)...
[11/19] ✓ THÀNH CÔNG: 10. Check Modules Sync (scripts/gen-modules.mjs --check) (0.11s)

[12/19] Bắt đầu: 11. Check TypeScript Types Sync (generate_types --check)...
[12/19] ✓ THÀNH CÔNG: 11. Check TypeScript Types Sync (generate_types --check) (1.10s)

[13/19] Bắt đầu: 11b. Check Design Tokens (scripts/check-tokens.mjs)...
[13/19] ✓ THÀNH CÔNG: 11b. Check Design Tokens (scripts/check-tokens.mjs) (0.09s)

[14/19] Bắt đầu: 11c. Check i18n Strings (scripts/check-i18n.mjs)...
[14/19] ✓ THÀNH CÔNG: 11c. Check i18n Strings (scripts/check-i18n.mjs) (0.32s)

[15/19] Bắt đầu: 12. Frontend TypeScript Check (tsc --noEmit)...
[15/19] ✓ THÀNH CÔNG: 12. Frontend TypeScript Check (tsc --noEmit) (8.28s)

[16/19] Bắt đầu: 13. Frontend ESLint (eslint .)...
[16/19] ✓ THÀNH CÔNG: 13. Frontend ESLint (eslint .) (6.81s)

[17/19] Bắt đầu: 14. Frontend Unit Tests (vitest run)...
[17/19] ✓ THÀNH CÔNG: 14. Frontend Unit Tests (vitest run) (143.91s)

[18/19] Bắt đầu: 15. Tooling Scripts Tests (node --test)...
✔ check-architecture: ca xanh trên cây hiện tại (11.0825ms)
✔ check-architecture: ca đỏ khi mod-a phụ thuộc mod-b (1.4727ms)
✔ check-architecture: ca đỏ khi có thư viện ngoài chưa duyệt (1.1396ms)
✔ check-architecture: ca đỏ khi có tên crate cấm (0.9329ms)
✔ check-i18n: ca xanh trên file TSX chuẩn i18n (0.9705ms)
✔ check-i18n: ca đỏ khi option chứa chữ viết cứng (1.1578ms)
✔ check-i18n: ca đỏ khi placeholder chứa chữ viết cứng (0.9412ms)
✔ check-i18n: ca đỏ khi title chứa chữ viết cứng (0.9392ms)
✔ check-i18n: ca đỏ khi aria-label chứa chữ viết cứng (0.9576ms)
✔ check-i18n: ca đỏ khi alt chứa chữ viết cứng (0.9754ms)
✔ check-i18n: ca đỏ khi label chứa chữ viết cứng (0.9575ms)
✔ check-lints: ca xanh trên cây hiện tại (313.2721ms)
✔ check-lints: ca đỏ khi có unwrap() ngoài test (1.4191ms)
✔ check-lints: ca đỏ khi có expect() ngoài test (1.0824ms)
✔ check-lints: ca đỏ khi có panic!() ngoài test (1.069ms)
✔ check-lints: ca đỏ khi có println!() ngoài test (1.0772ms)
✔ check-lints: ca đỏ khi có eprintln!() ngoài test (1.0617ms)
✔ check-lints: ca đỏ khi có dbg!() ngoài test (1.0592ms)
✔ check-lints: ca đỏ khi crate tầng T2 dùng thư viện tier T3 (1.0654ms)
✔ check-lints: ca đỏ khi phụ thuộc khai báo phiên bản riêng thay vì workspace = true (2.201ms)
✔ check-lints: ca đỏ khi gọi Command::new ngoài dandori-process (RS-SEC-001) (85.3195ms)
✔ check-sql: ca xanh trên cây hiện tại (181.1622ms)
✔ check-sql: ca xanh trên fixture SQL hợp lệ có bind (1.4666ms)
✔ check-sql: ca đỏ khi dùng format! trong sqlx::query* (1.6165ms)
✔ check-sql: ca đỏ khi format! nằm ở dòng sau (lời gọi bị ngắt nhiều dòng) (1.6246ms)
✔ check-sqlx-prepare: ca xanh khi .sqlx đồng bộ với migrations (18382.4036ms)
✔ check-sqlx-prepare: ca đỏ khi .sqlx query metadata bị sai lệch so với schema (3175.1618ms)
✔ check-tokens: ca xanh trên cây hiện tại (20.3263ms)
✔ check-tokens: ca đỏ khi file CSS chứa mã màu cứng ngoài thang token (3.8317ms)
✔ check-tokens: ca đỏ khi tokens.css thiếu token bắt buộc (13.9994ms)
✔ gen-modules: ca xanh khi kiểm tra module trên cây hiện tại (43.447ms)
✔ gen-modules: ca đỏ khi module thiếu module.json (2.4569ms)
✔ gen-modules: ca đỏ khi chu trình phụ thuộc 2 module (a -> b -> a) (3.8293ms)
✔ gen-modules: ca đỏ khi chu trình phụ thuộc 3 module bắc cầu (c -> d -> e -> c) (6.7553ms)
✔ gen-modules: ca đỏ khi requires tới module không tồn tại (4.1182ms)
✔ gen-modules: ca đỏ khi ui.contributes có slot không thuộc danh mục (2.4212ms)
✔ gen-modules: thêm mod-zz cập nhật dandori-modules mà không cần sửa Cargo.toml gốc hay server (15.9358ms)
✔ gen-modules: --check đỏ khi file sinh bị sửa tay (21.8306ms)
✔ generate_types: ca xanh khi shared/types.ts đồng bộ với Rust (765.2307ms)
✔ generate_types: ca đỏ khi shared/types.ts bị sửa tay khác với Rust (15163.4582ms)
✔ scan-secrets: ca xanh trên cây hiện tại (392.1749ms)
✔ scan-secrets: ca đỏ khi phát hiện API key bí mật (1.1931ms)
✔ scan-secrets: ca đỏ khi phát hiện private key (1.0504ms)
✔ ui-shots: findChromePath trả về đường dẫn hợp lệ hoặc null (1.5642ms)
✔ ui-shots: getFreePort cấp cổng mạng hợp lệ (18.9264ms)
✔ ui-shots: runUi07Scenario được xuất khẩu như một hàm (0.1851ms)
✔ ui-shots: --verify thoát với mã 42 và thông báo khi không có Chrome (72.0345ms)
ℹ tests 45
ℹ suites 0
ℹ pass 45
ℹ fail 0
ℹ cancelled 0
ℹ skipped 0
ℹ todo 0
ℹ duration_ms 21679.093
[18/19] ✓ THÀNH CÔNG: 15. Tooling Scripts Tests (node --test) (21.73s)

[19/19] Bắt đầu: 16. Browser UI Verification (scripts/ui-shots.mjs --verify)...
Khởi động Dandori máy chủ tại cổng 61653...
Khởi động Vite frontend tại cổng 61654...
Mở Chrome headless (CDP cổng 61655)...
Môi trường UI sẵn sàng: http://127.0.0.1:61654
=== Bắt đầu kịch bản UI-07: Walkthrough toàn bộ luồng chính ===
1. Đăng nhập với tài khoản admin...
warning: in the working copy of 'README.md', LF will be replaced by CRLF the next time Git touches it
2. Tạo Workspace và Dự án thật...
3. Trạng thái (a): Mới cài đặt...
📸 Chụp ảnh (1440x900): instructions\implementations\reports\ui\UI-07-fresh-board.png
📸 Chụp ảnh (1440x900): instructions\implementations\reports\ui\UI-07-fresh-disabled.png
4. Trạng thái (b): Bật gói Đầy đủ trong Cài đặt...
5. Đăng ký tài khoản fake agent...
6. Nạp task mẫu trên Board...
7. Giao task cho agent và theo dõi...
8. Duyệt qua 7 màn hình và chụp ảnh nghiệm thu...
Duyệt tới board (/)...
📸 Chụp ảnh (1440x900): instructions\implementations\reports\ui\UI-07-board.png
Duyệt tới agents (/agents)...
📸 Chụp ảnh (1440x900): instructions\implementations\reports\ui\UI-07-agents.png
Duyệt tới dashboard (/dashboard)...
📸 Chụp ảnh (1440x900): instructions\implementations\reports\ui\UI-07-dashboard.png
Duyệt tới calendar (/calendar)...
📸 Chụp ảnh (1440x900): instructions\implementations\reports\ui\UI-07-calendar.png
Duyệt tới knowledge (/knowledge)...
📸 Chụp ảnh (1440x900): instructions\implementations\reports\ui\UI-07-knowledge.png
Duyệt tới integrations (/integrations)...
📸 Chụp ảnh (1440x900): instructions\implementations\reports\ui\UI-07-integrations.png
Duyệt tới settings (/settings)...
📸 Chụp ảnh (1440x900): instructions\implementations\reports\ui\UI-07-settings.png
9. Kiểm tra console errors và exceptions...
=== Kịch bản UI-07 hoàn tất thành công! ===
ui-shots: Hoàn thành chụp trình duyệt.
ui-shots: Đã dọn dẹp sạch toàn bộ tiến trình và thư mục tạm.
[19/19] ✓ THÀNH CÔNG: 16. Browser UI Verification (scripts/ui-shots.mjs --verify) (48.39s)

===============================================================
  ✓ CỔNG KIỂM TRA VERIFY XANH (Toàn bộ 19 bước qua trong 432.45s)
===============================================================
```

---

## 4. Nhật ký Dọn dẹp Dung lượng Ổ Đĩa (Disk Space Delta)

Theo quy định bắt buộc của Task UI-08:
- Đã thực hiện xóa đúng hai thư mục tạm tạo ra bởi đợt UI:
  1. `D:\dandori-qc` (Bản sao sạch kiểm thử QC2)
  2. `D:\dandori-target-ui` (Thư mục target riêng cho build UI)
- Lệnh thực thi:
  `Remove-Item -LiteralPath "D:\dandori-qc" -Recurse -Force; Remove-Item -LiteralPath "D:\dandori-target-ui" -Recurse -Force`
- Thư mục mã nguồn và nhị phân chính `D:\19-dandori\target` được bảo toàn nguyên vẹn.

### Bảng số liệu dung lượng trống trên ổ D:
| Thời điểm | Dung lượng sử dụng (Used) | Dung lượng trống (Free) | Tương đương (GB) |
| :--- | :---: | :---: | :---: |
| **Trước khi dọn dẹp** | 189,008,834,560 B | 192,671,776,768 B | 179.44 GB |
| **Sau khi dọn dẹp** | 135,865,823,232 B | 245,814,788,096 B | 228.93 GB |
| **Chênh lệch (Giải phóng)** | **-53,143,011,328 B** | **+53,143,011,328 B** | **+49.49 GB** |

> Dung lượng trống hiện tại đạt **228.93 GB**, vượt xa ngưỡng an toàn tối thiểu **40 GB** bắt buộc cho các giai đoạn tiếp theo.

---

## 5. Danh sách Khác biệt Còn lại so với Mockup

Chi tiết trích xuất từ `instructions/implementations/reports/ui/DOI_CHIEU_MOCKUP.md`:

| Màn hình | Khác biệt còn lại | Lý do kỹ thuật & Quy chuẩn |
| :--- | :--- | :--- |
| **Board** | 1. Icon thẻ dùng bộ Lucide SVG thay vì logo 3rd party (Linear, Slack).<br>2. Chip "Dữ liệu minh họa" chỉ hiện khi vừa nạp mẫu. | RS-ARCH-012: Ứng dụng tự chứa font/icon nội bộ, không phụ thuộc tài nguyên mạng từ bên ngoài.<br>Tự ẩn khi có task thật từ người dùng. |
| **Agent & Tài khoản** | Hiển thị tài khoản `fake` thay vì tài khoản `Codex`. | Máy chủ cục bộ chưa cấu hình CLI Codex; sử dụng `fake-agent` để kiểm thử an toàn, không tốn quota API. |
| **Dashboard** | Đồ thị phân bổ dùng CSS tokens thanh trực quan thay vì biểu đồ SVG vẽ tay. | Tối giản, không kéo theo thư viện biểu đồ nặng ngoài danh mục đã duyệt (KISS/YAGNI). |
| **Lịch** | Trạng thái ban đầu trống task chờ cấu hình job. | Lưới lịch nạp động từ cơ sở dữ liệu thật thay vì dữ liệu cứng. |
| **Prompt & Tri thức** | Sử dụng Tabbed Slot Layout thay vì 2 cột tĩnh cố định. | Khả năng mở rộng ARIA tablist/tab: các module mở rộng có thể đóng góp thêm tab độc lập mà không làm vỡ bố cục. |
| **Tích hợp** | Token ghép nối thiết bị sinh mã ngẫu nhiên 1 lần, tự hủy khi đóng hộp thoại. | Quy tắc bảo mật RS-SEC (Rule 8 & Rule 17): Không lưu token một lần trong bộ nhớ trình duyệt sau khi đóng dialog. |
| **Cài đặt** | 1. Đã sửa lỗi dính chữ "Đang bật41/41" thành "Đang bật 41/41".<br>2. Có hộp thoại phân tích rủi ro & phụ thuộc.<br>3. Nút Hoàn tác ẩn khi backend chưa hỗ trợ. | Tuân thủ mục 7 của `04_GIAO_DIEN.md` và tài liệu đặc tả kiến trúc PDF. Tránh nút giả gây hiểu lầm. |

---

## 6. Danh mục Hạng mục Chưa kiểm chứng (`DAN-VERIFY-*`)

Các hạng mục kiểm thử thủ công đang chờ Chủ dự án (Owner) trực tiếp kiểm nghiệm:

- `DAN-VERIFY-001` (T08): Kho khóa Windows Credential Manager thật.
- `DAN-VERIFY-002` (T13): Chủ dự án xem giao diện thật so với mockup.
- `DAN-VERIFY-003` (P1-02/P1-09): Chạy thật với CLI agent đã đăng nhập (Claude Code / Agy).
- `DAN-VERIFY-004` (P1-02/P1-09): Cô lập HOME riêng biệt cho từng CLI.
- `DAN-VERIFY-005` (P1-09): Thực thi task thật trên repo git thật.
- `DAN-VERIFY-006` (P2-06): Kết nối GitHub / GitLab / Linear thật.
- `DAN-VERIFY-007` (P3-12): Thông báo qua Telegram Bot / Discord Webhook thật.
- `DAN-VERIFY-008` (P4-05): Thực thi task trên máy chủ Linux từ xa qua SSH.
- `DAN-VERIFY-009` (T14): Kiểm thử luồng P0 máy chủ thật.
- `DAN-VERIFY-010` (T14B): Biên dịch Linux cross-compile.
- `DAN-VERIFY-011` (P2-11): Dự phòng API key khi CLI cạn quota.
- `DAN-VERIFY-012` (P2-12): Đối soát token và bảng giá chi phí thật từ CLI.
- `DAN-VERIFY-013` (P3-06): Nghe giọng đọc báo cáo Web Speech API trên trình duyệt.
- `DAN-VERIFY-014` (P3-07): Kiểm tra ước tính ETA đối soát thực tế trên Kanban.
- `DAN-VERIFY-015` (P3-08): Nhập và duyệt tệp `SKILL.md` thật.
- `DAN-VERIFY-016` (P3-09): Đồng bộ tệp hướng dẫn AGENTS.md, CLAUDE.md và kiểm tra git exclude.
- `DAN-VERIFY-017` (P3-10): Kiểm tra duyệt tri thức, phòng thủ nhiễm độc bộ nhớ và lọc private key.
- `DAN-VERIFY-018` (P3-11): Kiểm tra tự động rút bài học vào bộ nhớ chung ở trạng thái chờ duyệt.
- `DAN-VERIFY-019` (P3-12): Gửi thông báo Telegram/Discord/Slack thật.
- `DAN-VERIFY-020` (P3-13): Duyệt từ điện thoại qua mạng riêng an toàn (Tailscale/VPN).
- `DAN-VERIFY-021` (P3-14): Kiểm tra toàn vẹn chuỗi băm audit log và phát hiện can thiệp dữ liệu.
- `DAN-VERIFY-022` (P3-15): Bài kiểm chạy qua đêm thật (Overnight run).
- `DAN-VERIFY-023` (UI-00): Trợ thủ chụp ảnh trình duyệt thật qua CDP (`scripts/ui-shots.mjs`).
- `DAN-VERIFY-024` (UI-08): **Bài kiểm tay 30 phút toàn diện giao diện thật nối máy chủ** (hướng dẫn chi tiết 8 bước tại `docs/manual-test-checklist.md`).

---

## 7. Các hạng mục chờ Chủ dự án phê duyệt

1. **Kiểm tra mắt và chạy bài kiểm tay 30 phút (`DAN-VERIFY-024`):** Thực hiện chuỗi 8 bước trên trình duyệt Chrome thật tại `http://localhost:5173`.
2. **Duyệt và thực hiện Git Commit cho Đợt nối giao diện:** Toàn bộ các thay đổi của UI-00…UI-08 trên nhánh `phase/ui-wire`.
3. **Task `pr-merge` (P2-07):** Tiếp tục giữ trạng thái `SKIPPED_AWAITING_OWNER` theo yêu cầu ban đầu.
4. **Quyết định thời điểm khởi động Giai đoạn 4 (P4):** Nhóm Agent **DỪNG LẠI TẠI ĐÂY**, tuyệt đối không tự ý bắt đầu P4.
