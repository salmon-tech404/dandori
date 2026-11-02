use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, TS)]
#[ts(
    export,
    export_to = "../../../frontend/app/src/features/shared-memory/types_generated.ts"
)]
pub struct MemoryEntry {
    pub id: String,
    pub workspace_id: String,
    pub project_id: Option<String>,
    pub category: String,
    pub content: String,
    pub source_type: String,
    pub source_ref: String,
    pub status: String,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../frontend/app/src/features/shared-memory/types_generated.ts"
)]
pub struct CreateMemoryPayload {
    pub project_id: Option<String>,
    pub category: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../frontend/app/src/features/shared-memory/types_generated.ts"
)]
pub struct UpdateMemoryStatusPayload {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../frontend/app/src/features/shared-memory/types_generated.ts"
)]
pub struct BatchUpdateStatusPayload {
    pub memory_ids: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../frontend/app/src/features/shared-memory/types_generated.ts"
)]
pub struct ProposeMemoryPayload {
    pub project_id: Option<String>,
    pub category: String,
    pub content: String,
    pub source_ref: String,
}

pub fn is_valid_category(category: &str) -> bool {
    matches!(
        category.to_lowercase().as_str(),
        "fact" | "decision" | "convention" | "lesson"
    )
}

pub fn is_valid_status(status: &str) -> bool {
    matches!(
        status.to_lowercase().as_str(),
        "pending" | "approved" | "rejected"
    )
}
