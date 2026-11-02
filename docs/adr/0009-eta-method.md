# ADR 0009: Phương pháp Dự tính Thời gian Hoàn thành (ETA) Đơn giản, Minh bạch và Tự đo Sai số

- **Trạng thái:** Đã duyệt (Accepted)
- **Ngày:** 2026-11-02
- **Tác giả:** Autonomous Engineering Team (Leader, Dev, QC1, QC2)
- **Phạm vi:** Task P3-07 (`crates/mod-eta`, `frontend/app/src/features/eta`)

---

## 1. Ngữ cảnh & Vấn đề

Khi một tác vụ (task) được giao việc cho Worker Agent thực thi, người dùng cần biết khoảng thời gian dự kiến tác vụ sẽ hoàn tất (Estimated Time of Arrival - ETA) để sắp xếp công việc và theo dõi tiến độ.

Tuy nhiên:
1. Bản chất của các mô hình ngôn ngữ lớn (LLM) và các công cụ agent là có tính bất định cao (non-deterministic). Thời lượng thực thi phụ thuộc vào độ phức tạp của bài toán, số lần gọi tool, phản hồi từ terminal, và độ trễ mạng của nhà cung cấp LLM.
2. Không có mô hình máy học (ML) hay mã nguồn mở nào chứng minh được độ tin cậy cao cho bài toán dự tính thời gian của tác tử lập trình tự động trên máy cục bộ.
3. Việc đưa ra con số ước lượng duy nhất (point estimate) dễ gây ảo tưởng về tính chính xác tuyệt đối, dẫn đến thất vọng khi tác vụ chạy lâu hơn cam kết.
4. Dự đoán khi chưa có đủ dữ liệu lịch sử là hành vi bịa đặt số liệu (vi phạm nguyên tắc cốt lõi A-03 của Dandori).

---

## 2. Quyết định Thiết kế

Chúng tôi chọn giải pháp **thống kê mô tả phi tham số (Non-parametric descriptive statistics)** đơn giản, minh bạch và có thể giải thích được:

### 2.1 Tiêu chí lọc mẫu lịch sử
Chỉ lấy các lần chạy **đã hoàn tất thành công** (`status = 'completed'` hoặc `'done'`) có cùng bộ 3 thuộc tính:
- Cùng dự án (`project_id`).
- Cùng vai trò (`role`, ví dụ: `worker`, `lead`, `reviewer`).
- Cùng nhà cung cấp agent (`provider`, ví dụ: `claude`, `agy`, `codex`).

Các lần chạy bị hủy (`cancelled`), lỗi (`failed`), hoặc bị timeout **tuyệt đối không được tính vào mẫu**, vì thời lượng của chúng không phản ánh thời gian hoàn thành công việc bình thường.

### 2.2 Ngưỡng kích hoạt tối thiểu (Sample Threshold)
- Chỉ hiển thị ước lượng ETA khi có **ít nhất $N$ mẫu** hợp lệ ($N \ge 5$).
- Khi có ít hơn $N$ mẫu ($N < 5$), hệ thống trả về `has_estimate = false` kèm thông điệp rõ ràng: `"chưa đủ dữ liệu (cần tối thiểu 5 lần chạy hoàn tất)"`. Tuyệt đối không đưa ra bất kỳ con số ước lượng nào.

### 2.3 Khoảng thời gian thay vì một con số đơn lẻ
ETA không được biểu diễn bằng một mốc giờ cố định duy nhất mà luôn ở dạng **khoảng thời gian (range)**:
$$\text{ETA} = \text{start\_time} + [\text{Median}, P_{80}]$$
- **Trung vị (Median / $P_{50}$):** Đại diện cho trường hợp phổ biến thông thường, ít bị ảnh hưởng bởi các giá trị ngoại lai (outliers).
- **Phân vị 80 ($P_{80}$):** 80% các tác vụ tương tự trong quá khứ hoàn tất trong khoảng thời gian này.
- Hiển thị người dùng: ví dụ *"Dự kiến 3 - 5 phút (khoảng 10:45 - 10:47)"*.

### 2.4 Cấp độ tin cậy (Confidence Level)
Độ tin cậy được xếp hạng dựa trên cỡ mẫu và độ phân tán của dữ liệu:
- **Thấp (Low):** 5 đến 9 mẫu, hoặc độ phân tán cao ($P_{80} / \text{Median} > 2.5$).
- **Vừa (Medium):** 10 đến 19 mẫu và độ phân tán ở mức vừa phải.
- **Cao (High):** $\ge 20$ mẫu và dữ liệu có độ lệch chuẩn/phân tán thấp.

### 2.5 Cơ chế Tự đo Sai số (Self-Measurement of Accuracy)
Hệ thống lưu lại mọi dự đoán ETA đã đưa ra vào bảng `m_eta_prediction`:
- Khi task kết thúc thực tế, ghi nhận `actual_duration_secs` và tính sai số tuyệt đối:
  $$\text{Absolute Error} = |\text{actual\_duration\_secs} - \text{predicted\_median\_secs}|$$
- Tính toán sai số trung vị tuyệt đối (Median Absolute Error - MAE) trên 20 lần chạy gần nhất.
- **Hạ cấp độ tin cậy tự động:** Nếu MAE gần đây vượt quá 50% thời lượng trung vị thực tế, hệ thống tự động hạ một bậc độ tin cậy hiển thị (ví dụ từ Cao xuống Vừa, từ Vừa xuống Thấp) kèm cảnh báo giải thích cho người dùng.

### 2.6 Tích hợp lỏng lẻo qua Sự kiện (Decoupled Event-Driven Integration)
- Module `eta` không gọi trực tiếp các module khác (`live-status`, `kanban`).
- Thay vào đó, `mod-eta` lắng nghe sự kiện `RunStarted` và `RunFinished` phát ra từ `EventBus` lõi.
- Giao diện người dùng (`frontend/app/src/features/eta/`) cung cấp hook `useTaskEta(taskId)` hoặc slot đóng góp hiển thị:
  - Khi module `eta` được bật: hiển thị badge hoặc chip ETA tại bảng Kanban và Live Status.
  - Khi module `eta` bị tắt: hiển thị ký hiệu `" — "` hoặc ẩn hoàn toàn, không ném lỗi hay phá vỡ giao diện.

---

## 3. Hệ quả & Rủi ro

### Ưu điểm:
- 100% giải thích được, minh bạch với người dùng.
- Không tiêu tốn tài nguyên GPU, không gọi API bên thứ ba, không tốn chi phí token.
- Ngăn ngừa tình trạng người dùng tin tưởng tuyệt đối vào một mốc giờ cố định.
- Cơ chế tự đo sai số giúp hệ thống phản ánh trung thực năng lực dự đoán của chính mình.

### Hạn chế:
- Với các task hoàn toàn mới (lần đầu thực hiện trên repo hoặc công nghệ mới), hệ thống sẽ không có ETA cho đến khi tích lũy đủ 5 lần chạy. Đây là sự đánh đổi có chủ đích để giữ vững nguyên tắc trung thực số liệu.
