# ADR 0007: Cơ chế Tự động Hợp nhất Pull Request (`pr-merge`) và Biện pháp Phòng vệ [CỔNG CHỦ]

- **Trạng thái:** Đang chờ chủ dự án phê duyệt (Pending Owner Approval)
- **Ngày:** 2026-11-01
- **Tác giả:** Dandori Team (theo yêu cầu Task P2-07 & 12_P2_VONG_LAM_VIEC.md)

---

## 1. Bối cảnh (Context)

Trong chu trình làm việc tự động hóa nâng cao của Dandori (Phase 2), sau khi agent hoàn thành công việc và vượt qua các cổng kiểm định chất lượng:
1. Nhiệm vụ chuyển sang trạng thái `ready_to_merge`.
2. Module `integrations` tạo Pull Request (PR) hoặc Merge Request (MR) trên nền tảng từ xa (GitHub / GitLab).
3. Hệ thống CI/CD trên remote chạy kiểm thử và trả về kết quả trạng thái.
4. Nhu cầu tự động hợp nhất mã nguồn (Auto-merge) vào nhánh mặc định (`main` / `master`) giúp tăng tốc độ phân phối và giảm thiểu thao tác thủ công lặp đi lặp lại.

Tuy nhiên, hành động hợp nhất mã nguồn tự động vào nhánh chính của sản phẩm tiềm ẩn các rủi ro an ninh và kỹ thuật rất lớn nếu không có sự kiểm soát chặt chẽ.

---

## 2. Mô hình Đe dọa An ninh (Threat Model)

1. **Hiểm họa Đưa mã độc vào nhánh chính (Malicious Code Infiltration):**
   Nếu mã nguồn do AI sinh ra chứa lỗ hổng bảo mật tiềm ẩn, backdoor hoặc bị thao túng qua Prompt Injection (từ tài liệu mở rộng hoặc issue bên ngoài), việc tự động merge mà không qua con người soát xét có thể trực tiếp làm thỏa hiệp toàn bộ ứng dụng production.
2. **Ca đua điều kiện kiểm tra CI (CI Race Condition & Stale Status):**
   Trạng thái CI có thể thay đổi hoặc commit mới có thể được đẩy vào nhánh giữa thời điểm Dandori kiểm tra trạng thái CI và thời điểm lệnh merge được thực thi.
3. **Phá vỡ Quy tắc Bảo vệ Nhánh (Branch Protection Bypass):**
   Nguy cơ cố gắng hợp nhất khi nhánh đích yêu cầu số lượng review tối thiểu từ con người hoặc yêu cầu chữ ký commit (signed commits).
4. **Vòng lặp Hợp nhất và Xung đột Mã nguồn (Merge Conflicts & Infinite Loops):**
   Khi có xung đột mã nguồn (merge conflict) hoặc lỗi mạng, nếu hệ thống tự động retry vô hạn sẽ gây tắc nghẽn tài nguyên và làm sai lệch lịch sử git.

---

## 3. Quyết định Kiến trúc & Biện pháp Phòng vệ (Mitigations)

### 3.1. Mặc định TẮT và Cổng Phê duyệt của Chủ Dự án ([CỔNG CHỦ])
- Thuộc tính `autoMerge` bắt buộc **MẶC ĐỊNH TẮT** (`false`) trên toàn hệ thống và trong cấu hình của từng dự án.
- Để kích hoạt tính năng tự động merge, chủ dự án phải phê duyệt tường minh ADR này và xác nhận rủi ro trên giao diện cài đặt tính năng.
- Không có bất kỳ chế độ ngầm định nào được tự động merge khi chủ dự án chưa bật.

### 3.2. Điều kiện Hợp nhất Nghiêm ngặt (Strict Merge Prerequisites)
Một Pull Request chỉ được xem xét tự động merge khi và chỉ khi thỏa mãn **toàn bộ** các điều kiện sau:
1. `autoMerge` được bật ở cấp dự án và xác nhận rủi ro hợp lệ.
2. Trạng thái kiểm tra CI trên remote trả về `success` (xanh) từ `GitHostProvider`.
3. Nếu module `approval-gate` được bật: Phải có quyết định `approved` hợp lệ từ người dùng hoặc người có thẩm quyền.
4. Trạng thái nhiệm vụ trong Dandori đang là `ready_to_merge`.
5. Phương thức merge tuân thủ cấu hình dự án (`squash`, `rebase`, hoặc `merge_commit`).

### 3.3. Tái kiểm tra Điều kiện Ngay trước khi Hợp nhất (Just-In-Time Re-validation)
- Để ngăn chặn ca đua (race condition), hệ thống bắt buộc thực hiện kiểm tra lại (re-check) trạng thái CI và trạng thái duyệt tại mili-giây ngay trước khi gửi lệnh gọi API `GitHostProvider.merge_pull_request`.
- Nếu trạng thái CI hoặc nhánh đích đã thay đổi trong khoảng thời gian chờ, thao tác merge bị hủy ngay lập tức và ghi nhận cảnh báo.

### 3.4. Ranh giới Git An toàn (Safe Git Boundaries)
- Tuyệt đối **CẤM** sử dụng `--force` (`git push --force`).
- Tuyệt đối **CẤM** đẩy trực tiếp lên nhánh mặc định (`main`, `master`). Chỉ đẩy nhánh nhiệm vụ `task/*`.
- Không tự động xóa nhánh từ xa sau khi merge nếu chưa được cấu hình rõ ràng.

### 3.5. Cơ chế Ngắt Lỗi (Error Handling & Circuit Breaker)
- Nếu thao tác merge gặp lỗi (xung đột mã nguồn, máy chủ Git từ chối, lỗi mạng):
  - Tuyệt đối **KHÔNG** thử lại vô hạn (không retry loop).
  - Tự động chuyển trạng thái nhiệm vụ sang `needs_human` để con người can thiệp.
  - Phát sự kiện `EventPayload::NotificationRequested` thông báo lỗi chi tiết cho người dùng.

---

## 4. Trạng thái Thực thi trong Phase 2

Theo chỉ thị của đề bài Phase 2:
- Do ADR 0007 đang ở trạng thái **CHỜ CHỦ PHÊ DUYỆT**, Task P2-07 chưa được viết mã chức năng `pr-merge`.
- Trạng thái của Task P2-07 trong `00_TRANG_THAI.md` được đánh dấu là `⏳ SKIPPED_AWAITING_OWNER`.
- Sau khi chủ dự án xem xét, đối chiếu an ninh và phê duyệt ADR này, mã nguồn module `crates/mod-pr-merge` sẽ được triển khai độc lập theo đúng các nguyên tắc bảo vệ nêu trên.
