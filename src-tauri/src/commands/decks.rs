use crate::error::{AppError, AppResult};
use crate::state::AppState;
use anki::prelude::TimestampSecs;
use anki::search::SortMode;
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct DeckSummary {
    pub id: i64,
    pub name: String,
    pub level: u32,
    pub new_count: u32,
    pub learn_count: u32,
    pub review_count: u32,
}

#[tauri::command]
pub async fn list_decks(state: State<'_, AppState>) -> AppResult<Vec<DeckSummary>> {
    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;
    let tree = col.deck_tree(Some(TimestampSecs::now()))?;

    let mut out = Vec::new();
    walk(&tree, 0, &mut out);
    Ok(out)
}

#[derive(Serialize, Debug)]
pub struct DeckStats {
    pub total_cards: u32,
    pub total_notes: u32,
    pub new_cards: u32,
    pub learn_cards: u32,
    pub review_cards: u32,
    pub suspended: u32,
    pub buried: u32,
}

#[derive(Serialize, Debug, Default)]
pub struct TodayStats {
    pub answer_count: u32,
    pub answer_millis: u32,
    pub correct_count: u32,
    pub mature_count: u32,
    pub mature_correct: u32,
    pub learn_count: u32,
    pub review_count: u32,
    pub relearn_count: u32,
}

#[derive(Serialize, Debug, Default)]
pub struct FutureDueBucket {
    pub day: i32,
    pub count: u32,
}

#[derive(Serialize, Debug, Default)]
pub struct DeckGraphStats {
    pub today: TodayStats,
    pub future_due: Vec<FutureDueBucket>,
    pub future_due_total: u32,
    pub future_due_avg_per_day: f32,
    pub future_due_have_backlog: bool,
    pub daily_load: u32,
}

#[tauri::command]
pub async fn deck_graph_stats(
    deck_id: i64,
    days: u32,
    state: State<'_, AppState>,
) -> AppResult<DeckGraphStats> {
    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;

    let search = format!("did:{}", deck_id);
    let resp = col.graph_data_for_search(&search, days)?;

    let today = resp.today.unwrap_or_default();
    let fd = resp.future_due.unwrap_or_default();

    let max_day = days as i32;
    let mut buckets: Vec<FutureDueBucket> = fd
        .future_due
        .into_iter()
        .filter(|(d, _)| *d >= 0 && *d < max_day)
        .map(|(day, count)| FutureDueBucket { day, count })
        .collect();
    buckets.sort_by_key(|b| b.day);
    let total: u32 = buckets.iter().map(|b| b.count).sum();
    let avg = if max_day > 0 {
        total as f32 / max_day as f32
    } else {
        0.0
    };

    Ok(DeckGraphStats {
        today: TodayStats {
            answer_count: today.answer_count,
            answer_millis: today.answer_millis,
            correct_count: today.correct_count,
            mature_count: today.mature_count,
            mature_correct: today.mature_correct,
            learn_count: today.learn_count,
            review_count: today.review_count,
            relearn_count: today.relearn_count,
        },
        future_due: buckets,
        future_due_total: total,
        future_due_avg_per_day: avg,
        future_due_have_backlog: fd.have_backlog,
        daily_load: fd.daily_load,
    })
}

#[tauri::command]
pub async fn deck_stats(
    deck_id: i64,
    state: State<'_, AppState>,
) -> AppResult<DeckStats> {
    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;

    let count = |col: &mut anki::collection::Collection, q: &str| -> u32 {
        col.search_cards(q, SortMode::NoOrder)
            .map(|v| v.len() as u32)
            .unwrap_or(0)
    };

    let did = format!("did:{}", deck_id);
    let total_cards = count(col, &did);
    let new_cards = count(col, &format!("{did} is:new"));
    let learn_cards = count(col, &format!("{did} is:learn"));
    let review_cards = count(col, &format!("{did} is:review"));
    let suspended = count(col, &format!("{did} is:suspended"));
    let buried = count(col, &format!("{did} is:buried"));

    // Distinct notes via SQL on the search results (cheap shortcut: pull
    // note ids from cards). For accuracy we could also do a notes query.
    let total_notes = col
        .search_cards(&did, SortMode::NoOrder)
        .map(|cids| {
            let mut nids = std::collections::HashSet::<i64>::new();
            for cid in cids {
                if let Ok(Some(c)) = col.storage.get_card(cid) {
                    nids.insert(c.note_id().0);
                }
            }
            nids.len() as u32
        })
        .unwrap_or(0);

    Ok(DeckStats {
        total_cards,
        total_notes,
        new_cards,
        learn_cards,
        review_cards,
        suspended,
        buried,
    })
}

#[tauri::command]
pub async fn create_deck(
    name: String,
    state: State<'_, AppState>,
) -> AppResult<i64> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::Anyhow(anyhow::anyhow!("deck name is empty")));
    }
    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;
    let deck = col.get_or_create_normal_deck(trimmed)?;
    Ok(deck.id.0)
}

#[tauri::command]
pub async fn rename_deck(
    deck_id: i64,
    new_name: String,
    state: State<'_, AppState>,
) -> AppResult<()> {
    let trimmed = new_name.trim();
    if trimmed.is_empty() {
        return Err(AppError::Anyhow(anyhow::anyhow!("deck name is empty")));
    }
    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;
    col.rename_deck(anki::prelude::DeckId(deck_id), trimmed)?;
    Ok(())
}

#[tauri::command]
pub async fn delete_deck(
    deck_id: i64,
    state: State<'_, AppState>,
) -> AppResult<usize> {
    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;
    let out = col.remove_decks_and_child_decks(&[anki::prelude::DeckId(deck_id)])?;
    Ok(out.output)
}

fn walk(node: &anki_proto::decks::DeckTreeNode, level: u32, out: &mut Vec<DeckSummary>) {
    if node.deck_id != 0 {
        out.push(DeckSummary {
            id: node.deck_id,
            name: node.name.clone(),
            level,
            new_count: node.new_count,
            learn_count: node.learn_count,
            review_count: node.review_count,
        });
    }
    for child in &node.children {
        walk(child, level + 1, out);
    }
}
