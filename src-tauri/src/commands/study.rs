use crate::error::{AppError, AppResult};
use crate::state::{AppState, CachedQueueEntry};
use anki::prelude::{DeckId, TimestampMillis};
use anki::scheduler::answering::{CardAnswer, Rating};
use anki::template::RenderedNode;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tauri::State;

#[derive(Serialize, Debug)]
pub struct StudyCard {
    pub card_id: i64,
    pub note_id: i64,
    pub question_html: String,
    pub answer_html: String,
    pub css: String,
    /// Counts left in the queue (not including the current card itself).
    pub remaining: Counts,
}

#[derive(Serialize, Debug, Default)]
pub struct Counts {
    pub new: u32,
    pub learning: u32,
    pub review: u32,
}

#[derive(Serialize, Debug)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NextCard {
    Card(StudyCard),
    Done(Counts),
}

fn render_nodes(nodes: &[RenderedNode]) -> String {
    nodes
        .iter()
        .map(|n| match n {
            RenderedNode::Text { text } => text.clone(),
            RenderedNode::Replacement { current_text, .. } => current_text.clone(),
        })
        .collect()
}

/// Set the active deck (used by the scheduler queue) and clear any cached
/// previous QueuedCard. Call before requesting the first card of a session.
#[tauri::command]
pub async fn start_study(
    deck_id: i64,
    state: State<'_, AppState>,
) -> AppResult<()> {
    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;
    col.set_current_deck(DeckId(deck_id))?;
    *state.last_queued.lock().await = None;
    Ok(())
}

#[tauri::command]
pub async fn get_next_card(state: State<'_, AppState>) -> AppResult<NextCard> {
    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;

    let queued = col.get_queued_cards(1, false)?;
    let counts = Counts {
        new: queued.new_count as u32,
        learning: queued.learning_count as u32,
        review: queued.review_count as u32,
    };

    let Some(qc) = queued.cards.into_iter().next() else {
        *state.last_queued.lock().await = None;
        return Ok(NextCard::Done(counts));
    };

    let card_id = qc.card.id();
    let note_id = qc.card.note_id();
    let rendered = col.render_existing_card(card_id, false, false)?;
    let card = StudyCard {
        card_id: card_id.0,
        note_id: note_id.0,
        question_html: render_nodes(&rendered.qnodes),
        answer_html: render_nodes(&rendered.anodes),
        css: rendered.css,
        remaining: counts,
    };

    *state.last_queued.lock().await = Some(CachedQueueEntry {
        card_id,
        states: qc.states,
        shown_at: Instant::now(),
    });
    Ok(NextCard::Card(card))
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AnswerRating {
    Again,
    Hard,
    Good,
    Easy,
}

#[tauri::command]
pub async fn answer_card_now(
    rating: AnswerRating,
    milliseconds_taken: Option<u32>,
    state: State<'_, AppState>,
) -> AppResult<()> {
    let entry = state
        .last_queued
        .lock()
        .await
        .clone()
        .ok_or_else(|| AppError::Anyhow(anyhow::anyhow!("no card queued")))?;

    let (rating_enum, new_state) = match rating {
        AnswerRating::Again => (Rating::Again, entry.states.again.clone()),
        AnswerRating::Hard => (Rating::Hard, entry.states.hard.clone()),
        AnswerRating::Good => (Rating::Good, entry.states.good.clone()),
        AnswerRating::Easy => (Rating::Easy, entry.states.easy.clone()),
    };

    let elapsed_ms = milliseconds_taken
        .unwrap_or_else(|| entry.shown_at.elapsed().as_millis().min(u32::MAX as u128) as u32);

    let mut answer = CardAnswer {
        card_id: entry.card_id,
        current_state: entry.states.current.clone(),
        new_state,
        rating: rating_enum,
        answered_at: TimestampMillis::now(),
        milliseconds_taken: elapsed_ms,
        custom_data: None,
        from_queue: true,
    };

    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;
    col.answer_card(&mut answer)?;

    // Invalidate cache so frontend must call get_next_card again.
    *state.last_queued.lock().await = None;
    Ok(())
}
