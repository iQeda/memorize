/** store の busy / busyReason / lastError ライフサイクルを一元化するヘルパー。
 *
 *  busy フラグを立てて fn を実行し、finally で必ずリセットする。エラーは
 *  lastError に格納して null を返す (opts.rethrow で再 throw に変更可)。
 *
 *  すべての非同期 store 操作を押し込む抽象ではない — sync.syncNow の
 *  ようにメッセージルーティングが絡むものや、collection.open のように
 *  state の形が違うものは対象外 (各実装のコメント参照)。 */
export interface BusyState {
  busy: boolean;
  busyReason: string | null;
  lastError: string | null;
}

export async function runAsync<T>(
  s: BusyState,
  fn: () => Promise<T>,
  opts?: { reason?: string; rethrow?: boolean },
): Promise<T | null> {
  s.busy = true;
  s.busyReason = opts?.reason ?? null;
  s.lastError = null;
  try {
    return await fn();
  } catch (e) {
    s.lastError = String(e);
    if (opts?.rethrow) throw e;
    return null;
  } finally {
    s.busy = false;
    s.busyReason = null;
  }
}
