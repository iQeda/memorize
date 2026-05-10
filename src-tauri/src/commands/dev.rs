//! Dev-only commands. Compiled out of release builds via `#![cfg(...)]`.
//!
//! Production builds prompt the user to pick a real `.anki2` from the welcome
//! screen. That makes hands-on dev iteration painful: every `pnpm tauri dev`
//! launch starts at an empty collection, so reproducing UI bugs that depend on
//! real card data (e.g. browse search) requires re-picking a collection each
//! time. This module's `bootstrap_dev_collection` opens a stable path under
//! the repo (`.memorize-dev/collection.anki2`), seeding a small English vocab
//! deck on first use, so dev always lands in a usable state.

#![cfg(debug_assertions)]
// Tauri's #[tauri::command] + cfg-gated registration in `generate_handler!`
// confuses rustc's dead_code lint (it can't see through the macro indirection),
// so the seed constants and entry point look unused even though the runtime
// registers and calls them.
#![allow(dead_code)]

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use anki::collection::{Collection, CollectionBuilder};
use anki::notes::Note;
use anki::prelude::DeckId;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::State;

fn dev_collection_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or(manifest_dir);
    repo_root.join(".memorize-dev").join("collection.anki2")
}

const DECK_NAME: &str = "Dev Deck";

// Words chosen so common substrings (`ex`, `ci`, `el`, `for`, `gar`, `har`)
// each match multiple notes — useful for verifying browse search filters.
const SAMPLE_NOTES: &[(&str, &str)] = &[
    ("example", "an instance illustrating something"),
    ("exit", "to leave a place"),
    ("exam", "a formal test"),
    ("exotic", "from another part of the world"),
    ("exact", "precise and accurate"),
    ("expand", "to make larger"),
    ("export", "to send goods abroad"),
    ("circle", "a round shape"),
    ("citrus", "a family of acidic fruits"),
    ("civil", "polite, related to citizens"),
    ("cinema", "a movie theater"),
    ("circuit", "a closed electrical loop"),
    ("digital", "using digits, computer-related"),
    ("dilemma", "a difficult choice"),
    ("diary", "a daily journal"),
    ("district", "a defined administrative area"),
    ("elephant", "a large mammal with a trunk"),
    ("elegant", "graceful and stylish"),
    ("electric", "powered by electricity"),
    ("evening", "the late part of the day"),
    ("forest", "a large area of trees"),
    ("fortune", "wealth, or luck"),
    ("folder", "a container for documents"),
    ("format", "a layout or arrangement"),
    ("garden", "a cultivated outdoor space"),
    ("global", "worldwide"),
    ("gravity", "the force pulling toward Earth"),
    ("harbor", "a sheltered port"),
    ("harmony", "agreement or pleasant combination"),
    ("hostile", "unfriendly, antagonistic"),
];

#[derive(Serialize, Debug)]
pub struct DevBootstrapInfo {
    pub path: String,
    pub seeded: bool,
    pub note_count: usize,
}

/// Add `SAMPLE_NOTES` to `col` only if the collection has no notes yet.
/// Returns `true` when a seed was inserted, `false` when the collection
/// already had content (the function is idempotent — re-running on an
/// already-seeded collection is a no-op).
fn seed_dev_notes_if_empty(col: &mut Collection) -> AppResult<bool> {
    if !col.search_notes_unordered("")?.is_empty() {
        return Ok(false);
    }
    let deck = col.get_or_create_normal_deck(DECK_NAME)?;
    let deck_id = DeckId(deck.id.0);

    // Pick the first non-cloze stock notetype with at least 2 fields. In a
    // freshly-built rslib collection this is "Basic" (or its localized
    // equivalent) with [Front, Back]. Positional set_field works regardless
    // of locale.
    let basic_nt = col
        .get_all_notetypes()?
        .into_iter()
        .find(|nt| nt.config.kind == 0 && nt.fields.len() >= 2)
        .ok_or_else(|| {
            AppError::Anyhow(anyhow::anyhow!(
                "no normal notetype with >=2 fields in fresh collection"
            ))
        })?;

    for (front, back) in SAMPLE_NOTES {
        let mut note = Note::new(&basic_nt);
        note.set_field(0, *front)?;
        note.set_field(1, *back)?;
        col.add_note(&mut note, deck_id)?;
    }
    Ok(true)
}

#[tauri::command]
pub async fn bootstrap_dev_collection(
    state: State<'_, AppState>,
) -> AppResult<DevBootstrapInfo> {
    let path = dev_collection_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::Anyhow(anyhow::Error::new(e).context("create .memorize-dev dir")))?;
    }

    let col = CollectionBuilder::new(&path)
        .set_shared_progress_state(state.progress.clone())
        .build()?;

    let mut guard = state.col.lock().await;
    if let Some(prev) = guard.take() {
        let _ = prev.close(None);
    }
    *guard = Some(col);
    *state.col_path.lock().await = Some(path.clone());

    let col = guard.as_mut().expect("just inserted");
    let seeded = seed_dev_notes_if_empty(col)?;
    let note_count = col.search_notes_unordered("")?.len();

    Ok(DevBootstrapInfo {
        path: path.to_string_lossy().to_string(),
        seeded,
        note_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn fresh_collection() -> (TempDir, Collection) {
        let tmp = TempDir::new().expect("tmpdir");
        let col = CollectionBuilder::new(tmp.path().join("test.anki2"))
            .build()
            .expect("build");
        (tmp, col)
    }

    #[test]
    fn seed_inserts_all_sample_notes_into_empty_collection() {
        let (_tmp, mut col) = fresh_collection();
        let inserted = seed_dev_notes_if_empty(&mut col).unwrap();
        assert!(inserted, "expected first seed to insert");
        let nids = col.search_notes_unordered("").unwrap();
        assert_eq!(nids.len(), SAMPLE_NOTES.len());
    }

    #[test]
    fn seed_creates_the_dev_deck() {
        let (_tmp, mut col) = fresh_collection();
        seed_dev_notes_if_empty(&mut col).unwrap();
        // get_or_create returns the existing one without reinserting.
        let deck = col.get_or_create_normal_deck(DECK_NAME).unwrap();
        // All sample cards should be in the dev deck — one card per Basic
        // note, so card count == sample count.
        let cids = col
            .search_cards(&format!("did:{}", deck.id.0), anki::search::SortMode::NoOrder)
            .unwrap();
        assert_eq!(cids.len(), SAMPLE_NOTES.len());
    }

    #[test]
    fn seed_is_idempotent_on_already_populated_collection() {
        let (_tmp, mut col) = fresh_collection();
        let first = seed_dev_notes_if_empty(&mut col).unwrap();
        let second = seed_dev_notes_if_empty(&mut col).unwrap();
        assert!(first);
        assert!(!second, "second call must be a no-op");
        let nids = col.search_notes_unordered("").unwrap();
        assert_eq!(
            nids.len(),
            SAMPLE_NOTES.len(),
            "no duplicates on re-seed"
        );
    }
}
