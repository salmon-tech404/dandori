# 17. Đợt thành phần dùng chung và làm đẹp theo mockup (UI-09…UI-17)

**Vì sao có đợt này.** Sau đợt nối giao diện (`16`, đã DONE_WITH_GAPS) chủ dự án dùng thử thật ngày 2026-11-03 và báo lỗi. Claude đã đo bằng trình duyệt thật (ngày 2026-11-03) và xác nhận các nguyên nhân gốc dưới đây. **Bằng chứng nằm ở đây, không phải suy đoán.**

## Bằng chứng đo được (đừng tranh cãi, hãy sửa)

| Hiện tượng chủ dự án thấy | Nguyên nhân gốc đã xác nhận | Số liệu |
| :--- | :--- | :--- |
| Dashboard, Lịch, Prompt và tri thức trông như HTML thô | `tailwindcss` có trong `package.json` nhưng **chưa nối** (không `@import "tailwindcss"`, không plugin Vite). Các class Tailwind trong mã **không có tác dụng** | **351** chỗ `className` kiểu Tailwind ở các module |
| Hộp thoại (ví dụ "Tạo prompt") không phủ màn hình mà chèn thẳng vào dòng nội dung | Cùng nguyên nhân: `fixed inset-0 ...` của Tailwind không có tác dụng (đo: `position: static`) | **14 module / ~20 hộp thoại** |
| Nút có hai dấu `++` | Chữ trong từ điển đã có `+` ("+ Giao task") mà nút lại thêm icon `Plus` | 3 chỗ: `AppLayout` (Giao task), `PromptLibraryTab`, `TechProfileTab`. Ngoài ra còn nhãn bắt đầu bằng `+`: `btn_add_account`, `api_key_fallback_add_btn` |
| Bấm ra ngoài không đóng dropdown | Không có xử lý bấm-ra-ngoài/Escape | `core/ProjectSelector.tsx`, `features/kanban/KanbanCard.tsx` (menu "Chuyển tới"). 4 hộp thoại tự viết cũng không đóng bằng Escape/nền: `core/ProjectModals.tsx`, `core/DirectoryPicker.tsx`, `features/kanban/CreateTaskModal.tsx`, `features/worktree/WorktreeSection.tsx` |
| Chuyển trang bị giật, lệch nội dung rồi mới về chuẩn | Danh sách module của Cài đặt tải bất đồng bộ nên chiều cao trang nhảy 844 → 6604 px, thanh cuộn xuất hiện làm nội dung đổi rộng 1208 → 1193 px; các màn lazy (`Slot`) hiện trắng rồi mới có nội dung | Chuyển Tích hợp → Cài đặt: **CLS = 0,51** (trên 0,25 là mức kém). Các trang khác 0,001 đến 0,09 và trống lúc 30 ms. Đã sửa riêng phần thanh cuộn (`scrollbar-gutter: stable`), phần còn lại thuộc UI-13 |
| Tái sử dụng kém | Mỗi nơi tự viết style riêng | Nút: 31 dùng `<Button>` vs **175** `<button>` thô. Ô nhập: 6 `<Input>` vs **70** `<input>`, **22** `<select>`, **8** `<textarea>` thô. Tooltip: 1 `<Tooltip>` vs 28 thuộc tính `title`. Hộp thoại: 1 tệp dùng `ConfirmDialog`, ~24 chỗ tự dựng. Icon: tốt (24 tệp cùng `lucide-react`, không icon vẽ tay) |

## Đã làm sẵn (Claude, không làm lại)

Thành phần dùng chung: `ui/components/Switch.tsx` (thanh hình chữ nhật bo góc 40×22, vùng chạm 44 px nằm ngoài), `Avatar.tsx`, `EmptyState.tsx`; `core/UserMenu.tsx` (avatar + tên + bánh răng, menu có đóng khi bấm ra ngoài/Escape), `core/AppSidebar.tsx`, `core/menu.ts`; `core/DirectoryPicker.tsx`, `core/BranchSelect.tsx`; bố cục cố định `100vh` (thanh bên đứng yên, chỉ phần nội dung cuộn); `Slot` truyền `workspaceId`/`projectId` cho mọi màn module (cả `context` lẫn thuộc tính riêng); Cài đặt: nút phạm vi Hệ thống/Workspace/Dự án nạp đúng trạng thái; gói "Đầy đủ" = mọi module trừ module rủi ro cao; thông báo lỗi "Không kết nối được máy chủ"; API `GET /api/v1/fs/dirs`, `/fs/git-branches` (chỉ quản trị hệ thống); workspace mặc định cho tài khoản `.env`. Đừng sửa lại những thứ này trừ khi gặp lỗi (ghi vào báo cáo).

## Quyết định thiết kế (chủ dự án giao Claude chọn, 2026-11-03)

**Không nối Tailwind.** Giữ MỘT hệ thống kiểu duy nhất: thành phần dùng chung + design tokens (`ui/tokens.css`). Chuyển mọi class Tailwind sang thành phần dùng chung/token (UI-12), rồi gỡ `tailwindcss` khỏi `package.json` (cập nhật `pnpm-lock.yaml`; `pnpm install --frozen-lockfile` phải xanh).

## Luật riêng của đợt này

1. Đọc trước: `00_BAI_HOC_P1.md`, `P2`, `P3`, `04_GIAO_DIEN.md`, `reports/REVIEW_UI.md`, tệp này, và ảnh `ui-reference/*.png` + `instructions/system-ideas/Agent Hub – Giao diện chính.html`.
2. **Mọi nút, ô nhập, hộp thoại, menu thả xuống, tooltip, icon-button, thẻ, chip, tab đều dùng thành phần chung.** Cấm `<button>`, `<input>`, `<select>`, `<textarea>`, `position: fixed` tự viết và thuộc tính `title=` làm tooltip trong `features/**` và `core/**`; ngoại lệ phải viết rõ lý do trong báo cáo. UI-11 thêm kiểm tra tự động (xem bên dưới).
3. **Quy tắc nhãn nút:** chữ trong từ điển i18n **không** chứa ký hiệu trang trí (`+`, `×`, `›`, mũi tên...). Ký hiệu đi bằng icon của `Button` (`icon=`). Kiểm tra tự động bắt cả hai trường hợp: nhãn bắt đầu bằng ký hiệu, và nút có icon `Plus` mà nhãn đã có `+`.
4. Mọi thành phần mới có test (ARIA, bàn phím, Escape, bấm ra ngoài), phá thử bằng mutation (đỏ rồi hoàn lại).
5. **Chủ dự án và Claude sẽ tự chụp ảnh để kiểm**; ảnh của Antigravity chỉ là tham khảo. Báo cáo không được ghi "Khớp hoàn toàn" nếu ảnh không khớp; phần khác biệt phải liệt kê thật (bài học của UI-07).
6. Mọi test trình duyệt dùng `scripts/ui-shots.mjs` (CDP, không thư viện ngoài). Thêm chế độ đo **CLS** (PerformanceObserver `layout-shift`) cho UI-13.
7. **Dung lượng ổ:** trước khi biên dịch ở bản sao sạch phải còn ≥ 40 GB trống (nếu không thì dừng task, ghi `BLOCKED` vì đầy ổ). Mỗi lần chạy `verify` xong **xóa** `D:\dandori-qc` và `D:\dandori-target-ui` (đúng hai đường dẫn đó, `Remove-Item -LiteralPath`, không ký tự đại diện); không bao giờ xóa `D:\19-dandori\target`. Dùng MỘT bản sao tái sử dụng (chép đè tệp mã, giữ `target`) thay vì tạo bản mới mỗi task.
8. Không xóa/đọc/sửa `.env` của chủ dự án; không dùng cổng 3000 (cổng mặc định của Dandori là 3100; Vite 5173).
9. Các luật chung P1 đến P3 (không git ghi, không tắt máy, i18n, token, bảo mật hiển thị, không thư viện ngoài, trạng thái chuẩn, không "100%").

---

## UI-09. Bộ thành phần dùng chung và trang xem thử

- **Chạm vào:** `frontend/app/src/ui/components/**`, `ui/tokens.css`, `core/i18n.ts`, trang dev `/ui-kit` (chỉ hiện khi chạy `pnpm dev`, không vào bản dựng).
- **Việc phải làm:** hoàn thiện và chuẩn hóa bộ thành phần, mỗi cái một tệp, có `README` ngắn (props, ví dụ) trong `ui/components/README.md`:
  `Button` (primary/secondary/ghost/danger/outline, cỡ md/sm, `icon`, `loading`), `IconButton` (chỉ icon, **bắt buộc** `aria-label`), `Input`, `Textarea`, `Select` (kèm nhãn, lỗi, gợi ý, bám token), `Checkbox`, `Switch` (có sẵn), `Modal` (xem UI-10), `Dropdown`/`Popover` + hook `useDismiss` (xem UI-10), `Tooltip` (thay thuộc tính `title`), `Tabs` (ARIA), `Card`, `Chip`/`Badge`, `Spinner`, `Skeleton`, `PageHeader`, `FormField`, `EmptyState` (có sẵn), `Avatar` (có sẵn).
  Thêm token còn thiếu (`--radius-4/6` đã có), không viết màu cứng.
- **Nghiệm thu:** mỗi thành phần có test (render, ARIA, bàn phím, trạng thái disabled/loading/error); trang `/ui-kit` xem được tất cả ở chế độ sáng và tối; ảnh `UI-09-ui-kit-*.png`; `check-tokens` và `check-i18n` xanh; `verify` xanh.

## UI-10. Hộp thoại (Modal) và menu thả xuống (Dropdown) dùng chung, thay toàn bộ

- **Việc phải làm:**
  1. `Modal`: phủ toàn màn hình (`position: fixed`, `z-index` theo token), nền mờ, **đóng bằng Escape và bấm nền** (có thể tắt riêng cho hộp thoại nhập liệu đang dở, ghi trong props), khóa cuộn nền, đưa tiêu điểm vào hộp và trả lại khi đóng (focus trap), `role="dialog"`, `aria-modal`, `aria-labelledby`.
  2. `Dropdown`/`Popover` + `useDismiss`: đóng khi bấm ra ngoài, Escape, mất tiêu điểm; điều hướng bằng phím mũi tên; đặt vị trí không tràn khỏi màn hình.
  3. **Thay toàn bộ** ~24 hộp thoại đang tự viết hoặc dùng class Tailwind `fixed inset-0` (prompt-library 3, tech-profile 2, skills-md 2, cost-tracking 2, shared-memory, reports, quota-tracking, quiet-hours, notifications, mobile, md-jobs, instruction-sync, calendar, approval-gate, `core/ProjectModals`, `core/DirectoryPicker`, `kanban/CreateTaskModal`, `worktree/WorktreeSection`) và `ConfirmDialog` để dùng `Modal`. Thay `ProjectSelector`, menu "Chuyển tới" của `KanbanCard`, `UserMenu` để dùng `Dropdown`.
  3b. **Rà soát lại toàn bộ** mọi chỗ "mở ra rồi không đóng được khi bấm ra ngoài" (tìm bằng `useState` mở/đóng, `position: absolute`, `role="menu"`, `<details>`), lập bảng danh sách trong báo cáo (tệp, đã sửa chưa).
- **Nghiệm thu:** test từng hộp thoại/dropdown (Escape, bấm nền, bấm ra ngoài, focus trap); kịch bản trình duyệt thật: mở hộp "Tạo prompt" thấy nền mờ phủ cả màn hình (`position: fixed` đo được), bấm ra ngoài đóng được; mở ô chọn dự án rồi bấm ra ngoài thì đóng. Ảnh `UI-10-*.png`. Không còn `fixed inset-0` thô trong `features/**` (kiểm tự động).

## UI-11. Nút, icon-button, ô nhập dùng chung; bỏ "++"

- **Việc phải làm:**
  1. Thay **175** `<button>`, **70** `<input>`, **22** `<select>`, **8** `<textarea>` thô bằng `Button`/`IconButton`/`Input`/`Select`/`Textarea`/`Checkbox`. Tooltip dùng `Tooltip`, không dùng `title=`.
  2. **Bỏ "++":** sửa `btn_create_task`, `prompt_btn_create`, `tech_profile_btn_add`, `btn_add_account`, `api_key_fallback_add_btn` (bỏ `+` khỏi chữ, để icon `Plus` của `Button`). Rà soát toàn bộ i18n: chữ nhãn nút không chứa ký hiệu trang trí.
  3. Thêm kiểm tra tự động vào `scripts/check-i18n.mjs` (hoặc script mới, được thêm vào `verify`): (a) nhãn i18n bắt đầu bằng `+`, `×`, `›`, `→`; (b) `<button>`, `<input>`, `<select>`, `<textarea>` thô trong `features/**` và `core/**` (có danh sách ngoại lệ ghi lý do); (c) thuộc tính `title=` trên phần tử tương tác. Test công cụ chứng minh bắt được từng loại.
- **Nghiệm thu:** `Button` + icon cho mọi nút "thêm"; trình duyệt thật: Board "Giao task", Prompt "Tạo prompt", Hồ sơ kỹ thuật "Thêm mục" chỉ có MỘT dấu cộng (kiểm tự động bằng cách đếm `svg` icon Plus và ký tự `+` trong nhãn); số `<button>` thô trong `features/**` bằng 0 (hoặc nằm trong danh sách ngoại lệ có lý do); ảnh `UI-11-*.png`.

## UI-12. Gỡ class Tailwind, dùng thành phần dùng chung và token

- **Việc phải làm:** chuyển **351** chỗ `className` kiểu Tailwind (cost-tracking, live-status, calendar, shared-memory, prompt-library, approval-gate, tech-profile, skills-md, ...) sang `Card`, `PageHeader`, `FormField`, `Table`, token và bố cục dùng chung; không dùng `style={{ }}` dài dòng lặp lại (đưa vào thành phần). Sau khi xong: **gỡ `tailwindcss` khỏi `package.json`**, cập nhật `pnpm-lock.yaml`, `pnpm install --frozen-lockfile` và `verify` xanh. Thêm kiểm tra tự động: không còn `className` chứa các tiền tố Tailwind phổ biến (`space-y-`, `p-[0-9]`, `bg-`, `text-(xs|sm|base)`, `flex `, `grid-cols-`, `rounded` ...) trong `features/**`.
- **Nghiệm thu:** ảnh trước/sau cho mỗi module chạm vào; không lỗi console; `verify` xanh.

## UI-13. Chuyển trang mượt, không giật

- **Mục tiêu đo được:** CLS của mọi lần chuyển giữa 7 màn **< 0,1** (hiện: Tích hợp → Cài đặt = 0,51).
- **Việc phải làm:** (1) `Skeleton` có chiều cao giữ chỗ cho danh sách Cài đặt (34 hàng) và mọi vùng tải bất đồng bộ; (2) `Slot` hiện khung chờ đúng kích thước thay vì trống rồi nhảy; nạp trước (preload) các màn khi rê chuột lên mục menu; (3) giữ `scrollbar-gutter: stable` (đã có); (4) không để màn "Tính năng này đang tắt" nhấp nháy trước khi biết trạng thái module (dùng `loading` của `FeaturesProvider` để hiện khung chờ); (5) không nạp lại dữ liệu khi chỉ đổi đối tượng cùng nội dung (`useFeaturesLoader` phụ thuộc vào đối tượng `currentWorkspace/currentProject`, nên đổi sang phụ thuộc `id`).
- **Nghiệm thu:** chế độ `ui-shots --cls` đo 6 lần chuyển (mẫu: Claude đã có kịch bản `jank_probe`, bạn viết bản riêng), in bảng CLS từng lần chuyển, tất cả < 0,1, không có lần nào đổi chiều rộng nội dung; báo cáo kèm bảng số đo trước/sau.

## UI-14 đến UI-16. Làm đẹp từng màn theo mockup (chỉ dùng bộ thành phần chung)

Mỗi task: dựng lại giao diện màn theo ảnh mockup tương ứng, **chỉ dùng** thành phần ở UI-09; dữ liệu thật; trạng thái trống/lỗi/đang tải; chạy tốt ở 1440 px và 390 px; không lỗi console.

| Task | Màn | Mockup | Phải có |
| :--- | :--- | :--- | :--- |
| UI-14 | Agent và tài khoản | `02-agent-tai-khoan.png` | Khối "Vai trò mặc định" (4 ô chọn nhà cung cấp, không phải chuỗi "1. claude ↑↓"), lưới thẻ tài khoản 3 cột (mã chữ cái, chip trạng thái, thanh 5 giờ/tuần, chip ưu tiên), hộp thông tin auto-switch, khối API key dự phòng |
| UI-15 | Dashboard và Lịch | `03-dashboard.png`, `04-lich.png` | Dashboard: 4 thẻ KPI, bảng lịch sử chạy, khối "Cần bạn", "Agent đang làm việc" (sửa trạng thái "Mất kết nối" thường trực: tìm nguyên nhân luồng `/api/v1/m/live-status/stream`). Lịch: lưới tuần thật (7 cột, hàng giờ), thanh khung giờ yên tĩnh, danh sách hàng đợi qua đêm |
| UI-16 | Prompt và tri thức, Tích hợp | `05-prompt-tri-thuc.png`, `06-tich-hop.png` | Thẻ prompt, tab, hồ sơ kỹ thuật, kỹ năng, đồng bộ hướng dẫn, bộ nhớ chung; Tích hợp: thẻ kết nối GitHub/GitLab/Linear, thông báo, thiết bị di động |

Ngoài ra trong UI-14 đến UI-16: sửa luồng "tìm lần chạy của task" (`TaskLiveRunSection` gọi `GET /api/v1/runs?task_id=` nhưng máy chủ **không có** route này; thêm route đọc có kiểm quyền theo workspace + test 3 vai trò, hoặc dùng route có sẵn), và Board: thẻ task có vòng chữ nhà cung cấp, thanh tiến độ, chip hạn mức theo mockup, cột "Sẵn sàng merge" không bị cắt.

- **Nghiệm thu mỗi task:** ảnh 1440 px và 390 px, bảng "khác biệt còn lại" trung thực; Claude tự chụp lại và đối chiếu mockup (không dùng ảnh của Antigravity); `verify` xanh.

## UI-17. Cổng đợt thành phần dùng chung

- `TONG_KET_UI2.md`: trạng thái từng task (từ chuẩn), `git status --porcelain` nguyên văn, đầu ra `verify` nguyên văn trên bản sao sạch, bảng CLS trước/sau, danh sách khác biệt còn lại so với mockup, danh sách chưa kiểm chứng, dung lượng trống ổ D trước/sau. Dọn đúng hai thư mục tạm (xem luật 7). **Dừng, chờ Claude kiểm bằng trình duyệt thật.** Không bắt đầu P4.
