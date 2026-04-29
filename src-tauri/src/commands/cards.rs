use crate::error::{AppError, AppResult};
use crate::state::AppState;
use anki::search::SortMode;
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct CardSummary {
    pub id: i64,
    pub note_id: i64,
    pub deck_id: i64,
    pub template_idx: u16,
    /// First field of the underlying note (the "word" for vocab decks).
    pub text: String,
}

async fn collect_cards(
    state: &State<'_, AppState>,
    search: &str,
    limit: u32,
) -> AppResult<Vec<CardSummary>> {
    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;

    let cids = col.search_cards(search, SortMode::NoOrder)?;

    let mut out = Vec::new();
    for cid in cids.into_iter().take(limit as usize) {
        if let Some(card) = col.storage.get_card(cid)? {
            let text = col
                .storage
                .get_note(card.note_id())?
                .and_then(|n| n.fields().first().cloned())
                .unwrap_or_default();
            out.push(CardSummary {
                id: card.id().0,
                note_id: card.note_id().0,
                deck_id: card.deck_id().0,
                template_idx: card.template_idx(),
                text,
            });
        }
    }
    Ok(out)
}

#[tauri::command]
pub async fn list_cards(
    deck_id: Option<i64>,
    query: Option<String>,
    limit: u32,
    state: State<'_, AppState>,
) -> AppResult<Vec<CardSummary>> {
    let mut parts: Vec<String> = Vec::new();
    if let Some(id) = deck_id {
        parts.push(format!("did:{}", id));
    }
    if let Some(q) = query.as_ref() {
        let trimmed = q.trim();
        if !trimmed.is_empty() {
            // Anki search: a bare token matches a substring across all fields.
            // Quote to keep multi-word phrases together.
            parts.push(format!("\"{}\"", trimmed.replace('"', "\\\"")));
        }
    }
    let search = parts.join(" ");
    collect_cards(&state, &search, limit).await
}

#[tauri::command]
pub async fn list_due_cards(
    deck_id: i64,
    limit: u32,
    state: State<'_, AppState>,
) -> AppResult<Vec<CardSummary>> {
    // Anki search syntax: cards eligible to study today in this deck.
    let search = format!("did:{} (is:new OR is:learn OR is:due)", deck_id);
    collect_cards(&state, &search, limit).await
}

