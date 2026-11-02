//! Chuẩn hóa và làm sạch tên nhánh git từ tiêu đề nhiệm vụ.

use crate::error::GitError;

fn remove_vietnamese_accents(c: char) -> char {
    match c {
        'à' | 'á' | 'ả' | 'ã' | 'ạ' | 'ă' | 'ằ' | 'ắ' | 'ẳ' | 'ẵ' | 'ặ' | 'â' | 'ầ' | 'ấ' | 'ẩ'
        | 'ẫ' | 'ậ' => 'a',
        'è' | 'é' | 'ẻ' | 'ẽ' | 'ẹ' | 'ê' | 'ề' | 'ế' | 'ể' | 'ễ' | 'ệ' => 'e',
        'ì' | 'í' | 'ỉ' | 'ĩ' | 'ị' => 'i',
        'ò' | 'ó' | 'ỏ' | 'õ' | 'ọ' | 'ô' | 'ồ' | 'ố' | 'ổ' | 'ỗ' | 'ộ' | 'ơ' | 'ờ' | 'ớ' | 'ở'
        | 'ỡ' | 'ợ' => 'o',
        'ù' | 'ú' | 'ủ' | 'ũ' | 'ụ' | 'ư' | 'ừ' | 'ứ' | 'ử' | 'ữ' | 'ự' => 'u',
        'ỳ' | 'ý' | 'ỷ' | 'ỹ' | 'ỵ' => 'y',
        'đ' => 'd',
        'À' | 'Á' | 'Ả' | 'Ã' | 'Ạ' | 'Ă' | 'Ằ' | 'Ắ' | 'Ẳ' | 'Ẵ' | 'Ặ' | 'Â' | 'Ầ' | 'Ấ' | 'Ẩ'
        | 'Ẫ' | 'Ậ' => 'a',
        'È' | 'É' | 'Ẻ' | 'Ẽ' | 'Ẹ' | 'Ê' | 'Ề' | 'Ế' | 'Ể' | 'Ễ' | 'Ệ' => 'e',
        'Ì' | 'Í' | 'Ỉ' | 'Ĩ' | 'Ị' => 'i',
        'Ò' | 'Ó' | 'Ỏ' | 'Õ' | 'Ọ' | 'Ô' | 'Ồ' | 'Ố' | 'Ổ' | 'Ỗ' | 'Ộ' | 'Ơ' | 'Ờ' | 'Ớ' | 'Ở'
        | 'Ỡ' | 'Ợ' => 'o',
        'Ù' | 'Ú' | 'Ủ' | 'Ũ' | 'Ụ' | 'Ư' | 'Ừ' | 'Ứ' | 'Ử' | 'Ữ' | 'Ự' => 'u',
        'Ỳ' | 'Ý' | 'Ỷ' | 'Ỹ' | 'Ỵ' => 'y',
        'Đ' => 'd',
        other => other,
    }
}

/// Chuẩn hóa tiêu đề task thành slug an toàn cho git branch.
#[must_use]
pub fn slugify(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut last_was_dash = false;

    for raw_char in text.chars() {
        let c = remove_vietnamese_accents(raw_char);
        if c.is_ascii_alphanumeric() {
            result.push(c.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            result.push('-');
            last_was_dash = true;
        }
    }

    let trimmed = result.trim_matches('-');
    if trimmed.is_empty() {
        "task".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Tạo tên nhánh an toàn có tiền tố `task/` và hậu tố ID ngắn.
///
/// # Errors
/// Trả về `GitError::InvalidBranchName` nếu tên nhánh vi phạm các quy tắc git ref format.
pub fn sanitize_branch_name(title: &str, short_id: &str) -> Result<String, GitError> {
    let raw_slug = slugify(title);
    let max_slug_len = 35;
    let truncated_slug: String = raw_slug.chars().take(max_slug_len).collect();
    let clean_slug = truncated_slug.trim_end_matches('-');

    let clean_id: String = short_id
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .take(12)
        .collect();

    let branch = if clean_id.is_empty() {
        format!("task/{clean_slug}")
    } else {
        format!("task/{clean_slug}-{clean_id}")
    };

    // Kiểm tra tính hợp lệ cơ bản của branch name
    if branch.contains("..")
        || branch.contains("@{")
        || branch.contains('~')
        || branch.contains('^')
        || branch.contains(':')
        || branch.contains('?')
        || branch.contains('*')
        || branch.contains('[')
        || branch.contains('\\')
        || branch.ends_with('/')
        || branch.ends_with(".lock")
    {
        return Err(GitError::InvalidBranchName(branch));
    }

    Ok(branch)
}

/// Kiểm tra tên nhánh/ref do người dùng nhập trước khi đưa vào dòng lệnh git.
///
/// Quan trọng nhất: không được bắt đầu bằng `-` (nếu không git hiểu là tùy chọn, ví dụ
/// `git diff --output=<tệp>` ghi đè tệp tùy ý).
///
/// # Errors
/// Trả về `GitError::InvalidBranchName` nếu tên rỗng, quá dài, bắt đầu bằng `-`/`/`,
/// chứa khoảng trắng, ký tự điều khiển hoặc dãy bị cấm bởi quy tắc ref của git.
pub fn validate_ref_name(name: &str) -> Result<(), GitError> {
    let bad = name.is_empty()
        || name.len() > 200
        || name.starts_with('-')
        || name.starts_with('/')
        || name.ends_with('/')
        || name.ends_with('.')
        || name.ends_with(".lock")
        || name == "@"
        || name.contains("..")
        || name.contains("//")
        || name.contains("@{")
        || name
            .chars()
            .any(|c| c.is_control() || c.is_whitespace() || "~^:?*[\\".contains(c));
    if bad {
        return Err(GitError::InvalidBranchName(name.to_string()));
    }
    Ok(())
}
