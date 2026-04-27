# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this project is

`memorize` is a Tauri 2 + SvelteKit + Svelte 5 (runes) desktop app that re-uses Anki's Rust core (`rslib`) as a library to stay binary-compatible with AnkiWeb sync, `.apkg`, and `.colpkg`. Long-term goal is English-vocabulary specialization. `vendor/anki` is a git submodule; we never re-implement Sync or Import/Export — every Tauri command is a thin DTO wrapper over `anki::*` calls.

## Common commands

```sh
# First-time / after a submodule update — patches MUST be re-applied.
git submodule update --init --recursive
./scripts/apply-vendor-patches.sh

# Dev (full Tauri shell with HMR).
pnpm tauri dev

# Frontend-only dev (Tauri commands won't work, but useful for layout work).
pnpm dev

# Production DMG.
pnpm tauri build

# Type / lint checks.
pnpm exec svelte-check
cargo check --manifest-path src-tauri/Cargo.toml

# Frontend production bundle alone (good for catching SSR-only Svelte errors that
# don't surface during HMR — see "Svelte tokenizer pitfalls" below).
pnpm build
```

`protoc` (`brew install protobuf`) is required by `rslib`'s build script. Set `PROTOC=/opt/homebrew/bin/protoc` if it isn't on PATH. Rust toolchain is pinned by `vendor/anki/rust-toolchain.toml`.

## Architecture you can't infer from a single file

### `src-tauri/Cargo.toml` MUST NOT contain `[workspace]`

`vendor/anki/rslib/Cargo.toml` declares `workspace = ".."`, so Cargo resolves `vendor/anki/Cargo.toml` as the workspace root. Adding `[workspace]` to `src-tauri/Cargo.toml` makes the `anki` crate belong to two workspaces and Cargo errors out with "package collides". Just dependency-link `anki = { path = "../vendor/anki/rslib", … }` and don't declare a workspace.

### `vendor/anki` carries local patches that must be applied

Three patches in `patches/` make otherwise-private rslib internals public so Tauri commands can call them:

- `0001-expose-progress-module.patch` — `pub mod progress;`
- `0002-tolerate-missing-original-size-header.patch` — fixes AnkiWeb `/upload` which returns plain `OK` without zstd headers
- `0003-expose-graph-data-for-search.patch` — `pub fn graph_data_for_search` for the deck stats panels

After every `git submodule update`, run `./scripts/apply-vendor-patches.sh` (idempotent — skips already-applied patches). Do not modify `vendor/anki` files directly; add a new patch.

### IPC DTO layer (intentional, non-obvious)

Tauri commands return *hand-written serde structs*, never the `anki_proto::*` prost types. This is the absorption layer that keeps the frontend insulated from rslib API changes. When adding a new command, define a `#[derive(Serialize)]` DTO in `src-tauri/src/commands/<area>.rs` and translate from rslib types — don't `#[derive(Serialize)]` on rslib-generated structs.

### Collection state (`src-tauri/src/state.rs`)

```rust
pub struct AppState {
    pub col: tokio::sync::Mutex<Option<anki::collection::Collection>>,
    pub http: reqwest::Client,        // Policy::none() — rslib does its own redirect handling
    pub progress: Arc<Mutex<ProgressState>>,
    pub col_path: Mutex<Option<PathBuf>>,
}
```

- `Collection` is `Send + !Sync` and held across `await`, so `tokio::sync::Mutex` is mandatory; `std::sync::Mutex` deadlocks under async.
- `reqwest::Client` is built with `redirect::Policy::none()` because rslib's sync code interprets HTTP 303 itself to switch shards (`sync13.ankiweb.net` etc.).

### CardFrame: HTML/CSS isolation + Japanese-weight workaround

`src/lib/components/CardFrame.svelte` renders Anki note templates inside an `<iframe srcdoc>` — never inline. The srcdoc bundles base CSS + user CSS + injected runtime script.

The injected script wraps every CJK run (`U+3000–U+9FFF` etc.) in `<span lang="ja">` so CSS `[lang="ja"]` can pin Japanese to regular weight while English stays bold. There is **no other reliable way** — `@font-face unicode-range` with weight pinning didn't survive macOS WebKit's font fallback.

**Svelte tokenizer pitfall**: the JS template literal that builds the srcdoc cannot contain the literal substring `<script>` or `</script>` — Svelte's HTML tokenizer scans the .svelte file naively and will close the outer `<script lang="ts">` block prematurely. Use `"<" + "script>"` concatenation tricks. This usually slips past dev (HMR's preprocessor is lenient) and only fails at `pnpm build` (SSR bundle), so always run `pnpm build` before declaring frontend work done.

### Deck stats counts: queue-based, not Anki-search-based

`commands/decks.rs::deck_stats` runs raw SQL against `col.storage.db()` and classifies cards by `c.queue`:
- `-1` → suspended, `-2|-3` → buried, `0` → new, `1|3` → learning, `2` → review.

Do **not** use `is:learn`/`is:review` etc. searches for these counts — those match by `c.type` (the underlying type, persists across suspension), not `c.queue`, so the categories overlap. A suspended-while-learning card was being counted in *both* "learn" and "suspended" before the rewrite.

### Chart components: keyed `{#each}` with the iterated value is unsafe

`src/lib/components/charts/*.svelte` previously used patterns like `{#each tickValues() as v (v)}` — when the data was sparse, `tickValues()` returned duplicates and Svelte crashed with "duplicate keys", silently breaking the entire `{#if graph}` block above the chart. All chart `each` loops use index-based keys (`as v, i (i)`). Keep that convention when adding charts.

### Frontend stores: Svelte 5 runes via `.svelte.ts`

Stores are class instances using `$state` and live in `src/lib/stores/*.svelte.ts`. The `.svelte.ts` extension (not just `.ts`) is required for runes to compile — same for `src/lib/i18n/index.svelte.ts`. Importing from `$lib/i18n/index.svelte` (no `.ts`) is the correct usage; SvelteKit resolves it.

### Routes layout

- `src/routes/+page.svelte` — Decks browser + the stats panel grid (Today / Future Due / Calendar / Reviews / Card Counts / Intervals / Card Ease / Retention / Hourly / Answer Buttons / Added). All panels are 260px fixed-height in a 2-column grid via `auto-rows-[260px]`.
- `src/routes/review/[deckId]/+page.svelte` — the reviewer. Cards render at full container width (no `max-w` clamp). Includes the Nani lookup feature: extracts the front word, populates an off-screen `<input>`, focuses+selects it, then `invoke('nani_lookup')` which `pbcopy`s the word and synthesizes Cmd+J via `osascript`.
- `src/routes/browse` — card browser.
- `src/routes/settings` — profile, theme, locale, sync, backup, import/export, shortcut rebinding.

### Shortcuts store (`stores/shortcuts.svelte.ts`)

Uses `Action = Rating | "nani"`. Adding a new bindable shortcut: extend the `Action` union, add a default key, expose `isXxx(key)` methods. Settings page automatically renders any entry of the shared list.

## Conventions

- Reply text is Japanese (per global `CLAUDE.md`); code identifiers stay English.
- Don't return prost types from Tauri commands — define a DTO.
- Don't use `is:state` Anki searches for analytical counts — use `c.queue` directly.
- Don't add `[workspace]` to `src-tauri/Cargo.toml`.
- Don't write literal `<script>` / `</script>` inside JS template literals in `.svelte` files.
- Use index-based keys in chart `{#each}` loops.
- After modifying files in `vendor/anki`: capture as a patch in `patches/`, not a direct submodule commit (we don't fork upstream).

## Persistent memory location

`/Users/iqeda/.claude/projects/-Users-iqeda-ghq-github-com-iQeda-memorize/memory/MEMORY.md` is the auto-memory index loaded into context across sessions. Update it when project facts change (phase status, architectural decisions, deadlines).
