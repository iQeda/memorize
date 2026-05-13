use crate::error::{AppError, AppResult};
use crate::state::AppState;
use anki::collection::Collection;
use anki::notes::Note;
use anki::prelude::{DeckId, NoteId, NotetypeId};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Serialize, Debug)]
pub struct NotetypeSummary {
    pub id: i64,
    pub name: String,
    pub field_names: Vec<String>,
}

#[tauri::command]
pub async fn list_notetypes(state: State<'_, AppState>) -> AppResult<Vec<NotetypeSummary>> {
    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;
    let nts = col.get_all_notetypes()?;
    Ok(nts
        .into_iter()
        .map(|nt| NotetypeSummary {
            id: nt.id.0,
            name: nt.name.clone(),
            field_names: nt.fields.iter().map(|f| f.name.clone()).collect(),
        })
        .collect())
}

#[derive(Serialize, Debug)]
pub struct NoteDetail {
    pub id: i64,
    pub notetype_id: i64,
    pub notetype_name: String,
    pub field_names: Vec<String>,
    pub fields: Vec<String>,
    pub tags: Vec<String>,
    /// Deck of a representative card belonging to this note.
    /// memorize's vocab notetypes are 1-card-per-note so this is unambiguous;
    /// for multi-card notetypes (Basic+reverse etc.) the first card by
    /// template order is used and `set_note_deck` aligns the rest.
    pub deck_id: i64,
}

#[tauri::command]
pub async fn get_note(note_id: i64, state: State<'_, AppState>) -> AppResult<NoteDetail> {
    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;
    let note = col
        .storage
        .get_note(NoteId(note_id))?
        .ok_or_else(|| AppError::Anyhow(anyhow::anyhow!("note not found")))?;
    let nt = col
        .get_notetype(note.notetype_id)?
        .ok_or_else(|| AppError::Anyhow(anyhow::anyhow!("notetype not found")))?;
    let cids = col
        .storage
        .all_card_ids_of_note_in_template_order(note.id)?;
    let deck_id = if let Some(cid) = cids.first() {
        col.storage
            .get_card(*cid)?
            .ok_or_else(|| AppError::Anyhow(anyhow::anyhow!("card not found")))?
            .deck_id()
            .0
    } else {
        0
    };
    Ok(NoteDetail {
        id: note.id.0,
        notetype_id: nt.id.0,
        notetype_name: nt.name.clone(),
        field_names: nt.fields.iter().map(|f| f.name.clone()).collect(),
        fields: note.fields().clone(),
        tags: note.tags.clone(),
        deck_id,
    })
}

#[derive(Deserialize, Debug)]
pub struct AddNoteInput {
    pub deck_id: i64,
    pub notetype_id: i64,
    pub fields: Vec<String>,
    pub tags: Vec<String>,
}

#[tauri::command]
pub async fn add_note(
    input: AddNoteInput,
    state: State<'_, AppState>,
) -> AppResult<i64> {
    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;
    let nt = col
        .get_notetype(NotetypeId(input.notetype_id))?
        .ok_or_else(|| AppError::Anyhow(anyhow::anyhow!("notetype not found")))?;
    let mut note = Note::new(&nt);
    for (i, value) in input.fields.into_iter().enumerate() {
        if i < nt.fields.len() {
            note.set_field(i, value)?;
        }
    }
    note.tags = input.tags;
    col.add_note(&mut note, DeckId(input.deck_id))?;
    Ok(note.id.0)
}

#[derive(Deserialize, Debug)]
pub struct UpdateNoteInput {
    pub note_id: i64,
    pub fields: Vec<String>,
    pub tags: Vec<String>,
}

#[tauri::command]
pub async fn update_note(
    input: UpdateNoteInput,
    state: State<'_, AppState>,
) -> AppResult<()> {
    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;
    let mut note = col
        .storage
        .get_note(NoteId(input.note_id))?
        .ok_or_else(|| AppError::Anyhow(anyhow::anyhow!("note not found")))?;
    for (i, value) in input.fields.into_iter().enumerate() {
        if i < note.fields().len() {
            note.set_field(i, value)?;
        }
    }
    note.tags = input.tags;
    col.update_note(&mut note)?;
    Ok(())
}

#[tauri::command]
pub async fn delete_notes(
    note_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> AppResult<usize> {
    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;
    let nids: Vec<NoteId> = note_ids.into_iter().map(NoteId).collect();
    let out = col.remove_notes(&nids)?;
    Ok(out.output)
}

#[derive(Deserialize, Debug)]
pub struct SetNoteDeckInput {
    pub note_id: i64,
    pub deck_id: i64,
}

fn set_note_deck_inner(col: &mut Collection, note_id: i64, deck_id: i64) -> AppResult<()> {
    let cids = col
        .storage
        .all_card_ids_of_note_in_template_order(NoteId(note_id))?;
    if cids.is_empty() {
        return Err(AppError::Anyhow(anyhow::anyhow!("note has no cards")));
    }
    col.set_deck(&cids, DeckId(deck_id))?;
    Ok(())
}

#[tauri::command]
pub async fn set_note_deck(
    input: SetNoteDeckInput,
    state: State<'_, AppState>,
) -> AppResult<()> {
    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;
    set_note_deck_inner(col, input.note_id, input.deck_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anki::collection::CollectionBuilder;
    use anki::prelude::DeckId;
    use tempfile::TempDir;

    fn test_collection() -> (TempDir, Collection) {
        let tmp = TempDir::new().expect("tmpdir");
        let path = tmp.path().join("test.anki2");
        let col = CollectionBuilder::new(&path).build().expect("build col");
        (tmp, col)
    }

    fn add_basic_note(col: &mut Collection, deck: DeckId, front: &str, back: &str) -> NoteId {
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
        note.id
    }

    fn make_deck(col: &mut Collection, name: &str) -> DeckId {
        col.get_or_create_normal_deck(name).expect("deck").id
    }

    #[test]
    fn set_note_deck_moves_all_cards_of_note_to_target_deck() {
        let (_tmp, mut col) = test_collection();
        let src = make_deck(&mut col, "Source");
        let dst = make_deck(&mut col, "Destination");
        let nid = add_basic_note(&mut col, src, "apple", "りんご");

        // Confirm baseline: the note's card(s) live in `src`.
        let cids = col
            .storage
            .all_card_ids_of_note_in_template_order(nid)
            .unwrap();
        assert!(!cids.is_empty(), "note must have at least one card");
        for cid in &cids {
            let c = col.storage.get_card(*cid).unwrap().unwrap();
            assert_eq!(c.deck_id(), src);
        }

        set_note_deck_inner(&mut col, nid.0, dst.0).expect("set_note_deck");

        for cid in &cids {
            let c = col.storage.get_card(*cid).unwrap().unwrap();
            assert_eq!(c.deck_id(), dst, "card {} should have moved to dst", cid.0);
        }
    }

    #[test]
    fn set_note_deck_errors_when_note_has_no_cards() {
        let (_tmp, mut col) = test_collection();
        let dst = make_deck(&mut col, "Destination");
        // Use a NoteId that no note exists for — same observable shape as
        // "note exists but has no cards" from set_note_deck_inner's POV.
        let bogus = 99_999_999_i64;
        let r = set_note_deck_inner(&mut col, bogus, dst.0);
        assert!(r.is_err(), "missing note should produce an error");
    }

    #[test]
    fn get_note_returns_current_deck_id() {
        let (_tmp, mut col) = test_collection();
        let deck = make_deck(&mut col, "Vocab");
        let nid = add_basic_note(&mut col, deck, "exit", "to leave");

        // Mirror the body of the `get_note` Tauri command but without
        // going through tauri::State.
        let note = col.storage.get_note(nid).unwrap().unwrap();
        let cids = col
            .storage
            .all_card_ids_of_note_in_template_order(note.id)
            .unwrap();
        let first_card = col.storage.get_card(cids[0]).unwrap().unwrap();
        assert_eq!(first_card.deck_id(), deck);
    }
}
