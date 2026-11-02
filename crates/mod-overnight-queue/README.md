# mod-overnight-queue

Module quản lý hàng đợi chạy tác vụ qua đêm (`Overnight Queue`) cho Dandori.

## Tính năng chính
- Hàng đợi có thứ tự gồm các task `backlog`.
- Giờ bắt đầu cấu hình được (mặc định 23:00), giới hạn song song tối đa N (mặc định 1).
- Chạy tuần tự theo thứ tự qua `TaskRunner` với nguồn `scheduler:overnight-queue` (tuân thủ `TransitionGuard` và khung giờ cấm P3-03).
- Dừng an toàn tự động: dừng sau K lần thất bại liên tiếp (mặc định 3) hoặc khi cạn hạn mức tài khoản.
- Tóm tắt sáng hôm sau: tạo bản tóm tắt danh sách PR/nhánh để duyệt ("Chuỗi để duyệt") và phát sự kiện thông báo `notification.requested`.
- Bền vững qua khởi động lại: tiếp tục xử lý các task còn lại trong hàng đợi, không lặp lại task đã xong.
- An toàn tuyệt đối: tuyệt đối không chứa hành vi tắt máy, ngủ hay khởi động lại máy tính.
