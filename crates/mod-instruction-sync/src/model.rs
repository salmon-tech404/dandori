use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, TS)]
#[ts(
    export,
    export_to = "../../../frontend/app/src/features/instruction-sync/types_generated.ts"
)]
pub struct InstructionSyncConfig {
    pub id: String,
    pub workspace_id: String,
    pub project_id: String,
    pub provider: String,
    pub target_filename: String,
    pub strategy: String,
    pub is_enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../frontend/app/src/features/instruction-sync/types_generated.ts"
)]
pub struct UpdateConfigPayload {
    pub provider: String,
    pub target_filename: String,
    pub strategy: String,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../frontend/app/src/features/instruction-sync/types_generated.ts"
)]
pub struct PreviewPayload {
    pub project_id: String,
    pub provider: String,
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../../frontend/app/src/features/instruction-sync/types_generated.ts"
)]
pub struct PreviewResult {
    pub target_filename: String,
    pub content: String,
    pub tech_profile_count: usize,
    pub skills_count: usize,
}
