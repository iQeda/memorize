<script lang="ts">
  import { ArrowLeft, RotateCcw, Eye, EyeOff, Pencil, Copy, BookA, Volume2, Repeat, SlidersHorizontal, X } from "lucide-svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { invoke } from "$lib/ipc";
  import { listen } from "@tauri-apps/api/event";
  import CardFrame from "$lib/components/CardFrame.svelte";
  import NoteEditor from "$lib/components/NoteEditor.svelte";
  import { onDestroy, onMount } from "svelte";
  import { fade, scale } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import { t } from "$lib/i18n/index.svelte";
  import { shortcuts } from "$lib/stores/shortcuts.svelte";
  import { sync } from "$lib/stores/sync.svelte";
  import { collection } from "$lib/stores/collection.svelte";
  import {
    speech,
    SPEECH_RATE_MIN,
    SPEECH_RATE_MAX,
    SENTENCE_PAUSE_MIN,
    SENTENCE_PAUSE_MAX,
    MAX_REPEAT_MIN,
    MAX_REPEAT_MAX,
    REPEAT_INTERVAL_MIN,
    REPEAT_INTERVAL_MAX,
    SPEECH_VOLUME_MIN,
    SPEECH_VOLUME_MAX,
  } from "$lib/stores/speech.svelte";

  type Counts = { new: number; learning: number; review: number };
  type StudyCard = {
    card_id: number;
    note_id: number;
    question_html: string;
    answer_html: string;
    css: string;
    remaining: Counts;
  };
  type NextCard =
    | {
        kind: "card";
        card_id: number;
        note_id: number;
        question_html: string;
        answer_html: string;
        css: string;
        remaining: Counts;
      }
    | { kind: "done"; new: number; learning: number; review: number };
  type RenderedCard = {
    question_html: string;
    answer_html: string;
    css: string;
  };

  const deckId = $derived(Number($page.params.deckId));

  let current = $state<StudyCard | null>(null);
  let totals = $state<Counts>({ new: 0, learning: 0, review: 0 });
  let initialTotal = $state(0);
  let cursor = $state(0);
  let showingAnswer = $state(false);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let shownAt = $state<number>(0);
  // セッション内の hide 状態。`l` で toggle、カード切替で `speech.hideDefault` にリセット。
  // 設定 (hideDefault) と分離しておくことで「設定 OFF でもこのカードだけ手で隠す」
  // 「設定 ON でも今だけ見たい」両方を許す。
  let hideActive = $state(false);

  const totalDue = $derived(totals.new + totals.learning + totals.review);
  const progress = $derived(
    initialTotal > 0 ? (cursor / initialTotal) * 100 : 0,
  );

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
    // 1 秒待ってから同じ frame のテキストを再抽出して再再生する。
    // 上書き再生 (新カード自動再生 / 手動 speak / カード切替) のときはバックエンドが
    // 旧プロセスを kill + 旧 cancel_rx に () を投げているため、このイベントは飛ばない。
    unlistenSpeech = await listen<void>("memorize://speech-finished", () => {
      if (!speech.repeat) return;
      // 最大回数到達: このカードではこれ以上ループしないが、チェックは維持。
      // 次カードに進んだら startSpeakCycle が repeatCount を 1 に戻すので、
      // 自動再生 (speakQuestionOnShow) ON 時は新カードでも 5 回ループが続く。
      if (speech.repeatCount >= speech.maxRepeat) {
        // ユーザーが「リピート完了で自動表示」を ON にしていれば、
        // hidden 状態を解除して答え合わせに進ませる。
        if (speech.autoRevealAfterRepeat && hideActive) {
          hideActive = false;
          applyHidden();
        }
        return;
      }
      const frame = lastSpokenFrame;
      if (!frame) return;
      if (repeatTimer) clearTimeout(repeatTimer);
      repeatTimer = setTimeout(() => {
        repeatTimer = null;
        speech.repeatCount += 1;
        speakFrame(frame);
      }, speech.repeatIntervalSec * 1000);
    });
    await startSession();
  });

  onDestroy(() => {
    if (unlistenSpeech) {
      unlistenSpeech();
      unlistenSpeech = null;
    }
    if (repeatTimer) {
      clearTimeout(repeatTimer);
      repeatTimer = null;
    }
    // speech.repeat / speech.repeatCount は意図的に触らない:
    // ・HMR や route 遷移で「新 mount の onMount が true → 旧 mount の
    //   onDestroy が false を上書き」というレースを防ぐ
    // ・「次の単語のレビューでもチェック状態は維持」というユーザー要件と一致
    // ・設定が ON なら次回 Reviewer 入りで onMount が改めて true をセットする
  });

  // `totals` は `get_next_card` の `remaining` でしか更新されないため、
  // 解答中に Sync が完了して別端末からの変更が入っても、次の rating まで
  // バッジが古いままになる。Sync 完了 (busy: true → false) を検知して
  // `list_decks` の最新値で塗り直す。`get_next_card` の remaining は
  // 「現在カードを除く残り」なのに対し `list_decks` は全件を返すので、
  // 現在カードの種類に応じて 1 引いて差を吸収する。
  let prevSyncBusy = false;
  $effect(() => {
    const isBusy = sync.busy;
    if (prevSyncBusy && !isBusy && current) {
      void refreshTotalsAfterSync();
    }
    prevSyncBusy = isBusy;
  });

  async function refreshTotalsAfterSync() {
    await collection.refreshDecks();
    const d = collection.decks.find((x) => x.id === deckId);
    if (!d) return;
    const next = { new: d.new_count, learning: d.learn_count, review: d.review_count };
    // 現在カードの 1 枚分は remaining から除外したいが、その種類は
    // フロントエンドに渡っていない。totals の前回値との差分で推定する：
    // 前回 totals に存在し、かつ今回の next で 0 でないカテゴリを 1 減算。
    const adjusted = { ...next };
    if (totals.new > 0 && adjusted.new > 0) adjusted.new -= 1;
    else if (totals.learning > 0 && adjusted.learning > 0) adjusted.learning -= 1;
    else if (totals.review > 0 && adjusted.review > 0) adjusted.review -= 1;
    totals = adjusted;
    // ヘッダーの "X / Y" 分母も Sync で増減した分だけ追従させる。
    // cursor は既に答えた枚数、+1 は現在カード、残り 3 カテゴリの合計が今後分。
    initialTotal = cursor + adjusted.new + adjusted.learning + adjusted.review + 1;
  }

  async function startSession() {
    loading = true;
    error = null;
    cursor = 0;
    try {
      await invoke("start_study", { deckId });
      await loadNext(/* isFirst */ true);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function loadNext(isFirst = false) {
    showingAnswer = false;
    hideActive = speech.hideDefault;
    try {
      const r = await invoke<NextCard>("get_next_card");
      if (r.kind === "card") {
        current = {
          card_id: r.card_id,
          note_id: r.note_id,
          question_html: r.question_html,
          answer_html: r.answer_html,
          css: r.css,
          remaining: r.remaining,
        };
        totals = r.remaining;
        if (isFirst) initialTotal = totals.new + totals.learning + totals.review + 1;
        shownAt = performance.now();
      } else {
        current = null;
        totals = { new: r.new, learning: r.learning, review: r.review };
      }
    } catch (e) {
      error = String(e);
    }
  }

  let editing = $state(false);
  // Audio settings popover の開閉状態。
  let audioMenuOpen = $state(false);

  // popover 外クリックで閉じる action。Svelte 5 でも `use:` action は同じ。
  function clickOutside(node: HTMLElement, callback: () => void) {
    const handler = (e: MouseEvent) => {
      if (!node.contains(e.target as Node)) callback();
    };
    document.addEventListener("mousedown", handler);
    return {
      destroy() {
        document.removeEventListener("mousedown", handler);
      },
    };
  }

  function openEditor() {
    if (!current) return;
    editing = true;
  }

  async function reloadCurrent() {
    if (!current) return;
    try {
      const r = await invoke<RenderedCard>("get_card_render", {
        cardId: current.card_id,
      });
      current = {
        ...current,
        question_html: r.question_html,
        answer_html: r.answer_html,
        css: r.css,
      };
    } catch (e) {
      console.error("get_card_render failed", e);
    }
  }

  function flip() {
    showingAnswer = !showingAnswer;
  }

  async function answer(rating: "again" | "hard" | "good" | "easy") {
    if (!current) return;
    const ms = Math.min(60_000, Math.round(performance.now() - shownAt));
    try {
      await invoke("answer_card_now", {
        rating,
        millisecondsTaken: ms,
      });
      cursor += 1;
      await loadNext();
      // サイドバーのデッキバッジは collection.decks を参照している。get_next_card
      // の remaining はヘッダー用に header の totals だけ更新するが、collection
      // ストアには反映されないので、答えるたびに list_decks を fire-and-forget で
      // 投げて全デッキのカウントを塗り直す。次カード描画はブロックしない。
      void collection.refreshDecks();
    } catch (e) {
      const msg = String(e);
      // rslib は楽観ロックで「フロントが提示された card state」と DB の最新値が
      // ずれていると `InvalidInput: card was modified` を返す。長時間 Reviewer を
      // 開きっぱなしで日付が跨ぎ elapsed_days が動いた、などで発生する。生の
      // エラー文言は出さず、ユーザー向けの一言を出してカードを reload する
      // (= loadNext で `last_queued` も最新 state で再構築される)。
      if (msg.includes("card was modified")) {
        flashInfo(t("reviewer.cardStateChanged"));
        await loadNext();
        return;
      }
      error = msg;
    }
  }

  let questionFrame = $state<HTMLIFrameElement | undefined>();
  let answerFrame = $state<HTMLIFrameElement | undefined>();
  let copyError = $state<string | null>(null);
  let copyInfo = $state<string | null>(null);
  let copyInfoTimer: ReturnType<typeof setTimeout> | null = null;
  let lastSpokenCardId = $state<number | null>(null);

  // リピート再生サイクル管理。`lastSpokenFrame` は finished イベント受信時に
  // 「どの iframe からテキストを再抽出するか」を解決するため保持する。
  // setTimeout は cleanup 用にハンドルを覚えておく必要があるが、speech store
  // ではなく Reviewer ローカルに置く: Reviewer を離れたら確実に止めたいから。
  let lastSpokenFrame: HTMLIFrameElement | null = null;
  let repeatTimer: ReturnType<typeof setTimeout> | null = null;
  let unlistenSpeech: (() => void) | null = null;

  // Anki テンプレートの answer 側は通常 `{{FrontSide}}<hr id=answer>{{Back}}` 構造で、
  // answer_html に質問部分が含まれる。フリップ後は答えだけ見せたいので hr#answer を
  // 境に前半 (質問) を削除した HTML を返す。hr#answer が無いカスタムテンプレートは
  // そのまま返す（破壊しない）。
  const answerOnlyHtml = $derived.by(() => {
    if (!current) return "";
    const html = current.answer_html;
    const doc = new DOMParser().parseFromString(html, "text/html");
    const hr = doc.querySelector('hr#answer, hr[id="answer"]');
    if (!hr || !hr.parentElement) return html;
    const parent = hr.parentElement;
    while (parent.firstChild && parent.firstChild !== hr) {
      parent.removeChild(parent.firstChild);
    }
    parent.removeChild(hr);
    return doc.body.innerHTML;
  });

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
    const frame = showingAnswer ? answerFrame : questionFrame;
    const win = frame?.contentWindow;
    const doc = frame?.contentDocument;
    if (!win || !doc) return;
    const host = doc.querySelector(".memorize-card-host");
    if (!host) return;

    // iframe 内の本文 div を全範囲選択。removeAllRanges → addRange で
    // 既存の選択を上書きするだけで、その後の操作で解除はしない。
    const range = doc.createRange();
    range.selectNodeContents(host);
    const sel = win.getSelection();
    sel?.removeAllRanges();
    sel?.addRange(range);

    // iframe にフォーカスを移し selection を AX 的に「active」にする。
    // ユーザーが手動で Cmd+J を押した際に Nani.app が現在選択中のテキストを
    // 読み取れるのは、フォーカスが当たっている要素の selection だけなので
    // 必須。後続の c/1/2/3/4/Space などは srcdoc 内の key bridge が
    // parent window に再ディスパッチするため引き続き反応する。
    win.focus();

    const text = (host.textContent ?? "").trim().replace(/\s+/g, " ");
    if (!text) return;

    copyInfo = null;
    try {
      await navigator.clipboard.writeText(text);
      copyError = null;
      flashInfo("Copied");
    } catch (e) {
      console.error("clipboard write failed", e);
      copyError = e instanceof Error ? e.message : String(e);
    }
    // Nani.app (Cmd+J 起動の辞書ランチャー) を発火する。Rust 側で CGEvent
    // 経由で keystroke を post するので、本番ビルド (Hardened Runtime +
    // ad-hoc 署名) でも動く。要アクセシビリティ権限 (初回プロンプトあり)。
    // Nani が未インストールなら Cmd+J は OS の他処理にフォールバックする
    // だけで Memorize 側に害はない。
    void invoke("start_nani_lookup", { word: text }).catch((e) => {
      console.error("start_nani_lookup failed", e);
    });
  }

  // iframe 内の本文テキストを抽出して、macOS の `say` に渡して読み上げる。
  // 設定オン時の自動発火 (新カード Question 表示) と、Speak ボタン /
  // k キーによる手動発火の両方で使う。osascript + Apple Events 方式は
  // 本番ビルド (ad-hoc + Hardened Runtime) で entitlement が無く動かない
  // ため、子プロセス起動だけで完結する `say` 経由に統一している。
  function speakFrame(frame: HTMLIFrameElement) {
    const run = () => {
      const doc = frame.contentDocument;
      if (!doc) return;
      const host = doc.querySelector(".memorize-card-host");
      if (!host) return;
      const text = (host.textContent ?? "").trim().replace(/\s+/g, " ");
      if (!text) return;
      void invoke("start_speak_text", {
        text,
        rate: speech.speechRate,
        sentencePauseMs: speech.sentencePauseMs,
        volume: speech.volume,
      }).catch((e) => {
        console.error("start_speak_text failed", e);
      });
    };
    if (
      frame.contentDocument?.readyState === "complete" &&
      frame.contentDocument.querySelector(".memorize-card-host")
    ) {
      run();
    } else {
      frame.addEventListener("load", run, { once: true });
    }
  }

  /** リピートサイクルを開始する: 進行中の setTimeout を捨て、count=1 から数え直す。 */
  function startSpeakCycle(frame: HTMLIFrameElement) {
    if (repeatTimer) {
      clearTimeout(repeatTimer);
      repeatTimer = null;
    }
    speech.repeatCount = 1;
    lastSpokenFrame = frame;
    speakFrame(frame);
  }

  function speakCardText() {
    const frame = showingAnswer ? answerFrame : questionFrame;
    if (!frame) return;
    startSpeakCycle(frame);
  }

  // 非表示モード: 親から questionFrame の body class を直接付け外しする。
  // CardFrame の srcdoc は theme/html/css の derived で、新 prop を入れると
  // iframe がフルリロードされ speech が中断するため、prop は介さず
  // contentDocument を直接操作する。answer 側は常に通常表示。
  // ラベル「[hidden mode]」は CSS ::after に頼ると iframe srcdoc が
  // HMR や user CSS で古いまま/上書きされる懸念があるので、inline-style 付き
  // の <div> を直接 body に挿入/撤去する。
  const HIDDEN_LABEL_ID = "memorize-hidden-label";
  const HIDDEN_LABEL_STYLE =
    "position:fixed; inset:0; display:flex; flex-direction:column; align-items:center; justify-content:center; gap:6px; " +
    "color:rgba(140,140,140,0.85); pointer-events:none; z-index:2147483647; visibility:visible;";
  function applyHidden() {
    const doc = questionFrame?.contentDocument;
    if (!doc?.body) return;
    doc.body.classList.toggle("memorize-hidden", hideActive);
    const existing = doc.getElementById(HIDDEN_LABEL_ID);
    if (hideActive) {
      if (!existing) {
        const el = doc.createElement("div");
        el.id = HIDDEN_LABEL_ID;
        el.setAttribute("style", HIDDEN_LABEL_STYLE);
        const main = doc.createElement("div");
        main.textContent = "[hidden mode]";
        main.setAttribute("style", "font-size:0.95rem; letter-spacing:0.05em;");
        const hint = doc.createElement("div");
        hint.textContent = t("reviewer.hideHint", { key: shortcuts.label("hide") });
        hint.setAttribute("style", "font-size:0.75rem; opacity:0.7;");
        el.appendChild(main);
        el.appendChild(hint);
        doc.body.appendChild(el);
      }
    } else if (existing) {
      existing.remove();
    }
  }

  function toggleHide() {
    hideActive = !hideActive;
    applyHidden();
  }

  // questionFrame は {#key current.card_id} で新カードごとに再生成されるため、
  // bind 変化のたびに load を待って hideActive を反映する。speakFrame と同じパターン。
  // ロード後に iframe 内 body クリックで toggle するハンドラも attach。
  $effect(() => {
    const f = questionFrame;
    if (!f) return;
    const run = () => {
      applyHidden();
      attachClickToggle();
    };
    if (
      f.contentDocument?.readyState === "complete" &&
      f.contentDocument.querySelector(".memorize-card-host")
    ) {
      run();
    } else {
      f.addEventListener("load", run, { once: true });
    }
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
    const id = current?.card_id;
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
    startSpeakCycle(frame);
  });

  // カード切替で「前のカードのリピート」を確実に止める。current?.card_id の
  // 変化を検知して進行中の setTimeout をキャンセル。新カードの自動再生 or
  // 手動 speak は startSpeakCycle 側で repeatCount=1 にリセットされる。
  let prevCardIdForRepeat: number | null = null;
  $effect(() => {
    const id = current?.card_id ?? null;
    if (id !== prevCardIdForRepeat) {
      if (repeatTimer) {
        clearTimeout(repeatTimer);
        repeatTimer = null;
      }
      prevCardIdForRepeat = id;
    }
  });

  function onKey(e: KeyboardEvent) {
    // Auto-repeat would re-fire copyCardText mid-flight, or apply a rating
    // while the user is still holding a key from a previous action.
    if (e.repeat) return;
    // The editor mounts its own inputs/textarea; let it handle its own keys.
    if (editing) return;
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
    if (!current) {
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
      if (showingAnswer) return;
      toggleHide();
      return;
    }
    if (e.key === " " || e.key === "Enter") {
      e.preventDefault();
      flip();
      return;
    }
    if (showingAnswer) {
      const rating = shortcuts.ratingFor(e.key);
      if (rating) {
        e.preventDefault();
        void answer(rating);
      }
    }
  }

  type Tone = "danger" | "warning" | "accent" | "success";
  type Rating = "again" | "hard" | "good" | "easy";
  const buttons = $derived<
    { rating: Rating; label: string; tone: Tone }[]
  >([
    { rating: "again", label: t("reviewer.again"), tone: "danger" },
    { rating: "hard", label: t("reviewer.hard"), tone: "warning" },
    { rating: "good", label: t("reviewer.good"), tone: "accent" },
    { rating: "easy", label: t("reviewer.easy"), tone: "success" },
  ]);

  const toneBg: Record<Tone, string> = {
    accent:
      "bg-(--color-accent-500) text-(--color-fg-onAccent) hover:bg-(--color-accent-600)",
    success:
      "bg-(--color-success) text-(--color-fg-onAccent) hover:brightness-110",
    warning:
      "bg-(--color-warning) text-(--color-bg-base) hover:brightness-105",
    danger:
      "bg-(--color-danger) text-(--color-fg-onAccent) hover:brightness-110",
  };
</script>

<div
  class="relative flex h-full flex-col transition-colors duration-300"
  class:bg-answer={showingAnswer}
>
  <div class="flex items-center justify-between px-6 py-3">
    <button
      type="button"
      onclick={() => goto("/")}
      class="flex items-center gap-1.5 rounded-(--radius-md) px-2 py-1 text-sm text-(--color-fg-muted) transition-colors hover:bg-(--color-bg-overlay) hover:text-(--color-fg-default)"
    >
      <ArrowLeft size={14} />
      {t("reviewer.back")}
      <span class="ml-1 font-mono text-[10px] opacity-70">Esc</span>
    </button>
    <p class="flex items-center gap-6 text-xs text-(--color-fg-subtle)">
      <span class="number-tabular">
        {cursor + (current ? 1 : 0)} / {initialTotal || totalDue || "—"}
      </span>
      <span class="hidden items-center gap-1.5 sm:flex">
        <span
          class="inline-flex items-center gap-1.5 rounded-full border border-(--color-border-default) bg-(--color-bg-elevated) px-2 py-0.5"
        >
          <span class="text-[10px] font-semibold tracking-wider text-(--color-fg-subtle) uppercase">
            {t("decks.new")}
          </span>
          <span class="number-tabular text-xs font-medium text-(--color-fg-default)">{totals.new}</span>
        </span>
        <span
          class="inline-flex items-center gap-1.5 rounded-full border border-(--color-border-default) bg-(--color-bg-elevated) px-2 py-0.5"
        >
          <span class="text-[10px] font-semibold tracking-wider text-(--color-fg-subtle) uppercase">
            {t("decks.learning")}
          </span>
          <span class="number-tabular text-xs font-medium text-(--color-fg-default)">{totals.learning}</span>
        </span>
        <span
          class="inline-flex items-center gap-1.5 rounded-full border border-(--color-border-default) bg-(--color-bg-elevated) px-2 py-0.5"
        >
          <span class="text-[10px] font-semibold tracking-wider text-(--color-fg-subtle) uppercase">
            {t("decks.review")}
          </span>
          <span class="number-tabular text-xs font-medium text-(--color-fg-default)">{totals.review}</span>
        </span>
      </span>
    </p>
    <div class="flex items-center gap-1">
      {#if current}
        <div class="relative" use:clickOutside={() => (audioMenuOpen = false)}>
          <button
            type="button"
            onclick={() => (audioMenuOpen = !audioMenuOpen)}
            aria-haspopup="dialog"
            aria-expanded={audioMenuOpen}
            class="flex h-7 items-center gap-1.5 rounded-(--radius-md) px-2 text-(--color-fg-muted) transition-colors hover:bg-(--color-bg-overlay) hover:text-(--color-fg-default) {audioMenuOpen
              ? 'bg-(--color-bg-overlay) text-(--color-fg-default)'
              : ''}"
            aria-label={t("reviewer.audioSettings")}
            title={t("reviewer.audioSettings")}
          >
            <SlidersHorizontal size={14} />
          </button>
          {#if audioMenuOpen}
            <div
              class="audio-popover absolute right-0 top-9 z-50 max-h-[80vh] w-[400px] overflow-y-auto rounded-(--radius-lg) border border-(--color-border-default) bg-(--color-bg-elevated) p-5 shadow-(--shadow-card)"
              role="dialog"
              aria-label={t("reviewer.audioSettings")}
            >
              <p class="mb-4 text-xs font-semibold tracking-wider text-(--color-fg-subtle) uppercase">
                {t("reviewer.audioSettings")}
              </p>
              <!-- Speak question on show -->
              <label class="mb-3 flex cursor-pointer items-center justify-between gap-2 text-sm text-(--color-fg-default) select-none">
                <span>{t("settings.speech.autoLabel")}</span>
                <input
                  type="checkbox"
                  checked={speech.speakQuestionOnShow}
                  onchange={(e) =>
                    speech.setSpeakQuestionOnShow(
                      (e.currentTarget as HTMLInputElement).checked,
                    )}
                  class="h-4 w-4 cursor-pointer accent-(--color-accent-500)"
                />
              </label>
              <!-- Enable repeat on question start -->
              <label class="mb-3 flex cursor-pointer items-center justify-between gap-2 text-sm text-(--color-fg-default) select-none">
                <span>{t("settings.speech.repeatOnStartLabel")}</span>
                <input
                  type="checkbox"
                  checked={speech.repeatOnQuestionStart}
                  onchange={(e) =>
                    speech.setRepeatOnQuestionStart(
                      (e.currentTarget as HTMLInputElement).checked,
                    )}
                  class="h-4 w-4 cursor-pointer accent-(--color-accent-500)"
                />
              </label>
              <!-- Maximum repeat count -->
              <div class="mb-3 flex items-center justify-between gap-2">
                <label for="audio-max-repeat" class="text-sm text-(--color-fg-default)">
                  {t("settings.speech.maxRepeatLabel")}
                </label>
                <div class="flex items-center gap-2">
                  <input
                    id="audio-max-repeat"
                    type="range"
                    min={MAX_REPEAT_MIN}
                    max={MAX_REPEAT_MAX}
                    step="1"
                    value={speech.maxRepeat}
                    oninput={(e) => {
                      const next = Number.parseInt(
                        (e.currentTarget as HTMLInputElement).value,
                        10,
                      );
                      if (Number.isFinite(next)) speech.setMaxRepeat(next);
                    }}
                    aria-label={t("settings.speech.maxRepeatLabel")}
                    class="w-40 accent-(--color-accent-500)"
                  />
                  <span class="number-tabular w-10 text-right text-xs text-(--color-fg-muted)">
                    {speech.maxRepeat}
                  </span>
                </div>
              </div>
              <!-- Repeat interval (seconds) -->
              <div class="mb-3 flex items-center justify-between gap-2">
                <label for="audio-interval" class="text-sm text-(--color-fg-default)">
                  {t("settings.speech.repeatIntervalLabel")}
                </label>
                <div class="flex items-center gap-2">
                  <input
                    id="audio-interval"
                    type="range"
                    min={REPEAT_INTERVAL_MIN}
                    max={REPEAT_INTERVAL_MAX}
                    step="0.01"
                    value={speech.repeatIntervalSec}
                    oninput={(e) => {
                      const next = Number.parseFloat(
                        (e.currentTarget as HTMLInputElement).value,
                      );
                      if (Number.isFinite(next)) speech.setRepeatIntervalSec(next);
                    }}
                    aria-label={t("settings.speech.repeatIntervalLabel")}
                    class="w-40 accent-(--color-accent-500)"
                  />
                  <span class="number-tabular w-12 text-right text-xs text-(--color-fg-muted)">
                    {speech.repeatIntervalSec.toFixed(2)}s
                  </span>
                </div>
              </div>
              <!-- Speech rate -->
              <div class="mb-3 flex items-center justify-between gap-2">
                <label for="audio-rate" class="text-sm text-(--color-fg-default)">
                  {t("settings.speech.rateLabel")}
                </label>
                <div class="flex items-center gap-2">
                  <input
                    id="audio-rate"
                    type="range"
                    min={SPEECH_RATE_MIN}
                    max={SPEECH_RATE_MAX}
                    step="10"
                    value={speech.speechRate}
                    oninput={(e) => {
                      const next = Number.parseInt(
                        (e.currentTarget as HTMLInputElement).value,
                        10,
                      );
                      if (Number.isFinite(next)) speech.setSpeechRate(next);
                    }}
                    aria-label={t("settings.speech.rateLabel")}
                    class="w-32 accent-(--color-accent-500)"
                  />
                  <span class="number-tabular w-10 text-right text-xs text-(--color-fg-muted)">
                    {speech.speechRate}
                  </span>
                  <button
                    type="button"
                    onclick={() => {
                      void invoke("start_speak_text", {
                        text: t("settings.speech.ratePreviewText"),
                        rate: speech.speechRate,
                        sentencePauseMs: speech.sentencePauseMs,
                        volume: speech.volume,
                      }).catch((e) => console.error("sample play failed", e));
                    }}
                    class="flex items-center gap-1 rounded-(--radius-md) border border-(--color-border-default) bg-(--color-bg-base) px-2 py-1.5 text-xs hover:bg-(--color-bg-overlay) active:scale-[0.97]"
                    title={t("settings.speech.ratePreview")}
                    aria-label={t("settings.speech.ratePreview")}
                  >
                    <Volume2 size={12} strokeWidth={2.25} />
                  </button>
                </div>
              </div>
              <!-- Volume -->
              <div class="mb-3 flex items-center justify-between gap-2">
                <label for="audio-volume" class="text-sm text-(--color-fg-default)">
                  {t("settings.speech.volumeLabel")}
                </label>
                <div class="flex items-center gap-2">
                  <input
                    id="audio-volume"
                    type="range"
                    min={SPEECH_VOLUME_MIN}
                    max={SPEECH_VOLUME_MAX}
                    step="1"
                    value={speech.volume}
                    oninput={(e) => {
                      const next = Number.parseInt(
                        (e.currentTarget as HTMLInputElement).value,
                        10,
                      );
                      if (Number.isFinite(next)) speech.setVolume(next);
                    }}
                    aria-label={t("settings.speech.volumeLabel")}
                    class="w-40 accent-(--color-accent-500)"
                  />
                  <span class="number-tabular w-12 text-right text-xs text-(--color-fg-muted)">
                    {speech.volume}%
                  </span>
                </div>
              </div>
              <!-- Sentence pause -->
              <div class="mb-3 flex items-center justify-between gap-2">
                <label for="audio-pause" class="text-sm text-(--color-fg-default)">
                  {t("settings.speech.sentencePauseLabel")}
                </label>
                <div class="flex items-center gap-2">
                  <input
                    id="audio-pause"
                    type="range"
                    min={SENTENCE_PAUSE_MIN}
                    max={SENTENCE_PAUSE_MAX}
                    step="100"
                    value={speech.sentencePauseMs}
                    oninput={(e) => {
                      const next = Number.parseInt(
                        (e.currentTarget as HTMLInputElement).value,
                        10,
                      );
                      if (Number.isFinite(next)) speech.setSentencePauseMs(next);
                    }}
                    aria-label={t("settings.speech.sentencePauseLabel")}
                    class="w-40 accent-(--color-accent-500)"
                  />
                  <span class="number-tabular w-14 text-right text-xs text-(--color-fg-muted)">
                    {speech.sentencePauseMs}ms
                  </span>
                </div>
              </div>
              <div class="my-4 border-t border-(--color-border-default)"></div>
              <!-- Repeat session toggle -->
              <label class="mb-3 flex cursor-pointer items-center justify-between gap-2 text-sm text-(--color-fg-default) select-none">
                <span class="flex items-center gap-2">
                  <Repeat size={14} />
                  {t("reviewer.repeat")}
                </span>
                <input
                  type="checkbox"
                  bind:checked={speech.repeat}
                  onchange={() => {
                    speech.repeatCount = 0;
                    if (!speech.repeat && repeatTimer) {
                      clearTimeout(repeatTimer);
                      repeatTimer = null;
                    }
                  }}
                  class="h-4 w-4 cursor-pointer accent-(--color-accent-500)"
                  aria-label={t("reviewer.repeat")}
                />
              </label>
              <!-- Auto-reveal after repeat -->
              <label class="flex cursor-pointer items-center justify-between gap-2 text-sm text-(--color-fg-default) select-none">
                <span>{t("reviewer.autoRevealAfterRepeat")}</span>
                <input
                  type="checkbox"
                  checked={speech.autoRevealAfterRepeat}
                  onchange={(e) =>
                    speech.setAutoRevealAfterRepeat(
                      (e.currentTarget as HTMLInputElement).checked,
                    )}
                  class="h-4 w-4 cursor-pointer accent-(--color-accent-500)"
                />
              </label>
            </div>
          {/if}
        </div>
        <button
          type="button"
          onclick={openEditor}
          class="flex h-7 items-center gap-1.5 rounded-(--radius-md) px-2 text-(--color-fg-muted) transition-colors hover:bg-(--color-bg-overlay) hover:text-(--color-fg-default)"
          aria-label={t("settings.shortcut.editNote")}
          title="{t('settings.shortcut.editNote')} (E)"
        >
          <Pencil size={14} />
          <span class="font-mono text-[10px] opacity-70">E</span>
        </button>
      {/if}
    </div>
  </div>

  <div
    class="bg-(--color-bg-overlay) transition-[height] duration-300 {showingAnswer
      ? 'h-1'
      : 'h-0.5'}"
  >
    <div
      class="h-full transition-all duration-500 {showingAnswer
        ? 'bg-(--color-success)'
        : 'bg-(--color-accent-500)'}"
      style="width: {progress}%; transition-timing-function: var(--ease-out-expo);"
    ></div>
  </div>

  <div class="flex flex-1 flex-col items-center px-6 pt-12 pb-6">
    {#if loading}
      <p class="text-sm text-(--color-fg-muted)">{t("reviewer.loading")}</p>
    {:else if error}
      <p class="text-sm text-(--color-danger)">{error}</p>
    {:else if !current}
      <div in:scale={{ duration: 240, start: 0.92, easing: cubicOut }} class="text-center">
        <p class="font-display text-3xl font-medium">{t("reviewer.done")}</p>
        <p class="mt-2 text-sm text-(--color-fg-muted)">{t("reviewer.sessionFinished")}</p>
        <button
          type="button"
          onclick={() => goto("/")}
          class="mt-6 inline-flex items-center gap-2 rounded-(--radius-md) border border-(--color-border-strong) px-4 py-1.5 text-sm transition-colors hover:bg-(--color-bg-overlay)"
        >
          {t("reviewer.backToDecks")}
          <span class="font-mono text-[10px] opacity-70">↵ / Space</span>
        </button>
      </div>
    {:else}
      <div class="relative w-full">
        <div
          class="absolute -top-3.5 left-1/2 z-10 flex -translate-x-1/2 items-center gap-1.5 rounded-full px-3 py-1 text-[10px] font-semibold tracking-[0.18em] uppercase shadow-(--shadow-subtle) transition-all duration-200 {showingAnswer
            ? 'bg-(--color-success) text-(--color-fg-onAccent) ring-1 ring-(--color-success)/30 ring-offset-2 ring-offset-(--color-bg-base)'
            : 'bg-(--color-bg-overlay) text-(--color-fg-muted)'}"
        >
          <span
            class="h-1.5 w-1.5 rounded-full {showingAnswer
              ? 'bg-(--color-fg-onAccent)/80'
              : 'bg-(--color-accent-500)'}"
          ></span>
          {showingAnswer ? t("reviewer.answer") : t("reviewer.question")}
        </div>
        {#key current.card_id}
          <article
            in:fade={{ duration: 220, easing: cubicOut, delay: 60 }}
            style="height: 420px; min-height: 420px; max-height: 420px; perspective: 2000px;"
            class="block w-full shrink-0"
          >
            <div
              class="relative h-full w-full transition-transform duration-500 ease-out"
              style="transform-style: preserve-3d; transform: rotateY({showingAnswer ? 180 : 0}deg);"
            >
              <div
                style="backface-visibility: hidden; -webkit-backface-visibility: hidden;"
                class="absolute inset-0 overflow-hidden rounded-(--radius-xl) border bg-(--color-bg-elevated) shadow-(--shadow-card) transition-[border-color,box-shadow] duration-200 {showingAnswer
                  ? 'border-(--color-success)/50 shadow-(--shadow-glow)'
                  : 'border-(--color-border-default)'}"
              >
                <CardFrame
                  bind:iframeEl={questionFrame}
                  html={current.question_html}
                  css={current.css}
                  side="question"
                />
              </div>
              <div
                style="backface-visibility: hidden; -webkit-backface-visibility: hidden; transform: rotateY(180deg);"
                class="absolute inset-0 overflow-hidden rounded-(--radius-xl) border bg-(--color-bg-elevated) shadow-(--shadow-card) transition-[border-color,box-shadow] duration-200 {showingAnswer
                  ? 'border-(--color-success)/50 shadow-(--shadow-glow)'
                  : 'border-(--color-border-default)'}"
              >
                <CardFrame
                  bind:iframeEl={answerFrame}
                  html={answerOnlyHtml}
                  css={current.css}
                  side="answer"
                />
              </div>
            </div>
          </article>
        {/key}
      </div>

      <div
        class="mt-8 flex w-full shrink-0 flex-col items-center gap-3"
      >
        <!-- 1段目: Nani / Speak / (front: Hide-Reveal, back: Show Question)。
             3 つ目の枠を front/back で使い分けることで位置を固定し、Show Answer 押下後に
             Rating が同じ「2段目位置」に出てきて手の移動なしで採点できる。 -->
        <div class="flex items-center justify-center gap-3">
          <button
            type="button"
            onclick={copyCardText}
            class="flex h-16 w-32 flex-col items-center justify-center gap-0.5 rounded-(--radius-md) border border-(--color-border-strong) bg-(--color-bg-elevated) px-5 py-2.5 text-(--color-fg-default) shadow-(--shadow-card) transition-all hover:-translate-y-0.5 hover:bg-(--color-bg-overlay) hover:shadow-(--shadow-glow) active:translate-y-0 active:scale-[0.97]"
            title="Nani"
          >
            <span class="flex items-center gap-1.5 text-sm font-medium">
              <BookA size={14} strokeWidth={2.25} />
              Nani
            </span>
            <span class="font-mono text-[10px] opacity-70">{shortcuts.label("copy")}</span>
          </button>
          <button
            type="button"
            onclick={speakCardText}
            class="flex h-16 w-32 flex-col items-center justify-center gap-0.5 rounded-(--radius-md) border border-(--color-border-strong) bg-(--color-bg-elevated) px-5 py-2.5 text-(--color-fg-default) shadow-(--shadow-card) transition-all hover:-translate-y-0.5 hover:bg-(--color-bg-overlay) hover:shadow-(--shadow-glow) active:translate-y-0 active:scale-[0.97]"
            title={t("reviewer.speak")}
          >
            <span class="flex items-center gap-1.5 text-sm font-medium">
              <Volume2 size={14} strokeWidth={2.25} />
              {t("reviewer.speak")}
            </span>
            <span class="font-mono text-[10px] opacity-70">{shortcuts.label("speak")}</span>
          </button>
          {#if showingAnswer}
            <button
              type="button"
              onclick={flip}
              class="flex h-16 w-32 flex-col items-center justify-center gap-0.5 rounded-(--radius-md) border border-(--color-border-strong) bg-(--color-bg-elevated) px-5 py-2.5 text-(--color-fg-default) shadow-(--shadow-card) transition-all hover:-translate-y-0.5 hover:bg-(--color-bg-overlay) hover:shadow-(--shadow-glow) active:translate-y-0 active:scale-[0.97]"
              title={t("reviewer.showQuestion")}
            >
              <span class="flex items-center gap-1.5 text-sm font-medium">
                <RotateCcw size={14} strokeWidth={2.25} />
                {t("reviewer.showQuestion")}
              </span>
              <span class="font-mono text-[10px] opacity-70">Space / ↵</span>
            </button>
          {:else}
            <button
              type="button"
              onclick={toggleHide}
              class="flex h-16 w-32 flex-col items-center justify-center gap-0.5 rounded-(--radius-md) border border-(--color-border-strong) bg-(--color-bg-elevated) px-5 py-2.5 text-(--color-fg-default) shadow-(--shadow-card) transition-all hover:-translate-y-0.5 hover:bg-(--color-bg-overlay) hover:shadow-(--shadow-glow) active:translate-y-0 active:scale-[0.97]"
              title={hideActive ? t("reviewer.reveal") : t("reviewer.hide")}
            >
              <span class="flex items-center gap-1.5 text-sm font-medium">
                {#if hideActive}
                  <Eye size={14} strokeWidth={2.25} />
                  {t("reviewer.reveal")}
                {:else}
                  <EyeOff size={14} strokeWidth={2.25} />
                  {t("reviewer.hide")}
                {/if}
              </span>
              <span class="font-mono text-[10px] opacity-70">{shortcuts.label("hide")}</span>
            </button>
          {/if}
        </div>
        <!-- 2段目: front は Show Answer (横長)、back は Rating x 4。
             同じ位置に置くことで Show Answer → Rating の手の移動がゼロ。 -->
        {#if showingAnswer}
          <div class="flex items-center justify-center gap-3">
            {#each buttons as b, i (b.rating)}
              <button
                type="button"
                onclick={() => answer(b.rating)}
                in:fade={{ duration: 200, delay: 40 + i * 30, easing: cubicOut }}
                class="flex min-w-[88px] flex-col items-center gap-0.5 rounded-(--radius-md) px-5 py-2.5 shadow-(--shadow-card) transition-all hover:-translate-y-0.5 hover:shadow-(--shadow-glow) active:translate-y-0 active:scale-[0.97] {toneBg[b.tone]}"
              >
                <span class="text-sm font-medium">{b.label}</span>
                <span class="font-mono text-[10px] opacity-70">{shortcuts.label(b.rating)}</span>
              </button>
            {/each}
          </div>
        {:else}
          <div class="flex items-center justify-center gap-3">
            <button
              type="button"
              onclick={flip}
              class="flex h-16 w-[420px] flex-col items-center justify-center gap-0.5 rounded-(--radius-md) border border-(--color-border-strong) bg-(--color-bg-elevated) px-5 py-2.5 text-(--color-fg-default) shadow-(--shadow-card) transition-all hover:-translate-y-0.5 hover:bg-(--color-bg-overlay) hover:shadow-(--shadow-glow) active:translate-y-0 active:scale-[0.97]"
              title={t("reviewer.showAnswer")}
            >
              <span class="flex items-center gap-1.5 text-base font-medium">
                <Eye size={16} strokeWidth={2.25} />
                {t("reviewer.showAnswer")}
              </span>
              <span class="font-mono text-[10px] opacity-70">Space / ↵</span>
            </button>
          </div>
        {/if}
      </div>
    {/if}
  </div>
  {#if copyError}
    <div
      role="alert"
      class="pointer-events-auto fixed bottom-6 left-1/2 z-20 flex max-w-lg -translate-x-1/2 gap-2 rounded-(--radius-md) border border-(--color-danger)/40 bg-(--color-danger)/10 px-4 py-2.5 text-xs text-(--color-danger) shadow-(--shadow-card) select-text"
    >
      <div class="min-w-0 flex-1">
        <p class="font-medium">Copy failed</p>
        <p class="mt-0.5 font-mono text-[11px] break-all opacity-90 select-all">{copyError}</p>
      </div>
      <div class="flex shrink-0 flex-col gap-1">
        <button
          type="button"
          onclick={copyErrorMessage}
          aria-label="Copy error"
          title="Copy error"
          class="grid h-5 w-5 place-items-center rounded text-(--color-danger) transition-colors hover:bg-(--color-danger)/20"
        >
          <Copy size={12} />
        </button>
        <button
          type="button"
          onclick={() => (copyError = null)}
          aria-label="Dismiss"
          title="Dismiss"
          class="grid h-5 w-5 place-items-center rounded text-(--color-danger) transition-colors hover:bg-(--color-danger)/20"
        >
          <X size={12} />
        </button>
      </div>
    </div>
  {:else if copyInfo}
    <div
      role="status"
      class="pointer-events-auto fixed bottom-6 left-1/2 z-20 -translate-x-1/2 rounded-(--radius-md) border border-(--color-border-default) bg-(--color-bg-elevated) px-3 py-1.5 text-xs text-(--color-fg-default) shadow-(--shadow-card) select-text"
    >
      {copyInfo}
    </div>
  {/if}

</div>

<svelte:window onkeydown={onKey} />

{#if editing && current}
  <NoteEditor
    mode="edit"
    noteId={current.note_id}
    onClose={() => (editing = false)}
    onSaved={() => void reloadCurrent()}
  />
{/if}
