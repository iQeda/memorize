use crate::error::{AppError, AppResult};
use crate::render::rendered_nodes_to_html;
use crate::state::AppState;
use anki::card::CardId;
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct RenderedCard {
    pub question_html: String,
    pub answer_html: String,
    pub css: String,
}

#[tauri::command]
pub async fn get_card_render(
    card_id: i64,
    state: State<'_, AppState>,
) -> AppResult<RenderedCard> {
    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;
    get_card_render_inner(col, card_id)
}

fn get_card_render_inner(
    col: &mut anki::collection::Collection,
    card_id: i64,
) -> AppResult<RenderedCard> {
    let cid = CardId(card_id);
    let rendered = col.render_existing_card(cid, false, false)?;

    Ok(RenderedCard {
        question_html: rendered_nodes_to_html(&rendered.qnodes),
        answer_html: rendered_nodes_to_html(&rendered.anodes),
        css: rendered.css,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use anki::collection::{Collection, CollectionBuilder};
    use anki::notes::Note;
    use anki::prelude::DeckId;
    use anki::search::SortMode;
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

    #[test]
    fn renders_existing_card_with_both_sides_and_css() {
        let (_tmp, mut col) = test_collection();
        let deck = col.get_or_create_normal_deck("Test").expect("deck").id;
        add_basic_note(&mut col, deck, "civil", "polite");

        let cids = col.search_cards("", SortMode::NoOrder).expect("search");
        assert_eq!(cids.len(), 1);

        let r = get_card_render_inner(&mut col, cids[0].0).expect("render");
        assert!(r.question_html.contains("civil"));
        assert!(r.answer_html.contains("polite"));
        // 解答面はテンプレート既定で問題面 (FrontSide) も含む。
        assert!(r.answer_html.contains("civil"));
        assert!(!r.css.is_empty(), "default notetype ships non-empty css");
    }

    #[test]
    fn unknown_card_id_is_an_error_not_a_panic() {
        let (_tmp, mut col) = test_collection();
        assert!(get_card_render_inner(&mut col, 999_999).is_err());
    }
}
