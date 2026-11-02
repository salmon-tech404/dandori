//! Xử lý biến mẫu, kiểm tra danh sách trắng và chống rò rỉ bí mật cho prompt.

/// Danh sách trắng các biến mẫu được phép sử dụng.
pub const VALID_VARIABLES: &[&str] = &[
    "{{task.title}}",
    "{{task.description}}",
    "{{project.name}}",
    "{{role}}",
];

/// Độ dài tối đa cho phép của nội dung prompt (ký tự).
pub const MAX_PROMPT_LENGTH: usize = 10_000;

/// Kiểm tra xem nội dung có vượt quá độ dài quy định không.
pub fn check_length(content: &str) -> Result<(), &'static str> {
    if content.len() > MAX_PROMPT_LENGTH {
        return Err("Nội dung prompt vượt quá giới hạn tối đa 10,000 ký tự");
    }
    Ok(())
}

/// Kiểm tra xem nội dung có chứa mẫu giống khóa bí mật hoặc token hay không (RS-SEC-005).
/// Dùng chung bộ nhận diện của `dandori-module-api` (nhận theo đầu từ, không khớp nhầm "task-").
pub fn check_secrets(content: &str) -> Result<(), &'static str> {
    if dandori_module_api::contains_secret_like(content) {
        return Err("Nội dung chứa chuỗi giống khóa bí mật hoặc API token bị từ chối");
    }
    Ok(())
}

/// Kiểm tra tính hợp lệ của tất cả các biến mẫu dạng `{{...}}`.
///
/// # Errors
/// Trả về lỗi nếu phát hiện biến mẫu lạ không thuộc danh sách trắng.
pub fn validate_variables(content: &str) -> Result<(), String> {
    let mut rest = content;
    while let Some(start_idx) = rest.find("{{") {
        let after_start = match rest.get(start_idx..) {
            Some(s) => s,
            None => break,
        };
        let end_idx = match after_start.find("}}") {
            Some(idx) => idx + 2,
            None => {
                return Err("Thẻ biến mẫu không đóng đúng cú pháp: thiếu '}}'".to_string());
            }
        };

        let var = match after_start.get(..end_idx) {
            Some(v) => v,
            None => break,
        };

        if !VALID_VARIABLES.contains(&var) {
            return Err(format!(
                "Biến mẫu không hợp lệ: '{var}'. Chỉ chấp nhận: {{task.title}}, {{task.description}}, {{project.name}}, {{role}}"
            ));
        }

        rest = match after_start.get(end_idx..) {
            Some(r) => r,
            None => break,
        };
    }

    Ok(())
}

/// Thay thế biến mẫu một lượt (single-pass non-recursive substitution).
///
/// Bảo đảm nội dung bên trong giá trị task (ví dụ task.title chứa `{{role}}`)
/// tuyệt đối không bị diễn giải đệ quy (AC2).
#[must_use]
pub fn render_template(
    template: &str,
    task_title: &str,
    task_desc: &str,
    project_name: &str,
    role_str: &str,
) -> String {
    let mut output = String::with_capacity(template.len() + 128);
    let mut rest = template;

    while !rest.is_empty() {
        if let Some(pos) = rest.find("{{") {
            if let Some(prefix) = rest.get(..pos) {
                output.push_str(prefix);
            }
            let candidate = match rest.get(pos..) {
                Some(c) => c,
                None => break,
            };

            if let Some(matched_var) = match_known_variable(candidate) {
                match matched_var {
                    "{{task.title}}" => output.push_str(task_title),
                    "{{task.description}}" => output.push_str(task_desc),
                    "{{project.name}}" => output.push_str(project_name),
                    "{{role}}" => output.push_str(role_str),
                    _ => {}
                }
                rest = candidate.get(matched_var.len()..).unwrap_or("");
            } else {
                output.push_str("{{");
                rest = candidate.get(2..).unwrap_or("");
            }
        } else {
            output.push_str(rest);
            break;
        }
    }

    output
}

fn match_known_variable(input: &str) -> Option<&'static str> {
    VALID_VARIABLES
        .iter()
        .find(|&&var| input.starts_with(var))
        .copied()
}
