use anki::progress::Progress;
use anki::progress::ProgressState;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::task::JoinHandle;

#[derive(Serialize, Debug, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProgressEvent {
    MediaSync {
        checked: usize,
        downloaded_files: usize,
        downloaded_deletions: usize,
        uploaded_files: usize,
        uploaded_deletions: usize,
    },
    NormalSync {
        stage: String,
        local_update: usize,
        local_remove: usize,
        remote_update: usize,
        remote_remove: usize,
    },
    FullSync {
        transferred_bytes: usize,
        total_bytes: usize,
    },
    Import {
        message: String,
    },
    Export {
        message: String,
    },
    Other,
}

fn convert(p: &Progress) -> ProgressEvent {
    match p {
        Progress::MediaSync(p) => ProgressEvent::MediaSync {
            checked: p.checked,
            downloaded_files: p.downloaded_files,
            downloaded_deletions: p.downloaded_deletions,
            uploaded_files: p.uploaded_files,
            uploaded_deletions: p.uploaded_deletions,
        },
        Progress::NormalSync(p) => ProgressEvent::NormalSync {
            stage: format!("{:?}", p.stage),
            local_update: p.local_update,
            local_remove: p.local_remove,
            remote_update: p.remote_update,
            remote_remove: p.remote_remove,
        },
        Progress::FullSync(p) => ProgressEvent::FullSync {
            transferred_bytes: p.transferred_bytes,
            total_bytes: p.total_bytes,
        },
        Progress::Import(_) => ProgressEvent::Import {
            message: "import".into(),
        },
        Progress::Export(_) => ProgressEvent::Export {
            message: "export".into(),
        },
        _ => ProgressEvent::Other,
    }
}

/// Spawn a tokio task that polls the shared progress state and emits
/// `progress` events to the frontend until the returned guard is dropped.
pub struct ProgressEmitter {
    handle: Option<JoinHandle<()>>,
}

impl ProgressEmitter {
    pub fn start(app: AppHandle, progress: Arc<Mutex<ProgressState>>) -> Self {
        // Clear stale progress before starting.
        if let Ok(mut g) = progress.lock() {
            g.last_progress = None;
        }
        let handle = tokio::spawn(async move {
            let mut last_emitted: Option<ProgressEvent> = None;
            loop {
                tokio::time::sleep(Duration::from_millis(120)).await;
                let snap = progress
                    .lock()
                    .ok()
                    .and_then(|g| g.last_progress)
                    .map(|p| convert(&p));
                if let Some(ev) = snap {
                    let differs = last_emitted
                        .as_ref()
                        .map(|prev| !same_event(prev, &ev))
                        .unwrap_or(true);
                    if differs {
                        let _ = app.emit("progress", ev.clone());
                        last_emitted = Some(ev);
                    }
                }
            }
        });
        Self {
            handle: Some(handle),
        }
    }
}

impl Drop for ProgressEmitter {
    fn drop(&mut self) {
        if let Some(h) = self.handle.take() {
            h.abort();
        }
    }
}

fn same_event(a: &ProgressEvent, b: &ProgressEvent) -> bool {
    use ProgressEvent::*;
    match (a, b) {
        (MediaSync { checked: a, .. }, MediaSync { checked: b, .. }) => a == b,
        (
            NormalSync {
                local_update: au,
                local_remove: ar,
                remote_update: bu,
                remote_remove: br,
                ..
            },
            NormalSync {
                local_update: au2,
                local_remove: ar2,
                remote_update: bu2,
                remote_remove: br2,
                ..
            },
        ) => au == au2 && ar == ar2 && bu == bu2 && br == br2,
        (
            FullSync { transferred_bytes: a, .. },
            FullSync { transferred_bytes: b, .. },
        ) => a == b,
        _ => std::mem::discriminant(a) == std::mem::discriminant(b),
    }
}
