/** ホームの統計パネルが受け取る Tauri コマンド (deck_stats /
 *  deck_graph_stats) の DTO 型。Rust 側 commands/decks/ の serde 構造体と
 *  フィールド名を一致させる。 */

export type DeckStats = {
  total_cards: number;
  total_notes: number;
  new_cards: number;
  learn_cards: number;
  review_cards: number;
  suspended: number;
  buried: number;
};

export type TodayStats = {
  answer_count: number;
  answer_millis: number;
  correct_count: number;
  mature_count: number;
  mature_correct: number;
  learn_count: number;
  review_count: number;
  relearn_count: number;
};

export type Bucket<K> = { key: K; value: number };

export type CardCountsBreakdown = {
  new_cards: number;
  learn: number;
  relearn: number;
  young: number;
  mature: number;
  suspended: number;
  buried: number;
};

export type ReviewsBucket = {
  day: number;
  learn: number;
  relearn: number;
  young: number;
  mature: number;
  filtered: number;
};

export type ButtonsCounts = { learning: number[]; young: number[]; mature: number[] };

export type ButtonsByRange = {
  one_month: ButtonsCounts;
  three_months: ButtonsCounts;
  one_year: ButtonsCounts;
};

export type HourBucket = { hour: number; total: number; correct: number };

export type HoursByRange = {
  one_month: HourBucket[];
  three_months: HourBucket[];
  one_year: HourBucket[];
};

export type TrueRetention = {
  young_passed: number;
  young_failed: number;
  mature_passed: number;
  mature_failed: number;
};

export type RetentionStats = {
  today: TrueRetention;
  yesterday: TrueRetention;
  week: TrueRetention;
  month: TrueRetention;
  year: TrueRetention;
  all_time: TrueRetention;
};

export type DeckGraphStats = {
  today: TodayStats;
  future_due: Bucket<number>[];
  future_due_total: number;
  future_due_avg_per_day: number;
  future_due_have_backlog: boolean;
  daily_load: number;
  card_counts_separate: CardCountsBreakdown;
  card_counts_combined: CardCountsBreakdown;
  intervals: Bucket<number>[];
  eases: Bucket<number>[];
  eases_average: number;
  reviews: ReviewsBucket[];
  added: Bucket<number>[];
  buttons: ButtonsByRange;
  hours: HoursByRange;
  retention: RetentionStats;
};
