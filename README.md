# memorize

Anki 互換の Rust 製デスクトップアプリ。最終形は英単語学習特化。

- **デスクトップシェル**: Tauri 2
- **フロントエンド**: SvelteKit + Svelte 5 (runes) + Tailwind v4
- **バックエンド**: Anki の [`rslib`](https://github.com/ankitects/anki/tree/main/rslib) を git submodule で取り込んで再利用（Sync・Import/Export 互換性を担保）

> **互換性方針**: AnkiWeb の認証・同期プロトコル、`.apkg` / `.colpkg` の Import/Export
> はすべて `rslib` の公開 API 経由。独自実装はしない。

## ステータス

| Phase | 内容 | 状態 |
|---|---|---|
| 0 | bootstrap (Tauri + SvelteKit + rslib リンク) | ✅ 完了 |
| 1 | 読み取り専用 IPC (open / list_decks / list_cards / get_card_render) | ✅ 完了 |
| 2 | UI ブラッシュアップ (4 画面、デザイントークン、アニメ) | ✅ 完了 |
| 3 | AnkiWeb Sync 統合 (login / normal_sync / full_upload-download / 自動 backup) | ✅ MVP 完了 |
| 4 | `.apkg` Import / Export, `.colpkg` Restore | ✅ MVP 完了 |
| 5 | 英単語特化機能 (発音・語源・専用 note type 等) | ⏳ 未着手 |

## セットアップ

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

`rslib/src/lib.rs` の `mod progress;` を `pub mod progress;` に変える 1 行パッチを当てている (`patches/0001-expose-progress-module.patch`)。
これは Tauri command 側で `Arc<Mutex<ProgressState>>` を構築するため必要。
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

## AnkiWeb Sync の安全な使い方

> ⚠️ **データ消失のリスクあり**。本番 AnkiWeb アカウントで試す前に必ずバックアップを取ってください。

`Collection::full_upload()` は **ローカルの内容で AnkiWeb サーバー側を上書きします**。
バグで壊れたローカルコレクションを upload すれば、サーバー側も壊れます。
AnkiWeb のサーバー側バックアップは Anki 公式に問い合わせないと復元できません。

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

`vendor/anki` 配下は AGPL-3.0-or-later (Anki 本体)。
memorize 自体のライセンスは未定 (要検討)。
