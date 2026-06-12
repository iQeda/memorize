<script lang="ts">
  import { RotateCcw, Eye, EyeOff, BookA, Volume2 } from "lucide-svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { listen } from "@tauri-apps/api/event";
  import NoteEditor from "$lib/components/NoteEditor.svelte";
  import { onDestroy, onMount } from "svelte";
  import { t } from "$lib/i18n/index.svelte";
  import { shortcuts } from "$lib/stores/shortcuts.svelte";
  import { sync } from "$lib/stores/sync.svelte";
  import { speech } from "$lib/stores/speech.svelte";
  import { stripQuestionFromAnswer } from "$lib/reviewer/answer-html";
  import { whenFrameReady } from "$lib/reviewer/frame-text";
  import { setHiddenOverlay } from "$lib/reviewer/hidden-overlay";
  import { SpeechCycle } from "$lib/reviewer/speech-cycle.svelte";
  import { ReviewSession } from "$lib/reviewer/session.svelte";
  import { speakFrameText } from "$lib/reviewer/speak";
  import { copyCardTextForNani } from "$lib/reviewer/copy-nani";
  import ReviewHeader from "$lib/components/reviewer/ReviewHeader.svelte";
  import CardStage from "$lib/components/reviewer/CardStage.svelte";
  import ReviewActionButton from "$lib/components/reviewer/ReviewActionButton.svelte";
  import RatingBar from "$lib/components/reviewer/RatingBar.svelte";
  import DoneScreen from "$lib/components/reviewer/DoneScreen.svelte";
  import CopyToast from "$lib/components/reviewer/CopyToast.svelte";

  const deckId = $derived(Number($page.params.deckId));

  // セッション内の hide 状態。`l` で toggle、カード切替で `speech.hideDefault` にリセット。
  // 設定 (hideDefault) と分離しておくことで「設定 OFF でもこのカードだけ手で隠す」
  // 「設定 ON でも今だけ見たい」両方を許す。
  let hideActive = $state(false);

  const session = new ReviewSession(
    () => deckId,
    (msg) => flashInfo(msg),
    () => (hideActive = speech.hideDefault),
  );

  let editing = $state(false);
  let questionFrame = $state<HTMLIFrameElement | undefined>();
  let answerFrame = $state<HTMLIFrameElement | undefined>();
  let copyError = $state<string | null>(null);
  let copyInfo = $state<string | null>(null);
  let copyInfoTimer: ReturnType<typeof setTimeout> | null = null;
  let lastSpokenCardId = $state<number | null>(null);
  let unlistenSpeech: (() => void) | null = null;

  const answerOnlyHtml = $derived(
    session.current ? stripQuestionFromAnswer(session.current.answer_html) : "",
  );

  // リピート再生サイクル (timer / 再生元 frame / カウンタ) は SpeechCycle が
  // 管理する。timer は Reviewer を離れたら確実に止めたいのでページ寿命に紐づく。
  const cycle = new SpeechCycle(speakFrameText);

  onMount(async () => {
    // 永続設定「問題開始時にリピートを有効にする」が ON なら、Reviewer に
    // 入った時点でチェックを入れた状態にする。onDestroy では repeat を
    // 触らないので、ユーザーが手動で OFF にした状態も次の Reviewer 入りまで
    // 維持される。
    if (speech.repeatOnQuestionStart) {
      speech.repeat = true;
      speech.repeatCount = 0;
    }
    hideActive = speech.hideDefault;
    // バックエンドの say が自然終了するたびに飛んでくる。リピート ON のあいだ、
    // interval 待機ののち同じ frame のテキストを再抽出して再再生する。
    // 上書き再生 (新カード自動再生 / 手動 speak / カード切替) のときはバックエンドが
    // 旧プロセスを kill + 旧 cancel_rx に () を投げているため、このイベントは飛ばない。
    unlistenSpeech = await listen<void>("memorize://speech-finished", () => {
      const outcome = cycle.onSpeechFinished();
      // 最大回数到達: このカードではこれ以上ループしないが、チェックは維持。
      // 次カードに進んだら cycle.start が repeatCount を 1 に戻すので、
      // 自動再生 (speakQuestionOnShow) ON 時は新カードでもループが続く。
      if (outcome === "max-reached") {
        // ユーザーが「リピート完了で自動表示」を ON にしていれば、
        // hidden 状態を解除して答え合わせに進ませる。
        if (speech.autoRevealAfterRepeat && hideActive) {
          hideActive = false;
          applyHidden();
        }
      }
    });
    await session.start();
  });

  onDestroy(() => {
    if (unlistenSpeech) {
      unlistenSpeech();
      unlistenSpeech = null;
    }
    cycle.cancelTimer();
    // speech.repeat / speech.repeatCount は意図的に触らない:
    // ・HMR や route 遷移で「新 mount の onMount が true → 旧 mount の
    //   onDestroy が false を上書き」というレースを防ぐ
    // ・「次の単語のレビューでもチェック状態は維持」というユーザー要件と一致
    // ・設定が ON なら次回 Reviewer 入りで onMount が改めて true をセットする
  });

  // Sync 完了 (busy: true → false) を検知してバッジを最新値で塗り直す。
  let prevSyncBusy = false;
  $effect(() => {
    const isBusy = sync.busy;
    if (prevSyncBusy && !isBusy && session.current) {
      void session.refreshTotalsAfterSync();
    }
    prevSyncBusy = isBusy;
  });

  function openEditor() {
    if (!session.current) return;
    editing = true;
  }

  function flashInfo(msg: string) {
    copyInfo = msg;
    if (copyInfoTimer) clearTimeout(copyInfoTimer);
    copyInfoTimer = setTimeout(() => {
      copyInfo = null;
      copyInfoTimer = null;
    }, 4500);
  }

  async function copyErrorMessage() {
    if (!copyError) return;
    try {
      await navigator.clipboard.writeText(copyError);
      flashInfo("Copied");
    } catch (e) {
      console.error("clipboard copy failed", e);
    }
  }

  async function copyCardText() {
    const frame = session.showingAnswer ? answerFrame : questionFrame;
    const outcome = await copyCardTextForNani(frame);
    if (outcome.kind === "nothing") return;
    copyInfo = null;
    if (outcome.kind === "copied") {
      copyError = null;
      flashInfo("Copied");
    } else {
      copyError = outcome.message;
    }
  }

  function speakCardText() {
    const frame = session.showingAnswer ? answerFrame : questionFrame;
    if (!frame) return;
    cycle.start(frame);
  }

  function applyHidden() {
    const doc = questionFrame?.contentDocument;
    if (!doc) return;
    setHiddenOverlay(doc, hideActive, t("reviewer.hideHint", { key: shortcuts.label("hide") }));
  }

  function toggleHide() {
    hideActive = !hideActive;
    applyHidden();
  }

  // questionFrame は {#key card_id} で新カードごとに再生成されるため、
  // bind 変化のたびに load を待って hideActive を反映する。speakFrame と同じパターン。
  // ロード後に iframe 内 body クリックで toggle するハンドラも attach。
  $effect(() => {
    const f = questionFrame;
    if (!f) return;
    whenFrameReady(f, () => {
      applyHidden();
      attachClickToggle();
    });
  });

  // iframe 内 body にクリックハンドラを 1 回だけ attach。
  // テキスト選択時 (sel.length > 0) や複数クリック (detail > 1) は toggle 抑止
  // — Nani 用の単語ダブルクリック選択や、コピー操作を壊さないため。
  function attachClickToggle() {
    const doc = questionFrame?.contentDocument;
    if (!doc?.body) return;
    if (doc.body.dataset.memorizeClickToggle === "1") return;
    doc.body.dataset.memorizeClickToggle = "1";
    doc.body.addEventListener("click", (e) => {
      if (e.detail > 1) return;
      const sel = doc.getSelection()?.toString() ?? "";
      if (sel.length > 0) return;
      toggleHide();
    });
  }

  $effect(() => {
    const id = session.current?.card_id;
    if (!id) return;
    // 新カード表示時に自動再生をかける条件:
    //   - 自動読み上げ ON (speakQuestionOnShow) — 既存挙動
    //   - もしくはリピート ON (speech.repeat) — 「問題開始時にリピートを有効にする」
    //     設定が ON で onMount が speech.repeat=true にしたケースを含む。
    //     1 回目の再生がないと finished イベントも飛ばず、リピートサイクルが
    //     永遠に始まらない。リピート希望時は 1 回目も自動でかける。
    if (!speech.speakQuestionOnShow && !speech.repeat) return;
    if (lastSpokenCardId === id) return;
    const frame = questionFrame;
    if (!frame) return;
    lastSpokenCardId = id;
    cycle.start(frame);
  });

  // カード切替で「前のカードのリピート」を確実に止める。card_id の変化を
  // 検知して進行中の setTimeout をキャンセル。新カードの自動再生 or
  // 手動 speak は cycle.start 側で repeatCount=1 にリセットされる。
  let prevCardIdForRepeat: number | null = null;
  $effect(() => {
    const id = session.current?.card_id ?? null;
    if (id !== prevCardIdForRepeat) {
      cycle.cancelTimer();
      prevCardIdForRepeat = id;
    }
  });

  // Audio popover のリピートチェック切替: カウンタを 0 に戻し、OFF なら
  // 進行中の再再生予約も破棄する。
  function onRepeatToggled() {
    speech.repeatCount = 0;
    if (!speech.repeat) cycle.cancelTimer();
  }

  function onKey(e: KeyboardEvent) {
    // Auto-repeat would re-fire copyCardText mid-flight, or apply a rating
    // while the user is still holding a key from a previous action.
    if (e.repeat) return;
    // The editor mounts its own inputs/textarea; let it handle its own keys.
    if (editing) return;
    // 意図的に $lib/utils/keyboard の isTextFieldTarget より狭いチェック
    // (input/textarea のみ + 上の editing フラグ)。reviewer には select /
    // contentEditable が無く、Nani 用オフスクリーン input との組み合わせを
    // 既存挙動のまま保つため、広いバリアントに置き換えないこと。
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
    // 修飾子付きのキーは OS / WebView の標準ショートカットに譲る。
    // 特に Cmd+J は Nani.app のグローバルホットキー — Copy が "j" にバインド
    // されている状態で preventDefault すると Nani に届かなくなる。同様に
    // Cmd+A/Cmd+S/Cmd+F なども rating キー (a/s/d/f) と衝突するため、修飾子
    // 付きは review 操作をすべて見送って素通しする。
    const hasModifier = e.metaKey || e.ctrlKey || e.altKey;
    // Esc — leave the review session and return to the deck overview.
    if (e.key === "Escape" && !hasModifier) {
      e.preventDefault();
      void goto("/");
      return;
    }
    if (hasModifier) return;
    // セッション終了画面 (current === null = "All cards reviewed") では
    // フリップやレーティング対象がないので、Enter / Space は "Back to decks"
    // としてホームへ戻すだけにする (Esc と同等の動線)。それ以外のキーは無視。
    if (!session.current) {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        void goto("/");
      }
      return;
    }
    // `e` (no modifiers) — open the note editor for the current card.
    // Anki Desktop と同じ慣習。
    if (e.key === "e" || e.key === "E") {
      e.preventDefault();
      openEditor();
      return;
    }
    // Copy works on either side — sometimes you need to look up a word
    // before flipping to the answer.
    if (shortcuts.isCopy(e.key)) {
      e.preventDefault();
      void copyCardText();
      return;
    }
    // Speak works on either side. The same flow as the Speak button: select
    // the iframe contents and trigger macOS Speak Selection via osascript.
    if (shortcuts.isSpeak(e.key)) {
      e.preventDefault();
      speakCardText();
      return;
    }
    // shift+L: 永続「デフォルト非表示」設定を反転（設定画面トグルと等価）。
    // hasModifier に shift は含めていないのでここまで到達する。固定キーとして扱う。
    if (e.shiftKey && (e.key === "L" || e.key === "l")) {
      e.preventDefault();
      speech.setHideDefault(!speech.hideDefault);
      hideActive = speech.hideDefault;
      applyHidden();
      return;
    }
    // `l` (no shift): カード内 hide toggle。次カードで hideDefault に戻る。
    // back 表示中は無効 — hide は front 専用なので、画面 UI (disabled ボタン) と挙動を揃える。
    if (!e.shiftKey && shortcuts.isHide(e.key)) {
      e.preventDefault();
      if (session.showingAnswer) return;
      toggleHide();
      return;
    }
    if (e.key === " " || e.key === "Enter") {
      e.preventDefault();
      session.flip();
      return;
    }
    if (session.showingAnswer) {
      const rating = shortcuts.ratingFor(e.key);
      if (rating) {
        e.preventDefault();
        void session.answer(rating);
      }
    }
  }
</script>

<div
  class="relative flex h-full flex-col transition-colors duration-300"
  class:bg-answer={session.showingAnswer}
>
  <ReviewHeader
    totals={session.totals}
    cursor={session.cursor}
    initialTotal={session.initialTotal}
    totalDue={session.totalDue}
    hasCard={session.current !== null}
    onBack={() => goto("/")}
    onEdit={openEditor}
    {onRepeatToggled}
  />

  <div
    class="bg-(--color-bg-overlay) transition-[height] duration-300 {session.showingAnswer
      ? 'h-1'
      : 'h-0.5'}"
  >
    <div
      class="h-full transition-all duration-500 {session.showingAnswer
        ? 'bg-(--color-success)'
        : 'bg-(--color-accent-500)'}"
      style="width: {session.progress}%; transition-timing-function: var(--ease-out-expo);"
    ></div>
  </div>

  <div class="flex flex-1 flex-col items-center px-6 pt-12 pb-6">
    {#if session.loading}
      <p class="text-sm text-(--color-fg-muted)">{t("reviewer.loading")}</p>
    {:else if session.error}
      <p class="text-sm text-(--color-danger)">{session.error}</p>
    {:else if !session.current}
      <DoneScreen onBack={() => goto("/")} />
    {:else}
      <CardStage
        card={session.current}
        answerHtml={answerOnlyHtml}
        showingAnswer={session.showingAnswer}
        bind:questionFrame
        bind:answerFrame
      />

      <div class="mt-8 flex w-full shrink-0 flex-col items-center gap-3">
        <!-- 1段目: Nani / Speak / (front: Hide-Reveal, back: Show Question)。
             3 つ目の枠を front/back で使い分けることで位置を固定し、Show Answer 押下後に
             Rating が同じ「2段目位置」に出てきて手の移動なしで採点できる。 -->
        <div class="flex items-center justify-center gap-3">
          <ReviewActionButton
            icon={BookA}
            label="Nani"
            hotkey={shortcuts.label("copy")}
            onclick={() => void copyCardText()}
            title="Nani"
          />
          <ReviewActionButton
            icon={Volume2}
            label={t("reviewer.speak")}
            hotkey={shortcuts.label("speak")}
            onclick={speakCardText}
          />
          {#if session.showingAnswer}
            <ReviewActionButton
              icon={RotateCcw}
              label={t("reviewer.showQuestion")}
              hotkey="Space / ↵"
              onclick={() => session.flip()}
            />
          {:else}
            <ReviewActionButton
              icon={hideActive ? Eye : EyeOff}
              label={hideActive ? t("reviewer.reveal") : t("reviewer.hide")}
              hotkey={shortcuts.label("hide")}
              onclick={toggleHide}
            />
          {/if}
        </div>
        <!-- 2段目: front は Show Answer (横長)、back は Rating x 4。
             同じ位置に置くことで Show Answer → Rating の手の移動がゼロ。 -->
        {#if session.showingAnswer}
          <RatingBar onRate={(r) => void session.answer(r)} />
        {:else}
          <div class="flex items-center justify-center gap-3">
            <ReviewActionButton
              icon={Eye}
              label={t("reviewer.showAnswer")}
              hotkey="Space / ↵"
              onclick={() => session.flip()}
              size="wide"
            />
          </div>
        {/if}
      </div>
    {/if}
  </div>

  <CopyToast
    error={copyError}
    info={copyInfo}
    onCopyError={() => void copyErrorMessage()}
    onDismiss={() => (copyError = null)}
  />
</div>

<svelte:window onkeydown={onKey} />

{#if editing && session.current}
  <NoteEditor
    mode="edit"
    noteId={session.current.note_id}
    onClose={() => (editing = false)}
    onSaved={() => void session.reloadCurrent()}
  />
{/if}
