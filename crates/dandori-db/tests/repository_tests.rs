//! Kiểm thử tích hợp CRUD cho các repository và kiểm tra cô lập sở hữu (AC2 và AC3).

use chrono::Utc;
use dandori_db::{DbError, DbPool};
use dandori_types::{
    FeatureAudit, FeatureAuditId, FeatureModule, FeatureModuleId, FeaturePack, FeaturePackId,
    FeatureSetting, Project, ProjectId, Run, RunEvent, RunEventId, RunId, Session, SessionId, Task,
    TaskId, User, UserId, Workspace, WorkspaceId, WorkspaceMember,
};

async fn setup_ws(pool: &DbPool) -> Result<WorkspaceId, DbError> {
    let ws_id = WorkspaceId::new();
    let ws = Workspace {
        id: ws_id,
        name: "WS".to_string(),
        created_at: Utc::now(),
    };
    pool.workspace_repo().create_workspace(&ws).await?;
    Ok(ws_id)
}

async fn setup_proj(pool: &DbPool) -> Result<(WorkspaceId, ProjectId), DbError> {
    let ws_id = setup_ws(pool).await?;
    let proj_id = ProjectId::new();
    let proj = Project {
        id: proj_id,
        workspace_id: ws_id,
        name: "P".to_string(),
        repo_path: "r".to_string(),
        default_branch: "m".to_string(),
        created_at: Utc::now(),
    };
    pool.project_repo().create_project(&proj).await?;
    Ok((ws_id, proj_id))
}

async fn setup_task(pool: &DbPool) -> Result<(ProjectId, TaskId), DbError> {
    let (_, proj_id) = setup_proj(pool).await?;
    let task_id = TaskId::new();
    let task = Task {
        id: task_id,
        project_id: proj_id,
        title: "T".to_string(),
        description: "D".to_string(),
        status: "todo".to_string(),
        lead_agent: "l".to_string(),
        worker_agent: "w".to_string(),
        workdir_path: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    pool.task_repo().create_task(&task).await?;
    Ok((proj_id, task_id))
}

#[tokio::test]
async fn test_user_crud() {
    let pool = DbPool::in_memory().await.expect("Tạo pool thất bại");
    let repo = pool.user_repo();
    let id = UserId::new();
    let user = User {
        id,
        username: "testuser".to_string(),
        password_hash: "hash_123".to_string(),
        system_role: "admin".to_string(),
        created_at: Utc::now(),
    };

    repo.create_user(&user).await.expect("Tạo user thất bại");
    let by_id = repo.get_user_by_id(&id).await.expect("Lỗi").expect("None");
    assert_eq!(by_id.username, "testuser");

    let by_name = repo
        .get_user_by_username("testuser")
        .await
        .expect("Lỗi")
        .expect("None");
    assert_eq!(by_name.id, id);

    repo.update_user_role(&id, "user")
        .await
        .expect("Cập nhật thất bại");
    let updated = repo.get_user_by_id(&id).await.expect("Lỗi").expect("None");
    assert_eq!(updated.system_role, "user");

    repo.delete_user(&id).await.expect("Xóa thất bại");
    assert!(repo.get_user_by_id(&id).await.expect("Lỗi").is_none());
}

#[tokio::test]
async fn test_session_crud() {
    let pool = DbPool::in_memory().await.expect("Tạo pool thất bại");
    let user_id = UserId::new();
    let u = User {
        id: user_id,
        username: "s_user".to_string(),
        password_hash: "h".to_string(),
        system_role: "user".to_string(),
        created_at: Utc::now(),
    };
    pool.user_repo()
        .create_user(&u)
        .await
        .expect("Tạo user thất bại");

    let repo = pool.session_repo();
    let sess_id = SessionId::new();
    let session = Session {
        id: sess_id,
        user_id,
        expires_at: Utc::now() + chrono::Duration::hours(2),
        token_hash: "h".to_string(),
    };

    repo.create_session(&session)
        .await
        .expect("Tạo session thất bại");
    let fetched = repo
        .get_session(&sess_id)
        .await
        .expect("Lỗi")
        .expect("None");
    assert_eq!(fetched.user_id, user_id);

    repo.delete_session(&sess_id)
        .await
        .expect("Xóa session thất bại");
    assert!(repo.get_session(&sess_id).await.expect("Lỗi").is_none());
}

#[tokio::test]
async fn test_workspace_crud() {
    let pool = DbPool::in_memory().await.expect("Tạo pool thất bại");
    let repo = pool.workspace_repo();
    let ws_id = WorkspaceId::new();
    let ws = Workspace {
        id: ws_id,
        name: "WS 1".to_string(),
        created_at: Utc::now(),
    };

    repo.create_workspace(&ws).await.expect("Tạo ws thất bại");
    let fetched = repo
        .get_workspace(&ws_id)
        .await
        .expect("Lỗi")
        .expect("None");
    assert_eq!(fetched.name, "WS 1");

    repo.update_workspace_name(&ws_id, "WS Renamed")
        .await
        .expect("Đổi tên thất bại");
    repo.delete_workspace(&ws_id).await.expect("Xóa thất bại");
    assert!(repo.get_workspace(&ws_id).await.expect("Lỗi").is_none());
}

#[tokio::test]
async fn test_workspace_member_crud() {
    let pool = DbPool::in_memory().await.expect("Tạo pool thất bại");
    let ws_id = setup_ws(&pool).await.expect("Setup thất bại");
    let user_id = UserId::new();
    let u = User {
        id: user_id,
        username: "m_user".to_string(),
        password_hash: "h".to_string(),
        system_role: "user".to_string(),
        created_at: Utc::now(),
    };
    pool.user_repo()
        .create_user(&u)
        .await
        .expect("Tạo user thất bại");

    let repo = pool.workspace_repo();
    let member = WorkspaceMember {
        workspace_id: ws_id,
        user_id,
        role: "admin".to_string(),
    };
    repo.add_member(&member)
        .await
        .expect("Thêm member thất bại");
    assert_eq!(repo.list_members(&ws_id).await.expect("Lỗi").len(), 1);

    repo.remove_member(&ws_id, &user_id)
        .await
        .expect("Xóa member thất bại");
    assert_eq!(repo.list_members(&ws_id).await.expect("Lỗi").len(), 0);
}

#[tokio::test]
async fn test_project_crud() {
    let pool = DbPool::in_memory().await.expect("Tạo pool thất bại");
    let ws_id = setup_ws(&pool).await.expect("Setup thất bại");
    let repo = pool.project_repo();
    let proj_id = ProjectId::new();
    let proj = Project {
        id: proj_id,
        workspace_id: ws_id,
        name: "P1".to_string(),
        repo_path: "r".to_string(),
        default_branch: "main".to_string(),
        created_at: Utc::now(),
    };

    repo.create_project(&proj).await.expect("Tạo proj thất bại");
    let fetched = repo
        .get_project(&ws_id, &proj_id)
        .await
        .expect("Lỗi")
        .expect("None");
    assert_eq!(fetched.name, "P1");

    repo.update_project(&ws_id, &proj_id, "P2", "dev")
        .await
        .expect("Sửa thất bại");
    assert_eq!(repo.list_projects(&ws_id).await.expect("Lỗi").len(), 1);

    repo.delete_project(&ws_id, &proj_id)
        .await
        .expect("Xóa thất bại");
    assert!(
        repo.get_project(&ws_id, &proj_id)
            .await
            .expect("Lỗi")
            .is_none()
    );
}

#[tokio::test]
async fn test_task_crud() {
    let pool = DbPool::in_memory().await.expect("Tạo pool thất bại");
    let (_, proj_id) = setup_proj(&pool).await.expect("Setup thất bại");
    let repo = pool.task_repo();
    let task_id = TaskId::new();
    let task = Task {
        id: task_id,
        project_id: proj_id,
        title: "T1".to_string(),
        description: "D".to_string(),
        status: "todo".to_string(),
        lead_agent: "l".to_string(),
        worker_agent: "w".to_string(),
        workdir_path: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    repo.create_task(&task).await.expect("Tạo task thất bại");
    let fetched = repo
        .get_task(&proj_id, &task_id)
        .await
        .expect("Lỗi")
        .expect("None");
    assert_eq!(fetched.title, "T1");

    repo.update_task_status(&proj_id, &task_id, "done")
        .await
        .expect("Sửa status thất bại");
    let updated = repo
        .get_task(&proj_id, &task_id)
        .await
        .expect("Lỗi")
        .expect("None");
    assert_eq!(updated.status, "done");

    assert_eq!(repo.list_tasks(&proj_id).await.expect("Lỗi").len(), 1);
    repo.delete_task(&proj_id, &task_id)
        .await
        .expect("Xóa task thất bại");
    assert!(
        repo.get_task(&proj_id, &task_id)
            .await
            .expect("Lỗi")
            .is_none()
    );
}

#[tokio::test]
async fn test_run_and_event_crud() {
    let pool = DbPool::in_memory().await.expect("Tạo pool thất bại");
    let (_, task_id) = setup_task(&pool).await.expect("Setup thất bại");
    let run_repo = pool.run_repo();
    let run_id = RunId::new();
    let run = Run {
        id: run_id,
        task_id,
        agent: "claude".to_string(),
        account: "a1".to_string(),
        status: "running".to_string(),
        started_at: Utc::now(),
        finished_at: None,
        exit_code: None,
    };
    run_repo.create_run(&run).await.expect("Tạo run thất bại");

    let ev_repo = pool.run_event_repo();
    let ev = RunEvent {
        id: RunEventId::new(),
        run_id,
        seq: 1,
        event_type: "log".to_string(),
        data: "{}".to_string(),
        created_at: Utc::now(),
    };
    ev_repo
        .create_run_event(&ev)
        .await
        .expect("Tạo event thất bại");
    assert_eq!(
        ev_repo.list_run_events(&run_id).await.expect("Lỗi").len(),
        1
    );

    run_repo
        .update_run_status(&task_id, &run_id, "success", Some(Utc::now()), Some(0))
        .await
        .expect("Sửa run thất bại");
    assert_eq!(run_repo.list_runs(&task_id).await.expect("Lỗi").len(), 1);
    run_repo
        .delete_run(&task_id, &run_id)
        .await
        .expect("Xóa run thất bại");
}

#[tokio::test]
async fn test_ownership_isolation_project() {
    let pool = DbPool::in_memory().await.expect("Tạo pool thất bại");
    let ws_a = setup_ws(&pool).await.expect("Setup thất bại");
    let ws_b = setup_ws(&pool).await.expect("Setup thất bại");
    let proj_a = ProjectId::new();
    let p = Project {
        id: proj_a,
        workspace_id: ws_a,
        name: "PA".to_string(),
        repo_path: "a".to_string(),
        default_branch: "m".to_string(),
        created_at: Utc::now(),
    };
    pool.project_repo()
        .create_project(&p)
        .await
        .expect("Tạo PA thất bại");

    // Workspace B không đọc được Project của Workspace A
    assert!(
        pool.project_repo()
            .get_project(&ws_b, &proj_a)
            .await
            .expect("Lỗi")
            .is_none()
    );
}

#[tokio::test]
async fn test_ownership_isolation_task() {
    let pool = DbPool::in_memory().await.expect("Tạo pool thất bại");
    let (_, proj_a) = setup_proj(&pool).await.expect("Setup thất bại");
    let (_, proj_b) = setup_proj(&pool).await.expect("Setup thất bại");
    let task_a = TaskId::new();
    let t = Task {
        id: task_a,
        project_id: proj_a,
        title: "TA".to_string(),
        description: "D".to_string(),
        status: "todo".to_string(),
        lead_agent: "l".to_string(),
        worker_agent: "w".to_string(),
        workdir_path: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    pool.task_repo()
        .create_task(&t)
        .await
        .expect("Tạo TA thất bại");

    // Project B không đọc được Task của Project A
    assert!(
        pool.task_repo()
            .get_task(&proj_b, &task_a)
            .await
            .expect("Lỗi")
            .is_none()
    );
    // Project B không cập nhật được Task của Project A
    assert!(
        pool.task_repo()
            .update_task_status(&proj_b, &task_a, "done")
            .await
            .is_err()
    );
}

#[tokio::test]
async fn test_feature_module_and_setting_crud() {
    let pool = DbPool::in_memory().await.expect("Tạo pool thất bại");
    let feat_repo = pool.feature_repo();
    let mod_id = FeatureModuleId::new("sample-hello");
    let feat_mod = FeatureModule {
        id: mod_id.clone(),
        group_name: "safety".to_string(),
        name: "Sample".to_string(),
        description: "Desc".to_string(),
        version: "0.1.0".to_string(),
        requires: vec![],
        scopes: vec!["system".to_string()],
        risk: "low".to_string(),
        effort: "low".to_string(),
        core: false,
    };
    feat_repo
        .upsert_module(&feat_mod)
        .await
        .expect("Upsert module thất bại");
    assert!(feat_repo.get_module(&mod_id).await.expect("Lỗi").is_some());

    let setting = FeatureSetting {
        module_id: mod_id.clone(),
        scope_kind: "system".to_string(),
        scope_id: "global".to_string(),
        enabled: true,
        locked: false,
        config: "{}".to_string(),
        changed_by: "admin".to_string(),
        changed_at: Utc::now(),
    };
    feat_repo
        .upsert_setting(&setting)
        .await
        .expect("Upsert setting thất bại");
    assert!(
        feat_repo
            .get_setting(&mod_id, "system", "global")
            .await
            .expect("Lỗi")
            .is_some()
    );
}

#[tokio::test]
async fn test_feature_pack_and_audit_crud() {
    let pool = DbPool::in_memory().await.expect("Tạo pool thất bại");
    let feat_repo = pool.feature_repo();
    let pack_id = FeaturePackId::new("solo");
    let pack = FeaturePack {
        id: pack_id.clone(),
        name: "S".to_string(),
        modules: vec![],
    };
    feat_repo
        .upsert_pack(&pack)
        .await
        .expect("Upsert pack thất bại");
    assert!(feat_repo.get_pack(&pack_id).await.expect("Lỗi").is_some());

    let audit = FeatureAudit {
        id: FeatureAuditId::new(),
        module_id: FeatureModuleId::new("sample-hello"),
        scope_kind: "system".to_string(),
        scope_id: "global".to_string(),
        action: "enable".to_string(),
        performed_by: "admin".to_string(),
        reason: "init".to_string(),
        created_at: Utc::now(),
    };
    feat_repo
        .create_audit(&audit)
        .await
        .expect("Tạo audit thất bại");
    assert_eq!(
        feat_repo
            .list_audits(&FeatureModuleId::new("sample-hello"))
            .await
            .expect("Lỗi")
            .len(),
        1
    );
}
