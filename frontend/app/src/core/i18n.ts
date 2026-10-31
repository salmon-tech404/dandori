/**
 * i18n — Hệ thống đa ngôn ngữ tiếng Việt tập trung cho Dandori (04_GIAO_DIEN.md mục 6)
 * Mọi chuỗi hiển thị giao diện bắt buộc khai báo tại đây.
 */

export const VI_DICTIONARY = {
  // Navigation & Menu
  nav_board: "Board",
  nav_agents: "Agent và tài khoản",
  nav_dashboard: "Dashboard",
  nav_calendar: "Lịch",
  nav_knowledge: "Prompt và tri thức",
  nav_integrations: "Tích hợp",
  nav_settings: "Cài đặt: Tính năng",

  // App Shell & Header
  app_title: "Agent Hub",
  server_running: "Server: đang chạy",
  server_stopped: "Server: mất kết nối",
  btn_create_task: "+ Giao task",
  active_features_note: "Đang bật {on} / {total} tính năng. Menu bên trái chỉ hiện những gì bạn đã bật.",
  project_selector_repos: "{count} repo",

  // Màn hình khi module tắt
  feature_disabled_title: "Tính năng này đang tắt",
  feature_disabled_desc: "Màn hình này thuộc về các tính năng chưa được kích hoạt.",
  go_to_settings: "Đi tới Cài đặt để bật tính năng",

  // Core Fallback Board
  core_task_list_title: "Danh sách nhiệm vụ (Lõi hệ thống)",
  core_task_list_desc: "Chế độ xem cơ bản khi module Kanban đang tắt.",
  core_task_list_empty: "Chưa có nhiệm vụ nào.",
  task_col_title: "Tiêu đề",
  task_col_status: "Trạng thái",
  task_col_created: "Ngày tạo",

  // Trạng thái Task
  status_backlog: "Backlog",
  status_running: "Đang chạy",
  status_needs_human: "Cần bạn",
  status_in_review: "Đang review",
  status_ready_to_merge: "Sẵn sàng merge",
  status_done: "Xong",
  status_failed: "Thất bại",
  status_cancelled: "Đã hủy",
  status_todo: "Cần làm",
  status_in_progress: "Đang xử lý",

  // Xác thực & Tài khoản
  setup_owner_title: "Khởi tạo Chủ sở hữu hệ thống",
  setup_owner_desc: "Đây là lần đầu khởi chạy Dandori. Hãy thiết lập tài khoản quản trị cao nhất.",
  login_title: "Đăng nhập Dandori",
  username_label: "Tên đăng nhập",
  password_label: "Mật khẩu",
  btn_login: "Đăng nhập",
  btn_setup_owner: "Khởi tạo hệ thống",
  btn_logout: "Đăng xuất",

  // Lỗi & Chi tiết kỹ thuật
  api_error_generic: "Yêu cầu không thể hoàn thành. Vui lòng thử lại sau.",
  technical_details: "Chi tiết kỹ thuật",
  request_id_label: "Mã yêu cầu (Request ID):",
  error_code_label: "Mã lỗi:",
  slot_error_title: "Không tải được {name}",
  slot_error_retry: "Thử tải lại",

  // Thành phần dùng chung
  loading: "Đang tải...",
  empty_state: "Không có dữ liệu.",
  btn_confirm: "Xác nhận",
  btn_cancel: "Hủy",
  btn_close: "Đóng",
  dialog_confirm_title: "Xác nhận thao tác",
  tooltip_more_info: "Thông tin chi tiết",

  // Màn hình Cài đặt > Tính năng (T13)
  settings_features_title: "Cài đặt Tính năng",
  settings_features_desc: "Bật tắt và cấu hình các module chức năng trong Dandori.",
  scope_system: "Hệ thống",
  scope_workspace: "Workspace",
  scope_project: "Dự án",
  search_features_placeholder: "Tìm kiếm tính năng theo tên hoặc mô tả...",
  filter_all: "Tất cả",
  filter_enabled: "Đang bật",
  filter_disabled: "Đang tắt",
  sample_packs_title: "Gói mẫu tính năng",
  sample_packs_desc: "Áp dụng cấu hình nhanh theo quy mô nhóm làm việc.",
  pack_solo: "Solo nhẹ",
  pack_team: "Nhóm nhỏ",
  pack_full: "Đầy đủ",
  pack_confirm_title: "Áp dụng gói mẫu {name}",
  pack_confirm_desc: "Thao tác này sẽ bật toàn bộ các module sau: {modules}.",
  group_safety: "An toàn & Kiểm soát",
  group_accounts: "Tài khoản & Hạn mức",
  group_interface: "Giao diện & Hiển thị",
  group_coordination: "Điều phối & Phân công",
  group_knowledge: "Tri thức & Kỹ năng",
  group_integrations: "Tích hợp & Thông báo",
  group_automation: "Tự động hóa & Lịch",
  group_infrastructure: "Hạ tầng & Thực thi",
  group_count_badge: "Đang bật {on}/{total}",
  requires_label: "Cần bật trước: {modules}",
  locked_by_parent: "Do quản trị viên cấp trên quy định",
  core_module_tooltip: "Luôn bật: Lõi hệ thống dựa vào tính năng này",
  modal_enable_dependencies_title: "Bật tính năng kèm theo",
  modal_enable_dependencies_msg: "Để bật tính năng này, hệ thống sẽ tự động bật kèm: {modules}.",
  modal_disable_dependents_title: "Cảnh báo tắt tính năng",
  modal_disable_dependents_msg: "Tắt tính năng này sẽ làm mất chức năng của các module: {modules}.",
  modal_risk_title: "Cảnh báo tính năng có rủi ro cao",
  modal_risk_msg: "Tính năng này có rủi ro cao: {note}. Bạn có chắc chắn muốn kích hoạt không?",
  worker_restart_badge: "Cần khởi động lại worker",
  effort_light: "Nhẹ",
  effort_medium: "Vừa",
  effort_heavy: "Nặng",
  risk_low: "Rủi ro thấp",
  risk_medium: "Rủi ro vừa",
  risk_high: "Rủi ro cao",
  configure_module: "Cấu hình module",
  audit_log_title: "Nhật ký thay đổi tính năng",
  btn_undo_latest: "Hoàn tác thay đổi gần nhất",
  undo_success: "Đã hoàn tác thay đổi gần nhất.",
  no_audit_records: "Chưa có thay đổi nào trong nhật ký.",
} as const;

export type TranslationKey = keyof typeof VI_DICTIONARY;

/**
 * Lấy chuỗi bản địa hóa tiếng Việt kèm thay thế tham số dạng {key}
 */
export function t(key: TranslationKey | string, params?: Record<string, string | number>): string {
  const template = VI_DICTIONARY[key as TranslationKey];
  if (!template) {
    return key;
  }
  if (!params) {
    return template;
  }
  let result: string = template;
  for (const [paramKey, paramVal] of Object.entries(params)) {
    result = result.replace(new RegExp(`\\{${paramKey}\\}`, "g"), String(paramVal));
  }
  return result;
}
