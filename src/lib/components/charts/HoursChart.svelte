<script lang="ts">
  type HourBucket = { hour: number; total: number; correct: number };
  let { hours }: { hours: HourBucket[] } = $props();

  const W = 720;
  const H = 200;
  const padL = 36;
  const padR = 36;
  const padT = 8;
  const padB = 28;
  const innerW = W - padL - padR;
  const innerH = H - padT - padB;

  const cols = 24;
  const colW = $derived(innerW / cols);
  const maxTotal = $derived(Math.max(1, ...hours.map((h) => h.total)));

  function tickValues(): number[] {
    const step = Math.max(1, Math.ceil(maxTotal / 4));
    const out: number[] = [];
    for (let v = 0; v <= maxTotal; v += step) out.push(v);
    if (out[out.length - 1] !== maxTotal) out.push(maxTotal);
    return out;
  }
</script>

<svg viewBox="0 0 {W} {H}" class="h-[200px] w-full" aria-label="Hours chart">
  {#each tickValues() as v (v)}
    {@const y = padT + innerH - (v / maxTotal) * innerH}
    <line x1={padL} x2={W - padR} y1={y} y2={y} stroke="currentColor" class="text-(--color-border-default)" stroke-width="0.5" />
    <text x={padL - 6} y={y + 3} text-anchor="end" class="fill-(--color-fg-subtle) text-[9px]">{v}</text>
  {/each}
  {#each Array(cols) as _, hour (hour)}
    {@const bucket = hours.find((h) => h.hour === hour) ?? { hour, total: 0, correct: 0 }}
    {@const x = padL + hour * colW}
    {@const h = (bucket.total / maxTotal) * innerH}
    {@const correctH = (bucket.correct / maxTotal) * innerH}
    <rect x={x + 1} y={padT + innerH - h} width={Math.max(1, colW - 2)} height={h - correctH} fill="var(--color-fg-subtle)" opacity="0.4" />
    <rect x={x + 1} y={padT + innerH - correctH} width={Math.max(1, colW - 2)} height={correctH} fill="var(--color-accent-500)" />
  {/each}
  {#each [0, 6, 12, 18, 23] as h (h)}
    {@const x = padL + h * colW + colW / 2}
    <text x={x} y={H - 8} text-anchor="middle" class="fill-(--color-fg-subtle) text-[9px]">{h}</text>
  {/each}
</svg>
