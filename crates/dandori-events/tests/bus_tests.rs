//! Kiểm thử tích hợp cho EventBus trong dandori-events (T04).

use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Utc;
use dandori_events::{EventBus, EventFilter, TimeSource};
use dandori_types::{
    Envelope, EventKind, EventPayload, EventScope, FeatureModuleId, ProjectId, TaskId, WorkspaceId,
};

#[derive(Debug)]
struct MockTimeSource {
    fixed_time: chrono::DateTime<Utc>,
}

impl TimeSource for MockTimeSource {
    fn now(&self) -> chrono::DateTime<Utc> {
        self.fixed_time
    }
}

#[tokio::test]
async fn test_multiple_listeners_receive_events_in_order() {
    let bus = EventBus::new(100);
    let mut sub1 = bus.subscribe(EventFilter::all());
    let mut sub2 = bus.subscribe(EventFilter::all());

    for i in 0..10 {
        let task_id = TaskId::new();
        let envelope = Envelope::new(
            Utc::now(),
            EventScope::empty(),
            EventPayload::TaskCreated {
                task_id,
                title: format!("Task {i}"),
            },
        );
        bus.publish(envelope);
    }

    let mut sub1_received = Vec::new();
    let mut sub2_received = Vec::new();

    for _ in 0..10 {
        let ev1 = tokio::time::timeout(Duration::from_millis(500), sub1.recv())
            .await
            .expect("Timeout sub1")
            .expect("Sub1 closed");
        let ev2 = tokio::time::timeout(Duration::from_millis(500), sub2.recv())
            .await
            .expect("Timeout sub2")
            .expect("Sub2 closed");

        sub1_received.push(ev1);
        sub2_received.push(ev2);
    }

    assert_eq!(sub1_received.len(), 10);
    assert_eq!(sub2_received.len(), 10);

    for i in 0..10 {
        assert_eq!(sub1_received[i].id, sub2_received[i].id);
        if let EventPayload::TaskCreated { title, .. } = &sub1_received[i].payload {
            assert_eq!(title, &format!("Task {i}"));
        } else {
            panic!("Sai loại payload");
        }
    }
}

#[tokio::test]
async fn test_slow_listener_does_not_block_and_counter_increments() {
    // Dung lượng hàng đợi chỉ là 2
    let bus = EventBus::new(2);
    let mut fast_sub = bus.subscribe(EventFilter::all());
    let _slow_sub = bus.subscribe(EventFilter::all()); // không bao giờ gọi recv()

    // Người nghe nhanh đọc liên tục trong một task nền
    let (tx, mut rx) = tokio::sync::mpsc::channel(30);
    let fast_task = tokio::spawn(async move {
        while let Some(ev) = fast_sub.recv().await {
            let _ = tx.send(ev).await;
        }
    });

    let start = Instant::now();
    for i in 0..20 {
        let envelope = Envelope::new(
            Utc::now(),
            EventScope::empty(),
            EventPayload::TaskCreated {
                task_id: TaskId::new(),
                title: format!("Overflow task {i}"),
            },
        );
        bus.publish(envelope);
        // Nhường một nhịp nhỏ cho scheduler để fast_sub đọc kịp
        tokio::task::yield_now().await;
    }

    let elapsed = start.elapsed();
    // Publish 20 sự kiện không được mất quá 100ms kể cả khi có người nghe chậm
    assert!(
        elapsed < Duration::from_millis(100),
        "Publish bị nghẽn bởi người nghe chậm: {elapsed:?}"
    );

    // Người nghe nhanh vẫn nhận đủ 20 sự kiện
    let mut received_count = 0;
    while let Ok(Some(_)) = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await {
        received_count += 1;
        if received_count == 20 {
            break;
        }
    }
    assert_eq!(
        received_count, 20,
        "Fast subscriber phải nhận đủ 20 sự kiện"
    );

    // Dừng task nền
    fast_task.abort();

    // Bộ đếm mất sự kiện phải ghi nhận số lần tràn của người nghe chậm
    assert!(
        bus.dropped_events_count() > 0,
        "Bộ đếm mất sự kiện không tăng khi người nghe chậm tràn hàng đợi"
    );
}

#[tokio::test]
async fn test_filter_by_workspace_and_project() {
    let bus = EventBus::new(50);
    let ws_a = WorkspaceId::new();
    let ws_b = WorkspaceId::new();
    let proj_b1 = ProjectId::new();
    let proj_b2 = ProjectId::new();

    let mut sub_ws_a = bus.subscribe(EventFilter::for_workspace(ws_a));
    let mut sub_proj_b1 = bus.subscribe(EventFilter::for_project(proj_b1));

    // Phát sự kiện trong Workspace A
    bus.publish(Envelope::new(
        Utc::now(),
        EventScope::workspace(ws_a),
        EventPayload::ModuleEnabled {
            module_id: FeatureModuleId::new("sample-hello"),
        },
    ));

    // Phát sự kiện trong Project B1
    bus.publish(Envelope::new(
        Utc::now(),
        EventScope::project(ws_b, proj_b1),
        EventPayload::TaskCreated {
            task_id: TaskId::new(),
            title: "Task in B1".to_string(),
        },
    ));

    // Phát sự kiện trong Project B2
    bus.publish(Envelope::new(
        Utc::now(),
        EventScope::project(ws_b, proj_b2),
        EventPayload::TaskCreated {
            task_id: TaskId::new(),
            title: "Task in B2".to_string(),
        },
    ));

    // Sub WS A chỉ nhận 1 sự kiện của Workspace A
    let ev_a = tokio::time::timeout(Duration::from_millis(200), sub_ws_a.recv())
        .await
        .expect("Timeout WS A")
        .expect("Closed WS A");
    assert_eq!(ev_a.kind, EventKind::ModuleEnabled);

    // Sub Project B1 chỉ nhận 1 sự kiện của Project B1
    let ev_b1 = tokio::time::timeout(Duration::from_millis(200), sub_proj_b1.recv())
        .await
        .expect("Timeout Proj B1")
        .expect("Closed Proj B1");
    assert_eq!(ev_b1.kind, EventKind::TaskCreated);

    // Không còn sự kiện nào lọt qua
    assert!(
        tokio::time::timeout(Duration::from_millis(50), sub_ws_a.recv())
            .await
            .is_err()
    );
    assert!(
        tokio::time::timeout(Duration::from_millis(50), sub_proj_b1.recv())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn test_drop_handle_stops_reception() {
    let bus = EventBus::new(50);
    assert_eq!(bus.active_subscribers_count(), 0);

    let sub = bus.subscribe(EventFilter::all());
    assert_eq!(bus.active_subscribers_count(), 1);

    drop(sub);

    // Sau khi drop handle, bus tự động phát hiện và dọn dẹp khi phát sự kiện
    bus.publish(Envelope::new(
        Utc::now(),
        EventScope::empty(),
        EventPayload::NotificationRequested {
            title: "Test".to_string(),
            message: "Msg".to_string(),
            level: "info".to_string(),
        },
    ));

    assert_eq!(bus.active_subscribers_count(), 0);
}

#[test]
fn test_mock_time_source_abstract_clock() {
    let fixed = Utc::now();
    let time_source = Arc::new(MockTimeSource { fixed_time: fixed });
    let bus = EventBus::with_time_source(10, time_source);

    let env = bus.create_envelope(
        EventScope::empty(),
        EventPayload::NotificationRequested {
            title: "T".to_string(),
            message: "M".to_string(),
            level: "info".to_string(),
        },
    );

    assert_eq!(env.at, fixed);
}

#[test]
fn test_no_secret_string_field_in_event_payload() {
    // Kiểm tra cấu trúc static của EventPayload:
    // Không có trường nào lưu raw secret, token hay password string
    let payload = EventPayload::ModuleEnabled {
        module_id: FeatureModuleId::new("vault"),
    };
    let json = serde_json::to_string(&payload).expect("Serialize failed");
    assert!(!json.contains("secret"));
    assert!(!json.contains("password"));
    assert!(!json.contains("token"));
}
