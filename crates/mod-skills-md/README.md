# mod-skills-md

Module quản lý kỹ năng Agent theo chuẩn `SKILL.md` (P3-08).

## Mục tiêu
- Nhập kỹ năng từ thư mục cục bộ hoặc tệp nén zip với cơ chế bảo vệ zip-slip và chống bom giải nén.
- Nhận diện các tệp script trong kỹ năng (`scripts/**`, `.sh`, `.py`, v.v.), tuyệt đối không thực thi script.
- Gắn kỹ năng với dự án và vai trò agent (`worker`, `lead`, v.v.).
- Đóng góp vào prompt của agent (`PromptContributor`) theo phương thức tiết kiệm: mặc định chỉ danh sách tên và mô tả, chỉ nhúng toàn bộ khi được cấu hình.
- Quản lý trạng thái phê duyệt: khi nội dung hoặc băm tệp thay đổi, tự động chuyển về `pending_approval`.
