//! deck_stats — queue ベースの排他カウント。
//! `is:learn` 等の Anki 検索は `c.type` ベースで排他にならないため使わない
//! (CLAUDE.md の queue-vs-type ルール)。

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use serde::Serialize;
use tauri::State;

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
    state
        .with_collection(|col| deck_stats_inner(col, deck_id))
        .await
}

/// 通常デッキ + filtered deck 退避中 (odid) のカードを対象にする共通フィルタ。
const IN_DECK: &str = "(did = ?1 OR (odid != 0 AND odid = ?1))";

/// `IN_DECK` フィルタ + 追加述語で COUNT(*) を撃つ。`select_expr` は
/// total_notes の COUNT(DISTINCT nid) だけが異なるため引数化している。
fn count_cards(
    db: &rusqlite::Connection,
    deck_id: i64,
    select_expr: &str,
    extra_predicate: &str,
) -> AppResult<u32> {
    let sql = format!("SELECT {select_expr} FROM cards WHERE {IN_DECK}{extra_predicate}");
    db.query_row(&sql, [deck_id], |r| r.get(0))
        .map_err(AppError::Db)
}

fn deck_stats_inner(
    col: &mut anki::collection::Collection,
    deck_id: i64,
) -> AppResult<DeckStats> {
    // Classify each card mutually exclusively by queue first.
    // queue: -1 = Suspended, -2/-3 = Buried, 0 = New, 1/3 = Learn,
    //        2 = Review.
    let db = col.storage.db();
    let count = |pred: &str| count_cards(db, deck_id, "COUNT(*)", pred);
    let total_cards = count("")?;
    let suspended = count(" AND queue = -1")?;
    let buried = count(" AND queue IN (-2, -3)")?;
    let new_cards = count(" AND queue = 0")?;
    let learn_cards = count(" AND queue IN (1, 3)")?;
    let review_cards = count(" AND queue = 2")?;
    let total_notes = count_cards(db, deck_id, "COUNT(DISTINCT nid)", "")?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use anki::collection::{Collection, CollectionBuilder};
    use anki::notes::Note;
    use anki::prelude::DeckId;
    use tempfile::TempDir;

    fn test_collection() -> (TempDir, Collection) {
        let tmp = TempDir::new().expect("tmpdir");
        let path = tmp.path().join("test.anki2");
        let col = CollectionBuilder::new(&path).build().expect("build col");
        (tmp, col)
    }

    fn add_basic_note(col: &mut Collection, deck: DeckId, front: &str) {
        let nt = col
            .get_all_notetypes()
            .expect("notetypes")
            .into_iter()
            .find(|nt| nt.config.kind == 0 && nt.fields.len() >= 2)
            .expect("a normal notetype with >=2 fields");
        let mut note = Note::new(&nt);
        note.set_field(0, front).unwrap();
        note.set_field(1, "back").unwrap();
        col.add_note(&mut note, deck).expect("add_note");
    }

    /// 追加順 (= id 昇順) でデッキ内のカード id を返す。
    fn card_ids(col: &Collection, deck: DeckId) -> Vec<i64> {
        let db = col.storage.db();
        let mut stmt = db
            .prepare("SELECT id FROM cards WHERE did = ?1 ORDER BY id")
            .unwrap();
        let ids = stmt
            .query_map([deck.0], |r| r.get(0))
            .unwrap()
            .collect::<Result<Vec<i64>, _>>()
            .unwrap();
        ids
    }

    fn set_card_state(col: &Collection, card_id: i64, queue: i64, ctype: i64) {
        col.storage
            .db()
            .execute(
                "UPDATE cards SET queue = ?1, type = ?2 WHERE id = ?3",
                [queue, ctype, card_id],
            )
            .unwrap();
    }

    #[test]
    fn deck_stats_classifies_by_queue_mutually_exclusively() {
        let (_tmp, mut col) = test_collection();
        let deck = col.get_or_create_normal_deck("Stats").expect("deck").id;
        for i in 0..5 {
            add_basic_note(&mut col, deck, &format!("w{i}"));
        }
        let ids = card_ids(&col, deck);
        assert_eq!(ids.len(), 5);

        // queue/type: new(0/0) はそのまま、残りを learn(1/1) / review(2/2) /
        // suspended(-1) / buried(-2) に振り分ける。
        set_card_state(&col, ids[1], 1, 1); // learning
        set_card_state(&col, ids[2], 2, 2); // review
        set_card_state(&col, ids[3], -1, 2); // suspended (review type)
        set_card_state(&col, ids[4], -2, 0); // buried (new type)

        let s = deck_stats_inner(&mut col, deck.0).unwrap();
        assert_eq!(s.total_cards, 5);
        assert_eq!(s.total_notes, 5);
        assert_eq!(s.new_cards, 1);
        assert_eq!(s.learn_cards, 1);
        assert_eq!(s.review_cards, 1);
        assert_eq!(s.suspended, 1);
        assert_eq!(s.buried, 1);
        // 相互排他: 各カテゴリの合計が総数に一致する。
        assert_eq!(
            s.new_cards + s.learn_cards + s.review_cards + s.suspended + s.buried,
            s.total_cards
        );
    }

    #[test]
    fn suspended_while_learning_counts_only_as_suspended() {
        // 過去バグの再発防止 (CLAUDE.md の queue-vs-type ルール):
        // type=1 (learning) のまま queue=-1 (suspended) になったカードが
        // learn と suspended の両方にカウントされてはいけない。
        let (_tmp, mut col) = test_collection();
        let deck = col.get_or_create_normal_deck("Stats").expect("deck").id;
        add_basic_note(&mut col, deck, "word");
        let ids = card_ids(&col, deck);

        set_card_state(&col, ids[0], -1, 1); // queue=suspended, type=learning

        let s = deck_stats_inner(&mut col, deck.0).unwrap();
        assert_eq!(s.suspended, 1);
        assert_eq!(s.learn_cards, 0, "type=learn but queue=suspended must not count as learn");
        assert_eq!(s.new_cards + s.learn_cards + s.review_cards + s.suspended + s.buried, 1);
    }

    #[test]
    fn deck_stats_ignores_cards_in_other_decks() {
        let (_tmp, mut col) = test_collection();
        let a = col.get_or_create_normal_deck("A").expect("deck").id;
        let b = col.get_or_create_normal_deck("B").expect("deck").id;
        add_basic_note(&mut col, a, "in_a");
        add_basic_note(&mut col, b, "in_b");

        let s = deck_stats_inner(&mut col, a.0).unwrap();
        assert_eq!(s.total_cards, 1);
        assert_eq!(s.new_cards, 1);
    }
}
