# Bộ thành phần dùng chung (Shared UI Components)

Bộ thành phần giao diện chuẩn hóa của Dandori (theo `04_GIAO_DIEN.md` và `17_UI_THANH_PHAN_DUNG_CHUNG.md`).
Toàn bộ thành phần đều bám sát hệ thống Design Tokens (`ui/tokens.css`), hỗ trợ đầy đủ phím tắt, ARIA, khả năng truy cập (vùng chạm tối thiểu 44px) và chế độ sáng/tối.

---

## 1. Nút & Tương tác

### `Button`
Nút bấm chuẩn hỗ trợ 5 biến thể, 3 kích cỡ, trạng thái đang tải và biểu tượng kèm theo.
- **Props:**
  - `variant`: `"primary"` | `"secondary"` | `"outline"` | `"ghost"` | `"danger"` (mặc định: `"secondary"`)
  - `size`: `"md"` | `"sm"` | `"icon"` (mặc định: `"md"`)
  - `loading`: `boolean`
  - `icon`: `ReactNode`
  - Các thuộc tính chuẩn của thẻ `<button>`
- **Ví dụ:**
  ```tsx
  <Button variant="primary" icon={<PlusIcon />}>Tạo mới</Button>
  <Button variant="danger" loading={isDeleting}>Xóa</Button>
  ```

### `IconButton`
Nút chỉ chứa biểu tượng. **Bắt buộc** truyền `aria-label` để đảm bảo khả năng tiếp cận.
- **Props:**
  - `aria-label`: `string` (bắt buộc)
  - `icon`: `ReactNode`
  - `variant`: Biến thể tương tự `Button` (mặc định: `"ghost"`)
  - `size`: `"md"` | `"sm"`
  - `loading`, `disabled`
- **Ví dụ:**
  ```tsx
  <IconButton aria-label="Đóng hộp thoại" icon={<CloseIcon />} onClick={onClose} />
  ```

### `Switch`
Công tắc bật/tắt chuẩn theo thiết kế, bo góc hình chữ nhật (40×22px) với vùng chạm hiệu dụng 44px.
- **Props:** `checked`, `onChange`, `disabled`, `aria-label`

---

## 2. Nhập liệu & Biểu mẫu

### `Input`
Ô nhập dữ liệu dạng văn bản với hỗ trợ nhãn (`label`), thông báo lỗi (`error`), và gợi ý (`hint`).
- **Props:** `label`, `error`, `hint`, các thuộc tính chuẩn `<input>`

### `Textarea`
Ô nhập văn bản nhiều dòng, hỗ trợ kiểu chữ đơn cách (`mono`) cho mã nguồn/cấu hình.
- **Props:** `label`, `error`, `hint`, `mono`: `boolean`, các thuộc tính `<textarea>`

### `Select`
Hộp chọn thả xuống chuẩn hóa theo token thiết kế.
- **Props:** `label`, `error`, `hint`, `options`: `Array<{ value, label, disabled? }>`

### `Checkbox`
Hộp kiểm tùy biến với nhãn, kiểm soát trạng thái bật/tắt và vùng chạm chuẩn 44px.
- **Props:** `label`, `error`, `checked`, `onChange`, `disabled`

### `FormField`
Khung bao bọc biểu mẫu chuẩn kết hợp nhãn, chỉ thị bắt buộc (`required`), thành phần nhập liệu và phản hồi lỗi/gợi ý.
- **Props:** `label`, `htmlFor`, `error`, `hint`, `required`: `boolean`, `children`

---

## 3. Lớp phủ & Điều hướng

### `Modal`
Hộp thoại che phủ toàn màn hình (`position: fixed`, `z-index: 1000`), nền mờ, khóa cuộn trang, tự bẫy tiêu điểm (focus trap), hỗ trợ phím `Escape` và bấm nền để đóng.
- **Props:**
  - `open`: `boolean`
  - `onClose`: `() => void`
  - `title`: `ReactNode`
  - `footer`: `ReactNode`
  - `size`: `"sm"` | `"md"` | `"lg"` | `"full"`
  - `closeOnEscape`: `boolean` (mặc định: `true`)
  - `closeOnBackdrop`: `boolean` (mặc định: `true`)

### `Dropdown` & `useDismiss`
Trình đơn thả xuống / Popover tự đóng khi bấm ra ngoài hoặc nhấn phím `Escape`. Điều hướng bàn phím bằng phím mũi tên `ArrowUp` / `ArrowDown`.
- **Props:** `trigger`, `content`, `align`: `"left"` | `"right"`, `open`, `onOpenChange`

### `Tooltip`
Hiển thị chú giải nổi khi rê chuột hoặc lấy tiêu điểm bàn phím. Thay thế hoàn toàn thuộc tính `title` thô.
- **Props:** `content`: `string`, `position`: `"top"` | `"bottom"` | `"left"` | `"right"`, `children`

### `Tabs`
Thanh chuyển đổi tab tuân thủ chuẩn WAI-ARIA (`role="tablist"`, `role="tab"`), hỗ trợ điều hướng bằng phím mũi tên trái/phải.
- **Props:** `tabs`: `Array<{ id, label, badge?, disabled? }>`, `activeTab`: `string`, `onChange`

---

## 4. Bố cục & Trạng thái

### `Card`
Thẻ chứa nội dung với bo góc `--radius-10`, viền `--ln`, nền `--sf`, hỗ trợ tiêu đề, phụ đề và các nút thao tác.
- **Props:** `title`, `subtitle`, `actions`, `padding`: `"none"` | `"sm"` | `"md"` | `"lg"`, `selected`, `onClick`

### `Chip` & `Badge`
Nhãn trạng thái viên thuốc (pill), chiều cao 22px theo token, hỗ trợ 7 nhóm màu: `default`, `blue`, `orange`, `teal`, `green`, `red`, `gray`.
- **Props:** `variant`, `size`: `"sm"` | `"md"`, `onRemove`: `() => void`

### `Spinner`
Biểu tượng xoay đang tải dạng vector (SVG) với `role="status"` và nhãn cho trình đọc màn hình.
- **Props:** `size`: `"sm"` | `"md"` | `"lg"`, `color`, `label`

### `Skeleton`
Khung chờ tải dữ liệu với hiệu ứng chuyển động mờ nhạt (pulse animation).
- **Props:** `width`, `height`, `radius`: `"sm"` | `"md"` | `"lg"` | `"pill"` | `"none"`

### `PageHeader`
Tiêu đề đầu trang chuẩn hóa theo thiết kế với đường phân cách, huy hiệu và nhóm hành động.
- **Props:** `title`, `subtitle`, `badges`, `actions`, `breadcrumbs`

### `Avatar` & `EmptyState`
Thành phần hiển thị đại diện người dùng/agent và khung trạng thái rỗng khi chưa có dữ liệu.
