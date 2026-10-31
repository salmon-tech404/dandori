// TỆP SINH TỰ ĐỘNG bởi generate_types (RS-TS-001); không sửa tay.

export type SystemStatus = "initializing" | "ready" | "degraded" | "stopped";

export type UserId = string;

export type SessionId = string;

export type WorkspaceId = string;

export type ProjectId = string;

export type TaskId = string;

export type RunId = string;

export type RunEventId = string;

export type AgentAccountId = string;

export type SecretId = string;

export type FeatureAuditId = string;

export type FeatureModuleId = string;

export type FeaturePackId = string;

export type User = { 
/**
 * Định danh người dùng.
 */
id: UserId, 
/**
 * Tên đăng nhập duy nhất.
 */
username: string, 
/**
 * Chuỗi băm mật khẩu (Argon2).
 */
password_hash: string, 
/**
 * Vai trò hệ thống (`admin` hoặc `user`).
 */
system_role: string, 
/**
 * Thời điểm tạo tài khoản.
 */
created_at: string, };

export type Session = { 
/**
 * Định danh phiên.
 */
id: SessionId, 
/**
 * Định danh người dùng sở hữu phiên.
 */
user_id: UserId, 
/**
 * Thời điểm hết hạn phiên.
 */
expires_at: string, 
/**
 * Chuỗi băm của token phiên.
 */
token_hash: string, };

export type Workspace = { 
/**
 * Định danh không gian làm việc.
 */
id: WorkspaceId, 
/**
 * Tên không gian làm việc.
 */
name: string, 
/**
 * Thời điểm khởi tạo.
 */
created_at: string, };

export type WorkspaceMember = { 
/**
 * Không gian làm việc.
 */
workspace_id: WorkspaceId, 
/**
 * Người dùng thành viên.
 */
user_id: UserId, 
/**
 * Vai trò (`admin` hoặc `member`).
 */
role: string, };

export type Project = { 
/**
 * Định danh dự án.
 */
id: ProjectId, 
/**
 * Không gian làm việc sở hữu dự án.
 */
workspace_id: WorkspaceId, 
/**
 * Tên dự án.
 */
name: string, 
/**
 * Đường dẫn thư mục kho mã nguồn (git repository).
 */
repo_path: string, 
/**
 * Nhánh git mặc định.
 */
default_branch: string, 
/**
 * Thời điểm tạo dự án.
 */
created_at: string, };

export type TaskStatus = "backlog" | "running" | "needs_human" | "in_review" | "ready_to_merge" | "done" | "failed" | "cancelled";

export type Task = { 
/**
 * Định danh task.
 */
id: TaskId, 
/**
 * Dự án sở hữu task.
 */
project_id: ProjectId, 
/**
 * Tiêu đề task.
 */
title: string, 
/**
 * Mô tả chi tiết yêu cầu công việc.
 */
description: string, 
/**
 * Trạng thái task (`todo`, `in_progress`, `done`, `failed`, `cancelled`).
 */
status: string, 
/**
 * Định danh agent trưởng phối hợp.
 */
lead_agent: string, 
/**
 * Định danh agent thực thi trực tiếp.
 */
worker_agent: string, 
/**
 * Đường dẫn thư mục làm việc độc lập (worktree), nếu có.
 */
workdir_path: string | null, 
/**
 * Thời điểm tạo task.
 */
created_at: string, 
/**
 * Thời điểm cập nhật trạng thái gần nhất.
 */
updated_at: string, };

export type Run = { 
/**
 * Định danh lần chạy.
 */
id: RunId, 
/**
 * Task được thực thi trong lần chạy này.
 */
task_id: TaskId, 
/**
 * Agent thực thi (`claude`, `gemini`, `codex`, `fake`...).
 */
agent: string, 
/**
 * Tài khoản agent được phân bổ.
 */
account: string, 
/**
 * Trạng thái lần chạy (`queued`, `running`, `success`, `failed`, `cancelled`).
 */
status: string, 
/**
 * Thời điểm bắt đầu chạy.
 */
started_at: string, 
/**
 * Thời điểm kết thúc chạy, nếu đã xong.
 */
finished_at: string | null, 
/**
 * Mã thoát của tiến trình (exit code), nếu có.
 */
exit_code: bigint | null, };

export type RunEvent = { 
/**
 * Định danh sự kiện lần chạy.
 */
id: RunEventId, 
/**
 * Định danh lần chạy gắn liền.
 */
run_id: RunId, 
/**
 * Thứ tự tuần tự của sự kiện trong lần chạy.
 */
seq: bigint, 
/**
 * Loại sự kiện (`output`, `tool_call`, `error`...).
 */
event_type: string, 
/**
 * Nội dung dữ liệu định dạng JSON.
 */
data: string, 
/**
 * Thời điểm ghi nhận sự kiện.
 */
created_at: string, };

export type SecretRecord = { 
/**
 * Định danh khóa bí mật.
 */
id: SecretId, 
/**
 * Nhãn mô tả khóa.
 */
label: string, 
/**
 * Giá trị nonce dùng cho thuật toán giải mã AEAD.
 */
nonce: string, 
/**
 * Dữ liệu đã mã hóa (ciphertext dạng byte).
 */
ciphertext: Array<number>, 
/**
 * Thời điểm lưu trữ khóa.
 */
created_at: string, };

export type AgentAccount = { 
/**
 * Định danh tài khoản.
 */
id: AgentAccountId, 
/**
 * Nhà cung cấp AI (`anthropic`, `google`, `openai`...).
 */
provider: string, 
/**
 * Nhãn hiển thị của tài khoản.
 */
label: string, 
/**
 * Thư mục HOME riêng biệt dành cho CLI của tài khoản này.
 */
home_dir: string, 
/**
 * Loại xác thực (`cli` hoặc `api_key`).
 */
auth_type: string, 
/**
 * Tham chiếu khóa bí mật mã hóa nếu dùng `api_key`.
 */
secret_id: SecretId | null, 
/**
 * Trạng thái đăng nhập (`not_installed`, `not_logged_in`, `ready`, `session_expired`, `unknown`).
 */
status: string, 
/**
 * Có đang được kích hoạt hay không.
 */
enabled: boolean, 
/**
 * Độ ưu tiên điều phối (số nhỏ = ưu tiên cao).
 */
priority: bigint, 
/**
 * Thời điểm hết hạn mức (nếu đang bị tạm ngừng).
 */
exhausted_until: string | null, 
/**
 * Thời điểm kiểm tra trạng thái gần nhất.
 */
last_checked_at: string | null, };

export type FeatureModule = { 
/**
 * Định danh module.
 */
id: FeatureModuleId, 
/**
 * Tên nhóm module.
 */
group_name: string, 
/**
 * Tên hiển thị module.
 */
name: string, 
/**
 * Mô tả chi tiết tính năng module.
 */
description: string, 
/**
 * Phiên bản module.
 */
version: string, 
/**
 * Danh sách định danh các module phụ thuộc bắc cầu.
 */
requires: Array<string>, 
/**
 * Các phạm vi được phép cấu hình (`system`, `workspace`, `project`).
 */
scopes: Array<string>, 
/**
 * Mức độ rủi ro (`low`, `medium`, `high`).
 */
risk: string, 
/**
 * Mức độ công sức triển khai (`low`, `medium`, `high`).
 */
effort: string, 
/**
 * Cờ đánh dấu module thuộc lõi hệ thống.
 */
core: boolean, };

export type FeatureSetting = { 
/**
 * Module được áp dụng cấu hình.
 */
module_id: FeatureModuleId, 
/**
 * Loại phạm vi (`system`, `workspace`, `project`).
 */
scope_kind: string, 
/**
 * Định danh của phạm vi đó.
 */
scope_id: string, 
/**
 * Trạng thái bật hoặc tắt.
 */
enabled: boolean, 
/**
 * Trạng thái khóa không cho cấp dưới ghi đè.
 */
locked: boolean, 
/**
 * Chuỗi JSON cấu hình chi tiết theo schema của module.
 */
config: string, 
/**
 * Người thực hiện thay đổi.
 */
changed_by: string, 
/**
 * Thời điểm thay đổi.
 */
changed_at: string, };

export type FeaturePack = { 
/**
 * Định danh gói mẫu.
 */
id: FeaturePackId, 
/**
 * Tên gói mẫu.
 */
name: string, 
/**
 * Danh sách các module thuộc gói.
 */
modules: Array<string>, };

export type FeatureAudit = { 
/**
 * Định danh sự kiện kiểm toán.
 */
id: FeatureAuditId, 
/**
 * Module bị tác động.
 */
module_id: FeatureModuleId, 
/**
 * Loại phạm vi (`system`, `workspace`, `project`).
 */
scope_kind: string, 
/**
 * Định danh phạm vi tác động.
 */
scope_id: string, 
/**
 * Hành động (`enable`, `disable`, `configure`, `rollback`).
 */
action: string, 
/**
 * Người hoặc agent thực hiện hành động.
 */
performed_by: string, 
/**
 * Lý do thực hiện.
 */
reason: string, 
/**
 * Thời điểm ghi nhận.
 */
created_at: string, };

export type EventKind = "module_enabled" | "module_disabled" | "task_created" | "task_status_changed" | "task_assigned" | "run_started" | "run_output" | "run_finished" | "run_failed" | "approval_requested" | "approval_decided" | "account_quota_updated" | "notification_requested" | "audit_recorded" | "worker_failed";

export type EventScope = { 
/**
 * Định danh không gian làm việc (nếu có).
 */
workspace_id: WorkspaceId | null, 
/**
 * Định danh dự án (nếu có).
 */
project_id: ProjectId | null, 
/**
 * Định danh task (nếu có).
 */
task_id: TaskId | null, 
/**
 * Định danh lần chạy agent (nếu có).
 */
run_id: RunId | null, };

export type EventPayload = { "kind": "module_enabled", "data": { 
/**
 * ID module được bật.
 */
module_id: FeatureModuleId, } } | { "kind": "module_disabled", "data": { 
/**
 * ID module bị tắt.
 */
module_id: FeatureModuleId, } } | { "kind": "task_created", "data": { 
/**
 * ID task được tạo.
 */
task_id: TaskId, 
/**
 * Tiêu đề task.
 */
title: string, } } | { "kind": "task_status_changed", "data": { 
/**
 * ID task.
 */
task_id: TaskId, 
/**
 * Trạng thái trước đó.
 */
old_status: string, 
/**
 * Trạng thái mới.
 */
new_status: string, } } | { "kind": "task_assigned", "data": { 
/**
 * ID task.
 */
task_id: TaskId, 
/**
 * Tên agent được phân bổ.
 */
agent: string, } } | { "kind": "run_started", "data": { 
/**
 * ID lần chạy.
 */
run_id: RunId, 
/**
 * Tên agent.
 */
agent: string, 
/**
 * Nhãn tài khoản agent.
 */
account: string, } } | { "kind": "run_output", "data": { 
/**
 * ID lần chạy.
 */
run_id: RunId, 
/**
 * Thứ tự tuần tự.
 */
seq: bigint, 
/**
 * Loại luồng (`stdout` hoặc `stderr`).
 */
stream: string, 
/**
 * Nội dung log chunk.
 */
chunk: string, } } | { "kind": "run_finished", "data": { 
/**
 * ID lần chạy.
 */
run_id: RunId, 
/**
 * Mã thoát của tiến trình (nếu có).
 */
exit_code: bigint | null, } } | { "kind": "run_failed", "data": { 
/**
 * ID lần chạy.
 */
run_id: RunId, 
/**
 * Thông điệp lỗi chi tiết.
 */
error_message: string, } } | { "kind": "approval_requested", "data": { 
/**
 * Định danh cổng phê duyệt.
 */
gate_id: string, 
/**
 * ID task cần phê duyệt.
 */
task_id: TaskId, 
/**
 * Lý do yêu cầu.
 */
reason: string, } } | { "kind": "approval_decided", "data": { 
/**
 * Định danh cổng phê duyệt.
 */
gate_id: string, 
/**
 * ID task.
 */
task_id: TaskId, 
/**
 * Được duyệt hay từ chối.
 */
approved: boolean, 
/**
 * Người hoặc agent đưa ra quyết định.
 */
decided_by: string, } } | { "kind": "account_quota_updated", "data": { 
/**
 * Định danh tài khoản agent.
 */
account_id: AgentAccountId, 
/**
 * Thời điểm hết hạn mức (nếu đang bị tạm ngừng).
 */
exhausted_until: string | null, } } | { "kind": "notification_requested", "data": { 
/**
 * Tiêu đề thông báo.
 */
title: string, 
/**
 * Nội dung chi tiết.
 */
message: string, 
/**
 * Mức độ thông báo (`info`, `warning`, `error`).
 */
level: string, } } | { "kind": "audit_recorded", "data": { 
/**
 * ID bản ghi kiểm toán.
 */
audit_id: FeatureAuditId, 
/**
 * Hành động tác động (`enable`, `disable`...).
 */
action: string, } } | { "kind": "worker_failed", "data": { 
/**
 * Định danh worker hoặc module sở hữu.
 */
worker_id: string, 
/**
 * Thông điệp lỗi.
 */
error_message: string, } };

export type Envelope = { 
/**
 * Định danh duy nhất của sự kiện (UUID v4).
 */
id: string, 
/**
 * Thời điểm phát sinh sự kiện (UTC timestamp).
 */
at: string, 
/**
 * Loại sự kiện.
 */
kind: EventKind, 
/**
 * Phạm vi tác động của sự kiện.
 */
scope: EventScope, 
/**
 * Dữ liệu chi tiết của sự kiện.
 */
payload: EventPayload, };
