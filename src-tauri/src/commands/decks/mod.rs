//! デッキ関連コマンド。CRUD はこのファイル、queue ベースの統計は
//! `stats`、グラフパネル用の集計は `graphs` に分割している。
//! lib.rs の generate_handler! は `commands::decks::*` パスのまま動くよう
//! glob re-export で公開する (tauri::command が生成する隠しマクロも含む)。

mod graphs;
mod stats;

pub use graphs::*;
pub use stats::*;

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use anki::prelude::TimestampSecs;
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
    state
        .with_collection(|col| {
            let tree = col.deck_tree(Some(TimestampSecs::now()))?;
            let mut out = Vec::new();
            walk(&tree, 0, &mut out);
            Ok(out)
        })
        .await
}

#[tauri::command]
pub async fn create_deck(
    name: String,
    state: State<'_, AppState>,
) -> AppResult<i64> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidInput("deck name is empty".into()));
    }
    state
        .with_collection(|col| {
            let deck = col.get_or_create_normal_deck(trimmed)?;
            Ok(deck.id.0)
        })
        .await
}

#[tauri::command]
pub async fn rename_deck(
    deck_id: i64,
    new_name: String,
    state: State<'_, AppState>,
) -> AppResult<()> {
    let trimmed = new_name.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidInput("deck name is empty".into()));
    }
    state
        .with_collection(|col| {
            col.rename_deck(anki::prelude::DeckId(deck_id), trimmed)?;
            Ok(())
        })
        .await
}

#[tauri::command]
pub async fn delete_deck(
    deck_id: i64,
    state: State<'_, AppState>,
) -> AppResult<usize> {
    state
        .with_collection(|col| {
            let out = col.remove_decks_and_child_decks(&[anki::prelude::DeckId(deck_id)])?;
            Ok(out.output)
        })
        .await
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

#[cfg(test)]
mod tests {
    use super::*;

    fn node(deck_id: i64, name: &str, children: Vec<anki_proto::decks::DeckTreeNode>) -> anki_proto::decks::DeckTreeNode {
        anki_proto::decks::DeckTreeNode {
            deck_id,
            name: name.into(),
            children,
            new_count: deck_id as u32,
            learn_count: deck_id as u32 * 2,
            review_count: deck_id as u32 * 3,
            ..Default::default()
        }
    }

    #[test]
    fn walk_skips_synthetic_root_and_records_depth() {
        // list_decks's caller passes the synthetic root (deck_id == 0). It
        // must not appear in the output, but its children do.
        //
        // Note on `level`: walk recurses with `level + 1` regardless of
        // whether the parent was pushed. So even though the synthetic root
        // is *skipped*, it still consumes one level — top-level decks land
        // at level 1, their children at level 2, etc. The frontend's deck
        // tree indentation depends on this offset.
        let tree = node(
            0,
            "",
            vec![
                node(1, "Default", vec![]),
                node(2, "Lang", vec![node(3, "Lang::FR", vec![])]),
            ],
        );

        let mut out = Vec::new();
        walk(&tree, 0, &mut out);

        let pairs: Vec<(i64, u32)> = out.iter().map(|d| (d.id, d.level)).collect();
        assert_eq!(pairs, vec![(1, 1), (2, 1), (3, 2)]);
    }

    #[test]
    fn walk_copies_per_node_counts() {
        let tree = node(0, "", vec![node(7, "Vocab", vec![])]);
        let mut out = Vec::new();
        walk(&tree, 0, &mut out);
        assert_eq!(out.len(), 1);
        let d = &out[0];
        assert_eq!(d.name, "Vocab");
        assert_eq!(d.new_count, 7);
        assert_eq!(d.learn_count, 14);
        assert_eq!(d.review_count, 21);
    }
}
