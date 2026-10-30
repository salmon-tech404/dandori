# Mẫu một dòng trong bảng công nghệ

<!-- Thêm vào docs/cong-nghe/README.md. Một công nghệ chỉ cần dòng này; viết thêm một file riêng khi giải thích dài. -->

| Công nghệ | Để làm gì | Phiên bản | Trạng thái | Vì sao chọn | Đã loại / không chọn |
| :--- | :--- | :--- | :--- | :--- | :--- |
| <tên> | <một câu: giúp dự án làm được việc gì> | <dán từ lệnh --version> | Đã kiểm / Dự kiến / Chưa cài | <lý do, liên kết ADR nếu có> | <phương án khác và lý do bỏ> |

## Khuôn giải thích một công cụ (dùng trong moi-truong/cai-dat.md)

Mỗi công cụ phải trả lời đủ năm câu bằng lời thường:

1. **Là gì?** Nói bằng một ví von (ví dụ "nhân viên kiểm hàng nhập kho").
2. **Cài những gì?** Liệt kê file hoặc thành phần thực sự được cài.
3. **Giúp gì cho dự án?** Gắn với một task hoặc rule cụ thể, không nói chung chung.
4. **Nếu không cài thì sao?** Điều gì hỏng hoặc không làm được.
5. **Dùng thế nào?** Một lệnh ví dụ.

Điều nào chưa kiểm chứng thì ghi `[kiến thức chung]`.

## Danh sách kiểm trước khi thêm một công nghệ mới

1. Nó giải quyết vấn đề cụ thể nào? Nếu chưa có vấn đề thì chưa cần.
2. Có cách đơn giản hơn (đã có sẵn trong công cụ khác) không?
3. Nó có còn được bảo trì không? (kiểm lần phát hành gần nhất, và cảnh báo bảo mật nếu có)
4. Giấy phép của nó có phù hợp không?
5. Gỡ nó đi sau này có khó không?
