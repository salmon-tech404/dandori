//! Định danh newtype cho mọi thực thể của Dandori (RS-TYPE-001).

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use ts_rs::TS;
use uuid::Uuid;

/// Macro sinh định danh newtype bọc UUID có đầy đủ trait chuẩn.
macro_rules! define_uuid_id {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS,
        )]
        #[ts(type = "string")]
        pub struct $name(pub Uuid);

        impl $name {
            #[doc = concat!("Tạo mới một `", stringify!($name), "` ngẫu nhiên theo chuẩn UUID v4.")]
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            #[doc = concat!("Lấy tham chiếu tới UUID bên trong `", stringify!($name), "`.")]
            #[must_use]
            pub fn as_uuid(&self) -> &Uuid {
                &self.0
            }

            #[doc = concat!("Chuyển đổi `", stringify!($name), "` thành UUID nguyên bản.")]
            #[must_use]
            pub fn into_inner(self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<Uuid> for $name {
            fn from(u: Uuid) -> Self {
                Self(u)
            }
        }

        impl FromStr for $name {
            type Err = uuid::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Uuid::parse_str(s).map(Self)
            }
        }
    };
}

define_uuid_id!(UserId, "Định danh duy nhất của người dùng.");
define_uuid_id!(SessionId, "Định danh duy nhất của phiên đăng nhập.");
define_uuid_id!(WorkspaceId, "Định danh duy nhất của không gian làm việc.");
define_uuid_id!(ProjectId, "Định danh duy nhất của dự án.");
define_uuid_id!(TaskId, "Định danh duy nhất của task công việc.");
define_uuid_id!(RunId, "Định danh duy nhất của một lần chạy agent.");
define_uuid_id!(
    RunEventId,
    "Định danh duy nhất của một sự kiện/dòng log lần chạy."
);
define_uuid_id!(
    AgentAccountId,
    "Định danh duy nhất của tài khoản agent kết nối."
);
define_uuid_id!(SecretId, "Định danh duy nhất của bản ghi khóa bí mật.");
define_uuid_id!(
    FeatureAuditId,
    "Định danh duy nhất của bản ghi nhật ký kiểm toán tính năng."
);

/// Định danh dạng chuỗi của một module tính năng (ví dụ `sample-hello`, `kanban`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
#[ts(type = "string")]
pub struct FeatureModuleId(pub String);

impl FeatureModuleId {
    /// Tạo mới một `FeatureModuleId` từ chuỗi.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Lấy tham chiếu chuỗi của module ID.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Chuyển thành chuỗi nguyên bản.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for FeatureModuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for FeatureModuleId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for FeatureModuleId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Định danh dạng chuỗi của một gói tính năng mẫu (ví dụ `solo-light`, `team-small`, `full`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
#[ts(type = "string")]
pub struct FeaturePackId(pub String);

impl FeaturePackId {
    /// Tạo mới một `FeaturePackId` từ chuỗi.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Lấy tham chiếu chuỗi của pack ID.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Chuyển thành chuỗi nguyên bản.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for FeaturePackId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for FeaturePackId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for FeaturePackId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}
