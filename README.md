# memorize

Anki 互換の Rust 製デスクトップアプリ。最終形は英単語学習特化。

- **デスクトップシェル**: Tauri 2
- **フロントエンド**: SvelteKit + Svelte 5 (runes) + Tailwind v4
- **バックエンド**: Anki の [`rslib`](https://github.com/ankitects/anki/tree/main/rslib) を git submodule で取り込んで再利用（Sync・Import/Export 互換性を担保）

> **互換性方針**: AnkiWeb の認証・同期プロトコル、`.apkg` / `.colpkg` の Import/Export
> はすべて `rslib` の公開 API 経由。独自実装はしない。

> **Disclaimer**: memorize is an unofficial, third-party client. It is not
> affiliated with, endorsed by, or sponsored by Ankitects Pty Ltd. "Anki" and
> "AnkiWeb" are trademarks of Ankitects Pty Ltd; their use here is solely for
> nominative description of compatibility, not as a product designation.

## ステータス

| Phase | 内容 | 状態 |
|---|---|---|
| 0 | bootstrap (Tauri + SvelteKit + rslib リンク) | ✅ 完了 |
| 1 | 読み取り専用 IPC (open / list_decks / list_cards / get_card_render) | ✅ 完了 |
| 2 | UI ブラッシュアップ (4 画面、デザイントークン、アニメ) | ✅ 完了 |
| 3 | AnkiWeb Sync 統合 (login / normal_sync / full_upload-download / 自動 backup) | ✅ MVP 完了 |
| 4 | `.apkg` Import / Export, `.colpkg` Restore | ✅ MVP 完了 |
| 4.5 | Decks 画面の Stats パネル全実装 (Today / Future Due / Calendar / Reviews / Card Counts / Intervals / Card Ease / Retention / Hourly / Answer Buttons / Added) | ✅ 完了 |
| 5 | 英単語特化機能 | ⏳ 着手中 (Nani lookup ✅) |

## インストール

### Homebrew (Apple Silicon)

```sh
brew tap iQeda/tap
brew install --cask memorize
```

cask は [iQeda/homebrew-tap](https://github.com/iQeda/homebrew-tap) で管理。
Apple Silicon (arm64) のみ。Intel Mac は非対応。

### 直接 DMG をダウンロード

[Releases](https://github.com/iQeda/memorize/releases) から
`memorize_<version>_aarch64.dmg` をダウンロード → マウント →
`memorize.app` を `/Applications` にドラッグ。未署名なので初回起動は
**右クリック → 開く → 開く** で Gatekeeper をバイパスしてください
(brew cask 経由なら postflight で自動的に xattr が剥がされるので不要)。

## 開発セットアップ

前提: `rustup` (Rust 1.92.0)、`pnpm`、`protoc`、`node` >= 22。

```sh
# submodule (anki + ネストした FTL 翻訳 repo) を取得
git submodule update --init --recursive

# vendor/anki への local patch を適用 (rslib の progress モジュールを公開)
./scripts/apply-vendor-patches.sh

# 依存インストール
pnpm install
```

`protoc` が PATH に無い場合 (Homebrew にあるが PATH が通っていない等):

```sh
export PROTOC=/opt/homebrew/bin/protoc
```

`rustup` は `vendor/anki/rust-toolchain.toml` を読んで Rust 1.92.0 を自動取得する。

## 開発

```sh
pnpm tauri dev
```

ブラウザだけで UI を確認したい場合 (Tauri command は呼べない):

```sh
pnpm dev
```

型チェック:

```sh
pnpm exec svelte-check
cargo check --manifest-path src-tauri/Cargo.toml
```

## 本番ビルド (DMG)

```sh
pnpm tauri build
```

- 出力: `src-tauri/target/release/bundle/dmg/memorize_<version>_<arch>.dmg`
  および `src-tauri/target/release/bundle/macos/memorize.app`
- `tauri.conf.json` の `bundle.active` は `true`、`bundle.targets` は `["app", "dmg"]`
- 初回ビルドは Anki rslib の release 最適化込みで 5–10 分程度かかる
- 署名・公証 (codesign / notarize) は未設定。配布時は別途設定が必要

## アーキテクチャ

```
memorize/
├── src/                          # SvelteKit (Svelte 5 runes、SPA モード)
│   ├── app.css                   # Tailwind v4 @theme トークン
│   ├── lib/
│   │   ├── ipc.ts                # invoke() ラッパ
│   │   ├── actions/draggable.ts  # window.startDragging() を mousedown で呼ぶ
│   │   ├── stores/               # collection / theme / sync (Svelte 5 class state)
│   │   ├── components/           # Sidebar / TitleBar / CardFrame / PageTransition
│   │   └── reviewer/             # Anki Reviewer JS の最小ポート (script 再評価)
│   └── routes/                   # /, /browse, /review/[deckId], /settings
├── src-tauri/                    # Tauri 2 + Rust
│   ├── Cargo.toml                # [workspace] を書かない (重要、後述)
│   ├── examples/smoke.rs         # スタンドアロン smoke test
│   └── src/
│       ├── state.rs              # Mutex<Option<Collection>> + http client + col_path
│       ├── error.rs              # serde-serializable AppError
│       └── commands/             # collection / decks / cards / reviewer / sync
└── vendor/anki/                  # git submodule → ankitects/anki @ 35b727a
```

### vendor/anki へのローカルパッチ

`patches/` 配下の差分を submodule の working tree に当てている (idempotent な
`./scripts/apply-vendor-patches.sh` で適用):

| パッチ | 内容 | 必要な理由 |
|---|---|---|
| `0001-expose-progress-module.patch` | `mod progress;` → `pub mod progress;` | Tauri command で `Arc<Mutex<ProgressState>>` を構築 |
| `0002-tolerate-missing-original-size-header.patch` | `io_monitor.rs` で zstd ヘッダ無しの応答を許容 | AnkiWeb `/upload` が plain `OK` を返す挙動への対処 |
| `0003-expose-graph-data-for-search.patch` | `Collection::graph_data_for_search` を pub に | Decks 画面の Stats パネルで内部 graph data API を利用 |

submodule 更新後は必ず `./scripts/apply-vendor-patches.sh` を再実行する。
将来は upstream に PR するか fork に切り替える。

### `[workspace]` を書かない理由

`vendor/anki/rslib/Cargo.toml` は `workspace = ".."` 指定で、`vendor/anki/Cargo.toml`
を workspace ルートとして解決される。memorize の `src-tauri/Cargo.toml` に
`[workspace]` を追加すると、`anki` crate が複数 workspace に属して
`error: package collides` になる。

### IPC 設計

Tauri commands は serde の独自 DTO を返す。`anki::pb::*` の prost 生成型を
直接フロントに返さない。これは **rslib の API 変更からフロントを守る吸収層** で、
Phase 0 から維持。

### CardFrame の iframe 隔離

カードの HTML/CSS は Anki ノートテンプレ由来でアプリ全体を汚染しうる。
`src/lib/components/CardFrame.svelte` は `<iframe srcdoc>` で隔離レンダリング、
ベース CSS と user CSS を内部で結合してから注入する。

加えて iframe 内部で短い JS を走らせて、CJK Unicode 範囲のテキストランを
`<span lang="ja">…</span>` で wrap している。これにより CSS `[lang="ja"]`
セレクタで日本語だけ regular weight に固定できる (英単語は `font-weight: 700`)。

> ⚠️ Svelte tokenizer 上の制約: srcdoc を組み立てる JS テンプレートリテラルに
> 文字列 `<script>` / `</script>` を直接書くと、Svelte の HTML パーサが
> 外側の `<script lang="ts">` ブロックを早期クローズしてしまう (HMR では
> 通るが `pnpm build` の SSR で fail する)。タグは `"<" + "script>"` 形式で
> 連結すること。

### Nani lookup (Phase 5)

解答画面の "Nani" ボタン (デフォルトショートカット `n`) で:

1. front (英単語) を抽出してオフスクリーン `<input>` に詰めて全選択
2. `osascript` 経由で Cmd+J を OS に送出
3. Nani の global hotkey (Cmd+J 想定) が macOS Accessibility API で
   memorize の focused input から選択テキストを読み取る

ショートカットキーは Settings > Keyboard shortcuts で再バインド可能。
(`shortcuts.svelte.ts` の `Action = Rating | "nani"`)

### Deck stats のカウント方式

`commands/decks.rs::deck_stats` は `c.queue` 値による排他カウントを直接 SQL で行う。
Anki search 構文 (`is:learn`, `is:suspended` 等) は `c.type` ベースと `c.queue`
ベースが混在しており排他にならないため使わない (suspended-while-learning な
カードが両方にカウントされる)。

## AnkiWeb Sync の安全な使い方

> ⚠️ **データ消失のリスクあり**。本番 AnkiWeb アカウントで試す前に必ずバックアップを取ってください。

`Collection::full_upload()` は **ローカルの内容で AnkiWeb サーバー側を上書きします**。
バグで壊れたローカルコレクションを upload すれば、サーバー側も壊れます。
AnkiWeb のサーバー側バックアップは Anki 公式に問い合わせないと復元できません。

### third-party クライアントである旨の注意

memorize は Ankitects の公式クライアントではありません。Anki 公式ドキュメント
[Third-party tools and apps](https://docs.ankiweb.net/third-party.html) は
third-party クライアントについて以下の通り述べています:

> No testing is done against them, and they tend to take time to catch up when
> the sync protocol changes, so they are not recommended.

これは memorize にも当てはまります:

- **Sync protocol 変更リスク**: AnkiWeb 側のプロトコルが変わると memorize は
  突然壊れる可能性があります。修正は `vendor/anki` の追従と patch 更新が必要
- **公式サポート対象外**: memorize 経由でデータ消失が起きても Ankitects の
  サポートは受けられません
- **将来的な選択肢**: 長期的には [Anki 公式の self-hosted sync server](https://docs.ankiweb.net/sync-server.html)
  を立てて、memorize / 公式 Anki どちらもそこに繋ぐ方が安全です

### 推奨手順

1. **事前ダウンロード**: https://ankiweb.net/decks/ から `.colpkg` をダウンロードして手元に保存
2. **テストアカウント**: 可能なら本番と別の AnkiWeb アカウントを作る
3. **テストコレクション**: 本番 `.anki2` を直接開かず、コピーで動作確認
   ```sh
   cp "$HOME/Library/Application Support/Anki2/User 1/collection.anki2" /tmp/memorize-test/
   ```
4. **フル同期は最終手段**: 通常 sync (`normal_sync`) が `FullSyncRequired` を返した場合、
   どちらが正なのか確認してから upload か download を選ぶ

### 認証情報の保管

ログイン後、host-key は **macOS Keychain** に `dev.iqeda.memorize` /
`ankiweb-credentials` として JSON 形式で保存される (`keyring` crate)。
パスワード本体は保存しない (host-key はパスワードと等価ではないが、
sync API には十分な権限を持つので慎重に)。

ログアウト or `keychain` の該当エントリを削除すれば破棄できる:

```sh
security delete-generic-password -s dev.iqeda.memorize -a ankiweb-credentials
```

## Import / Export / Backup

| 操作 | 拡張子 | 用途 | 操作場所 |
|---|---|---|---|
| Backup (export) | `.colpkg` | コレクション全体のスナップショット | Settings → Backup → 「今すぐバックアップ」 |
| 自動 Backup | `.colpkg` | 同期実行直前 | Settings → Backup → トグル ON (default) |
| Restore (import) | `.colpkg` | コレクション全体を上書き復元 | Settings → Backup → 「復元…」 |
| Import | `.apkg` | デッキを既存コレクションに追加マージ | Settings → Import / Export → 「ファイルを選択…」 |
| Export | `.apkg` | デッキを共有・移行用に書き出し | Settings → Import / Export → デッキ選択 → 「Export…」 |

`.colpkg` の Restore と全フル sync は **OS ネイティブ confirm dialog** が出る。

## 既知の限界

- Phase 5 (英単語特化 note type / 発音 / 語源 / TTS など) は未着手 — ユーザー要件待ち
- 同期前後のデッキ一覧 auto-refresh は手動 (Import 後のみ自動 refresh)
- vendor/anki に local patch 1 件 (`patches/0001-expose-progress-module.patch`) を当てているため、submodule update 後は `./scripts/apply-vendor-patches.sh` を実行する必要あり

## ライセンス

memorize は **GNU Affero General Public License v3.0 or later (AGPL-3.0-or-later)**
でライセンスされています。全文は [`LICENSE`](./LICENSE)、サードパーティ著作権の
帰属は [`NOTICE`](./NOTICE) を参照してください。

This project incorporates `rslib` from
[ankitects/anki](https://github.com/ankitects/anki) (Copyright Ankitects Pty Ltd
and Alex Fraser), which is also licensed under AGPL-3.0-or-later. ローカル改変
は `patches/` 以下に保管し、各 patch の冒頭で AGPL §5(a) の改変表記を行って
います。

### AGPL §6 (バイナリ配布における対応するソース)

GitHub Releases で配布する DMG にはアプリ本体のみを同梱しています。対応する
ソースコードは本リポジトリ (https://github.com/iQeda/memorize) の同一バージョン
タグから取得できます。Release notes に直接リンクを記載しています。
