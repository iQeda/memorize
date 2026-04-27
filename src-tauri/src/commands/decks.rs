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
