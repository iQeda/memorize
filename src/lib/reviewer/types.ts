/** Reviewer が受け取る Tauri コマンド (start_study / get_next_card /
 *  get_card_render) の DTO 型。Rust 側 commands/study.rs / reviewer.rs の
 *  serde 構造体とフィールド名を一致させる。 */

export type Counts = { new: number; learning: number; review: number };

export type StudyCard = {
  card_id: number;
  note_id: number;
  question_html: string;
  answer_html: string;
  css: string;
  remaining: Counts;
};

export type NextCard =
  | {
      kind: "card";
      card_id: number;
      note_id: number;
      question_html: string;
      answer_html: string;
      css: string;
      remaining: Counts;
    }
  | { kind: "done"; new: number; learning: number; review: number };

export type RenderedCard = {
  question_html: string;
  answer_html: string;
  css: string;
};
