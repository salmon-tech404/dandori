use dandori_module_api::{
    Manifest, ManifestError, ModuleEffort, ModuleGroup, ModuleRisk, ModuleScope, SettingSchema,
    table_name,
};

#[test]
fn test_valid_manifest_deserialization() {
    let json = r#"{
        "id": "cost-tracking",
        "group": "dashboard",
        "name": "Token và chi phí theo task",
        "description": "Một dòng mô tả hiện trong bảng Option",
        "version": "0.1.0",
        "requires": ["live-status"],
        "scopes": ["workspace", "project"],
        "effort": "medium",
        "risk": "low",
        "core": false,
        "requires_worker_restart": false,
        "settings": {
            "budgetPerTask": {
                "type": "number",
                "default": 100.0,
                "min": 0.0,
                "max": 10000.0
            }
        },
        "ui": {
            "contributes": [
                {
                    "id": "cost-card",
                    "slot": "board.card.badge",
                    "label": "Chi phí",
                    "order": 40
                }
            ]
        }
    }"#;

    let manifest: Manifest = serde_json::from_str(json).expect("manifest should parse");
    assert_eq!(manifest.id, "cost-tracking");
    assert_eq!(manifest.group, ModuleGroup::Dashboard);
    assert_eq!(manifest.effort, ModuleEffort::Medium);
    assert_eq!(manifest.risk, ModuleRisk::Low);
    assert_eq!(
        manifest.scopes,
        vec![ModuleScope::Workspace, ModuleScope::Project]
    );
    assert_eq!(manifest.requires, vec!["live-status"]);
    assert!(!manifest.core);
    assert!(!manifest.requires_worker_restart);
    assert!(manifest.validate().is_ok());
}

#[test]
fn test_manifest_rejects_unknown_fields() {
    let json = r#"{
        "id": "unknown-test",
        "group": "safety",
        "name": "Test",
        "description": "Test desc",
        "version": "0.1.0",
        "requires": [],
        "scopes": ["project"],
        "effort": "light",
        "risk": "low",
        "unexpected_field": 123
    }"#;

    let res: Result<Manifest, _> = serde_json::from_str(json);
    assert!(res.is_err(), "must reject unexpected_field");
}

#[test]
fn test_manifest_missing_required_fields() {
    let json = r#"{
        "id": "missing-desc",
        "group": "safety",
        "name": "Test"
    }"#;

    let res: Result<Manifest, _> = serde_json::from_str(json);
    assert!(res.is_err(), "must fail when fields are missing");
}

#[test]
fn test_manifest_risk_note_required_for_non_low_risk() {
    let json = r#"{
        "id": "risky-mod",
        "group": "runtime",
        "name": "Risky",
        "description": "Risky module",
        "version": "0.1.0",
        "requires": [],
        "scopes": ["workspace"],
        "effort": "heavy",
        "risk": "high"
    }"#;

    let manifest: Manifest = serde_json::from_str(json).expect("should parse JSON");
    let err = manifest.validate().unwrap_err();
    assert!(matches!(err, ManifestError::MissingRiskNote));
}

#[test]
fn test_manifest_invalid_id() {
    let json = r#"{
        "id": "Invalid_ID_Uppercase",
        "group": "safety",
        "name": "Bad ID",
        "description": "Desc",
        "version": "0.1.0",
        "requires": [],
        "scopes": ["project"],
        "effort": "light",
        "risk": "low"
    }"#;

    let manifest: Manifest = serde_json::from_str(json).expect("should parse JSON");
    let err = manifest.validate().unwrap_err();
    assert!(matches!(err, ManifestError::InvalidId(_)));
}

#[test]
fn test_manifest_invalid_setting_min_max() {
    let mut setting = SettingSchema::new("number", serde_json::json!(50));
    setting.min = Some(100.0);
    setting.max = Some(10.0);
    assert!(setting.validate().is_err());
}

#[test]
fn test_manifest_duplicate_ui_contribution_id() {
    let json = r#"{
        "id": "ui-dup",
        "group": "dashboard",
        "name": "UI Duplicate",
        "description": "Desc",
        "version": "0.1.0",
        "requires": [],
        "scopes": ["project"],
        "effort": "light",
        "risk": "low",
        "ui": {
            "contributes": [
                { "id": "badge", "slot": "board.card.badge", "label": "B1", "order": 10 },
                { "id": "badge", "slot": "board.card.badge", "label": "B2", "order": 20 }
            ]
        }
    }"#;

    let manifest: Manifest = serde_json::from_str(json).expect("should parse JSON");
    let err = manifest.validate().unwrap_err();
    assert!(matches!(err, ManifestError::DuplicateContributionId(_)));
}

#[test]
fn test_table_name_convention() {
    assert_eq!(
        table_name("sample-hello", "note").expect("valid"),
        "m_sample_hello_note"
    );
    assert_eq!(
        table_name("cost-tracking", "task_budget").expect("valid"),
        "m_cost_tracking_task_budget"
    );
    assert!(table_name("bad name!", "note").is_err());
    assert!(table_name("sample-hello", "").is_err());
}
