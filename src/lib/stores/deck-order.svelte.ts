import { browser } from "$app/environment";

const ORDER_KEY = "memorize:deck-order";

/** Sidebar のトップレベルデッキの並び順を管理。子デッキは親に追従するので
 *  並び順 ID は level=0 のデッキのみを対象とする (実装簡略化)。
 *  Anki 側には並び順フィールドがないため memorize ローカルに localStorage で保持。 */
class DeckOrderStore {
  ids = $state<number[]>([]);

  constructor() {
    if (browser) {
      try {
        const raw = localStorage.getItem(ORDER_KEY);
        if (raw) {
          const parsed = JSON.parse(raw);
          if (Array.isArray(parsed)) {
            this.ids = parsed.filter((n) => typeof n === "number");
          }
        }
      } catch {}
    }
  }

  set(ids: number[]) {
    this.ids = ids;
    if (browser) localStorage.setItem(ORDER_KEY, JSON.stringify(ids));
  }

  /** 与えられたトップレベル ID 配列を保存済み順で並べる。
   *  - 保存順にあるが入力に無い ID (= 削除済みデッキ) はスキップ
   *  - 入力にあるが保存順に無い ID (= 新規デッキ) は元の順で末尾に積む */
  applyOrder(topIds: number[]): number[] {
    const known = new Set(topIds);
    const orderMap = new Map<number, number>();
    this.ids.forEach((id, i) => {
      if (known.has(id)) orderMap.set(id, i);
    });
    return [...topIds].sort((a, b) => {
      const ia = orderMap.get(a);
      const ib = orderMap.get(b);
      if (ia === undefined && ib === undefined) return 0;
      if (ia === undefined) return 1;
      if (ib === undefined) return -1;
      return ia - ib;
    });
  }

  /** ソース ID を、ターゲット ID の直前 (target が null なら末尾) に移動。
   *  内部 store はトップレベル ID の最新スナップショットで上書きする。 */
  move(topIds: number[], sourceId: number, targetId: number | null) {
    const current = this.applyOrder(topIds);
    const without = current.filter((id) => id !== sourceId);
    let next: number[];
    if (targetId === null) {
      next = [...without, sourceId];
    } else {
      const idx = without.indexOf(targetId);
      next =
        idx === -1
          ? [...without, sourceId]
          : [...without.slice(0, idx), sourceId, ...without.slice(idx)];
    }
    this.set(next);
  }
}

export const deckOrder = new DeckOrderStore();
