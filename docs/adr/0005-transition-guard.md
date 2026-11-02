# ADR 0005: Điểm mở rộng `TransitionGuard` và Kiểm soát Chuyển đổi Trạng thái

- **Trạng thái:** Đã chốt (Accepted)
- **Ngày:** 2026-11-01
- **Tác giả:** Dandori Team (theo yêu cầu P2-03 & 01_KIEN_TRUC.md mục 4.5)

## 1. Bối cảnh (Context)
Trong hệ thống Dandori, các tác vụ (`Task`) di chuyển qua nhiều trạng thái trong vòng đời làm việc (từ `backlog`, `todo`, `in_progress`, `in_review`, đến `ready_to_merge`, `done`, `cancelled`, hoặc `needs_human`).

Khi người dùng hoặc hệ thống tự động kích hoạt chuyển đổi trạng thái (ví dụ từ `in_review` sang `ready_to_merge`), có những chuyển đổi mang tính nhạy cảm hoặc rủi ro cao:
- Đưa mã nguồn vào nhánh merge mà chưa có sự kiểm duyệt của con người (cần cổng duyệt `approval-gate`).
- Đưa tác vụ vào chạy tự động ngoài giờ làm việc hoặc trong khung giờ cấm (`quiet-hours` trong P3).
- Chạy tác vụ khi ngân sách hoặc hạn mức sử dụng đã cạn kiệt.

Hệ thống cần một điểm mở rộng chuẩn hóa cho phép các module đăng ký can thiệp vào quá trình chuyển trạng thái của máy trạng thái Kanban lõi mà không làm suy giảm tính toàn vẹn và độ tin cậy của hệ thống.

## 2. Quyết định (Decision)

### 2.1 Điểm mở rộng `TransitionGuard`
Định nghĩa trait `TransitionGuard` trong `crates/dandori-module-api/src/extension.rs`:

```rust
pub trait TransitionGuard: fmt::Debug + Send + Sync {
    fn name(&self) -> &'static str;

    fn check(
        &self,
        task: &Task,
        from: TaskStatus,
        to: TaskStatus,
        actor: &TransitionActor,
    ) -> BoxFuture<'static, Result<GuardDecision, GuardError>>;
}
```

Trong đó:
- `TransitionActor`:
  - `User(String)`: Người dùng thao tác qua giao diện hoặc API (chứa `user_id`).
  - `System(SystemSource)`: Hệ thống tự động kích hoạt, với `SystemSource` là `Scheduler(String)` (`scheduler:<module-id>`) hoặc `Module(String)` (`module:<module-id>`). Phân định rõ tác nhân thực thi phục vụ chính sách duyệt và kịch bản P3-03.
- `GuardDecision`:
  - `Allow`: Cho phép chuyển đổi tiếp tục.
  - `Deny { reason: String, retry_after: Option<DateTime<Utc>> }`: Từ chối chuyển đổi ngay lập tức kèm lý do và thời gian có thể thử lại.
  - `NeedsApproval { kind: String, reason: String }`: Chuyển hướng tác vụ sang `needs_human`, phát sự kiện `ApprovalRequested` để con người phê duyệt.

### 2.2 Quy trình thực thi tại máy chủ Lõi (`dandori-server`)
Tại điểm thực thi chuyển trạng thái tác vụ (`execute_task_transition`):
1. Kiểm tra tính hợp lệ của bước chuyển theo máy trạng thái cơ sở (`current_status.can_transition_to(target_status)`). Nếu không hợp lệ, trả về lỗi 400.
2. Lấy danh sách các `TransitionGuard` đang hoạt động từ `FeatureLifecycleManager`.
3. Nếu không có guard nào đăng ký, tác vụ chuyển trạng thái bình thường như hành vi cũ.
4. Lần lượt gọi `guard.check(...)`:
   - Nếu bất kỳ guard nào trả về `Deny`: Dừng chuỗi đánh giá, từ chối yêu cầu chuyển đổi với mã lỗi và lý do tương ứng.
   - Nếu guard trả về `NeedsApproval`: Chuyển trạng thái tác vụ sang `needs_human`, phát sự kiện `ApprovalRequested` lên `EventBus`, lưu yêu cầu phê duyệt và trả về tác vụ ở trạng thái `needs_human`.
   - Nếu tất cả các guard đều trả về `Allow`: Tiến hành cập nhật trạng thái tác vụ sang `target_status` và phát sự kiện `TaskStatusChanged`.

## 3. Phân tích mối nguy và Biện pháp giảm thiểu (Threats & Mitigations)

| Mối nguy cơ | Tác động | Biện pháp giảm thiểu của Dandori |
| :--- | :--- | :--- |
| **Treo tác vụ vô hạn (Hanging Tasks)** | Guard không đưa ra quyết định hoặc chờ phản hồi từ con người quá lâu khiến tác vụ bị kẹt vĩnh viễn ở `needs_human`. | **Thời hạn hết hạn (Expiry):** Mọi yêu cầu phê duyệt đều có trường `expires_at`. Worker nền định kỳ quét các yêu cầu quá hạn, tự động đánh dấu `expired` và hoàn tác tác vụ về trạng thái trước đó (`from_status`). |
| **Vòng lặp chuyển trạng thái** | Khi guard từ chối hoặc yêu cầu duyệt, nếu hệ thống tự động cố thử lại ngay lập tức sẽ gây vòng lặp vô hạn (Infinite Transition Loop). | **Tác nhân `System` & `retry_after`:** Guard phân biệt rõ tác nhân `User` hay `System`. Các tác nhân tự động phải tôn trọng `retry_after`. Chuyển sang `needs_human` chỉ xảy ra 1 lần cho mỗi yêu cầu duyệt. |
| **Tự phê duyệt không hợp lệ (Self-Approval)** | Người tạo hoặc người yêu cầu chuyển trạng thái tự duyệt yêu cầu của chính mình để vượt qua kiểm soát. | **Chính sách cấm tự duyệt (`disallow_self_approval`):** Khi chính sách này bật, API từ chối với HTTP 400 nếu `decider_id == requester_id`. |
| **Tranh chấp điều kiện (Race Condition)** | Hai người duyệt cùng lúc hoặc cùng duyệt/từ chối 1 yêu cầu. | **Khóa lạc quan / Ràng buộc trạng thái duy nhất:** Câu lệnh cập nhật SQL sử dụng điều kiện `WHERE id = ? AND status = 'pending'`. Chỉ đúng 1 giao dịch thành công chuyển trạng thái sang `approved` hoặc `rejected`. |
| **Khóa chặt khi tắt module** | Khi module `approval-gate` bị tắt, các tác vụ bị kẹt hoặc guard vẫn còn can thiệp. | **Tắt là gỡ bỏ hoàn toàn:** Khi module bị tắt, `FeatureLifecycleManager` gỡ bỏ guard khỏi danh sách hoạt động ngay lập tức. Máy chủ chuyển trạng thái trở lại như cũ mà không bị cản trở. Dữ liệu các yêu cầu cũ vẫn được bảo toàn trong SQLite để tái sử dụng khi module bật lại. |

## 4. Hệ quả kiến trúc (Consequences)
- Tuân thủ nghiêm ngặt nguyên tắc phụ thuộc một chiều: `dandori-module-api` định nghĩa trait và enum; `dandori-registry` quản lý vòng đời; `dandori-server` gọi điểm mở rộng; `mod-approval-gate` hiện thực trait.
- Hệ thống hỗ trợ mở rộng linh hoạt cho các guard tiếp theo (như `quiet-hours` trong P3) mà không cần thay đổi hợp đồng giao diện của lõi.
