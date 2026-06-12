use serde::{Serialize, Serializer};
use std::error::Error;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("collection is not open")]
    CollectionNotOpen,

    /// ユーザー入力の検証エラー。メッセージはそのままフロントに表示される。
    #[error("{0}")]
    InvalidInput(String),

    #[error(transparent)]
    Anki(#[from] anki::error::AnkiError),

    /// コマンドが col.storage.db() に直接撃つ生 SQL のエラー。
    #[error(transparent)]
    Db(#[from] rusqlite::Error),

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collection_not_open_serializes_to_its_display_message() {
        let err = AppError::CollectionNotOpen;
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, "\"collection is not open\"");
    }

    #[test]
    fn anyhow_chain_is_flattened_into_full_message() {
        // Build a deliberately layered anyhow chain so we can verify each
        // layer ends up in the serialized string. This is the path Tauri
        // takes to surface backend errors to the frontend.
        let inner = anyhow::anyhow!("disk full");
        let wrapped = inner.context("write credentials").context("save sync state");
        let err = AppError::Anyhow(wrapped);
        let msg = serde_json::to_string(&err).unwrap();
        assert!(msg.contains("save sync state"), "got: {msg}");
        assert!(msg.contains("write credentials"), "got: {msg}");
        assert!(msg.contains("disk full"), "got: {msg}");
    }

    #[test]
    fn invalid_input_serializes_to_its_message_verbatim() {
        // フロントの表示文字列なので、variant 移行で文言が変わらないこと。
        let err = AppError::InvalidInput("deck name is empty".into());
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, "\"deck name is empty\"");
    }

    #[test]
    fn db_error_serializes_to_rusqlite_display() {
        let err = AppError::Db(rusqlite::Error::QueryReturnedNoRows);
        let json = serde_json::to_string(&err).unwrap();
        // rusqlite の Display と一致する (transparent)。
        assert_eq!(
            json,
            serde_json::to_string(&rusqlite::Error::QueryReturnedNoRows.to_string()).unwrap()
        );
    }

    #[test]
    fn duplicate_segments_are_not_repeated() {
        // full_message should skip a source whose Display is already a
        // substring of the accumulated message — otherwise wrapping the same
        // text twice would produce "X: X" tails.
        let inner = anyhow::anyhow!("boom");
        let wrapped = inner.context("boom");
        let err = AppError::Anyhow(wrapped);
        let msg = serde_json::to_string(&err).unwrap();
        // Quoted exactly once.
        assert_eq!(msg.matches("boom").count(), 1, "got: {msg}");
    }
}
