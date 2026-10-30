# Sổ tay dự án Dandori

Nơi ghi lại **đã làm gì, dùng công nghệ gì, vì sao chọn cái này mà không chọn cái kia**, để nhìn lại sau vài tháng vẫn hiểu.

> Phân biệt với hai thư mục khác:
> - `instructions/` là **kế hoạch và thiết kế** (do Claude viết, chỉ đọc).
> - `instructions/implementations/reports/` là **báo cáo của agent** sau mỗi task (bằng chứng để kiểm tra, không phải sổ tay).
> - `docs/` (thư mục này) là **sổ tay của bạn**: ngắn, theo thời gian, viết cho chính bạn đọc lại.

## Năm loại ghi chú

| Bạn muốn nhớ... | Ghi ở đâu | Ví dụ |
| :--- | :--- | :--- |
| **Hôm nay đã làm gì** | `nhat-ky/AAAA-MM-DD.md` | "Cài Rust, tạo repo git, kiểm môi trường" |
| **Dự án dùng công nghệ gì, phiên bản nào, vì sao chọn** | `cong-nghe/README.md` (bảng) | "Rust 1.99.1, chọn vì giống Vibe Kanban; đã loại nightly" |
| **Một lựa chọn lớn giữa nhiều phương án** | `adr/NNNN-ten.md` (ADR) | "SSE thay vì WebSocket vì..." |
| **Cài đặt máy và ý nghĩa của từng lệnh** | `moi-truong/cai-dat.md` và `moi-truong/lenh-hay-dung.md` | `cargo --version` nghĩa là gì |
| **Một lỗi đã gặp và cách sửa** | `su-co/AAAA-MM-DD-ten-loi.md` | "cargo không được nhận ra" |

Từ khó ở `thuat-ngu.md`. Mẫu để sao chép nằm ở `_mau/`.

## Khi nào ghi gì (quy tắc ngón tay cái)

1. **Cài hoặc đổi một công cụ** → thêm một dòng vào bảng `cong-nghe/` và một bước vào `moi-truong/cai-dat.md`.
2. **Phải chọn giữa hai hay nhiều cách** → viết một ADR (xem `_mau/mau-quyet-dinh.md`). Phần quan trọng nhất của ADR là mục *"Đã cân nhắc và không chọn"*, đó chính là câu trả lời cho "vì sao cài cái này mà không cài cái kia".
3. **Gặp lỗi mất hơn 10 phút để sửa** → viết một ghi chú sự cố. Lần sau gặp lại bạn tra là xong.
4. **Gõ một lệnh mà chưa hiểu nó làm gì** → thêm vào `lenh-hay-dung.md` kèm một câu giải thích.
5. **Cuối buổi làm việc** → 5 dòng vào `nhat-ky/` của ngày đó.

## Năm quy tắc để sổ tay không biến thành đống chữ vô dụng

1. **Ghi ngày thật** (lấy từ máy, không đoán). Công nghệ thay đổi nhanh, ghi chú không có ngày thì không biết còn đúng không.
2. **Dán nguyên văn đầu ra của lệnh** khi ghi phiên bản hoặc kết quả. Đừng gõ lại bằng trí nhớ.
3. **Mỗi công cụ trả lời đủ năm câu:** là gì (có ví von), cài những gì, giúp gì cho dự án (gắn với một task hay rule cụ thể), nếu không cài thì sao, dùng thế nào. Một công cụ không giải thích được vì sao cần thì có lẽ chưa cần. Khuôn đầy đủ ở `_mau/mau-cong-nghe.md`; ví dụ mẫu ở `moi-truong/cai-dat.md`.
4. **Đánh dấu rõ trạng thái:** `Đã kiểm` (đã chạy và thấy kết quả), `Dự kiến` (kế hoạch, chưa cài), `Chưa kiểm` (nghe nói, chưa thử).
5. **Tuyệt đối không ghi bí mật:** mật khẩu, khóa API, token, nội dung thư mục đăng nhập của CLI. Cần nhắc tới thì ghi `[đã che]`.

Ngoài ra: **liên kết thay vì chép lại**. Nếu một nội dung đã nằm ở `instructions/` hoặc ADR thì trỏ tới đó.

## Cách tìm lại

| Câu hỏi | Mở |
| :--- | :--- |
| Dự án đang dùng công nghệ gì? | `cong-nghe/README.md` |
| Vì sao chọn X mà không chọn Y? | `cong-nghe/README.md` (cột "Đã loại"), rồi `adr/` nếu cần chi tiết |
| Tuần trước mình đã làm gì? | `nhat-ky/` |
| Lỗi này mình gặp rồi thì sửa sao? | `su-co/` |
| Cài lại máy thì làm gì? | `moi-truong/cai-dat.md` |
| Lệnh này nghĩa là gì? | `moi-truong/lenh-hay-dung.md` |
| Từ này là gì? | `thuat-ngu.md` |

## Mục lục hiện có

- `cong-nghe/README.md` · bảng công nghệ
- `moi-truong/cai-dat.md` · nhật ký cài đặt máy
- `moi-truong/lenh-hay-dung.md` · sổ lệnh
- `nhat-ky/2026-10-30.md` · nhật ký ngày đầu
- `su-co/2026-10-30-cargo-khong-nhan-ra.md` · sự cố đầu tiên
- `thuat-ngu.md` · từ điển
- `_mau/` · bốn mẫu để sao chép
- `adr/` · *chưa có; task T01 sẽ tạo thư mục này cùng ba ADR đầu tiên*
