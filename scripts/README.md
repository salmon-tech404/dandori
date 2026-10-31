# Thư mục Scripts của Dandori

Thư mục `scripts/` chứa các kịch bản thực thi tự động (viết bằng Node.js `.mjs` chạy đa nền tảng Windows và Linux) phục vụ cổng xác minh chất lượng, kiểm tra kiến trúc và sinh mã nguồn.

## Các kịch bản chính

- `verify.mjs`: Cổng kiểm tra chất lượng duy nhất của dự án (`pnpm run verify`), thực thi tuần tự và dừng ngay khi gặp lỗi đầu tiên.
- `check-architecture.mjs`: Kiểm tra luật phụ thuộc giữa các crate, quy tắc hexagonal, danh mục thư viện được duyệt và tên crate cấm.
- `check-lints.mjs`: Kiểm tra cấu hình lints tập trung, cấm `#![allow]`, cấm panic abort và kiểm tra MSRV.
- `check-sql.mjs`: Chặn `format!` và ghép chuỗi trong lời gọi truy vấn SQLx (phòng chống SQL injection).
- `scan-secrets.mjs`: Quét mẫu khóa bí mật, token và private key trên toàn bộ tệp được git theo dõi.
- `gen-modules.mjs`: Thu thập cấu hình `module.json` và đồng bộ crate cầu nối `dandori-modules`.

## Kiểm thử các kịch bản

Mỗi script đều có bài kiểm thử tương ứng (cả ca thành công và ca vi phạm thất bại) trong thư mục `tests/tooling/`, được thực thi bằng:

```powershell
pnpm run test:tooling
```
