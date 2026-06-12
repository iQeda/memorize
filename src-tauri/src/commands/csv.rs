//! TSV / CSV import.
//!
//! Thin DTO wrappers over rslib's `Collection::get_csv_metadata` +
//! `Collection::import_csv`. The file is treated as self-describing: meta lines
//! like `#separator:tab`, `#deck:Hacker News`, `#notetype:Basic`,
//! `#tags column:3` are parsed by rslib's auto-detection, so the frontend only
//! needs to confirm what was detected and optionally override how duplicates are
//! resolved. The default duplicate behaviour is UPDATE — re-importing the same
//! deck with the first field (front) as the key overwrites existing notes, which
//! is the "add & update words daily" workflow.

use super::package::ImportReport;
use crate::error::AppResult;
use crate::progress::ProgressEmitter;
use crate::state::AppState;
use anki::collection::Collection;
use anki::import_export::text::csv::metadata::{
    CsvDeck, CsvMetadata, CsvNotetype, Delimiter, DupeResolution,
};
use anki::prelude::{DeckId, NotetypeId};
use serde::Serialize;
use tauri::{AppHandle, State};

/// What rslib detected for a CSV/TSV file, surfaced to the confirmation dialog.
#[derive(Serialize, Debug)]
pub struct CsvPreview {
    /// Human-readable destination deck name (or "column N" for per-row decks).
    pub deck: String,
    /// Notetype name (or "column N" for per-row notetypes).
    pub notetype: String,
    /// Delimiter code: tab / comma / semicolon / colon / pipe / space.
    pub delimiter: String,
    /// Duplicate-resolution code: update / preserve / duplicate.
    pub dupe_resolution: String,
    /// Number of columns detected.
    pub columns: u32,
    /// First few rows for display (rslib caps this at 5).
    pub preview_rows: Vec<Vec<String>>,
    /// Whether field contents are treated as raw HTML.
    pub is_html: bool,
    /// 1-based tags column, 0 if none.
    pub tags_column: u32,
}

fn delimiter_code(d: Delimiter) -> &'static str {
    match d {
        Delimiter::Tab => "tab",
        Delimiter::Pipe => "pipe",
        Delimiter::Semicolon => "semicolon",
        Delimiter::Colon => "colon",
        Delimiter::Comma => "comma",
        Delimiter::Space => "space",
    }
}

fn dupe_code(d: DupeResolution) -> &'static str {
    match d {
        DupeResolution::Update => "update",
        DupeResolution::Preserve => "preserve",
        DupeResolution::Duplicate => "duplicate",
    }
}

fn dupe_from_code(code: &str) -> Option<DupeResolution> {
    match code {
        "update" => Some(DupeResolution::Update),
        "preserve" => Some(DupeResolution::Preserve),
        "duplicate" => Some(DupeResolution::Duplicate),
        _ => None,
    }
}

fn deck_label(col: &mut Collection, deck: Option<&CsvDeck>) -> String {
    match deck {
        Some(CsvDeck::DeckId(id)) => col
            .get_deck(DeckId(*id))
            .ok()
            .flatten()
            .map(|d| d.human_name())
            .unwrap_or_else(|| format!("#{id}")),
        Some(CsvDeck::DeckName(name)) => name.clone(),
        Some(CsvDeck::DeckColumn(n)) => format!("column {n}"),
        None => String::new(),
    }
}

fn notetype_label(col: &mut Collection, notetype: Option<&CsvNotetype>) -> String {
    match notetype {
        Some(CsvNotetype::GlobalNotetype(global)) => col
            .get_notetype(NotetypeId(global.id))
            .ok()
            .flatten()
            .map(|nt| nt.name.clone())
            .unwrap_or_else(|| format!("#{}", global.id)),
        Some(CsvNotetype::NotetypeColumn(n)) => format!("column {n}"),
        None => String::new(),
    }
}

fn build_preview(col: &mut Collection, meta: &CsvMetadata) -> CsvPreview {
    let deck = deck_label(col, meta.deck.as_ref());
    let notetype = notetype_label(col, meta.notetype.as_ref());
    CsvPreview {
        deck,
        notetype,
        delimiter: delimiter_code(meta.delimiter()).to_string(),
        dupe_resolution: dupe_code(meta.dupe_resolution()).to_string(),
        columns: meta.column_labels.len() as u32,
        preview_rows: meta.preview.iter().map(|row| row.vals.clone()).collect(),
        is_html: meta.is_html,
        tags_column: meta.tags_column,
    }
}

fn csv_preview_inner(col: &mut Collection, path: &str) -> AppResult<CsvPreview> {
    // Pass None for every override so rslib's directives + defaults drive
    // detection (separator, deck, notetype, html, tags column, dupe scope).
    let meta = col.get_csv_metadata(path, None, None, None, None)?;
    Ok(build_preview(col, &meta))
}

fn import_csv_inner(
    col: &mut Collection,
    path: &str,
    dupe_resolution: Option<DupeResolution>,
) -> AppResult<ImportReport> {
    let mut meta = col.get_csv_metadata(path, None, None, None, None)?;
    if let Some(dupe) = dupe_resolution {
        meta.dupe_resolution = dupe as i32;
    }
    let out = col.import_csv(path, meta)?;
    Ok(ImportReport::from_note_log(&out.output))
}

/// Detect and return import settings for a CSV/TSV file without modifying the
/// collection. Used to populate the confirmation dialog.
#[tauri::command]
pub async fn csv_metadata(in_path: String, state: State<'_, AppState>) -> AppResult<CsvPreview> {
    state
        .with_collection(|col| csv_preview_inner(col, &in_path))
        .await
}

/// Import a CSV/TSV file. `dupe_resolution` (update/preserve/duplicate) overrides
/// whatever the file/config specified; unknown or absent values keep the
/// detected default.
#[tauri::command]
pub async fn import_tsv(
    in_path: String,
    dupe_resolution: Option<String>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<ImportReport> {
    let dupe = dupe_resolution.as_deref().and_then(dupe_from_code);
    let _emitter = ProgressEmitter::start(app, state.progress.clone());
    state
        .with_collection(|col| import_csv_inner(col, &in_path, dupe))
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use anki::collection::CollectionBuilder;
    use std::io::Write;
    use tempfile::TempDir;

    fn test_collection() -> (TempDir, Collection) {
        let tmp = TempDir::new().expect("tmpdir");
        let path = tmp.path().join("test.anki2");
        let col = CollectionBuilder::new(&path).build().expect("build col");
        (tmp, col)
    }

    /// Name of the seeded Basic notetype (the normal notetype with >=2 fields),
    /// so meta lines work regardless of the collection's locale.
    fn basic_notetype_name(col: &mut Collection) -> String {
        col.get_all_notetypes()
            .expect("notetypes")
            .into_iter()
            .find(|nt| nt.config.kind == 0 && nt.fields.len() >= 2)
            .expect("a normal notetype with >=2 fields")
            .name
            .clone()
    }

    fn write_file(dir: &TempDir, name: &str, body: &str) -> String {
        let p = dir.path().join(name);
        let mut f = std::fs::File::create(&p).expect("create file");
        f.write_all(body.as_bytes()).expect("write file");
        p.to_string_lossy().into_owned()
    }

    fn tsv_with(nt: &str, rows: &str) -> String {
        format!("#separator:tab\n#html:true\n#deck:Daily Vocab\n#notetype:{nt}\n{rows}")
    }

    #[test]
    fn preview_detects_tab_delimiter_and_deck_from_meta_lines() {
        let (tmp, mut col) = test_collection();
        let nt = basic_notetype_name(&mut col);
        let path = write_file(
            &tmp,
            "vocab.tsv",
            &tsv_with(&nt, "apple\tりんご\nbanana\tバナナ\n"),
        );

        let preview = csv_preview_inner(&mut col, &path).expect("preview");

        assert_eq!(preview.delimiter, "tab");
        assert_eq!(preview.deck, "Daily Vocab");
        assert_eq!(preview.notetype, nt);
        assert_eq!(preview.dupe_resolution, "update");
        assert!(preview.is_html);
        assert_eq!(preview.columns, 2);
        assert_eq!(preview.preview_rows.len(), 2);
        assert_eq!(preview.preview_rows[0], vec!["apple", "りんご"]);
    }

    #[test]
    fn import_adds_new_notes_into_named_deck() {
        let (tmp, mut col) = test_collection();
        let nt = basic_notetype_name(&mut col);
        let path = write_file(
            &tmp,
            "vocab.tsv",
            &tsv_with(&nt, "apple\tりんご\nbanana\tバナナ\n"),
        );

        let report = import_csv_inner(&mut col, &path, None).expect("import");

        assert_eq!(report.new, 2);
        assert_eq!(report.updated, 0);

        // The "Daily Vocab" deck was created on the fly and holds both cards.
        assert!(
            col.get_deck_id("Daily Vocab")
                .expect("deck lookup")
                .is_some(),
            "deck created"
        );
        let cards = col
            .search_cards("deck:\"Daily Vocab\"", anki::search::SortMode::NoOrder)
            .expect("search");
        assert_eq!(cards.len(), 2);
    }

    /// Field 1 (front) of every note matching `front`. Used to assert that an
    /// UPDATE overwrote the back side in place.
    fn back_fields(col: &mut Collection, front: &str) -> Vec<String> {
        col.search_notes(front, anki::search::SortMode::NoOrder)
            .expect("search notes")
            .into_iter()
            .map(|nid| col.storage.get_note(nid).unwrap().unwrap().fields()[1].clone())
            .collect()
    }

    #[test]
    fn reimport_updates_existing_note_in_place() {
        let (tmp, mut col) = test_collection();
        let nt = basic_notetype_name(&mut col);

        let first = write_file(&tmp, "day1.tsv", &tsv_with(&nt, "apple\tりんご\n"));
        let r1 = import_csv_inner(&mut col, &first, Some(DupeResolution::Update)).expect("import 1");
        assert_eq!(r1.new, 1);

        // Same first field "apple", new back side — UPDATE overwrites in place.
        let second = write_file(&tmp, "day2.tsv", &tsv_with(&nt, "apple\tリンゴ (改訂)\n"));
        let r2 = import_csv_inner(&mut col, &second, Some(DupeResolution::Update)).expect("import 2");

        assert_eq!(r2.new, 0, "no new note");
        // No GUID column, so a first-field-keyed update is reported under
        // `first_field_match` (not `updated`, which is reserved for GUID matches).
        assert_eq!(r2.first_field_match, 1, "existing note matched & updated");
        assert_eq!(
            back_fields(&mut col, "apple"),
            vec!["リンゴ (改訂)".to_string()],
            "back side overwritten, no duplicate created"
        );
    }

    #[test]
    fn duplicate_mode_keeps_both_notes() {
        let (tmp, mut col) = test_collection();
        let nt = basic_notetype_name(&mut col);

        let first = write_file(&tmp, "day1.tsv", &tsv_with(&nt, "apple\tりんご\n"));
        import_csv_inner(&mut col, &first, Some(DupeResolution::Duplicate)).expect("import 1");

        let second = write_file(&tmp, "day2.tsv", &tsv_with(&nt, "apple\tリンゴ\n"));
        import_csv_inner(&mut col, &second, Some(DupeResolution::Duplicate)).expect("import 2");

        // DUPLICATE keeps both: two notes now share the first field "apple".
        assert_eq!(back_fields(&mut col, "apple").len(), 2);
    }

    #[test]
    fn preserve_mode_leaves_existing_untouched() {
        let (tmp, mut col) = test_collection();
        let nt = basic_notetype_name(&mut col);

        let first = write_file(&tmp, "day1.tsv", &tsv_with(&nt, "apple\tりんご\n"));
        import_csv_inner(&mut col, &first, Some(DupeResolution::Preserve)).expect("import 1");

        let second = write_file(&tmp, "day2.tsv", &tsv_with(&nt, "apple\tリンゴ\n"));
        import_csv_inner(&mut col, &second, Some(DupeResolution::Preserve)).expect("import 2");

        assert_eq!(
            back_fields(&mut col, "apple"),
            vec!["りんご".to_string()],
            "existing back side preserved, new data discarded"
        );
    }

    #[test]
    fn dupe_code_round_trips() {
        for code in ["update", "preserve", "duplicate"] {
            let parsed = dupe_from_code(code).expect("known code");
            assert_eq!(dupe_code(parsed), code);
        }
        assert!(dupe_from_code("nonsense").is_none());
    }
}
