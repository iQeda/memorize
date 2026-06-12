<script lang="ts">
  import { CHART_W, CHART_H, inner, tickValues } from "./chart-utils";
  type Series = { label: string; values: number[]; color: string };
  type Props = {
    /** Each value is a column. Multiple series stack within a column. */
    columns: number;
    series: Series[];
    /** xFormat for axis label */
    xFormat?: (col: number) => string;
    minCol?: number;
    /** Inverted day axis (negative days for past, 0 today). */
    xLabels?: number[];
  };
  let {
    columns,
    series,
    xFormat = (i) => `${i}`,
    minCol = 0,
    xLabels,
  }: Props = $props();

  const W = CHART_W;
  const H = CHART_H;
  const { w: innerW, h: innerH, pad } = inner();
  const padL = pad.l;
  const padR = pad.r;
  const padT = pad.t;

  const colW = $derived(innerW / Math.max(1, columns));
  const totals = $derived(
    Array.from({ length: columns }, (_, i) =>
      series.reduce((s, ser) => s + (ser.values[i] ?? 0), 0),
    ),
  );
  const maxTotal = $derived(Math.max(1, ...totals));


  function xTicks(): number[] {
    if (xLabels) return xLabels;
    return [0, Math.floor(columns / 4), Math.floor(columns / 2), Math.floor((columns * 3) / 4), columns - 1];
  }
</script>

<svg viewBox="0 0 {W} {H}" class="h-[140px] w-full" preserveAspectRatio="none" aria-label="Stacked bar chart">
  {#each tickValues(maxTotal) as v, i (i)}
    {@const y = padT + innerH - (v / maxTotal) * innerH}
    <line
      x1={padL}
      x2={W - padR}
      y1={y}
      y2={y}
      stroke="currentColor"
      class="text-(--color-border-default)"
      stroke-width="0.5"
    />
    <text x={padL - 6} y={y + 3} text-anchor="end" class="fill-(--color-fg-subtle) text-[9px]">
      {v}
    </text>
  {/each}
  {#each Array.from({ length: columns }) as _, i (i)}
    {@const x = padL + i * colW}
    <g>
      {#each series as ser, sIdx (sIdx)}
        {@const value = ser.values[i] ?? 0}
        {@const before = series
          .slice(0, sIdx)
          .reduce((s, p) => s + (p.values[i] ?? 0), 0)}
        {@const h = (value / maxTotal) * innerH}
        {@const yBase = (before / maxTotal) * innerH}
        <rect
          x={x + 1}
          y={padT + innerH - h - yBase}
          width={Math.max(1, colW - 2)}
          height={h}
          fill={ser.color}
          opacity="0.9"
        />
      {/each}
    </g>
  {/each}
  {#each xTicks() as i, idx (idx)}
    {@const x = padL + i * colW + colW / 2}
    <text x={x} y={H - 8} text-anchor="middle" class="fill-(--color-fg-subtle) text-[9px]">
      {xFormat(i + minCol)}
    </text>
  {/each}
</svg>

<div class="mt-2 flex flex-wrap gap-3 text-[11px]">
  {#each series as ser, i (i)}
    <span class="flex items-center gap-1.5">
      <span class="h-2 w-2 rounded-sm" style="background:{ser.color}"></span>
      {ser.label}
    </span>
  {/each}
</div>
