---
name: refactor
description: memorize コードベース全体の段階的リファクタリング計画と実行手順。「リファクタリングして」「リファクタの続きをやって」「Phase N を実行して」のような指示が出たとき、または重複排除・巨大ファイル分割・テスト補強・設定のゴミ掃除の作業を開始/再開するときに使う。2026-06-12 の全体監査（多段検証済み findings 32 件）に基づく 7 フェーズ構成。進捗は本ファイルのチェックボックスで管理する。memorize リポジトリ専用。
---

# memorize 全体リファクタリング計画

2026-06-12 実施の全体監査（4 次元並列監査 → 敵対的検証 → 漏れスイープ、9 エージェント）で確定した無駄を、**7 フェーズ**で解消する。スコープは **挙動不変の純リファクタ + テスト・型補強 + 依存/設定のゴミ掃除**。UX 変更・機能追加は含まない。

> **行番号について**: 本ファイルの行番号はすべて **2026-06-12 時点（commit `8cb9775`）** のもの。フェーズが進むとドリフトするため、実行時は行番号ではなく **記載のコードパターンを grep で再特定** すること。

## 進捗トラッカー

実行セッションはフェーズ完了ごとにここを更新する（タスク単位のチェックは各フェーズ内）。

- [x] Phase 1 — リポジトリ衛生（config / docs / i18n 死にキー）
- [x] Phase 2 — フロントエンド基盤（共有ユーティリティと store 統一）
- [x] Phase 3 — Rust テスト補強（構造変更前の安全網）
- [x] Phase 4 — Rust 構造整理（with_collection / decks 分割 / SQL 統合）
- [x] Phase 5 — チャート共通化 + ホーム画面分割
- [x] Phase 6 — Settings 画面分割（1308 行 → 92 行）
- [ ] Phase 7 — Reviewer 画面分割（1176 行 → 約 350 行、最高リスクなので最後）

依存関係:

```
P1 ──► (どこからでも)
P2 ──► P6 ──► P7        SPEECH_LIMITS → SpeechControls rows → popover 転用
P3 ──► P4               Rust はテスト追加が構造変更より先（厳守）
P5 ⟂ Rust トラック       チャート/ホームは独立。P2 完了後ならいつでも可
```

1 フェーズ ≈ 1 セッション。フェーズ内はタスクごとに 1 commit（コミット規約は下記）。**フェーズ途中でセッションが終わる場合は、完了タスクのチェックボックスを更新してから終えること。**

## 全フェーズ共通の鉄則

1. **検証ゲート（フェーズ完了の条件）** — 全部 green になるまでフェーズ完了と言わない:

   ```sh
   pnpm test:run
   PROTOC=/opt/homebrew/bin/protoc cargo test --manifest-path src-tauri/Cargo.toml
   pnpm exec svelte-check --tsconfig ./tsconfig.json
   cargo check --manifest-path src-tauri/Cargo.toml
   pnpm build          # .svelte を触ったフェーズは必須（CardFrame の <script> tokenizer 罠）
   ```

2. **テスト規律（CLAUDE.md 準拠）** — 非自明な変更は同一 commit にテスト同梱。挙動不変の純粋な移動・リネームは免除だが、**commit メッセージに「純リファクタ・挙動不変」と明記**する。
3. **コミット規約** — 日本語 `refactor(area): 内容`。例: `refactor(settings): トグルスイッチ 6 箇所を ToggleSwitch.svelte に統合（純リファクタ・挙動不変）`。
4. **vendor/anki は触らない**。`git add -A` / `git add .` 禁止（submodule の dirty を拾うため）。常にファイル名を明示して add。
5. **push はユーザーの明示指示があるときだけ**。push 前は CLAUDE.md の pre-push チェックリストを通す。リリースするなら version bump は patch（リファクタのみのため）。
6. **守るべき既存設計**（リファクタで「ついでに直したく」なっても触らない）:
   - Tauri コマンドは手書き DTO（prost 型を返さない）
   - deck 統計は `c.queue` ベースの SQL（`is:learn` 等の検索は使わない）
   - `src-tauri/Cargo.toml` に `[workspace]` を足さない
   - チャートの `{#each}` は index キー（`as v, i (i)`）
   - store は `.svelte.ts` の runes クラス
   - `.svelte` 内の JS テンプレートリテラルに `<script>` リテラルを書かない

## 監査で棄却・降格された項目（やらないことリスト）

実行セッションがこれらを「発見」して着手しないこと。検証済みの棄却理由:

| 項目 | 判定 | 理由 |
|---|---|---|
| `custom-protocol` feature の削除（src-tauri/Cargo.toml:48） | **削除禁止** | tauri-cli が `tauri build` 時に `--features custom-protocol` で外部から有効化する。repo 内 grep でヒットしないのは当然で、削除すると本番 DMG ビルドが壊れ、検証ゲートでは検出できない。→ Phase 1 でコメント追記のみ |
| KeyDispatcher store（キーボード処理の一元化） | **過剰設計** | 3 つの onKey は意図的にポリシーが違う（+layout は ⌘S/⌘, をテキストフィールド内でも発火、review は modifier 全拒否 + `editing` フラグ、home は `defaultPrevented` チェック）。統一すると退行リスクだけ増える。→ Phase 2 で述語関数の最小抽出のみ |
| speech.rs（377 行）のモジュール分割 | **不要** | 377 行中 143 行は `#[cfg(test)]`。実体は約 230 行・1 コマンド + テスト済み純ヘルパー 5 個で凝集している |
| StatCard を home countCard と settings import-stats で共用 | **統合しない** | settings 側（834-852 行）は既に `{#each}` データ駆動で重複なし。home の countCard はトーングラデーション + アニメーション付きで別物。無理に統合すると prop だらけの何でもコンポーネントになる |
| `@testing-library/svelte` の導入 | **導入しない** | 既存テスト基盤は vitest + jsdom + `$app/environment` スタブのみ。コンポーネントマウントテストより「純ロジックを関数に抽出して jsdom でテスト」が本リポジトリの流儀 |
| app.rs のユニットテスト | **免除** | `confirm_exit` は AtomicBool 反転 + `app.exit(0)` の 14 行。AppHandle のモックはフレームワークのモックをテストするだけになる |

---

## Phase 1 — リポジトリ衛生（規模 S・半セッション）

**ゴール**: ゼロリスク項目を全部焼き払い、以降の diff をきれいにする。

- [x] **1.1 README の DMG 名ドリフト修正** — `README.md` 44 行・99 行の `memorize_<ver>_aarch64.dmg` → `Memorize_<ver>_aarch64.dmg`（`tauri.conf.json` の productName が `Memorize` のため CI 出力は大文字始まり）。テスト不要（ドキュメント）。
- [x] **1.2 実行不能な permission の削除** — `.claude/settings.json` から `Bash(npm run build:check)` と `Bash(pnpm lint*)` を削除（対応するスクリプトが package.json に存在しない）。※permission 分類器が Claude からの編集を拒否したためユーザーが手動対応（2026-06-12）。
- [x] **1.3 pnpm バージョンの単一ソース化** — `package.json` に `"packageManager": "pnpm@10.x"`（手元の実バージョンに合わせる）と `"engines": { "node": ">=22" }` を追加。**同一 commit で** `.github/workflows/release.yml` の `pnpm/action-setup` から `version: 10` 入力を削除（`packageManager` と `version` が両方あると action-setup がエラーになる）。検証: `pnpm install --frozen-lockfile` が通ること。
- [x] **1.4 custom-protocol feature に警告コメント** — `src-tauri/Cargo.toml` の `[features]` に「tauri-cli が build 時に有効化する。未参照に見えても削除禁止」とコメント追記。
- [x] **1.5 i18n の parity テスト + 死にキー削除**（確定 2 件に加え、監査で `decks.parentPath` / `sync.fullRequired` もゼロ参照と判明し計 4 キー削除） — 順序が重要:
  1. 先に `src/lib/i18n/i18n.test.ts` へ **en↔ja キー集合一致テスト** を追加（`Object.keys(messages.en)` と `Object.keys(messages.ja)` の set 等価。テストファイル自体は存在するが、この assert はまだ無い）
  2. 確定死にキー `nav.home`（en:30 行 / ja:347 行付近）と `reviewer.reload`（en:84 / ja:401 付近）を **両ロケールから** 削除
  3. 全キー監査: messages.ts の各キーを `src/`（messages.ts 自身を除く）に対して grep。動的キー構築が無いことは検証済み（`t(\`` のヒットなし）だが、削除前に再確認すること。grep でゼロ参照のキーのみ削除
- [x] **1.6 ハードコード日本語の i18n 化** — `src/lib/stores/package.svelte.ts`（61, 92, 109 行）の `"Import 中…"` / `"Export 中…"` を `t("io.importing")` / `t("io.exporting")` に置換。messages.ts の en/ja 両方にキー追加。`package.svelte.test.ts` がリテラルを assert していれば追従。

**ゲート**: 共通ゲート全部（.svelte は触らないが `pnpm build` も一応通す）。

---

## Phase 2 — フロントエンド基盤（規模 M〜L・1 セッション）

**ゴール**: 後続フェーズが import する共有モジュールを、それぞれ単体テスト付きで先に作る。アプリ挙動は不変。

- [x] **2.1 キーボード述語の抽出** — 新規 `src/lib/utils/keyboard.ts`:

  ```ts
  /** フォーカス先がテキスト入力か（input / textarea / select / contentEditable） */
  export function isTextFieldTarget(target: EventTarget | null): boolean;
  /** meta / ctrl / alt のいずれかが押下されているか */
  export function hasModifier(e: KeyboardEvent): boolean;
  ```

  実装は `+layout.svelte` 39-49 行の広いバリアント（tag + contentEditable）を移す。採用先: `+layout.svelte` と `src/routes/+page.svelte` 192-200 行（意味的に同一なことを検証済み）。**review ページの inline チェックは意図的に狭い（input/textarea のみ + `editing` フラグ）ので触らず**、その旨のコメントだけ追記。テスト: `keyboard.test.ts`（jsdom で input / textarea / select / contentEditable div / 素の div / null）。
- [x] **2.2 localStorage キーの集約** — 新規 `src/lib/storage-keys.ts` に `STORAGE_KEYS` 定数オブジェクト。現在 **定義箇所 7 ファイル**（shortcuts / collection / theme / speech / sync / deck-order の各 store + `i18n/index.svelte.ts`）に分散する **17 個** の `memorize:*` キーを集約（`+layout.svelte` と review ページにも参照があるが、それらは store 経由のため定義の集約だけでよい）。**文字列値はバイト同一を維持**（変えるとユーザー設定が消える。マイグレーションはスコープ外）。テスト: 全値のスナップショット assert（将来のリネームで利用者データが silent に孤児化するのを防ぐ）+ 値のユニーク性。
- [x] **2.3 busy/error パターンの共通化** — 新規 `src/lib/stores/run-async.ts`:

  ```ts
  export interface BusyState { busy: boolean; busyReason: string | null; lastError: string | null }
  export async function runAsync<T>(
    s: BusyState, fn: () => Promise<T>,
    opts?: { reason?: string; rethrow?: boolean },
  ): Promise<T | null>;  // finally で busy/busyReason を必ずリセット
  ```

  採用先は **無理なく合う箇所だけ**: `package.svelte.ts`（3 箇所）、`notes.svelte.ts`、`sync.svelte.ts` の login/logout/manualBackup/restore、`collection.svelte.ts::open`。`sync.svelte.ts` の `syncNow`/`runWithAutoBackup`（メッセージルーティングが絡む）は **無理に合わせない** — 4 store の状態形が違うものを 1 つの抽象に押し込むのが過剰抽象の罠。テスト: `run-async.test.ts`（成功 / throw+swallow / throw+rethrow / finally での busy リセット）。**注意: sync.svelte.ts は現在テストファイルが無い**ので、これを触る commit で `sync.svelte.test.ts` を新設し busy ライフサイクル（エラー経路で busy=false に戻る、を含む）を assert する。
- [x] **2.4 invoke の型注釈** — 型パラメータ無しの `invoke()` 呼び出しに明示注釈（戻り値未使用なら `invoke<void>`）。判明箇所: `collection.svelte.ts` 75 行付近、`sync.svelte.ts` 227 行付近。他も sweep。`src/lib/ipc.ts` のラッパー（generic デフォルト unknown）は既存のまま。検証は svelte-check（型のみの変更）。
- [x] **2.5 SPEECH_LIMITS への定数統合** — `src/lib/stores/speech.svelte.ts`（14-47 行）の個別 export 定数 15 個を 1 つに:

  ```ts
  export const SPEECH_LIMITS = {
    maxRepeat:         { min: 1,   max: 10,   default: 3 },
    repeatIntervalSec: { min: 0,   max: 10,   default: 1 },
    rateWpm:           { min: 100, max: 400,  default: 150 },
    sentencePauseMs:   { min: 0,   max: 5000, default: 500 },
    volume:            { min: 0,   max: 200,  default: 100 },
  } as const;
  ```

  （値は **現行コードから転記** すること。上記は形の例）。消費側更新: `settings/+page.svelte`、`review/[deckId]/+page.svelte`、`speech.svelte.test.ts`。**Phase 6 の SpeechControls がこの API 前提なので、必ず P6 より先に**。既存 speech テストが守る。
- [x] **2.6 CalendarHeatmap のハードコード色** — `src/lib/components/charts/CalendarHeatmap.svelte` 25 行の `rgba(124, 138, 255, opacity)` を `color-mix(in srgb, var(--color-accent-500) ${pct}%, transparent)` に置換（app.css のトークン形式を先に確認）。純表示変更につき免除明記 + 目視確認を commit に記す。
- [x] **2.7 draggable のテスト** — `src/lib/actions/draggable.ts` の `isInteractive`（純粋な DOM ツリーウォーク）を export し、`draggable.test.ts` を新設（jsdom: ネストした button / anchor / `data-no-drag` 祖先 / 素の div）。`@tauri-apps/api/window` は `vi.mock`。新規依存は追加しない。

**ゲート**: 共通ゲート + `pnpm build` 必須（2.5/2.6 で .svelte を触る）。

---

## Phase 3 — Rust テスト補強(規模 L・1 セッション)

**ゴール**: Phase 4 の構造変更より**前に**安全網を張る。変更は追加 + 微小な抽出のみ。テストパターンは既存の `cards.rs`(tempfile::TempDir + CollectionBuilder)に倣う。

- [x] **3.1 RenderedNode 変換の共通化** — 新規 `src-tauri/src/render.rs` に `pub fn rendered_nodes_to_html(nodes: &[RenderedNode]) -> String`。`reviewer.rs::render_node_to_html`(43-52 行、ノード単位 map)と `study.rs::render_nodes`(35-43 行、スライス処理)が同一出力なことを確認済み。両者を置換し、`lib.rs` に `mod render;` 追加。テスト: Text variant / Replacement variant / 空スライス。
- [x] **3.2 reviewer.rs のテスト** — `get_card_render_inner(col: &mut Collection, card_id: i64)` を抽出し、DB-backed テスト追加(note 作成 → render → q/a HTML が非空 + css round-trip)。
- [x] **3.3 study.rs のテスト** — `start_study` / `get_next_card` / `answer_card_now` から `*_inner(col, ...)` を抽出し、DB-backed テスト追加(collection 構築 → note 追加 → study 開始 → next card 取得 → Good 回答 → queue 遷移を assert)。
- [x] **3.4 sync.rs の credentials テスト** — `credentials_path(app)` を `credentials_path_in(dir: &Path)` にリファクタ(コマンド側は AppHandle からディレクトリを解決して渡す)。テスト: TempDir で save→load→delete round-trip(61 行付近の **0600 chmod も assert**)+ `auth_from` のホスト番号/endpoint マッピング。`sync_now`/`full_sync` はネットワーク依存のため対象外と module doc コメントに明記。
- [x] **3.5 deck_stats の回帰テストを先に追加(Phase 4 の安全網)** — **現行コードに対して** DB-backed `deck_stats` テストを書く: 各 queue 状態(new/learn/review/suspended/buried)のカードを作り、**suspended-while-learning のカードが suspended のみにカウントされる**こと(過去バグの再発防止。CLAUDE.md の queue-vs-type ルールのテスト化)とカテゴリの相互排他を assert。必要なら `deck_stats_inner(col, deck_id)` を抽出してテスト可能な形にする。
- [x] **3.6 app.rs の免除を文書化** — テスト追加はしない(冒頭の棄却リスト参照)。module doc コメントに免除理由を 1 行残す。

**ゲート**: 共通ゲート。cargo test のテスト数が明確に増えていること(現状: reviewer 0 / study 0 / sync 0)。

---

## Phase 4 — Rust 構造整理(規模 L・1 セッション、P3 完了が前提)

**ゴール**: バックエンドの重複排除と分割。P3 のテストが守る。

- [x] **4.1 with_collection ヘルパー** — `src-tauri/src/state.rs` に追加:

  ```rust
  impl AppState {
      pub async fn with_collection<T>(
          &self,
          f: impl FnOnce(&mut Collection) -> Result<T, AppError>,
      ) -> Result<T, AppError> {
          let mut guard = self.col.lock().await;
          let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;
          f(col)
      }
  }
  ```

  `let mut guard = state.col.lock().await; let col = guard.as_mut().ok_or(...)?` パターンは**実測 30 箇所・9 ファイル**(監査初報の 38 は過大)。採用は **closure が Collection に対して同期的な箇所だけ**(decks CRUD / cards / notes / reviewer / collection_info / csv / package / backup ≒ 24 コマンド)。**適用除外**: `sync.rs::full_sync`(282 行で Collection を `.take()` してムーブする)と `study.rs`(`last_queued` の第二 mutex と交錯 — ロック順序が変わる)。除外箇所には手動パターン維持の理由コメントを残す。純リファクタ・既存テストがゲート。
- [x] **4.2 エラーハンドリング統一** — `error.rs` に `#[error("{0}")] InvalidInput(String)` と `#[error(transparent)] Db(#[from] rusqlite::Error)` variant を追加。`decks.rs` の `AppError::Anyhow(anyhow::anyhow!(...))`(309, 350, 366 行)と `.map_err` 混在を新 variant に統一。**エラーメッセージ文字列は不変に保つ**(フロントに表示されるため)。テスト: `error.rs` 既存の serde テストを新 variant に拡張。
- [x] **4.3 deck_stats の SQL 統合** — `decks.rs` 311-330 行の 7 つの `format!` SELECT を統合:

  ```rust
  fn count_cards(db: &rusqlite::Connection, deck_id: i64, extra_predicate: &str) -> Result<u32, AppError>
  ```

  ベースの `(did = ?1 OR (odid != 0 AND odid = ?1))` フィルタと **queue 述語は逐語維持**。P3.5 の回帰テストが green のまま = 意味保存の証明。
- [x] **4.4 decks.rs の分割(最後に実施)** — 489 行(うち約 90 行はテスト)を `commands/decks/mod.rs`(CRUD + walk + DeckSummary)/ `commands/decks/stats.rs`(deck_stats + DTO + count_cards)/ `commands/decks/graphs.rs`(deck_graph_stats + 変換ヘルパー)へ純移動。**`lib.rs` の `generate_handler!` リストは `decks/mod.rs` からの re-export で無変更に保つ**(マクロのパス churn ゼロ)。移動は 4.1-4.3 で最終形になった後に行う(移動とロジック変更を同一 commit に混ぜない)。

**ゲート**: 共通ゲート。フロントは触らないが全件実行。

---

## Phase 5 — チャート共通化 + ホーム分割(規模 M〜L・1 セッション)

**ゴール**: ホーム(672 行)を約 250 行へ。チャート数値ロジックを一元化しテストで固定。

- [x] **5.1 chart-utils の新設** — 新規 `src/lib/components/charts/chart-utils.ts`:

  ```ts
  export const CHART_W = 720; export const CHART_H = 140;
  export type ChartPad = { l: number; r: number; t: number; b: number };
  export const DEFAULT_PAD: ChartPad = { l: 28, r: 28, t: 6, b: 20 };
  export function inner(pad?: Partial<ChartPad>): { w: number; h: number; pad: ChartPad };
  export function tickValues(max: number, steps?: number): number[];
  ```

  **注意: 定数は完全一致ではない** — `ButtonsChart` は `padR=6, padB=24`(他は 28/20)。override 引数で吸収する。テスト: `chart-utils.test.ts` — tickValues を max 1 / 4 / 5 / 97 で(重複なし・昇順・末尾 === max)。**疎データで重複 tick が出るケースを必ず含める**(過去に keyed `{#each}` の duplicate-key クラッシュを起こした事故のテスト化)。
- [x] **5.2 5 チャートへ採用** — Histogram / FutureDue / StackedBar / Hours / Buttons の各 .svelte からローカル定数と tickValues 実装を削除し chart-utils を import。**`{#each ... as v, i (i)}` の index キーは絶対に維持**。y 軸グリッドの markup 自体は各チャートに残す(SVG サブコンポーネント化は 16 行の節約に対し複雑度が見合わない — 数式だけ共有)。
- [x] **5.3 RangeTabs コンポーネント** — 新規 `src/lib/components/RangeTabs.svelte`(`<script generics="T extends string | number">`)。props: `{ options: { value: T; label: string }[]; value: T; onSelect: (v: T) => void }`。class は現行スニペットから逐語コピー。`src/routes/+page.svelte` 611-651 行の `rangeBtn` / `hRangeBtn` / `bRangeBtn` 3 スニペットを置換。
- [x] **5.4 ホームの分割** — 純移動中心:
  - `src/lib/stats/types.ts` — DTO 型(DeckStats 〜 DeckGraphStats、約 80 行)を移動
  - `src/lib/stats/transform.ts` — 純関数 `buildReviewsCols` / `buildAddedCols` / `buildCalendarPerDay` / `formatDuration`(166-172, 360-393 行)を抽出。**テスト必須**: `transform.test.ts`(疎な日付 / 負の day マッピング / 空入力 — 現在テストゼロの実ロジック)
  - `src/lib/components/home/WelcomeScreen.svelte` — `!collection.isOpen` ブランチ(248-286 行)
  - `src/lib/components/home/StatsPanelGrid.svelte` — `panel` スニペット + 11 パネル + `hoursRange`/`buttonsRange` ローカル state + RangeTabs。props: `{ graph, graphDays, onDaysChange }`
  - `countCard` スニペットは `+page.svelte` に残す(棄却リスト参照)

**ゲート**: 共通ゲート + `pnpm build` 必須 + ホームの統計グリッドを `pnpm tauri dev` で目視スモーク(チャートは green チェックが画素の正しさを最も保証しない場所)。

---

## Phase 6 — Settings 分割(規模 L・1 セッション、P2 完了が前提)

**ゴール**: `src/routes/settings/+page.svelte` 1308 行 → 約 150 行の composition root。共有プリミティブを確立し、SpeechControls(rows 版)を誕生させる。

ビルド順(プリミティブ → セクション → ページ差し替え):

- [x] **6.1 ToggleSwitch.svelte** — 新規 `src/lib/components/ToggleSwitch.svelte`。props: `{ checked: boolean; onToggle: (next: boolean) => void; label: string /* aria-label */; disabled?: boolean }`。`h-5 w-9 rounded-full` + inner span `left-[18px]/left-0.5` パターンの **6 箇所**(sync 自動同期 / backup 自動 / autostart / speech 自動読み上げ / repeat-on-start / hide-default。630-644, 735-744, 978-987, 1018-1027, 1042-1051, 1233-1242 行)を置換。
- [x] **6.2 SettingRow.svelte** — 新規 `src/lib/components/settings/SettingRow.svelte`。props: `{ icon, label: string, subtitle?: string, divider?: boolean, action: Snippet }`。`flex items-center justify-between gap-4` の **18+ 箇所** と `mt-0.5 text-xs text-(--color-fg-subtle)` subtitle class の **15+ 箇所** を吸収。
- [x] **6.3 SettingsSection.svelte / SettingsToc.svelte** — セクションの章 chrome(`<section id scroll-mt-20>` + h3 + カード枠)と左サイドの目次 nav を部品化。
- [x] **6.4 SpeechControls.svelte(rows 版)** — 新規 `src/lib/components/SpeechControls.svelte`。props: `{ layout: "rows" | "popover" }`(popover 実装は Phase 7)。speech store を直接読み書きし、共有 7 コントロール(自動読み上げ / repeat-on-start / maxRepeat / interval / rate+プレビュー / volume+プレビュー / sentence pause)を一度だけ実装。**プレビューボタンの `invoke("start_speak_text")` は単一の `preview()` ハンドラに統合**(settings 内の完全重複 2 箇所 [1132-1147, 1175-1190 行] をここで吸収。review popover の同一 invoke も P7 で合流)。SPEECH_LIMITS(P2.5)前提。
- [x] **6.5 セクションコンポーネント群** — `src/lib/components/settings/` 配下に、それぞれ自分の script ロジックを持ち上げて移動:
  `UpdatesCard.svelte`(updater 状態機械、177-501 行)/ `CollectionSection.svelte`(508-555)/ `SyncSection.svelte`(ログインフォーム + 同期ボタン、557-713)/ `BackupSection.svelte`(715-800)/ `ImportExportSection.svelte`(803-900)/ `LanguageSection.svelte`(907-930)/ `AppearanceSection.svelte`(932-955)/ `StartupSection.svelte`(autostart、137-175 + 957-996)/ `SpeechSection.svelte`(SpeechControls rows のラップ + hideDefault 行 + macOnly 注記)/ `ShortcutsSection.svelte`(ショートカット録製、110-126 + 1254-1306)。
- [x] **6.6 +page.svelte の書き換え** — TOC データ + ページグリッド + セクション composition のみの約 150 行へ。
- [x] **6.7 テスト成果物** — コンポーネントはマウントテストしない方針(棄却リスト)のため、このフェーズのテストは: `UpdatesCard` のダウンロード進捗計算(received/total → percent)を `src/lib/updater-progress.ts` に純関数抽出して単体テスト。既存 store テスト全 green 維持。移動のみの commit は純リファクタ明記。

**手動スモーク(commit/PR に結果を記録)**: 全トグルがリロード後も永続 / TOC アンカースクロール / ショートカット再バインド / アップデートチェック表示 / speech プレビュー再生。

**ゲート**: 共通ゲート + `pnpm build` 必須(大規模 .svelte 手術)。

---

## Phase 7 — Reviewer 分割(規模 L・1 セッション、P2+P6 完了が前提。意図的に最終)

**ゴール**: `src/routes/review/[deckId]/+page.svelte` 1176 行 → 約 350 行。タイミング依存の speech/hide ロジックをテスト済み純モジュール化し、speech 設定 UI の二重実装をアプリから根絶する。

- [ ] **7.1 ロジック抽出(本丸。各モジュールにテスト必須)** — `src/lib/reviewer/` 配下(render.ts + render.test.ts は既存):
  - `types.ts` — Counts / StudyCard / NextCard / RenderedCard DTO(30-54 行から移動)
  - `answer-html.ts` — `stripQuestionFromAnswer(html: string): string`(`hr#answer` ロジック、298-310 行)。テスト: hr#answer あり / なし / 親なし
  - `frame-text.ts` — `extractCardText(doc: Document): string | null` と `whenFrameReady(frame, run)`(3 箇所に重複した readyState/host チェック [398-406, 469-484, copy 経路] を統合)。テスト: jsdom
  - `hidden-overlay.ts` — `setHiddenOverlay(doc, hidden, hintText)`(432-459 行)。テスト: ラベル挿入/除去 / class トグル / 冪等性
  - `speech-cycle.svelte.ts` — `repeatTimer` / `lastSpokenFrame` / `repeatCount` の絡み(90-112, 408-417, 520-532 行)を `SpeechCycle` クラスに(constructor が `speak: (frame) => void` を受ける)。テスト: `vi.useFakeTimers()` で max-repeat 停止 / interval 遵守 / stop() のキャンセル / 新サイクルでのカウントリセット
- [ ] **7.2 コンポーネント抽出** — `src/lib/components/reviewer/` 配下:
  `AudioSettingsPopover.svelte`(**SpeechControls layout="popover" を実装し、review の 700-924 行を削除** — speech UI 二重実装がここで根絶)/ `ReviewActionButton.svelte`(Nani/Speak/Hide/ShowQuestion の同一 class ボタン 4 箇所 [1037-1088 行]。props: `{ icon, label, hotkey, onclick, title?, size?: "normal" | "wide" }`)/ `RatingBar.svelte`(4 トーンボタン)/ `CardStage.svelte`(3D flip + CardFrame ×2、iframe は `$bindable`)/ `ReviewHeader.svelte` / `DoneScreen.svelte` / `CopyToast.svelte`。
- [ ] **7.3 +page.svelte の書き換え** — セッション state / sync-refresh effect / onKey(意図的に狭い text-field チェックは維持)/ composition の約 350 行へ。**`onDestroy` の「speech.repeat 設定を触らない」セマンティクスと既存コメント(116-130 行)を必ず温存**。

**手動スモーク(必須。観察結果を commit/PR に記録)**: repeat サイクル(max 回数で停止・interval)/ auto-reveal / hide モードのクリックトグル / Nani の copy + フォーカス挙動 / Audio popover の全コントロールが settings と双方向同期。

**ゲート**: 共通ゲート + `pnpm build` 必須。

---

## 完了の定義(全フェーズ後)

- [ ] 進捗トラッカーの 7 フェーズ全チェック
- [ ] 巨大ファイル解消の実測: `settings/+page.svelte` ≤ 200 行 / `review/[deckId]/+page.svelte` ≤ 400 行 / `+page.svelte` ≤ 300 行 / `decks.rs`(分割後の各ファイル) ≤ 250 行
- [ ] テストゼロだったモジュール(reviewer.rs / study.rs / sync.rs / sync.svelte.ts / draggable.ts / stats transform / chart 数式)にテストが存在
- [ ] 検証ゲート 5 コマンド全 green
- [ ] CLAUDE.md の「Architecture you can't infer from a single file」に新モジュール(render.rs / with_collection / SpeechControls / chart-utils)の記述を追記し、本スキルの監査時点情報(行番号など)が古くなったことを明記
- [ ] リリースする場合: version bump は patch(/release スキル参照)
