//! Tiến trình nền và bộ lắng nghe sự kiện của module `diff-preview`.

use std::sync::Arc;
use tokio_util::sync::CancellationToken;

use crate::manager::PreviewManager;
use dandori_module_api::{EventBus, EventFilter};
use dandori_types::{EventKind, EventPayload};

/// Lắng nghe các sự kiện thay đổi trạng thái task để tự động dọn dẹp tiến trình xem trước.
pub async fn run_preview_worker(
    event_bus: Arc<EventBus>,
    manager: Arc<PreviewManager>,
    cancel_token: CancellationToken,
) {
    let filter = EventFilter::for_kinds(vec![EventKind::TaskStatusChanged]);
    let mut sub = event_bus.subscribe(filter);

    loop {
        tokio::select! {
            () = cancel_token.cancelled() => {
                manager.stop_all().await;
                break;
            }
            envelope_opt = sub.recv() => {
                match envelope_opt {
                    Some(env) => {
                        handle_task_event(&manager, &env).await;
                    }
                    None => {
                        manager.stop_all().await;
                        break;
                    }
                }
            }
        }
    }
}

async fn handle_task_event(manager: &PreviewManager, env: &dandori_types::Envelope) {
    let EventPayload::TaskStatusChanged {
        task_id,
        new_status,
        ..
    } = &env.payload
    else {
        return;
    };
    if matches!(new_status.as_str(), "done" | "cancelled" | "failed") {
        manager.stop_preview(&task_id.to_string()).await;
    }
}
