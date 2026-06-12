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

    let cid = CardId(card_id);
    let rendered = col.render_existing_card(cid, false, false)?;

    Ok(RenderedCard {
        question_html: rendered_nodes_to_html(&rendered.qnodes),
        answer_html: rendered_nodes_to_html(&rendered.anodes),
        css: rendered.css,
    })
}
