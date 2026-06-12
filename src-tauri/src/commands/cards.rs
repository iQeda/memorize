use crate::error::{AppError, AppResult};
use crate::state::AppState;
use anki::collection::Collection;
use anki::prelude::CardId;
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

/// Build the raw-SQL form used by `list_cards` to filter cards by deck and a
/// substring against the sort field. Returns the SQL string with positional
/// placeholders and the parameter values in order. Extracted as a pure
/// function so it can be unit-tested without standing up a Collection.
fn build_list_cards_sql(
    deck_id: Option<i64>,
    query: Option<&str>,
    limit: u32,
) -> (String, Vec<rusqlite::types::Value>) {
    let mut sql =
        String::from("SELECT c.id FROM cards c JOIN notes n ON c.nid = n.id WHERE 1=1");
    let mut args: Vec<rusqlite::types::Value> = Vec::new();

    if let Some(id) = deck_id {
        let i = args.len() + 1;
        sql.push_str(&format!(
            " AND (c.did = ?{i} OR (c.odid != 0 AND c.odid = ?{i}))"
        ));
        args.push(rusqlite::types::Value::Integer(id));
    }
    if let Some(q) = query.map(str::trim).filter(|s| !s.is_empty()) {
        let i = args.len() + 1;
        sql.push_str(&format!(" AND CAST(n.sfld AS TEXT) LIKE ?{i} ESCAPE '\\'"));
        // Escape SQL LIKE wildcards (`%`, `_`) and the escape char itself so
        // user input becomes a literal substring match.
        let escaped = q
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        args.push(rusqlite::types::Value::Text(format!("%{escaped}%")));
    }
    sql.push_str(&format!(" LIMIT {limit}"));
    (sql, args)
}

fn enrich_card_ids(col: &mut Collection, cids: Vec<i64>) -> AppResult<Vec<CardSummary>> {
    let mut out = Vec::new();
    for cid in cids {
        if let Some(card) = col.storage.get_card(CardId(cid))? {
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

/// Browse search is a vocab lookup: it should match the Word, not the meaning
/// text. Scope substring match to `n.sfld` (the sort field, which is the first
/// field for memorize's stock notetypes) instead of going through Anki search
/// syntax — `UnqualifiedText` searches both `n.sfld` and `n.flds`, so e.g.
/// typing "it" would match "civil" via the meaning "polite, related to
/// citizens".
fn list_cards_inner(
    col: &mut Collection,
    deck_id: Option<i64>,
    query: Option<&str>,
    limit: u32,
) -> AppResult<Vec<CardSummary>> {
    let (sql, args) = build_list_cards_sql(deck_id, query, limit);
    let cids: Vec<i64> = {
        let db = col.storage.db();
        let mut stmt = db
            .prepare(&sql)
            .map_err(|e| AppError::Anyhow(anyhow::anyhow!(e)))?;
        let rows = stmt
            .query_map(rusqlite::params_from_iter(args.iter()), |r| {
                r.get::<_, i64>(0)
            })
            .map_err(|e| AppError::Anyhow(anyhow::anyhow!(e)))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Anyhow(anyhow::anyhow!(e)))?
    };
    enrich_card_ids(col, cids)
}

fn list_due_cards_inner(
    col: &mut Collection,
    deck_id: i64,
    limit: u32,
) -> AppResult<Vec<CardSummary>> {
    // Anki search syntax: cards eligible to study today in this deck. The
    // `did:` clause depends on patches/0004 wrapping its OR'd terms in an
    // outer paren so the AND with the queue states actually intersects.
    let search = format!("did:{deck_id} (is:new OR is:learn OR is:due)");
    let cids = col.search_cards(&search, SortMode::NoOrder)?;
    enrich_card_ids(col, cids.into_iter().take(limit as usize).map(|c| c.0).collect())
}

#[tauri::command]
pub async fn list_cards(
    deck_id: Option<i64>,
    query: Option<String>,
    limit: u32,
    state: State<'_, AppState>,
) -> AppResult<Vec<CardSummary>> {
    state
        .with_collection(|col| list_cards_inner(col, deck_id, query.as_deref(), limit))
        .await
}

#[tauri::command]
pub async fn list_due_cards(
    deck_id: i64,
    limit: u32,
    state: State<'_, AppState>,
) -> AppResult<Vec<CardSummary>> {
    state
        .with_collection(|col| list_due_cards_inner(col, deck_id, limit))
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use anki::collection::CollectionBuilder;
    use anki::notes::Note;
    use anki::prelude::DeckId;
    use rusqlite::types::Value;
    use tempfile::TempDir;

    // ---- pure SQL builder ----

    #[test]
    fn build_sql_with_no_filter_lists_everything_under_limit() {
        let (sql, args) = build_list_cards_sql(None, None, 50);
        assert_eq!(
            sql,
            "SELECT c.id FROM cards c JOIN notes n ON c.nid = n.id WHERE 1=1 LIMIT 50"
        );
        assert!(args.is_empty());
    }

    #[test]
    fn build_sql_with_deck_only_filters_by_did_and_odid() {
        // Cards moved into a filtered deck have `c.did` rewritten and the
        // original deck stashed in `c.odid`. Both must be considered.
        let (sql, args) = build_list_cards_sql(Some(123), None, 500);
        assert!(sql.contains("(c.did = ?1 OR (c.odid != 0 AND c.odid = ?1))"));
        assert_eq!(args, vec![Value::Integer(123)]);
    }

    #[test]
    fn build_sql_with_query_only_substring_matches_sfld() {
        let (sql, args) = build_list_cards_sql(None, Some("foo"), 500);
        assert!(sql.contains("CAST(n.sfld AS TEXT) LIKE ?1 ESCAPE '\\'"));
        // n.flds (all fields concatenated) is intentionally NOT searched —
        // that's the whole point of restricting to the Word.
        assert!(!sql.contains("n.flds"));
        assert_eq!(args, vec![Value::Text("%foo%".into())]);
    }

    #[test]
    fn build_sql_escapes_sql_like_wildcards_in_query() {
        let (_sql, args) = build_list_cards_sql(None, Some("a_b%c\\d"), 500);
        // Backslash first so we don't double-escape introduced backslashes.
        assert_eq!(args, vec![Value::Text("%a\\_b\\%c\\\\d%".into())]);
    }

    #[test]
    fn build_sql_treats_whitespace_only_query_as_no_filter() {
        let (sql, args) = build_list_cards_sql(Some(7), Some("   "), 10);
        assert!(!sql.contains("LIKE"));
        assert_eq!(args, vec![Value::Integer(7)]);
    }

    #[test]
    fn build_sql_combines_deck_and_query_with_consecutive_placeholders() {
        let (sql, args) = build_list_cards_sql(Some(42), Some("hello"), 100);
        assert!(sql.contains("c.did = ?1"));
        assert!(sql.contains("LIKE ?2 ESCAPE '\\'"));
        assert_eq!(
            args,
            vec![Value::Integer(42), Value::Text("%hello%".into())]
        );
    }

    // ---- integration: real Collection ----

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

    fn make_deck(col: &mut Collection, name: &str) -> DeckId {
        let deck = col.get_or_create_normal_deck(name).expect("deck");
        deck.id
    }

    #[test]
    fn list_cards_substring_matches_sort_field_only() {
        let (_tmp, mut col) = test_collection();
        let deck = make_deck(&mut col, "Test");
        add_basic_note(&mut col, deck, "civil", "polite, related to citizens");
        add_basic_note(&mut col, deck, "exit", "to leave");
        add_basic_note(&mut col, deck, "garden", "outdoor");

        // "it" appears in `civil`'s back field (citizens), but only `exit`
        // contains "it" in the front field. Substring search must NOT pick
        // up the meaning-text hit.
        let r = list_cards_inner(&mut col, Some(deck.0), Some("it"), 100).unwrap();
        let words: Vec<_> = r.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(words, vec!["exit"]);
    }

    #[test]
    fn list_cards_returns_all_cards_in_deck_when_query_is_empty() {
        let (_tmp, mut col) = test_collection();
        let a = make_deck(&mut col, "A");
        let b = make_deck(&mut col, "B");
        add_basic_note(&mut col, a, "alpha", "x");
        add_basic_note(&mut col, a, "beta", "y");
        add_basic_note(&mut col, b, "gamma", "z");

        let r = list_cards_inner(&mut col, Some(a.0), None, 100).unwrap();
        assert_eq!(r.len(), 2);
        let r_all = list_cards_inner(&mut col, None, None, 100).unwrap();
        assert_eq!(r_all.len(), 3);
    }

    #[test]
    fn list_cards_does_not_match_meaning_field() {
        // Even if a meaning text contains the query, sfld-scope must hide it.
        let (_tmp, mut col) = test_collection();
        let deck = make_deck(&mut col, "Test");
        add_basic_note(&mut col, deck, "civil", "contains foo in meaning");
        add_basic_note(&mut col, deck, "other", "no match here");

        let r = list_cards_inner(&mut col, Some(deck.0), Some("foo"), 100).unwrap();
        assert!(r.is_empty(), "search hit meaning field, expected sfld-only");
    }

    #[test]
    fn list_cards_treats_user_underscore_as_literal() {
        // SQL LIKE treats `_` as "any single char". User input must be
        // escaped so a literal underscore matches only the same underscore.
        let (_tmp, mut col) = test_collection();
        let deck = make_deck(&mut col, "Test");
        add_basic_note(&mut col, deck, "abc", "x");
        add_basic_note(&mut col, deck, "a_c", "x");

        let r = list_cards_inner(&mut col, Some(deck.0), Some("a_c"), 100).unwrap();
        let words: Vec<_> = r.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(words, vec!["a_c"]);
    }

    #[test]
    fn list_cards_respects_limit() {
        let (_tmp, mut col) = test_collection();
        let deck = make_deck(&mut col, "Test");
        for i in 0..5 {
            add_basic_note(&mut col, deck, &format!("word{i}"), "x");
        }
        let r = list_cards_inner(&mut col, Some(deck.0), None, 3).unwrap();
        assert_eq!(r.len(), 3);
    }

    // ---- regression: vendor/anki SQL precedence (patches/0004) ----

    #[test]
    fn deck_filter_combined_with_field_search_correctly_intersects() {
        // Without patches/0004, `did:X "alpha"` builds SQL of shape
        //   `c.did in (X) OR (c.odid != 0 AND c.odid in (X)) AND (...fields...)`
        // and SQL precedence (AND tighter than OR) makes the deck filter
        // short-circuit the OR — every card in the deck is returned.
        // With the patch, the OR'd deck clauses are wrapped in an outer
        // paren so the AND with the field search actually intersects.
        let (_tmp, mut col) = test_collection();
        let deck = make_deck(&mut col, "Test");
        add_basic_note(&mut col, deck, "alpha", "x");
        add_basic_note(&mut col, deck, "beta", "x");
        add_basic_note(&mut col, deck, "gamma", "x");

        let cids = col
            .search_cards(&format!("did:{} \"alpha\"", deck.0), SortMode::NoOrder)
            .unwrap();
        assert_eq!(cids.len(), 1, "deck filter must AND with field search");
    }

    #[test]
    fn list_due_cards_does_not_leak_cards_from_other_decks() {
        // Indirectly exercises the patch: list_due_cards combines `did:X`
        // with `(is:new OR is:learn OR is:due)`. Pre-patch this leaked
        // every card in the deck; this test asserts the filter intersects
        // properly so we don't return more than was added.
        let (_tmp, mut col) = test_collection();
        let a = make_deck(&mut col, "A");
        let b = make_deck(&mut col, "B");
        add_basic_note(&mut col, a, "in_a_1", "x");
        add_basic_note(&mut col, a, "in_a_2", "x");
        add_basic_note(&mut col, b, "in_b_1", "x");

        // Fresh notes are all in queue=0 (new), so all three would be due,
        // but the deck filter must keep B's card out of A's result.
        let r = list_due_cards_inner(&mut col, a.0, 100).unwrap();
        let words: Vec<_> = r.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(words.len(), 2);
        assert!(words.iter().all(|w| w.starts_with("in_a_")));
    }
}
