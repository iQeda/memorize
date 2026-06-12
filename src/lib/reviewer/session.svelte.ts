/** Reviewer のセッション状態 (現在カード / 残数 / 進捗) と
 *  start_study / get_next_card / answer_card_now の呼び出しを持つ
 *  runes クラス。ページはこのインスタンスを composition root として使う。 */

import { invoke } from "$lib/ipc";
import { collection } from "$lib/stores/collection.svelte";
import { t } from "$lib/i18n/index.svelte";
import type { Counts, NextCard, RenderedCard, StudyCard } from "./types";
import { adjustRemainingAfterSync } from "./totals";

export class ReviewSession {
  current = $state<StudyCard | null>(null);
  totals = $state<Counts>({ new: 0, learning: 0, review: 0 });
  initialTotal = $state(0);
  cursor = $state(0);
  showingAnswer = $state(false);
  loading = $state(false);
  error = $state<string | null>(null);
  shownAt = $state<number>(0);

  /** カード状態が裏で更新されたとき (card was modified) のユーザー通知。 */
  constructor(
    private deckId: () => number,
    private notify: (msg: string) => void,
    /** 新カード表示時のセッションローカル状態リセット (hideActive など)。 */
    private onCardChange: () => void,
  ) {}

  get totalDue(): number {
    return this.totals.new + this.totals.learning + this.totals.review;
  }

  get progress(): number {
    return this.initialTotal > 0 ? (this.cursor / this.initialTotal) * 100 : 0;
  }

  async start(): Promise<void> {
    this.loading = true;
    this.error = null;
    this.cursor = 0;
    try {
      await invoke<void>("start_study", { deckId: this.deckId() });
      await this.loadNext(/* isFirst */ true);
    } catch (e) {
      this.error = String(e);
    } finally {
      this.loading = false;
    }
  }

  async loadNext(isFirst = false): Promise<void> {
    this.showingAnswer = false;
    this.onCardChange();
    try {
      const r = await invoke<NextCard>("get_next_card");
      if (r.kind === "card") {
        this.current = {
          card_id: r.card_id,
          note_id: r.note_id,
          question_html: r.question_html,
          answer_html: r.answer_html,
          css: r.css,
          remaining: r.remaining,
        };
        this.totals = r.remaining;
        if (isFirst) {
          this.initialTotal =
            this.totals.new + this.totals.learning + this.totals.review + 1;
        }
        this.shownAt = performance.now();
      } else {
        this.current = null;
        this.totals = { new: r.new, learning: r.learning, review: r.review };
      }
    } catch (e) {
      this.error = String(e);
    }
  }

  flip(): void {
    this.showingAnswer = !this.showingAnswer;
  }

  async answer(rating: "again" | "hard" | "good" | "easy"): Promise<void> {
    if (!this.current) return;
    const ms = Math.min(60_000, Math.round(performance.now() - this.shownAt));
    try {
      await invoke<void>("answer_card_now", {
        rating,
        millisecondsTaken: ms,
      });
      this.cursor += 1;
      await this.loadNext();
      // サイドバーのデッキバッジは collection.decks を参照している。get_next_card
      // の remaining はヘッダー用に header の totals だけ更新するが、collection
      // ストアには反映されないので、答えるたびに list_decks を fire-and-forget で
      // 投げて全デッキのカウントを塗り直す。次カード描画はブロックしない。
      void collection.refreshDecks();
    } catch (e) {
      const msg = String(e);
      // rslib は楽観ロックで「フロントが提示された card state」と DB の最新値が
      // ずれていると `InvalidInput: card was modified` を返す。長時間 Reviewer を
      // 開きっぱなしで日付が跨ぎ elapsed_days が動いた、などで発生する。生の
      // エラー文言は出さず、ユーザー向けの一言を出してカードを reload する
      // (= loadNext で `last_queued` も最新 state で再構築される)。
      if (msg.includes("card was modified")) {
        this.notify(t("reviewer.cardStateChanged"));
        await this.loadNext();
        return;
      }
      this.error = msg;
    }
  }

  /** ノート編集後など、現在カードのレンダリングだけ取り直す。 */
  async reloadCurrent(): Promise<void> {
    if (!this.current) return;
    try {
      const r = await invoke<RenderedCard>("get_card_render", {
        cardId: this.current.card_id,
      });
      this.current = {
        ...this.current,
        question_html: r.question_html,
        answer_html: r.answer_html,
        css: r.css,
      };
    } catch (e) {
      console.error("get_card_render failed", e);
    }
  }

  /** Sync 完了後にバッジと分母を list_decks の最新値で塗り直す。
   *  (`totals` は get_next_card の remaining でしか更新されないため。) */
  async refreshTotalsAfterSync(): Promise<void> {
    await collection.refreshDecks();
    const d = collection.decks.find((x) => x.id === this.deckId());
    if (!d) return;
    const adjusted = adjustRemainingAfterSync(this.totals, {
      new: d.new_count,
      learning: d.learn_count,
      review: d.review_count,
    });
    this.totals = adjusted;
    // ヘッダーの "X / Y" 分母も Sync で増減した分だけ追従させる。
    // cursor は既に答えた枚数、+1 は現在カード、残り 3 カテゴリの合計が今後分。
    this.initialTotal =
      this.cursor + adjusted.new + adjusted.learning + adjusted.review + 1;
  }
}
