<script lang="ts">
  import { CHART_W, CHART_H, inner, tickValues } from "./chart-utils";
  type HourBucket = { hour: number; total: number; correct: number };
  let { hours }: { hours: HourBucket[] } = $props();

  const W = CHART_W;
  const H = CHART_H;
  const { w: innerW, h: innerH, pad } = inner();
  const padL = pad.l;
  const padR = pad.r;
  const padT = pad.t;

  const cols = 24;
  const colW = $derived(innerW / cols);
  const maxTotal = $derived(Math.max(1, ...hours.map((h) => h.total)));

</script>

<svg viewBox="0 0 {W} {H}" class="h-[140px] w-full" preserveAspectRatio="none" aria-label="Hours chart">
  {#each tickValues(maxTotal) as v, i (i)}
    {@const y = padT + innerH - (v / maxTotal) * innerH}
    <line x1={padL} x2={W - padR} y1={y} y2={y} stroke="currentColor" class="text-(--color-border-default)" stroke-width="0.5" />
    <text x={padL - 6} y={y + 3} text-anchor="end" class="fill-(--color-fg-subtle) text-[9px]">{v}</text>
  {/each}
  {#each Array.from({ length: cols }) as _, hour (hour)}
    {@const bucket = hours.find((h) => h.hour === hour) ?? { hour, total: 0, correct: 0 }}
    {@const x = padL + hour * colW}
    {@const h = (bucket.total / maxTotal) * innerH}
    {@const correctH = (bucket.correct / maxTotal) * innerH}
    <rect x={x + 1} y={padT + innerH - h} width={Math.max(1, colW - 2)} height={h - correctH} fill="var(--color-fg-subtle)" opacity="0.4" />
    <rect x={x + 1} y={padT + innerH - correctH} width={Math.max(1, colW - 2)} height={correctH} fill="var(--color-accent-500)" />
  {/each}
  {#each [0, 6, 12, 18, 23] as h, i (i)}
    {@const x = padL + h * colW + colW / 2}
    <text x={x} y={H - 8} text-anchor="middle" class="fill-(--color-fg-subtle) text-[9px]">{h}</text>
  {/each}
</svg>
