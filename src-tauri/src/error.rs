use serde::{Serialize, Serializer};
use std::error::Error;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("collection is not open")]
    CollectionNotOpen,

    #[error(transparent)]
    Anki(#[from] anki::error::AnkiError),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl AppError {
    /// Format the error chain. AnkiError variants like `SyncError { source }`
    /// only show the variant name in their Display impl; the actual details
    /// live in the `source` chain via std::error::Error.
    fn full_message(&self) -> String {
        let mut s = self.to_string();
        let mut current: Option<&dyn Error> = self.source();
        while let Some(src) = current {
            let msg = src.to_string();
            if !msg.is_empty() && !s.contains(&msg) {
                s.push_str(": ");
                s.push_str(&msg);
            }
            current = src.source();
        }
        s
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.full_message())
    }
}

pub type AppResult<T> = Result<T, AppError>;
