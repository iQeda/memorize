---
name: release
description: memorize の本番リリース手順。「push and release」「リリースして」「公開して」「main に上げて」のような指示が出たとき、または手元の変更を本番（GitHub Release / DMG / Homebrew cask）に届ける必要があるときに使う。バージョン bump 判断、vendor/anki の dirty な submodule 状態の扱い、commit 規約、release workflow の確認手順をまとめる。memorize リポジトリ専用。
---

# memorize Release

memorize の本番リリースは **`main` への push で完全自動**。`.github/workflows/release.yml` が tauri-action でビルドし、GitHub Release を作って Latest 化、`iQeda/homebrew-tap` の cask も自動更新する。

## 前提運用ルール（毎回これを思い出す）

1. **バージョンは据え置きが慣習**
   - `package.json` / `src-tauri/tauri.conf.json` / `src-tauri/Cargo.toml` の `version` は通常 bump しない
   - tag 名は workflow 内で `v${VERSION}-${DATE}-${SHA}` として組み立てるので、同じ version でも DATE+SHA で一意になる
   - `git log -- package.json` で過去の bump 履歴を見ると、memorize 自体の package.json はほとんど触られていない（bump コミットが見えるのは vendor/anki 側）
   - 明示的なメジャー / マイナー区切りを切りたいときだけ手動で bump する。普段の機能追加コミットでは触らない

2. **`vendor/anki` の dirty は絶対に commit しない**
   - `apply-vendor-patches.sh` が `vendor/anki/rslib/...` に local patch を当てるので、submodule は常に dirty 状態（`m vendor/anki`）
   - これは設計通りの状態。CI でも同じ patch を当てるので、submodule の中身を新しい SHA に進める commit を作ってはいけない
   - `git add -A` / `git add .` は厳禁。常に **ファイル名を明示** して `git add <path...>`

3. **commit メッセージは日本語の `type(area): 内容` 形式**
   - 過去履歴例:
     - `feat(reviewer): カードを答えるたびにサイドバーのデッキバッジを再取得`
     - `feat(shortcuts): edit を E に戻し、Study Now / Back to decks に Space を追加`
     - `fix(note-editor): Front 自動フォーカスを bind:this 経由で確実に発火`
     - `ci: tauri-action を v0.5.25 に pin して release 作成失敗を解消`
     - `chore(deps): ...`
   - area は `reviewer` / `shortcuts` / `sync` / `settings` / `note-editor` / `home` / `layout` / `ci` / `chore` など、変更箇所のドメイン名
   - 末尾に `Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>` を必ず入れる

## 実行手順

### 1. 事前ビルドチェック（任意だが推奨）

リリース前に CI が落ちる原因を手元で潰しておく:

```sh
pnpm exec svelte-check                          # フロント型チェック
cargo check --manifest-path src-tauri/Cargo.toml # Rust コンパイルチェック
pnpm build                                       # SSR ビルド (CardFrame の <script> tokenizer 罠を catch)
```

CardFrame.svelte 周辺を触ったときは特に `pnpm build` を必ず通す（CLAUDE.md の "Svelte tokenizer pitfall" 参照）。

### 2. 状態確認

```sh
git status                # vendor/anki の m は無視。それ以外を確認
git diff --stat           # 規模感
git log --oneline -5      # commit メッセージの過去スタイル参照
```

### 3. 明示的に stage

```sh
# ファイル名を一つずつ列挙する。-A や . は使わない。
git add path/to/file1 path/to/file2 ...
git status                # vendor/anki が "Changes not staged" のまま残ることを確認
```

### 4. commit

HEREDOC で commit メッセージを渡す（フォーマット崩れを避けるため）:

```sh
git commit -m "$(cat <<'EOF'
feat(area): 1 行サマリ

(必要なら本文。何を変えたかではなく WHY を中心に。
 機能名やキーバインド、デフォルト値など読み手が知りたい事実を含める)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

### 5. push（ここから先は副作用が外に出る）

```sh
git push origin main
```

ユーザーから明示的にリリース指示を受けてから push する。push と同時に release workflow が起動するため、push 自体がリリースの実行アクションになる点に注意。

### 6. workflow の確認

```sh
sleep 5 && gh run list --workflow=release.yml --limit 1
```

`in_progress` であることを確認したら、ビルド完了 (過去履歴で 4-7 分) まで待つ。

### 7. 完了確認と URL 報告

5-7 分後、または `gh run watch <run-id>` で完了を待ってから:

```sh
gh run list --workflow=release.yml --limit 1                # success/failure
gh release list --limit 1                                   # 新 tag を確認
gh release view <tag> --json url -q .url                    # release URL
```

完了したらユーザーに以下を伝える:
- 新しい tag (例: `v0.4.19-20260507-1134-16aa9a5`)
- Release ページの URL
- homebrew tap 自動 bump が走ったか（成功なら `gh release view` の本文に何も注記なし。`HOMEBREW_TAP_TOKEN` 未設定時は warning でスキップする）

## 失敗時の対処

### workflow が失敗した場合

```sh
gh run view <run-id> --log-failed
```

過去に踏んだ罠:
- **tauri-action @v0 が v0.6 に勝手に上がって失敗**（2026-04-29 頃）→ `v0.5.25` に pin して解決済み（`commit 9906ced`）。再発したら release.yml の `tauri-apps/tauri-action@<ver>` を確認
- **codesign 検証失敗**（"Signature=adhoc" が出ない / `--verify --deep --strict` 失敗）→ workflow 内 "Verify bundle signature" ステップで `error::bundle integrity` が出る。bundle 構造を変えるような Tauri 設定変更を疑う
- **homebrew tap bump がスキップ**（warning 出力）→ `HOMEBREW_TAP_TOKEN` secret 未設定。リリース自体は成功しているので追加対応不要、必要なら secret 設定後に再 push

### push 後に「やっぱり戻したい」場合

GitHub Release は `gh release delete <tag>` + tag 削除 (`git push --delete origin <tag>`) で消せるが、homebrew-tap への自動 bump commit はすでに別 repo に push されているので別途 revert が必要。基本は **push 前に確認しておく** のが鉄則で、push 後の rollback はユーザー明示確認が必須。

## チェックリスト（迷ったらこれを順に通す）

- [ ] `git status` で変更確認、`vendor/anki` の dirty 以外を把握
- [ ] バージョン bump は本当に必要か？（普段は不要）
- [ ] ビルドチェック通った？（svelte-check / cargo check / pnpm build）
- [ ] `git add` でファイル名明示（`-A` 使ってない）
- [ ] commit メッセージは `feat(area): ...` 日本語スタイル + Co-Authored-By
- [ ] ユーザーから明示的に push 指示があるか
- [ ] push 後に workflow 起動を `gh run list` で確認
- [ ] 完了確認 → release URL をユーザーに報告
