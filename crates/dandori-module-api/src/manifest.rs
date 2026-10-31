//! Định nghĩa Manifest và kiểm tra hợp lệ của tệp module.json (01_KIEN_TRUC.md mục 4.1).

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;
use ts_rs::TS;

/// Lỗi kiểm tra Manifest.
#[derive(Debug, Error, PartialEq)]
pub enum ManifestError {
    /// ID module không đúng định dạng chữ thường, số, gạch ngang.
    #[error("ID module không hợp lệ (phải là chữ thường, số, gạch ngang): {0}")]
    InvalidId(String),

    /// Tên module rỗng.
    #[error("Tên module không được để trống")]
    EmptyName,

    /// Mô tả module rỗng.
    #[error("Mô tả module không được để trống")]
    EmptyDescription,

    /// Phiên bản không hợp lệ.
    #[error("Phiên bản module không được để trống")]
    EmptyVersion,

    /// Module có rủi ro khác Low nhưng thiếu ghi chú rủi ro.
    #[error("Module có mức rủi ro không phải low bắt buộc phải có risk_note giải thích")]
    MissingRiskNote,

    /// Phụ thuộc không hợp lệ.
    #[error("Phụ thuộc không hợp lệ: {0}")]
    InvalidDependency(String),

    /// Schema cấu hình không hợp lệ.
    #[error("Schema cấu hình {0} không hợp lệ: {1}")]
    InvalidSettingSchema(String, String),

    /// Trùng ID contribution giao diện trong cùng module.
    #[error("Trùng ID contribution giao diện trong module: {0}")]
    DuplicateContributionId(String),

    /// Slot giao diện rỗng hoặc không đúng định dạng.
    #[error("Slot giao diện không hợp lệ: {0}")]
    InvalidSlot(String),
}

/// Nhóm phân loại module theo 01_KIEN_TRUC.md mục 4.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ModuleGroup {
    /// Tài khoản và hạn mức
    Accounts,
    /// Điều phối và phân công
    Orchestration,
    /// Môi trường thực thi
    Runtime,
    /// Bảng điều khiển và thống kê
    Dashboard,
    /// Lịch trình và tác vụ định kỳ
    Scheduling,
    /// Tri thức và prompt
    Knowledge,
    /// Tích hợp bên ngoài
    Integrations,
    /// An toàn và kiểm soát
    Safety,
}

/// Phạm vi hoạt động của module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ModuleScope {
    /// Cấp toàn hệ thống
    System,
    /// Cấp workspace
    Workspace,
    /// Cấp dự án
    Project,
}

/// Mức công sức triển khai/vận hành.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ModuleEffort {
    /// Nhẹ
    Light,
    /// Trung bình
    Medium,
    /// Nặng
    Heavy,
}

/// Mức độ rủi ro khi bật module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ModuleRisk {
    /// Thấp, không ảnh hưởng lớn
    Low,
    /// Trung bình, cần lưu ý
    Medium,
    /// Cao, có thể gây mất an toàn nếu cấu hình sai
    High,
}

/// Định nghĩa schema một mục cấu hình trong settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct SettingSchema {
    /// Kiểu dữ liệu: "string", "number", "boolean", "array", "object".
    #[serde(rename = "type")]
    pub type_name: String,

    /// Giá trị mặc định.
    #[ts(type = "unknown")]
    pub default: serde_json::Value,

    /// Danh sách giá trị cho phép (nếu có).
    #[serde(default)]
    #[serde(rename = "enum")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(type = "unknown[]")]
    pub enum_values: Option<Vec<serde_json::Value>>,

    /// Giá trị nhỏ nhất (nếu là số).
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,

    /// Giá trị lớn nhất (nếu là số).
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
}

impl SettingSchema {
    /// Khởi tạo một SettingSchema mới.
    pub fn new(type_name: impl Into<String>, default: serde_json::Value) -> Self {
        Self {
            type_name: type_name.into(),
            default,
            enum_values: None,
            min: None,
            max: None,
        }
    }

    /// Kiểm tra tính hợp lệ của schema.
    pub fn validate(&self) -> Result<(), String> {
        let valid_types = ["string", "number", "boolean", "array", "object"];
        if !valid_types.contains(&self.type_name.as_str()) {
            return Err(format!(
                "Kiểu dữ liệu không được hỗ trợ: {}",
                self.type_name
            ));
        }

        if let (Some(min), Some(max)) = (self.min, self.max)
            && min > max
        {
            return Err(format!("min ({min}) không được lớn hơn max ({max})"));
        }

        Ok(())
    }
}

/// Định nghĩa một ô cắm (slot contribution) giao diện do module đóng góp.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct UiContribution {
    /// ID định danh ô cắm, duy nhất trong nội bộ module.
    pub id: String,

    /// Mã slot đích (thuộc danh mục slot hợp lệ).
    pub slot: String,

    /// Nhãn hiển thị trên giao diện.
    pub label: String,

    /// Thứ tự sắp xếp hiển thị trong slot.
    pub order: i32,
}

/// Khai báo phần giao diện đóng góp của module.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct UiDeclaration {
    /// Danh sách các ô cắm giao diện.
    #[serde(default)]
    pub contributes: Vec<UiContribution>,
}

/// Khai báo đầy đủ của một module theo tệp module.json (01_KIEN_TRUC.md mục 4.1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// ID định danh module (chữ thường, số, gạch ngang, khớp thư mục `mod-<id>`).
    pub id: String,

    /// Nhóm phân loại module.
    pub group: ModuleGroup,

    /// Tên hiển thị tiếng Việt.
    pub name: String,

    /// Một dòng mô tả hiển thị trong bảng Option.
    pub description: String,

    /// Phiên bản semver của module.
    pub version: String,

    /// Danh sách ID các module cần có trước.
    #[serde(default)]
    pub requires: Vec<String>,

    /// Các cấp phạm vi áp dụng.
    #[serde(default)]
    pub scopes: Vec<ModuleScope>,

    /// Mức công sức.
    pub effort: ModuleEffort,

    /// Mức độ rủi ro.
    pub risk: ModuleRisk,

    /// Ghi chú rủi ro khi risk != low.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk_note: Option<String>,

    /// Module lõi bắt buộc, không thể tắt.
    #[serde(default)]
    pub core: bool,

    /// Cần khởi động lại worker nền khi thay đổi cấu hình.
    #[serde(default)]
    pub requires_worker_restart: bool,

    /// Khai báo các mục cấu hình tùy chọn.
    #[serde(default)]
    pub settings: HashMap<String, SettingSchema>,

    /// Đóng góp giao diện vào các slot.
    #[serde(default)]
    pub ui: UiDeclaration,
}

impl Manifest {
    /// Kiểm tra tính hợp lệ toàn diện của manifest.
    pub fn validate(&self) -> Result<(), ManifestError> {
        // 1. Kiểm tra ID: chữ thường, số, dấu gạch ngang
        if self.id.is_empty()
            || !self
                .id
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(ManifestError::InvalidId(self.id.clone()));
        }

        // 2. Tên và mô tả
        if self.name.trim().is_empty() {
            return Err(ManifestError::EmptyName);
        }
        if self.description.trim().is_empty() {
            return Err(ManifestError::EmptyDescription);
        }
        if self.version.trim().is_empty() {
            return Err(ManifestError::EmptyVersion);
        }

        // 3. Rủi ro và ghi chú rủi ro
        if self.risk != ModuleRisk::Low {
            match &self.risk_note {
                Some(note) if !note.trim().is_empty() => {}
                _ => return Err(ManifestError::MissingRiskNote),
            }
        }

        // 4. Kiểm tra settings
        for (key, schema) in &self.settings {
            if let Err(msg) = schema.validate() {
                return Err(ManifestError::InvalidSettingSchema(key.clone(), msg));
            }
        }

        // 5. Kiểm tra UI contributes: không trùng ID trong module
        let mut seen_ids = HashSet::new();
        for item in &self.ui.contributes {
            if item.id.trim().is_empty() {
                return Err(ManifestError::DuplicateContributionId(item.id.clone()));
            }
            if !seen_ids.insert(&item.id) {
                return Err(ManifestError::DuplicateContributionId(item.id.clone()));
            }
            if item.slot.trim().is_empty() {
                return Err(ManifestError::InvalidSlot(item.slot.clone()));
            }
        }

        Ok(())
    }
}
