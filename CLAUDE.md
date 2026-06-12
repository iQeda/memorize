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

# Unit tests.
pnpm test:run     # Vitest, jsdom env, $app/environment is stubbed (test/mocks/app-environment.ts).
pnpm test:rust    # cargo test --manifest-path src-tauri/Cargo.toml (needs PROTOC).

# Frontend production bundle alone (good for catching SSR-only Svelte errors that
# don't surface during HMR — see "Svelte tokenizer pitfalls" below).
pnpm build
```

`protoc` (`brew install protobuf`) is required by `rslib`'s build script. Set `PROTOC=/opt/homebrew/bin/protoc` if it isn't on PATH. Rust toolchain is pinned by `vendor/anki/rust-toolchain.toml`.

## Architecture you can't infer from a single file

### `src-tauri/Cargo.toml` MUST NOT contain `[workspace]`

`vendor/anki/rslib/Cargo.toml` declares `workspace = ".."`, so Cargo resolves `vendor/anki/Cargo.toml` as the workspace root. Adding `[workspace]` to `src-tauri/Cargo.toml` makes the `anki` crate belong to two workspaces and Cargo errors out with "package collides". Just dependency-link `anki = { path = "../vendor/anki/rslib", … }` and don't declare a workspace.

### `vendor/anki` carries local patches that must be applied

The patches in `patches/` make otherwise-private rslib internals public so Tauri commands can call them, plus one upstream-bug fix:

- `0001-expose-progress-module.patch` — `pub mod progress;`
- `0002-tolerate-missing-original-size-header.patch` — fixes AnkiWeb `/upload` which returns plain `OK` without zstd headers
- `0003-expose-graph-data-for-search.patch` — `pub fn graph_data_for_search` for the deck stats panels
- `0004-paren-wrap-did-search-or-clause.patch` — wraps `DeckIdsWithoutChildren` SQL emission in an outer paren so `did:X "foo"` AND-intersects with the field search instead of OR-leaking every card in the deck (regression in upstream commit `159681d9f`)

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

### Shared modules introduced by the 2026-06-12 refactor (Phases 1–7)

- `src-tauri/src/render.rs` — `rendered_nodes_to_html` is the single RenderedNode→HTML conversion; reviewer.rs / study.rs both call it.
- `AppState::with_collection` (`src-tauri/src/state.rs`) — the canonical "lock col, error if closed" helper. study.rs / backup.rs / sync.rs keep manual lock patterns on purpose (see comments there) — don't "clean them up".
- `src-tauri/src/commands/decks/` — split into `mod.rs` (CRUD) / `stats.rs` (queue-based deck_stats) / `graphs.rs`. `lib.rs`'s `generate_handler!` relies on the glob re-exports in `decks/mod.rs`.
- `src/lib/storage-keys.ts` — every localStorage key. Values are frozen by a snapshot test; renaming one orphans user settings.
- `src/lib/stores/run-async.ts` — shared busy/busyReason/lastError lifecycle. `collection.open` and `sync.syncNow` intentionally don't use it.
- `src/lib/stores/speech.svelte.ts` exports `SPEECH_LIMITS` (min/max/default per speech param) — the only source for those ranges.
- `src/lib/components/SpeechControls.svelte` — the single speech-settings UI, `layout="rows"` (settings) / `layout="popover"` (reviewer Audio popover). Don't re-inline speech sliders anywhere.
- `src/lib/components/charts/chart-utils.ts` — chart geometry + `tickValues`. ButtonsChart overrides pad via `inner({ r: 6, b: 24 })`.
- `src/lib/stats/` (types + transform) and `src/lib/components/home/` — the home page is a thin composition root.
- `src/lib/reviewer/` — reviewer logic as tested pure modules: `session.svelte.ts` (ReviewSession), `speech-cycle.svelte.ts` (repeat timer), `answer-html.ts`, `frame-text.ts`, `hidden-overlay.ts`, `speak.ts`, `copy-nani.ts`, `totals.ts`. `src/lib/components/reviewer/` holds the UI pieces.
- `.claude/skills/refactor/SKILL.md` の行番号・実測値は監査時点 (commit `8cb9775`) のもので、Phase 1–7 完了後の現状とはズレている。

### Routes layout

- `src/routes/+page.svelte` — Decks browser + the stats panel grid (Today / Future Due / Calendar / Reviews / Card Counts / Intervals / Card Ease / Retention / Hourly / Answer Buttons / Added). All panels are 260px fixed-height in a 2-column grid via `auto-rows-[260px]`.
- `src/routes/review/[deckId]/+page.svelte` — the reviewer. Cards render at full container width (no `max-w` clamp). Includes the Nani lookup feature: selects the card iframe text, copies it to the clipboard, then `invoke('start_nani_lookup')` opens the `naniapp://translate?source=<word>` deep link via `/usr/bin/open` (no CGEvent / osascript / accessibility permission needed).
- `src/routes/browse` — card browser.
- `src/routes/settings` — profile, theme, locale, sync, backup, import/export, shortcut rebinding.

### Shortcuts store (`stores/shortcuts.svelte.ts`)

Uses `Action = Rating | "copy" | "speak" | "hide"` (rebindable in Settings → Keyboard shortcuts). Adding a new bindable shortcut: extend the `Action` union, add a default key in `defaults`, expose `isXxx(key)` methods, and add it to `ratingShortcuts` in `src/lib/components/settings/ShortcutsSection.svelte`.

Global (non-rebindable) keys handled directly in `+layout.svelte` and route components:
- `⌘,` → Settings, `⌘S` → Sync now, `⌘F` / `⌘K` → quick deck launcher. Ctrl modifier is rejected on these (macOS only; Ctrl alias was removed to prevent ambiguous bindings).
- `Esc` → leave reviewer / close launcher / close editor.
- `Space` / `↵` → flip card in reviewer; "back to decks" on the done screen.
- `Shift+L` → toggle the persistent "Hidden by default" setting from the reviewer.

Reviewer-only audio controls live in the Audio popover (gear button in the reviewer header) and are mirrored from `speech.svelte.ts`. Removed `R` hotkey for repeat — repeat is now toggled via the popover checkbox.

## Conventions

- Reply text is Japanese (per global `CLAUDE.md`); code identifiers stay English.
- Don't return prost types from Tauri commands — define a DTO.
- Don't use `is:state` Anki searches for analytical counts — use `c.queue` directly.
- Don't add `[workspace]` to `src-tauri/Cargo.toml`.
- Don't write literal `<script>` / `</script>` inside JS template literals in `.svelte` files.
- Use index-based keys in chart `{#each}` loops.
- After modifying files in `vendor/anki`: capture as a patch in `patches/`, not a direct submodule commit (we don't fork upstream).

## Testing & push discipline

### Unit tests are required for every non-trivial change

Any new logic, behavior change, or bug fix must ship with a corresponding unit test in the same commit.

- **Rust**: add a `#[cfg(test)] mod tests` block in the same file. Existing patterns:
  - Pure helpers — see `commands/nani.rs` (`url_encode`) and `error.rs` (serde output).
  - DB-backed integration — see `commands/cards.rs` (`tempfile::TempDir` + `CollectionBuilder` to spin up a real `Collection`, then exercise the `*_inner` helper). Run with `pnpm test:rust` (needs `PROTOC`).
- **Frontend**: add a `*.test.ts` next to the module. Vitest is configured with jsdom and a `$app/environment` stub (`test/mocks/app-environment.ts`). Run with `pnpm test:run`.
- For bug fixes, write the reproduction test first so it goes red, then make it green with the fix. The vendor/anki SQL precedence regression caught by `cards::tests::deck_filter_combined_with_field_search_correctly_intersects` is an example.
- If the production code is awkward to test, refactor it into a testable shape first (extract a pure helper, take `&mut Collection` instead of `State<AppState>`, etc.) and test the shape.

### Pre-push checklist (all three must pass before `git push`)

1. **Tests for the pushed diff exist.** Inspect `git log --stat origin/<branch>..HEAD` and confirm new/changed logic has a matching test. Pure refactors with no behavior change are exempt but should say so in the commit message.
2. **All tests green.** Run both:
   ```sh
   pnpm test:run
   PROTOC=/opt/homebrew/bin/protoc cargo test --manifest-path src-tauri/Cargo.toml
   ```
   `pnpm test:rust` is the same Rust invocation if `PROTOC` is already on PATH. Don't push with any failures or skips.
3. **Lint / type-check clean.** Run both:
   ```sh
   pnpm exec svelte-check --tsconfig ./tsconfig.json
   cargo check --manifest-path src-tauri/Cargo.toml
   ```
   Zero errors. Warnings are OK only if they're `#[cfg]`-gated dead-code false positives (e.g. `commands/dev.rs`).

When the user says "push", "release", "main に上げて" etc., **don't push immediately** — run the three checks first, report each result on a separate line, then proceed.

## Persistent memory location

`/Users/iqeda/.claude/projects/-Users-iqeda-ghq-github-com-iQeda-memorize/memory/MEMORY.md` is the auto-memory index loaded into context across sessions. Update it when project facts change (phase status, architectural decisions, deadlines).
