use crate::error::{AppError, AppResult};
use crate::state::AppState;
use anki_proto::import_export::ExportAnkiPackageOptions;
use anki_proto::import_export::ImportAnkiPackageOptions;
use anki_proto::import_export::ImportAnkiPackageUpdateCondition;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Serialize, Debug)]
pub struct ImportReport {
    pub new: u32,
    pub updated: u32,
    pub duplicate: u32,
    pub conflicting: u32,
    pub first_field_match: u32,
    pub missing_notetype: u32,
    pub missing_deck: u32,
    pub empty_first_field: u32,
    pub found_notes: u32,
}

#[tauri::command]
pub async fn import_apkg(
    in_path: String,
    state: State<'_, AppState>,
) -> AppResult<ImportReport> {
    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;

    let options = ImportAnkiPackageOptions {
        merge_notetypes: true,
        update_notes: ImportAnkiPackageUpdateCondition::Always as i32,
        update_notetypes: ImportAnkiPackageUpdateCondition::Always as i32,
        with_scheduling: true,
        with_deck_configs: true,
    };
    let out = col.import_apkg(&in_path, options)?;
    let log = out.output;

    Ok(ImportReport {
        new: log.new.len() as u32,
        updated: log.updated.len() as u32,
        duplicate: log.duplicate.len() as u32,
        conflicting: log.conflicting.len() as u32,
        first_field_match: log.first_field_match.len() as u32,
        missing_notetype: log.missing_notetype.len() as u32,
        missing_deck: log.missing_deck.len() as u32,
        empty_first_field: log.empty_first_field.len() as u32,
        found_notes: log.found_notes,
    })
}

#[derive(Deserialize, Debug)]
pub struct ExportAllInput {
    pub out_path: String,
    pub with_scheduling: bool,
    pub with_media: bool,
    pub with_deck_configs: bool,
    pub legacy: bool,
}

#[derive(Serialize, Debug)]
pub struct ExportReport {
    pub note_count: u32,
}

#[tauri::command]
pub async fn export_all_apkg(
    input: ExportAllInput,
    state: State<'_, AppState>,
) -> AppResult<ExportReport> {
    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;

    let options = ExportAnkiPackageOptions {
        with_scheduling: input.with_scheduling,
        with_deck_configs: input.with_deck_configs,
        with_media: input.with_media,
        legacy: input.legacy,
    };
    // Empty search string is parsed as SearchNode::WholeCollection by rslib.
    let count = col.export_apkg(&input.out_path, options, "", None)?;
    Ok(ExportReport {
        note_count: count as u32,
    })
}
