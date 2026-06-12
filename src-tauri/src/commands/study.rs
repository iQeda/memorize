use crate::error::{AppError, AppResult};
use crate::render::rendered_nodes_to_html;
use crate::state::{AppState, CachedQueueEntry};
use anki::prelude::{DeckId, TimestampMillis};
use anki::scheduler::answering::{CardAnswer, Rating};
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

// このモジュールは AppState::with_collection を使わない: 各コマンドが
// `col` と `last_queued` の 2 つの mutex を跨いで操作するため、closure 化
// するとロック取得順序が変わり交錯のリスクがある。手動パターンを維持する。

/// Set the active deck (used by the scheduler queue) and clear any cached
/// previous QueuedCard. Call before requesting the first card of a session.
#[tauri::command]
pub async fn start_study(
    deck_id: i64,
    state: State<'_, AppState>,
) -> AppResult<()> {
    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;
    start_study_inner(col, deck_id)?;
    *state.last_queued.lock().await = None;
    Ok(())
}

fn start_study_inner(col: &mut anki::collection::Collection, deck_id: i64) -> AppResult<()> {
    col.set_current_deck(DeckId(deck_id))?;
    Ok(())
}

#[tauri::command]
pub async fn get_next_card(state: State<'_, AppState>) -> AppResult<NextCard> {
    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;
    let (next, entry) = get_next_card_inner(col)?;
    *state.last_queued.lock().await = entry;
    Ok(next)
}

/// Pull the next queued card and build its DTO. Returns the cache entry the
/// caller must stash so a later answer_card_now can construct the CardAnswer
/// (None when the queue is exhausted).
fn get_next_card_inner(
    col: &mut anki::collection::Collection,
) -> AppResult<(NextCard, Option<CachedQueueEntry>)> {
    let queued = col.get_queued_cards(1, false)?;
    let counts = Counts {
        new: queued.new_count as u32,
        learning: queued.learning_count as u32,
        review: queued.review_count as u32,
    };

    let Some(qc) = queued.cards.into_iter().next() else {
        return Ok((NextCard::Done(counts), None));
    };

    let card_id = qc.card.id();
    let note_id = qc.card.note_id();
    let rendered = col.render_existing_card(card_id, false, false)?;
    let card = StudyCard {
        card_id: card_id.0,
        note_id: note_id.0,
        question_html: rendered_nodes_to_html(&rendered.qnodes),
        answer_html: rendered_nodes_to_html(&rendered.anodes),
        css: rendered.css,
        remaining: counts,
    };

    let entry = CachedQueueEntry {
        card_id,
        states: qc.states,
        shown_at: Instant::now(),
    };
    Ok((NextCard::Card(card), Some(entry)))
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

    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;
    answer_card_now_inner(col, &entry, rating, milliseconds_taken)?;

    // Invalidate cache so frontend must call get_next_card again.
    *state.last_queued.lock().await = None;
    Ok(())
}

fn answer_card_now_inner(
    col: &mut anki::collection::Collection,
    entry: &CachedQueueEntry,
    rating: AnswerRating,
    milliseconds_taken: Option<u32>,
) -> AppResult<()> {
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
    col.answer_card(&mut answer)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anki::collection::{Collection, CollectionBuilder};
    use anki::notes::Note;
    use tempfile::TempDir;

    fn test_collection() -> (TempDir, Collection) {
        let tmp = TempDir::new().expect("tmpdir");
        let path = tmp.path().join("test.anki2");
        let col = CollectionBuilder::new(&path).build().expect("build col");
        (tmp, col)
    }

    fn add_basic_note(col: &mut Collection, deck: DeckId, front: &str, back: &str) {
        let nt = col
            .get_all_notetypes()
            .expect("notetypes")
            .into_iter()
            .find(|nt| nt.config.kind == 0 && nt.fields.len() >= 2)
            .expect("a normal notetype with >=2 fields");
        let mut note = Note::new(&nt);
        note.set_field(0, front).unwrap();
        note.set_field(1, back).unwrap();
        col.add_note(&mut note, deck).expect("add_note");
    }

    fn card_queue(col: &Collection, card_id: i64) -> i64 {
        col.storage
            .db()
            .query_row("SELECT queue FROM cards WHERE id = ?1", [card_id], |r| {
                r.get(0)
            })
            .expect("card queue")
    }

    #[test]
    fn full_study_flow_serves_card_then_good_moves_it_to_learning() {
        let (_tmp, mut col) = test_collection();
        let deck = col.get_or_create_normal_deck("Study").expect("deck").id;
        add_basic_note(&mut col, deck, "civil", "polite");

        start_study_inner(&mut col, deck.0).expect("start");

        let (next, entry) = get_next_card_inner(&mut col).expect("next");
        let NextCard::Card(card) = next else {
            panic!("expected a card, got Done");
        };
        let entry = entry.expect("entry must accompany a served card");
        assert_eq!(card.card_id, entry.card_id.0);
        assert!(card.question_html.contains("civil"));
        assert!(card.answer_html.contains("polite"));
        // 新規 1 枚だけのデッキ: remaining は現在カードを含むカウント。
        assert_eq!(card.remaining.new, 1);
        assert_eq!(card.remaining.learning, 0);
        assert_eq!(card.remaining.review, 0);

        // 回答前: queue = 0 (new)。Good で learning queue (1 or 3) へ遷移する。
        assert_eq!(card_queue(&col, card.card_id), 0);
        answer_card_now_inner(&mut col, &entry, AnswerRating::Good, Some(1200))
            .expect("answer");
        let q = card_queue(&col, card.card_id);
        assert!(q == 1 || q == 3, "expected learning queue after Good, got {q}");
    }

    #[test]
    fn empty_deck_returns_done_with_zero_counts_and_no_entry() {
        let (_tmp, mut col) = test_collection();
        let deck = col.get_or_create_normal_deck("Empty").expect("deck").id;
        start_study_inner(&mut col, deck.0).expect("start");

        let (next, entry) = get_next_card_inner(&mut col).expect("next");
        assert!(entry.is_none());
        let NextCard::Done(counts) = next else {
            panic!("expected Done for an empty deck");
        };
        assert_eq!((counts.new, counts.learning, counts.review), (0, 0, 0));
    }

    #[test]
    fn answering_again_keeps_card_in_learning_and_queue_serves_it_again() {
        let (_tmp, mut col) = test_collection();
        let deck = col.get_or_create_normal_deck("Study").expect("deck").id;
        add_basic_note(&mut col, deck, "exit", "to leave");
        start_study_inner(&mut col, deck.0).expect("start");

        let (_, entry) = get_next_card_inner(&mut col).expect("next");
        let entry = entry.expect("entry");
        answer_card_now_inner(&mut col, &entry, AnswerRating::Again, Some(500))
            .expect("answer");

        // Again はカードを learning に留めるので、もう一度引ける。
        let (next, _) = get_next_card_inner(&mut col).expect("next again");
        let NextCard::Card(card) = next else {
            panic!("expected the card to be re-served after Again");
        };
        assert_eq!(card.card_id, entry.card_id.0);
    }
}
