# Module `mod-eta`

Module dự tính thời gian hoàn thành tác vụ (ETA) theo phương pháp thống kê mô tả phi tham số minh bạch, trung thực và tự đo lường sai số.

## Chức năng chính
- Lấy mẫu thời lượng từ các lần chạy hoàn tất trong lịch sử có cùng `(project_id, role, provider)`.
- Tính toán khoảng dự kiến `[Median, P80]`.
- Yêu cầu tối thiểu $N \ge 5$ mẫu; ít hơn sẽ trả về "chưa đủ dữ liệu", tuyệt đối không bịa số liệu.
- Lưu lại mọi dự đoán và tự động đối soát với thời lượng thực tế khi tác vụ hoàn thành để tính sai số tuyệt đối (MAE).
- Tự động hạ bậc độ tin cậy hiển thị nếu sai số gần đây vượt quá 50% thời lượng trung vị.
