use anki::collection::Collection;
use anki::prelude::CardId;
use anki::progress::ProgressState;
use anki::scheduler::states::SchedulingStates;
use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Instant;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct CachedQueueEntry {
    pub card_id: CardId,
    pub states: SchedulingStates,
    /// Wall-clock when the card was handed to the frontend, used to
    /// compute `milliseconds_taken` if the frontend doesn't pass it.
    pub shown_at: Instant,
}

pub struct AppState {
    pub col: Mutex<Option<Collection>>,
    pub col_path: Mutex<Option<PathBuf>>,
    pub http: reqwest::Client,
    /// Cached state for the card currently shown to the frontend;
    /// needed to construct CardAnswer when the user picks a rating.
    pub last_queued: Mutex<Option<CachedQueueEntry>>,
    /// Shared with every Collection we open so a background polling task
    /// can read sync/import/export progress.
    pub progress: Arc<StdMutex<ProgressState>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            col: Mutex::new(None),
            col_path: Mutex::new(None),
            http: reqwest::Client::new(),
            last_queued: Mutex::new(None),
            progress: Arc::new(StdMutex::new(ProgressState::default())),
        }
    }
}
