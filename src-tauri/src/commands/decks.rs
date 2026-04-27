use crate::error::{AppError, AppResult};
use crate::state::AppState;
use anki::prelude::TimestampSecs;
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct DeckSummary {
    pub id: i64,
    pub name: String,
    pub level: u32,
    pub new_count: u32,
    pub learn_count: u32,
    pub review_count: u32,
}

#[tauri::command]
pub async fn list_decks(state: State<'_, AppState>) -> AppResult<Vec<DeckSummary>> {
    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;
    let tree = col.deck_tree(Some(TimestampSecs::now()))?;

    let mut out = Vec::new();
    walk(&tree, 0, &mut out);
    Ok(out)
}

#[derive(Serialize, Debug)]
pub struct DeckStats {
    pub total_cards: u32,
    pub total_notes: u32,
    pub new_cards: u32,
    pub learn_cards: u32,
    pub review_cards: u32,
    pub suspended: u32,
    pub buried: u32,
}

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
    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;

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

#[tauri::command]
pub async fn deck_stats(
    deck_id: i64,
    state: State<'_, AppState>,
) -> AppResult<DeckStats> {
    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;

    // Classify each card mutually exclusively by queue first.
    // queue: -1 = Suspended, -2/-3 = Buried, 0 = New, 1/3 = Learn,
    //        2 = Review.
    let db = col.storage.db();
    let count = |sql: &str| -> AppResult<u32> {
        db.query_row(sql, [deck_id], |r| r.get(0))
            .map_err(|e| AppError::Anyhow(anyhow::anyhow!(e)))
    };
    let in_deck = "(did = ?1 OR (odid != 0 AND odid = ?1))";
    let total_cards = count(&format!("SELECT COUNT(*) FROM cards WHERE {in_deck}"))?;
    let suspended = count(&format!(
        "SELECT COUNT(*) FROM cards WHERE {in_deck} AND queue = -1"
    ))?;
    let buried = count(&format!(
        "SELECT COUNT(*) FROM cards WHERE {in_deck} AND queue IN (-2, -3)"
    ))?;
    let new_cards = count(&format!(
        "SELECT COUNT(*) FROM cards WHERE {in_deck} AND queue = 0"
    ))?;
    let learn_cards = count(&format!(
        "SELECT COUNT(*) FROM cards WHERE {in_deck} AND queue IN (1, 3)"
    ))?;
    let review_cards = count(&format!(
        "SELECT COUNT(*) FROM cards WHERE {in_deck} AND queue = 2"
    ))?;
    let total_notes = count(&format!(
        "SELECT COUNT(DISTINCT nid) FROM cards WHERE {in_deck}"
    ))?;

    Ok(DeckStats {
        total_cards,
        total_notes,
        new_cards,
        learn_cards,
        review_cards,
        suspended,
        buried,
    })
}

#[tauri::command]
pub async fn create_deck(
    name: String,
    state: State<'_, AppState>,
) -> AppResult<i64> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::Anyhow(anyhow::anyhow!("deck name is empty")));
    }
    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;
    let deck = col.get_or_create_normal_deck(trimmed)?;
    Ok(deck.id.0)
}

#[tauri::command]
pub async fn rename_deck(
    deck_id: i64,
    new_name: String,
    state: State<'_, AppState>,
) -> AppResult<()> {
    let trimmed = new_name.trim();
    if trimmed.is_empty() {
        return Err(AppError::Anyhow(anyhow::anyhow!("deck name is empty")));
    }
    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;
    col.rename_deck(anki::prelude::DeckId(deck_id), trimmed)?;
    Ok(())
}

#[tauri::command]
pub async fn delete_deck(
    deck_id: i64,
    state: State<'_, AppState>,
) -> AppResult<usize> {
    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;
    let out = col.remove_decks_and_child_decks(&[anki::prelude::DeckId(deck_id)])?;
    Ok(out.output)
}

fn walk(node: &anki_proto::decks::DeckTreeNode, level: u32, out: &mut Vec<DeckSummary>) {
    if node.deck_id != 0 {
        out.push(DeckSummary {
            id: node.deck_id,
            name: node.name.clone(),
            level,
            new_count: node.new_count,
            learn_count: node.learn_count,
            review_count: node.review_count,
        });
    }
    for child in &node.children {
        walk(child, level + 1, out);
    }
}
