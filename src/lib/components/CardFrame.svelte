<script lang="ts">
  import { theme } from "$lib/stores/theme.svelte";

  type Props = {
    html: string;
    css: string;
    side: "question" | "answer";
    iframeEl?: HTMLIFrameElement;
  };
  let { html, css, side, iframeEl = $bindable() }: Props = $props();

  // Tags split to keep Svelte's tokenizer from prematurely closing this block.
  const SCRIPT_OPEN = "<" + "script>";
  const SCRIPT_CLOSE = "</" + "script>";

  const wrapJaScript = `(function() {
    var RE = /[\\u3000-\\u303F\\u3040-\\u309F\\u30A0-\\u30FF\\u31F0-\\u31FF\\u3400-\\u4DBF\\u4E00-\\u9FFF\\uF900-\\uFAFF\\uFE30-\\uFE4F\\uFF00-\\uFFEF]+/g;
    // Skip walking entirely for English-only cards (the common case for
    // vocab learning) — saves a full text-node traversal per render.
    if (!RE.test(document.body.textContent || '')) return;
    RE.lastIndex = 0;
    var walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT, null);
    var nodes = [];
    var n;
    while ((n = walker.nextNode())) {
      if (n.parentNode && (n.parentNode.tagName === 'SCRIPT' || n.parentNode.tagName === 'STYLE')) continue;
      nodes.push(n);
    }
    for (var i = 0; i < nodes.length; i++) {
      var t = nodes[i];
      var s = t.nodeValue || '';
      if (!RE.test(s)) continue;
      RE.lastIndex = 0;
      var frag = document.createDocumentFragment();
      var last = 0, m;
      while ((m = RE.exec(s)) !== null) {
        if (m.index > last) frag.appendChild(document.createTextNode(s.slice(last, m.index)));
        var span = document.createElement('span');
        span.setAttribute('lang', 'ja');
        span.textContent = m[0];
        frag.appendChild(span);
        last = m.index + m[0].length;
      }
      if (last < s.length) frag.appendChild(document.createTextNode(s.slice(last)));
      t.parentNode.replaceChild(frag, t);
    }
  })();`;

  // Copy ボタンは iframe にフォーカスを移すことで本文 selection を AX 的に
  // active にする（手動の Cmd+J で Nani.app がその選択を読めるように）。
  // ただしフォーカスが iframe にあると c/1/2/3/4/Space 等のキーは
  // <svelte:window onkeydown> に届かなくなるので、iframe 内の keydown を
  // parent window へ再ディスパッチして親側 onKey にイベントを引き渡す。
  // Cmd+J は親側で何も preventDefault しないため、元イベントが OS に
  // そのまま伝播し Nani のグローバルホットキーが発火する。
  const keyBridgeScript = `(function() {
    window.addEventListener('keydown', function(e) {
      try {
        var ev = new KeyboardEvent('keydown', {
          key: e.key,
          code: e.code,
          ctrlKey: e.ctrlKey,
          shiftKey: e.shiftKey,
          altKey: e.altKey,
          metaKey: e.metaKey,
          repeat: e.repeat,
          bubbles: true,
          cancelable: true,
        });
        parent.dispatchEvent(ev);
        if (ev.defaultPrevented) e.preventDefault();
      } catch (_) {}
    });
  })();`;

  const srcdoc = $derived.by(() => {
    const cardClass = side === "answer" ? "card answer" : "card";
    const baseTextColor =
      theme.resolved === "dark" ? "#f3f4f6" : "#0f172a";
    const baseBg = theme.resolved === "dark" ? "#1c1d22" : "#fafafb";
    const accentColor = "#7c8aff";
    const scrollThumb = theme.resolved === "dark" ? "#3a3b40" : "#c9cad0";
    const scrollThumbHover = theme.resolved === "dark" ? "#4a4b52" : "#a8a9b1";
    return `<!doctype html>
<html>
<head>
<meta charset="utf-8" />
<style>
  html, body {
    margin: 0;
    padding: 0;
    color: ${baseTextColor};
    font-family:
      "-apple-system", "BlinkMacSystemFont", "SF Pro Text",
      "Inter Variable", "Inter",
      "Hiragino Kaku Gothic ProN", "Hiragino Sans",
      "Yu Gothic Medium", "Yu Gothic",
      "Noto Sans JP", system-ui, sans-serif;
    font-feature-settings: "kern", "liga", "calt", "palt";
    font-synthesis: none;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    text-rendering: optimizeLegibility;
    overflow-wrap: anywhere;
    word-break: normal;
    line-break: strict;
  }
  html, body { height: 100%; }
  body {
    box-sizing: border-box;
    padding: 28px 36px;
    font-size: 1.125rem;
    line-height: 1.75;
    letter-spacing: 0.003em;
    background: ${baseBg};
    text-align: center;
    overflow-y: auto;
    overflow-x: hidden;
    scrollbar-gutter: stable;
  }
  /* 上下中央寄せのために二重 wrapper を使う。
     - outer (.memorize-card-frame) を flex column container にして
       inner を justify-content で iframe 高さの中央に置く。
     - inner (.memorize-card-host) は普通の block。flex container の
       直下にすると inline 子（<b>, <i>, text node, <br>）が flex item
       として block 化されて縦並びになるので、必ず 1 段挟む。 */
  .memorize-card-frame {
    min-height: 100%;
    display: flex;
    flex-direction: column;
    justify-content: safe center;
  }
  /* English vocab card front renders bold by default. A small script below
     wraps any Japanese run in <span lang="ja">…</span>, which we then
     pull back to regular weight here. Anki user CSS may override. */
  .card {
    font-weight: 700;
    font-feature-settings: "kern", "liga", "calt", "ss01";
  }
  .card [lang="ja"] {
    font-weight: 400;
    font-family:
      "Hiragino Kaku Gothic ProN", "Hiragino Sans",
      "Yu Gothic Medium", "Yu Gothic", "Noto Sans JP", sans-serif;
  }
  .card em, .card i { font-style: italic; }
  /* Visible-on-hover scrollbar instead of macOS overlay-only one. */
  body::-webkit-scrollbar {
    width: 8px;
    height: 8px;
  }
  body::-webkit-scrollbar-thumb {
    background: ${scrollThumb};
    border-radius: 4px;
  }
  body::-webkit-scrollbar-thumb:hover {
    background: ${scrollThumbHover};
  }
  body::-webkit-scrollbar-track {
    background: transparent;
  }
  img { max-width: 100%; height: auto; }
  pre, code { white-space: pre-wrap; word-break: break-word; }
  table { max-width: 100%; }
  hr#answer {
    border: none;
    border-top: 1px dashed ${accentColor}55;
    margin: 22px auto;
    height: 0;
    width: 60%;
  }
  /* Anki user CSS follows; allowed to override the above. */
  ${css}
</style>
</head>
<body class="${cardClass}"><div class="memorize-card-frame"><div class="memorize-card-host">${html}</div></div>${SCRIPT_OPEN}${wrapJaScript}${SCRIPT_CLOSE}${SCRIPT_OPEN}${keyBridgeScript}${SCRIPT_CLOSE}</body>
</html>`;
  });
</script>

<iframe
  bind:this={iframeEl}
  title="Card content"
  sandbox="allow-scripts allow-same-origin"
  style="width: 100%; height: 100%; border: 0; display: block;"
  class="bg-transparent"
  {srcdoc}
></iframe>
