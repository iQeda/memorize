use crate::error::{AppError, AppResult};
use crate::state::AppState;
use anki::prelude::DeckId;
use anki::search::SortMode;
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct CardSummary {
    pub id: i64,
    pub note_id: i64,
    pub deck_id: i64,
    pub template_idx: u16,
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
            out.push(CardSummary {
                id: card.id().0,
                note_id: card.note_id().0,
                deck_id: card.deck_id().0,
                template_idx: card.template_idx(),
            });
        }
    }
    Ok(out)
}

#[tauri::command]
pub async fn list_cards(
    deck_id: i64,
    limit: u32,
    state: State<'_, AppState>,
) -> AppResult<Vec<CardSummary>> {
    let search = format!("did:{}", deck_id);
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

#[derive(Debug, Serialize)]
pub struct StudyQueue {
    pub cards: Vec<CardSummary>,
    pub new_count: u32,
    pub learning_count: u32,
    pub review_count: u32,
}

#[tauri::command]
pub async fn get_study_queue(
    deck_id: i64,
    limit: u32,
    state: State<'_, AppState>,
) -> AppResult<StudyQueue> {
    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;

    col.set_current_deck(DeckId(deck_id))?;
    let queued = col.get_queued_cards(limit as usize, false)?;

    let cards = queued
        .cards
        .iter()
        .map(|qc| CardSummary {
            id: qc.card.id().0,
            note_id: qc.card.note_id().0,
            deck_id: qc.card.deck_id().0,
            template_idx: qc.card.template_idx(),
        })
        .collect();

    Ok(StudyQueue {
        cards,
        new_count: queued.new_count as u32,
        learning_count: queued.learning_count as u32,
        review_count: queued.review_count as u32,
    })
}
