# mod-auto-lessons

Module tự động ghi nhận bài học sau mỗi task trong Dandori (P3-11).
- Khi task chuyển sang trạng thái `done` hoặc `failed`, tự động chạy một lần chạy ngắn vai PM để trích xuất bài học.
- Đưa bài học vào module `shared-memory` dưới dạng mục `lesson` ở trạng thái `pending` (không tự duyệt).
- Giới hạn: Tối đa một lần chạy bài học cho mỗi task (chống lặp). Bỏ qua task quá ngắn hoặc thiếu thông tin.
- Lọc và che toàn bộ bí mật (khóa riêng đa dòng và API tokens) trong bài học trước khi lưu trữ.
- Cho phép cấu hình bật/tắt theo dự án và cấu hình ngân sách token.
