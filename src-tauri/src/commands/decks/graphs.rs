//! deck_graph_stats — ホーム画面の統計パネル群が使うグラフ集計。
//! rslib の `graph_data_for_search` (patches/0003 で公開) の応答を
//! 手書き DTO へ変換する。

use crate::error::AppResult;
use crate::state::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Serialize, Debug, Default)]
pub struct TodayStats {
    pub answer_count: u32,
    pub answer_millis: u32,
    pub correct_count: u32,
    pub mature_count: u32,
    pub mature_correct: u32,
    pub learn_count: u32,
    pub review_count: u32,
    pub relearn_count: u32,
}

#[derive(Serialize, Debug, Default)]
pub struct Bucket<K: Serialize> {
    pub key: K,
    pub value: u32,
}

#[derive(Serialize, Debug, Default)]
pub struct CardCountsBreakdown {
    pub new_cards: u32,
    pub learn: u32,
    pub relearn: u32,
    pub young: u32,
    pub mature: u32,
    pub suspended: u32,
    pub buried: u32,
}

#[derive(Serialize, Debug, Default)]
pub struct ReviewsBucket {
    pub day: i32,
    pub learn: u32,
    pub relearn: u32,
    pub young: u32,
    pub mature: u32,
    pub filtered: u32,
}

#[derive(Serialize, Debug, Default)]
pub struct ButtonsCounts {
    pub learning: Vec<u32>,
    pub young: Vec<u32>,
    pub mature: Vec<u32>,
}

#[derive(Serialize, Debug, Default)]
pub struct ButtonsByRange {
    pub one_month: ButtonsCounts,
    pub three_months: ButtonsCounts,
    pub one_year: ButtonsCounts,
}

#[derive(Serialize, Debug, Default)]
pub struct HourBucket {
    pub hour: u32,
    pub total: u32,
    pub correct: u32,
}

#[derive(Serialize, Debug, Default)]
pub struct HoursByRange {
    pub one_month: Vec<HourBucket>,
    pub three_months: Vec<HourBucket>,
    pub one_year: Vec<HourBucket>,
}

#[derive(Serialize, Debug, Default)]
pub struct TrueRetention {
    pub young_passed: u32,
    pub young_failed: u32,
    pub mature_passed: u32,
    pub mature_failed: u32,
}

#[derive(Serialize, Debug, Default)]
pub struct RetentionStats {
    pub today: TrueRetention,
    pub yesterday: TrueRetention,
    pub week: TrueRetention,
    pub month: TrueRetention,
    pub year: TrueRetention,
    pub all_time: TrueRetention,
}

#[derive(Serialize, Debug, Default)]
pub struct DeckGraphStats {
    pub today: TodayStats,
    pub future_due: Vec<Bucket<i32>>,
    pub future_due_total: u32,
    pub future_due_avg_per_day: f32,
    pub future_due_have_backlog: bool,
    pub daily_load: u32,
    pub card_counts_separate: CardCountsBreakdown,
    pub card_counts_combined: CardCountsBreakdown,
    pub intervals: Vec<Bucket<u32>>,
    pub eases: Vec<Bucket<u32>>,
    pub eases_average: f32,
    pub reviews: Vec<ReviewsBucket>,
    pub added: Vec<Bucket<i32>>,
    pub buttons: ButtonsByRange,
    pub hours: HoursByRange,
    pub retention: RetentionStats,
}

fn map_to_buckets<K: Ord + Copy + Serialize>(m: std::collections::HashMap<K, u32>) -> Vec<Bucket<K>> {
    let mut v: Vec<Bucket<K>> = m
        .into_iter()
        .map(|(key, value)| Bucket { key, value })
        .collect();
    v.sort_by_key(|b| b.key);
    v
}

fn convert_buttons(b: anki_proto::stats::graphs_response::buttons::ButtonCounts) -> ButtonsCounts {
    ButtonsCounts {
        learning: b.learning,
        young: b.young,
        mature: b.mature,
    }
}

fn convert_hours(hs: Vec<anki_proto::stats::graphs_response::hours::Hour>) -> Vec<HourBucket> {
    hs.into_iter()
        .enumerate()
        .map(|(i, h)| HourBucket {
            hour: i as u32,
            total: h.total,
            correct: h.correct,
        })
        .collect()
}

fn convert_retention(
    r: anki_proto::stats::graphs_response::true_retention_stats::TrueRetention,
) -> TrueRetention {
    TrueRetention {
        young_passed: r.young_passed,
        young_failed: r.young_failed,
        mature_passed: r.mature_passed,
        mature_failed: r.mature_failed,
    }
}

#[tauri::command]
pub async fn deck_graph_stats(
    deck_id: i64,
    days: u32,
    state: State<'_, AppState>,
) -> AppResult<DeckGraphStats> {
    tracing::info!(deck_id, days, "deck_graph_stats called");
    state
        .with_collection(|col| deck_graph_stats_inner(col, deck_id, days))
        .await
}

fn deck_graph_stats_inner(
    col: &mut anki::collection::Collection,
    deck_id: i64,
    days: u32,
) -> AppResult<DeckGraphStats> {
    let search = format!("did:{}", deck_id);
    let resp = col.graph_data_for_search(&search, days)?;
    tracing::info!("deck_graph_stats: graph_data_for_search ok");

    let today = resp.today.unwrap_or_default();
    let fd = resp.future_due.unwrap_or_default();
    let cc = resp.card_counts.unwrap_or_default();
    let intervals = resp.intervals.unwrap_or_default();
    let eases = resp.eases.unwrap_or_default();
    let reviews_proto = resp.reviews.unwrap_or_default();
    let added_proto = resp.added.unwrap_or_default();
    let buttons = resp.buttons.unwrap_or_default();
    let hours = resp.hours.unwrap_or_default();
    let retention = resp.true_retention.unwrap_or_default();

    let max_day = days as i32;
    let mut future_due_buckets: Vec<Bucket<i32>> = fd
        .future_due
        .into_iter()
        .filter(|(d, _)| *d >= 0 && *d < max_day)
        .map(|(key, value)| Bucket { key, value })
        .collect();
    future_due_buckets.sort_by_key(|b| b.key);
    let total: u32 = future_due_buckets.iter().map(|b| b.value).sum();
    let avg = if max_day > 0 {
        total as f32 / max_day as f32
    } else {
        0.0
    };

    let cc_sep = cc.excluding_inactive.unwrap_or_default();
    let cc_comb = cc.including_inactive.unwrap_or_default();
    let convert_cc = |c: anki_proto::stats::graphs_response::card_counts::Counts| {
        CardCountsBreakdown {
            new_cards: c.new_cards,
            learn: c.learn,
            relearn: c.relearn,
            young: c.young,
            mature: c.mature,
            suspended: c.suspended,
            buried: c.buried,
        }
    };

    let mut reviews_vec: Vec<ReviewsBucket> = reviews_proto
        .count
        .into_iter()
        .map(|(day, r)| ReviewsBucket {
            day,
            learn: r.learn,
            relearn: r.relearn,
            young: r.young,
            mature: r.mature,
            filtered: r.filtered,
        })
        .collect();
    reviews_vec.sort_by_key(|b| b.day);

    Ok(DeckGraphStats {
        today: TodayStats {
            answer_count: today.answer_count,
            answer_millis: today.answer_millis,
            correct_count: today.correct_count,
            mature_count: today.mature_count,
            mature_correct: today.mature_correct,
            learn_count: today.learn_count,
            review_count: today.review_count,
            relearn_count: today.relearn_count,
        },
        future_due: future_due_buckets,
        future_due_total: total,
        future_due_avg_per_day: avg,
        future_due_have_backlog: fd.have_backlog,
        daily_load: fd.daily_load,
        card_counts_separate: convert_cc(cc_sep),
        card_counts_combined: convert_cc(cc_comb),
        intervals: map_to_buckets(intervals.intervals),
        eases: map_to_buckets(eases.eases),
        eases_average: eases.average,
        reviews: reviews_vec,
        added: map_to_buckets(added_proto.added),
        buttons: ButtonsByRange {
            one_month: buttons.one_month.map(convert_buttons).unwrap_or_default(),
            three_months: buttons.three_months.map(convert_buttons).unwrap_or_default(),
            one_year: buttons.one_year.map(convert_buttons).unwrap_or_default(),
        },
        hours: HoursByRange {
            one_month: convert_hours(hours.one_month),
            three_months: convert_hours(hours.three_months),
            one_year: convert_hours(hours.one_year),
        },
        retention: RetentionStats {
            today: retention.today.map(convert_retention).unwrap_or_default(),
            yesterday: retention.yesterday.map(convert_retention).unwrap_or_default(),
            week: retention.week.map(convert_retention).unwrap_or_default(),
            month: retention.month.map(convert_retention).unwrap_or_default(),
            year: retention.year.map(convert_retention).unwrap_or_default(),
            all_time: retention.all_time.map(convert_retention).unwrap_or_default(),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn map_to_buckets_sorts_by_key_ascending() {
        let mut m: HashMap<i32, u32> = HashMap::new();
        m.insert(5, 50);
        m.insert(-1, 10);
        m.insert(2, 20);
        let buckets = map_to_buckets(m);
        let keys: Vec<i32> = buckets.iter().map(|b| b.key).collect();
        let values: Vec<u32> = buckets.iter().map(|b| b.value).collect();
        assert_eq!(keys, vec![-1, 2, 5]);
        assert_eq!(values, vec![10, 20, 50]);
    }

    #[test]
    fn map_to_buckets_empty_input() {
        let buckets = map_to_buckets(HashMap::<u32, u32>::new());
        assert!(buckets.is_empty());
    }

    #[test]
    fn convert_hours_assigns_index_as_hour_label() {
        let proto = vec![
            anki_proto::stats::graphs_response::hours::Hour { total: 1, correct: 1 },
            anki_proto::stats::graphs_response::hours::Hour { total: 5, correct: 3 },
            anki_proto::stats::graphs_response::hours::Hour { total: 0, correct: 0 },
        ];
        let out = convert_hours(proto);
        assert_eq!(out.len(), 3);
        assert_eq!((out[0].hour, out[0].total, out[0].correct), (0, 1, 1));
        assert_eq!((out[1].hour, out[1].total, out[1].correct), (1, 5, 3));
        assert_eq!((out[2].hour, out[2].total, out[2].correct), (2, 0, 0));
    }
}
