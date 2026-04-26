use crate::error::{AppError, AppResult};
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

    let question_html = rendered
        .qnodes
        .iter()
        .map(render_node_to_html)
        .collect::<String>();
    let answer_html = rendered
        .anodes
        .iter()
        .map(render_node_to_html)
        .collect::<String>();

    Ok(RenderedCard {
        question_html,
        answer_html,
        css: rendered.css,
    })
}

fn render_node_to_html(node: &anki::template::RenderedNode) -> String {
    match node {
        anki::template::RenderedNode::Text { text } => text.clone(),
        anki::template::RenderedNode::Replacement {
            field_name: _,
            current_text,
            filters: _,
        } => current_text.clone(),
    }
}
