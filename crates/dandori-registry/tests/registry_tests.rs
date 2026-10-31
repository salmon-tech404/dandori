use dandori_db::DbPool;
use dandori_registry::{
    ChangeRequest, FeatureRegistry, RegistryError, RegistryRole, RegistryScope,
};

async fn create_test_registry() -> Result<(DbPool, FeatureRegistry), Box<dyn std::error::Error>> {
    let pool = DbPool::in_memory().await?;
    let registry = FeatureRegistry::new(pool.inner().clone());
    registry.seed_modules_and_packs().await?;
    Ok((pool, registry))
}

// -----------------------------------------------------------------------------
// TIÊU CHÍ 1: BẢNG TEST TRẠNG THÁI HIỆU LỰC (01_KIEN_TRUC.md MỤC 5 & T06 AC1)
// -----------------------------------------------------------------------------

#[tokio::test]
async fn test_effective_state_1_no_value_is_disabled() {
    let (_pool, registry) = create_test_registry().await.expect("test registry");
    let state = registry
        .effective_state("sample-hello", "ws-1", "proj-1")
        .await
        .expect("query state");

    assert!(!state.enabled, "Chưa có cấu hình thì mặc định phải tắt");
    assert_eq!(state.source_scope, "default");
    assert!(!state.locked);
}

#[tokio::test]
async fn test_effective_state_2_system_only_enabled() {
    let (_pool, registry) = create_test_registry().await.expect("test registry");
    registry
        .change_state(ChangeRequest {
            module_id: "sample-hello".into(),
            scope: RegistryScope::System,
            enabled: true,
            locked: false,
            config: None,
            reason: "System enable".into(),
            performed_by: "user-admin".into(),
            actor_role: RegistryRole::SystemOwner,
            risk_acknowledged: false,
        })
        .await
        .expect("system enable");

    let state = registry
        .effective_state("sample-hello", "ws-1", "proj-1")
        .await
        .expect("query state");

    assert!(
        state.enabled,
        "Hệ thống bật thì dự án phải có trạng thái hiệu lực bật"
    );
    assert_eq!(state.source_scope, "system");
}

#[tokio::test]
async fn test_effective_state_3_workspace_disabled_overrides_system_enabled() {
    let (_pool, registry) = create_test_registry().await.expect("test registry");
    // 1. Hệ thống bật
    registry
        .change_state(ChangeRequest {
            module_id: "sample-hello".into(),
            scope: RegistryScope::System,
            enabled: true,
            locked: false,
            config: None,
            reason: "System enable".into(),
            performed_by: "user-admin".into(),
            actor_role: RegistryRole::SystemOwner,
            risk_acknowledged: false,
        })
        .await
        .expect("sys enable");

    // 2. Workspace tắt ghi đè
    registry
        .change_state(ChangeRequest {
            module_id: "sample-hello".into(),
            scope: RegistryScope::Workspace("ws-1".into()),
            enabled: false,
            locked: false,
            config: None,
            reason: "Workspace disable".into(),
            performed_by: "ws-admin".into(),
            actor_role: RegistryRole::WorkspaceAdmin,
            risk_acknowledged: false,
        })
        .await
        .expect("ws disable");

    let state = registry
        .effective_state("sample-hello", "ws-1", "proj-1")
        .await
        .expect("query state");

    assert!(!state.enabled, "Workspace tắt phải ghi đè hệ thống bật");
    assert_eq!(state.source_scope, "workspace");
}

#[tokio::test]
async fn test_effective_state_4_project_enabled_overrides_workspace_disabled() {
    let (_pool, registry) = create_test_registry().await.expect("test registry");
    // Workspace tắt
    registry
        .change_state(ChangeRequest {
            module_id: "sample-hello".into(),
            scope: RegistryScope::Workspace("ws-1".into()),
            enabled: false,
            locked: false,
            config: None,
            reason: "Workspace disable".into(),
            performed_by: "ws-admin".into(),
            actor_role: RegistryRole::WorkspaceAdmin,
            risk_acknowledged: false,
        })
        .await
        .expect("ws disable");

    // Dự án bật ghi đè
    registry
        .change_state(ChangeRequest {
            module_id: "sample-hello".into(),
            scope: RegistryScope::Project {
                workspace_id: "ws-1".into(),
                project_id: "proj-1".into(),
            },
            enabled: true,
            locked: false,
            config: None,
            reason: "Project enable".into(),
            performed_by: "proj-owner".into(),
            actor_role: RegistryRole::ProjectOwner,
            risk_acknowledged: false,
        })
        .await
        .expect("project enable");

    let state = registry
        .effective_state("sample-hello", "ws-1", "proj-1")
        .await
        .expect("query state");

    assert!(state.enabled, "Dự án bật phải ghi đè workspace tắt");
    assert_eq!(state.source_scope, "project");
}

#[tokio::test]
async fn test_effective_state_5_system_locked_enabled_rejects_project_disable() {
    let (_pool, registry) = create_test_registry().await.expect("test registry");
    // Hệ thống bật và LOCKED
    registry
        .change_state(ChangeRequest {
            module_id: "sample-hello".into(),
            scope: RegistryScope::System,
            enabled: true,
            locked: true,
            config: None,
            reason: "System locked enable".into(),
            performed_by: "user-admin".into(),
            actor_role: RegistryRole::SystemOwner,
            risk_acknowledged: false,
        })
        .await
        .expect("sys lock");

    // Dự án cố tắt -> PHẢI BỊ TỪ CHỐI
    let err = registry
        .change_state(ChangeRequest {
            module_id: "sample-hello".into(),
            scope: RegistryScope::Project {
                workspace_id: "ws-1".into(),
                project_id: "proj-1".into(),
            },
            enabled: false,
            locked: false,
            config: None,
            reason: "Project try disable".into(),
            performed_by: "proj-owner".into(),
            actor_role: RegistryRole::ProjectOwner,
            risk_acknowledged: false,
        })
        .await
        .unwrap_err();

    assert!(
        matches!(err, RegistryError::ScopeLocked { locked_at_scope, .. } if locked_at_scope == "system")
    );

    // Trạng thái hiệu lực vẫn là hệ thống bật
    let state = registry
        .effective_state("sample-hello", "ws-1", "proj-1")
        .await
        .expect("query state");
    assert!(state.enabled);
    assert!(state.locked);
    assert_eq!(state.source_scope, "system");
}

#[tokio::test]
async fn test_effective_state_6_workspace_locked_rejects_project_change() {
    let (_pool, registry) = create_test_registry().await.expect("test registry");
    // Workspace khóa bật
    registry
        .change_state(ChangeRequest {
            module_id: "sample-hello".into(),
            scope: RegistryScope::Workspace("ws-1".into()),
            enabled: true,
            locked: true,
            config: None,
            reason: "Workspace lock".into(),
            performed_by: "ws-admin".into(),
            actor_role: RegistryRole::WorkspaceAdmin,
            risk_acknowledged: false,
        })
        .await
        .expect("ws lock");

    // Dự án cố đổi -> BỊ TỪ CHỐI
    let err = registry
        .change_state(ChangeRequest {
            module_id: "sample-hello".into(),
            scope: RegistryScope::Project {
                workspace_id: "ws-1".into(),
                project_id: "proj-1".into(),
            },
            enabled: false,
            locked: false,
            config: None,
            reason: "Project try change".into(),
            performed_by: "proj-owner".into(),
            actor_role: RegistryRole::ProjectOwner,
            risk_acknowledged: false,
        })
        .await
        .unwrap_err();

    assert!(
        matches!(err, RegistryError::ScopeLocked { locked_at_scope, .. } if locked_at_scope == "workspace")
    );
}

#[tokio::test]
async fn test_effective_state_7_system_unlocked_workspace_locked_disabled_rejects_project_enable() {
    let (_pool, registry) = create_test_registry().await.expect("test registry");
    // Hệ thống bật nhưng KHÔNG khóa
    registry
        .change_state(ChangeRequest {
            module_id: "sample-hello".into(),
            scope: RegistryScope::System,
            enabled: true,
            locked: false,
            config: None,
            reason: "System enable".into(),
            performed_by: "user-admin".into(),
            actor_role: RegistryRole::SystemOwner,
            risk_acknowledged: false,
        })
        .await
        .expect("sys enable");

    // Workspace tắt VÀ KHÓA
    registry
        .change_state(ChangeRequest {
            module_id: "sample-hello".into(),
            scope: RegistryScope::Workspace("ws-1".into()),
            enabled: false,
            locked: true,
            config: None,
            reason: "Workspace lock disable".into(),
            performed_by: "ws-admin".into(),
            actor_role: RegistryRole::WorkspaceAdmin,
            risk_acknowledged: false,
        })
        .await
        .expect("ws lock disable");

    // Dự án cố bật -> BỊ TỪ CHỐI
    let err = registry
        .change_state(ChangeRequest {
            module_id: "sample-hello".into(),
            scope: RegistryScope::Project {
                workspace_id: "ws-1".into(),
                project_id: "proj-1".into(),
            },
            enabled: true,
            locked: false,
            config: None,
            reason: "Project try enable".into(),
            performed_by: "proj-owner".into(),
            actor_role: RegistryRole::ProjectOwner,
            risk_acknowledged: false,
        })
        .await
        .unwrap_err();

    assert!(
        matches!(err, RegistryError::ScopeLocked { locked_at_scope, .. } if locked_at_scope == "workspace")
    );
}

// -----------------------------------------------------------------------------
// TIÊU CHÍ 2: PLAN_ENABLE & PLAN_DISABLE VỚI PHỤ THUỘC BẮC CẦU
// -----------------------------------------------------------------------------

#[tokio::test]
async fn test_plan_enable_and_disable_transitive_dependencies() {
    let (_pool, registry) = create_test_registry().await.expect("test registry");

    // Đăng ký 3 module bắc cầu: mod-3 -> mod-2 -> mod-1
    registry
        .register_test_module("mod-1", vec![], false, false)
        .await
        .expect("reg mod-1");
    registry
        .register_test_module("mod-2", vec!["mod-1".into()], false, false)
        .await
        .expect("reg mod-2");
    registry
        .register_test_module("mod-3", vec!["mod-2".into()], false, true) // mod-3 có risk cao
        .await
        .expect("reg mod-3");

    // Lập kế hoạch bật mod-3
    let enable_plan = registry
        .plan_enable("mod-3", &RegistryScope::Workspace("ws-1".into()))
        .await
        .expect("plan enable");

    assert_eq!(
        enable_plan.will_enable_with,
        vec!["mod-1", "mod-2", "mod-3"],
        "Phải sắp xếp theo thứ tự topo từ gốc đến ngọn"
    );
    assert!(
        enable_plan.needs_risk_ack,
        "mod-3 có risk cao nên cần xác nhận"
    );

    // Thực hiện bật toàn bộ chuỗi
    for m in &enable_plan.will_enable_with {
        registry
            .change_state(ChangeRequest {
                module_id: m.clone(),
                scope: RegistryScope::Workspace("ws-1".into()),
                enabled: true,
                locked: false,
                config: None,
                reason: "Enable chain".into(),
                performed_by: "ws-admin".into(),
                actor_role: RegistryRole::WorkspaceAdmin,
                risk_acknowledged: true,
            })
            .await
            .expect("enable item in chain");
    }

    // Lập kế hoạch tắt mod-1 (mod-2 và mod-3 đang cần nó)
    let disable_plan = registry
        .plan_disable("mod-1", &RegistryScope::Workspace("ws-1".into()))
        .await
        .expect("plan disable");

    assert!(disable_plan.will_break.contains(&"mod-2".to_string()));
    assert!(disable_plan.will_break.contains(&"mod-3".to_string()));
}

// -----------------------------------------------------------------------------
// TIÊU CHÍ 3: PHÂN QUYỀN THEO CẤP
// -----------------------------------------------------------------------------

#[tokio::test]
async fn test_role_based_permissions_across_scopes() {
    let (_pool, registry) = create_test_registry().await.expect("test registry");

    // 1. Quản trị viên Workspace không được đổi cấp Hệ thống
    let err_sys = registry
        .change_state(ChangeRequest {
            module_id: "sample-hello".into(),
            scope: RegistryScope::System,
            enabled: true,
            locked: false,
            config: None,
            reason: "Unauth".into(),
            performed_by: "user".into(),
            actor_role: RegistryRole::WorkspaceAdmin,
            risk_acknowledged: false,
        })
        .await
        .unwrap_err();
    assert!(matches!(err_sys, RegistryError::PermissionDenied { .. }));

    // 2. Chủ dự án không được đổi cấp Workspace
    let err_ws = registry
        .change_state(ChangeRequest {
            module_id: "sample-hello".into(),
            scope: RegistryScope::Workspace("ws-1".into()),
            enabled: true,
            locked: false,
            config: None,
            reason: "Unauth".into(),
            performed_by: "user".into(),
            actor_role: RegistryRole::ProjectOwner,
            risk_acknowledged: false,
        })
        .await
        .unwrap_err();
    assert!(matches!(err_ws, RegistryError::PermissionDenied { .. }));

    // 3. Member không được đổi cấp Dự án
    let err_proj = registry
        .change_state(ChangeRequest {
            module_id: "sample-hello".into(),
            scope: RegistryScope::Project {
                workspace_id: "ws-1".into(),
                project_id: "proj-1".into(),
            },
            enabled: true,
            locked: false,
            config: None,
            reason: "Unauth".into(),
            performed_by: "user".into(),
            actor_role: RegistryRole::Member,
            risk_acknowledged: false,
        })
        .await
        .unwrap_err();
    assert!(matches!(err_proj, RegistryError::PermissionDenied { .. }));
}

// -----------------------------------------------------------------------------
// TIÊU CHÍ 4 & 5: NHẬT KÝ KIỂM TOÁN VÀ HOÀN TÁC
// -----------------------------------------------------------------------------

#[tokio::test]
async fn test_audit_logging_sequence() {
    let (_pool, registry) = create_test_registry().await.expect("test registry");
    let scope = RegistryScope::Workspace("ws-audit".into());

    // 1. Bật module
    registry
        .change_state(ChangeRequest {
            module_id: "sample-hello".into(),
            scope: scope.clone(),
            enabled: true,
            locked: false,
            config: None,
            reason: "Initial enable".into(),
            performed_by: "admin-1".into(),
            actor_role: RegistryRole::WorkspaceAdmin,
            risk_acknowledged: false,
        })
        .await
        .expect("enable");

    // 2. Đổi cấu hình
    registry
        .change_state(ChangeRequest {
            module_id: "sample-hello".into(),
            scope: scope.clone(),
            enabled: true,
            locked: false,
            config: Some(serde_json::json!({ "greeting": "Hi" })),
            reason: "Update greeting".into(),
            performed_by: "admin-1".into(),
            actor_role: RegistryRole::WorkspaceAdmin,
            risk_acknowledged: false,
        })
        .await
        .expect("change config");

    let audits = registry.list_audits("sample-hello").await.expect("audits");
    assert_eq!(audits.len(), 2);
    assert_eq!(audits[0].action, "enable");
    assert_eq!(audits[1].action, "configure");
}

#[tokio::test]
async fn test_audit_undo_restores_previous_state() {
    let (_pool, registry) = create_test_registry().await.expect("test registry");
    let scope = RegistryScope::Workspace("ws-undo".into());

    // 1. Bật module
    registry
        .change_state(ChangeRequest {
            module_id: "sample-hello".into(),
            scope: scope.clone(),
            enabled: true,
            locked: false,
            config: None,
            reason: "Enable".into(),
            performed_by: "admin-1".into(),
            actor_role: RegistryRole::WorkspaceAdmin,
            risk_acknowledged: false,
        })
        .await
        .expect("enable");

    // 2. Tắt module
    registry
        .change_state(ChangeRequest {
            module_id: "sample-hello".into(),
            scope: scope.clone(),
            enabled: false,
            locked: false,
            config: None,
            reason: "Disable".into(),
            performed_by: "admin-1".into(),
            actor_role: RegistryRole::WorkspaceAdmin,
            risk_acknowledged: false,
        })
        .await
        .expect("disable");

    // 3. Hoàn tác -> module phải bật lại
    registry
        .undo_last_change(
            "sample-hello",
            &scope,
            "admin-1",
            RegistryRole::WorkspaceAdmin,
        )
        .await
        .expect("undo");

    let state = registry
        .effective_state("sample-hello", "ws-undo", "proj-x")
        .await
        .expect("state");
    assert!(state.enabled, "Hoàn tác xong module phải bật lại");

    let audits = registry.list_audits("sample-hello").await.expect("audits");
    assert_eq!(audits.len(), 3);
    assert_eq!(audits.last().expect("last").action, "rollback");
}

// -----------------------------------------------------------------------------
// TIÊU CHÍ 6: MODULE CORE KHÔNG THỂ BỊ TẮT
// -----------------------------------------------------------------------------

#[tokio::test]
async fn test_core_module_cannot_be_disabled() {
    let (_pool, registry) = create_test_registry().await.expect("test registry");

    // Đăng ký module core
    registry
        .register_test_module("auth-login", vec![], true, false)
        .await
        .expect("reg core module");

    let err = registry
        .change_state(ChangeRequest {
            module_id: "auth-login".into(),
            scope: RegistryScope::System,
            enabled: false,
            locked: false,
            config: None,
            reason: "Try disable core".into(),
            performed_by: "admin".into(),
            actor_role: RegistryRole::SystemOwner,
            risk_acknowledged: false,
        })
        .await
        .unwrap_err();

    assert!(matches!(
        err,
        RegistryError::CoreModuleCannotBeDisabled(id) if id == "auth-login"
    ));
}
