# memorize

Anki 互換の Rust 製デスクトップアプリ。最終形は英単語学習特化。

- デスクトップシェル: Tauri 2
- フロントエンド: SvelteKit + Svelte 5 + Tailwind v4
- バックエンド: Anki の `rslib` を git submodule で取り込んで再利用（Sync・Import/Export 互換性を担保）

## セットアップ

前提: `rustup`、`pnpm`、`protoc`、`node` >= 22

```sh
# submodule（anki + ネストした FTL 翻訳 repo）を取得
git submodule update --init --recursive

# 依存インストール
pnpm install

# Rust ツールチェイン (rust-toolchain.toml が 1.92.0 を pin している)
rustup show
```

`protoc` が PATH に無い場合は明示指定:

```sh
export PROTOC=/opt/homebrew/bin/protoc
```

## 開発

```sh
pnpm tauri dev
```

ブラウザだけで UI を見たい場合:

```sh
pnpm dev
```

## アーキテクチャ

```
memorize/
├── src/                  # SvelteKit (Svelte 5 runes)
│   ├── lib/
│   │   ├── ipc.ts        # invoke() ラッパ
│   │   ├── stores/       # collection / theme
│   │   ├── components/   # Sidebar / TitleBar
│   │   └── reviewer/     # Anki Reviewer JS の最小ポート
│   └── routes/           # /, /browse, /review/[deckId], /settings
├── src-tauri/            # Tauri 2 + Rust
│   └── src/
│       ├── state.rs      # Mutex<Option<Collection>>
│       ├── error.rs
│       └── commands/     # open/close/list_decks/list_cards/get_card_render
└── vendor/anki/          # git submodule → ankitects/anki
```

`src-tauri/Cargo.toml` には `[workspace]` セクションを書かない。`anki` crate は
`vendor/anki/Cargo.toml` を workspace ルートとして解決される。
