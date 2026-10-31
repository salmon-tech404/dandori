# Frontend của Dandori

Thư mục `frontend/` là không gian làm việc (pnpm workspace) chứa ứng dụng web giao diện người dùng của Dandori cùng các cấu hình tooling liên quan.

## Cách chạy

Từ thư mục gốc dự án:

- **Cài đặt thư viện:**
  ```powershell
  pnpm install
  ```
- **Kiểm tra khóa thư viện:**
  ```powershell
  pnpm install --frozen-lockfile
  ```
- **Chạy môi trường phát triển (Dev server):**
  ```powershell
  pnpm --filter app dev
  ```
- **Xây dựng bản phát hành (Production build):**
  ```powershell
  pnpm --filter app build
  ```
- **Xem trước bản dựng (Preview):**
  ```powershell
  pnpm --filter app preview
  ```

## Cấu trúc thư mục ứng dụng (`frontend/app/`)

Ứng dụng web dùng Vite + React + TypeScript. Dưới đây là cấu trúc **đích** (ADR 0020); mục đánh dấu *(chưa có)* sẽ được tạo ở task ghi bên cạnh.

```
frontend/
├── app/
│   ├── src/
│   │   ├── main.tsx      # điểm nhập ứng dụng React
│   │   ├── App.tsx       # thành phần gốc (hiện là trang chào tạm)
│   │   ├── index.css     # CSS gốc toàn trang
│   │   ├── ui/           # tokens.css (bộ token tạm) và thành phần dùng chung
│   │   ├── core/         # (chưa có, T12) khung ứng dụng, client API, vùng chứa ô cắm
│   │   ├── features/     # (chưa có, T12 trở đi) mỗi module một thư mục con
│   │   └── i18n/         # (chưa có, T12) chuỗi tiếng Việt
│   ├── e2e/              # (chưa có, T13) kịch bản Playwright
│   ├── public/           # (chưa có) tài nguyên tĩnh, phông tự chứa
│   ├── index.html        # khung HTML gốc
│   ├── package.json      # phụ thuộc của ứng dụng
│   ├── tsconfig.json     # cấu hình TypeScript
│   └── vite.config.ts    # cấu hình Vite
└── tooling/              # (chưa có, T02) bản sao cấu hình ESLint của Kantanda
```

## Quy chuẩn giao diện

- **Design Tokens:** Sử dụng biến CSS tại `src/ui/tokens.css` (hiện là bộ token **tạm**; T12 thay bằng bộ ở `04_GIAO_DIEN.md` mục 5). Tuyệt đối không dùng mã màu hex (`#...`) hay kích thước `rem` gán cứng trực tiếp trong component.
- **Tuân thủ Kantanda:** Phong cách thiết kế, kích thước mục bấm (tối thiểu 44px), màu sắc và phản hồi đều tham chiếu theo hồ sơ `standards/profile.md` và tài liệu `instructions/implementations/04_GIAO_DIEN.md`.
