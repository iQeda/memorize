//! rslib のレンダリング結果 (RenderedNode) をフロントに渡す HTML 文字列へ
//! 変換する共通ヘルパー。reviewer / study の両コマンドが使う。

use anki::template::RenderedNode;

/// RenderedNode のスライスを 1 つの HTML 文字列に平坦化する。
/// Text はそのまま、Replacement はフィルタ適用済みの current_text を採用。
pub fn rendered_nodes_to_html(nodes: &[RenderedNode]) -> String {
    nodes
        .iter()
        .map(|n| match n {
            RenderedNode::Text { text } => text.clone(),
            RenderedNode::Replacement { current_text, .. } => current_text.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_slice_renders_to_empty_string() {
        assert_eq!(rendered_nodes_to_html(&[]), "");
    }

    #[test]
    fn text_nodes_are_passed_through() {
        let nodes = vec![RenderedNode::Text {
            text: "<b>hello</b>".into(),
        }];
        assert_eq!(rendered_nodes_to_html(&nodes), "<b>hello</b>");
    }

    #[test]
    fn replacement_nodes_use_current_text() {
        let nodes = vec![RenderedNode::Replacement {
            field_name: "Front".into(),
            current_text: "word".into(),
            filters: vec!["cloze".into()],
        }];
        assert_eq!(rendered_nodes_to_html(&nodes), "word");
    }

    #[test]
    fn mixed_nodes_concatenate_in_order() {
        let nodes = vec![
            RenderedNode::Text { text: "<div>".into() },
            RenderedNode::Replacement {
                field_name: "Front".into(),
                current_text: "civil".into(),
                filters: vec![],
            },
            RenderedNode::Text { text: "</div>".into() },
        ];
        assert_eq!(rendered_nodes_to_html(&nodes), "<div>civil</div>");
    }
}
