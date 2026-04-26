<script lang="ts">
  import { theme } from "$lib/stores/theme.svelte";

  type Props = {
    html: string;
    css: string;
    side: "question" | "answer";
  };
  let { html, css, side }: Props = $props();

  const srcdoc = $derived.by(() => {
    const cardClass = side === "answer" ? "card answer" : "card";
    const baseTextColor =
      theme.resolved === "dark" ? "#f3f4f6" : "#0f172a";
    const baseBg = theme.resolved === "dark" ? "#1c1d22" : "#fafafb";
    const accentColor = "#7c8aff";
    return `<!doctype html>
<html>
<head>
<meta charset="utf-8" />
<style>
  html { height: 100%; }
  html, body {
    margin: 0;
    padding: 0;
    color: ${baseTextColor};
    font-family: "Inter Variable", "Inter", "Hiragino Sans", "Noto Sans JP", system-ui, sans-serif;
    -webkit-font-smoothing: antialiased;
  }
  body {
    min-height: 100%;
    box-sizing: border-box;
    padding: 36px 44px;
    line-height: 1.6;
    background: ${baseBg};
    display: block;
    text-align: center;
  }
  img { max-width: 100%; height: auto; }
  hr#answer {
    border: none;
    border-top: 1px dashed ${accentColor}55;
    margin: 28px auto;
    height: 0;
    width: 60%;
  }
  /* Anki user CSS follows; allowed to override the above. */
  ${css}
</style>
</head>
<body class="${cardClass}">
${html}
</body>
</html>`;
  });
</script>

<iframe
  title="Card content"
  sandbox="allow-scripts allow-same-origin"
  style="width: 100%; height: 100%; border: 0; display: block;"
  class="bg-transparent"
  {srcdoc}
></iframe>
