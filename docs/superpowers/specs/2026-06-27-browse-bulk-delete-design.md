# Browse 一括削除（チェックボックス選択 + bulk delete）設計

## 背景・目的

Browse 画面（`src/routes/browse/+page.svelte`）は現状、単語（カード）を一覧表示し、行クリックで
1 件ずつ編集／削除できるのみ。複数の単語をまとめて削除する手段がない。本変更でチェックボックス選択 +
一括削除を追加する。

「単語の削除」= ノート（`note_id`）の削除であり、そのノートに属する全カードが Anki コアの
`remove_notes` によって削除される。

## スコープ

- **変更**: `src/routes/browse/+page.svelte`（UI・選択状態・削除フロー）
- **追加**: `src/lib/browse/selection.ts`（純粋ヘルパー）+ `src/lib/browse/selection.test.ts`
- **追加**: `src/lib/i18n/messages.ts` に i18n キー（en/ja）
- **変更なし（既存を再利用）**:
  - バックエンド `delete_notes` コマンド（`src-tauri/src/commands/notes.rs`、`note_ids: Vec<i64>` →
    `col.remove_notes`）
  - ストア `notes.deleteNotes(noteIds): Promise<number>`（`src/lib/stores/notes.svelte.ts`）
  - 確認ダイアログ `@tauri-apps/plugin-dialog` の `confirm`（NoteEditor / Sidebar と同パターン）

## 選択モデル

- Browse の各行は **カード**。削除対象は **ノート**。
- 選択状態は `let selected = $state(new Set<number>())` に **カード id**（`c.id`、行ごとに一意）を保持。
- 削除時に選択カードの `note_id` を抽出・重複排除して `notes.deleteNotes(noteIds)` に渡す。
  memorize の語彙ノートは 1 カード = 1 ノートだが、多カードノートでも重複排除により安全に動作する。
- リスト再読込（デッキ変更・検索変更・削除完了後）のたびに選択をクリアする。再読込後の
  カード集合に存在しない id が選択に残ると、選択件数表示やヘッダーチェック状態がずれるため。

## UI（選択バーを上部に表示）

```
┌────────────────────────────────────────┐
│ 120枚                      [+ 単語を追加] │  ← 既存ヘッダー
├────────────────────────────────────────┤
│ ☑ 3件選択中  [すべて選択] [解除] [🗑 削除] │  ← 1件以上選択時のみ出現
├────────────────────────────────────────┤
│ ☑ │ Word    │ Note    │ #0              │  ← 先頭にチェックボックス列を追加
│ ☑ │ apple   │ 15…     │ #0              │
│ ☐ │ banana  │ 15…     │ #0              │
└────────────────────────────────────────┘
```

- テーブルに先頭チェックボックス列を追加。`<thead>` のチェックボックスは **全選択トグル**。
  - 全行選択時はチェック ON、一部のみ選択時は `indeterminate`（バインドで設定）、0 件選択時は OFF。
- 選択バーはヘッダー（`<header>`）とテーブルの間に配置し、`selected.size > 0` のときだけ表示。
  - 内容: `{count}件選択中` ラベル、`すべて選択`（表示中の全行を選択）、`解除`（全選択クリア）、
    `🗑 削除` ボタン。
  - `すべて選択` は全行選択済み（`allSelected`）のときは disabled。部分選択から「全件に広げる」
    導線として、ヘッダーチェックボックスより発見しやすい位置に置く。
- チェックボックスの `click` / `change` は `stopPropagation` し、行クリック（`openEdit` で編集モーダルを
  開く）を誤発火させない。

### キーボードショートカット

- `⌘A`（macOS）: 表示中の全カードをチェック状態にする（`selected = new Set(全カード id)`）。
  - **検索入力欄にフォーカスがあるときは横取りしない**（テキスト全選択の既定動作を優先）。
    `event.target` が `<input>` / `<textarea>` のときはスルー。それ以外で Browse 画面にいる場合のみ
    `preventDefault` して全選択を実行。
  - 編集モーダル（`NoteEditor`）が開いているときは無効。
- `Esc`: 編集モーダルが閉じている状態で選択があれば選択をクリア（モーダルを開いていれば従来通り
  モーダルを閉じる方を優先）。

## 削除フロー

1. `🗑 削除` クリック → 選択カードから `selectedNoteIds()` で note_id 配列を算出。
2. `@tauri-apps/plugin-dialog` の `confirm` を `kind: "warning"` で表示。
   - title: `browse.deleteConfirmTitle`、body: `browse.deleteConfirmBody`（件数埋め込み）、
     OK/Cancel: 既存 `note.deleteOk` / `note.deleteCancel` を再利用。
3. 承認時のみ `notes.deleteNotes(noteIds)` を呼ぶ。
4. 完了後に `load(filterDeckId, query)` で再読込（= 選択クリア）し、`collection.refreshDecks()` で
   サイドバーのデッキ別カウントを更新。

## i18n（`messages.ts` に en/ja 追加）

| key | en | ja |
| --- | --- | --- |
| `browse.selectedCount` | `{count} selected` | `{count}件選択中` |
| `browse.clearSelection` | `Clear` | `解除` |
| `browse.deleteSelected` | `Delete` | `削除` |
| `browse.selectAll` | `Select all` | `すべて選択`（ヘッダーチェックボックスの aria-label + 選択バーのボタン文言を兼用） |
| `browse.deleteConfirmTitle` | `Delete words` | `単語を削除` |
| `browse.deleteConfirmBody` | `Delete {count} words? This cannot be undone.` | `{count}件の単語を削除します。この操作は取り消せません。` |

ダイアログのボタンラベルは既存 `note.deleteOk` / `note.deleteCancel` を再利用し、キー増殖を避ける。

## テスト（CLAUDE.md のユニットテスト必須ルール準拠）

純粋ロジックを `src/lib/browse/selection.ts` に抽出し Vitest でテストする。

```ts
type Row = { id: number; note_id: number };

// 選択カード id 集合 → 重複排除した note_id 配列（出現順）
export function selectedNoteIds(cards: Row[], selectedIds: Set<number>): number[];

// 表示中の全カードが選択済みか（ヘッダーチェック ON 判定）
export function allSelected(cards: Row[], selectedIds: Set<number>): boolean;

// 1 件以上選択されているか（indeterminate 判定に使用：some && !all）
export function someSelected(cards: Row[], selectedIds: Set<number>): boolean;

// 表示中の全カード id 集合（「すべて選択」/ ⌘A 用）
export function allCardIds(cards: Row[]): Set<number>;
```

`selection.test.ts` のケース:
- 空選択 → `selectedNoteIds` は空、`allSelected` は false、`someSelected` は false。
- 部分選択 → 該当 note_id のみ、`allSelected` false、`someSelected` true。
- 全選択 → 全 note_id、`allSelected` true。
- 多カード 1 ノート（同 note_id の 2 カードを選択）→ note_id が重複排除され 1 件。
- 空リスト（`cards` が空）→ `allSelected` は false（空集合を「全選択」と誤判定しない）。
- `allCardIds` → 表示中の全カード id を含む集合（「すべて選択」/ ⌘A の結果と一致）。

UI 結線（チェックボックスのトグル・選択バー / すべて選択ボタンの表示・⌘A / Esc のハンドリング）は
Svelte コンポーネント側で薄く実装し、ロジックは上記純粋関数に委譲する。

## 非対象（YAGNI）

- 500 件上限を超える「全件選択」やページング：現状 `list_cards` の `limit: 500` のまま、選択は
  表示中の行に限定する。
- 一括でのデッキ移動・タグ付け等の他バルク操作：本変更は削除のみ。
- Undo/復元 UI：Anki コアの操作履歴に委ねる（独自実装しない）。
