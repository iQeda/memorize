use crate::error::{AppError, AppResult};
use crate::state::AppState;
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
    Ok(NoteDetail {
        id: note.id.0,
        notetype_id: nt.id.0,
        notetype_name: nt.name.clone(),
        field_names: nt.fields.iter().map(|f| f.name.clone()).collect(),
        fields: note.fields().clone(),
        tags: note.tags.clone(),
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
