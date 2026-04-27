<script lang="ts">
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

  const W = 720;
  const H = 200;
  const padL = 36;
  const padR = 36;
  const padT = 8;
  const padB = 28;
  const innerW = W - padL - padR;
  const innerH = H - padT - padB;

  const colW = $derived(innerW / Math.max(1, columns));
  const totals = $derived(
    Array.from({ length: columns }, (_, i) =>
      series.reduce((s, ser) => s + (ser.values[i] ?? 0), 0),
    ),
  );
  const maxTotal = $derived(Math.max(1, ...totals));

  function tickValues(): number[] {
    const step = Math.max(1, Math.ceil(maxTotal / 4));
    const out: number[] = [];
    for (let v = 0; v <= maxTotal; v += step) out.push(v);
    if (out[out.length - 1] !== maxTotal) out.push(maxTotal);
    return out;
  }

  function xTicks(): number[] {
    if (xLabels) return xLabels;
    return [0, Math.floor(columns / 4), Math.floor(columns / 2), Math.floor((columns * 3) / 4), columns - 1];
  }
</script>

<svg viewBox="0 0 {W} {H}" class="h-[200px] w-full" aria-label="Stacked bar chart">
  {#each tickValues() as v (v)}
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
  {#each Array(columns) as _, i (i)}
    {@const x = padL + i * colW}
    <g>
      {#each series as ser, sIdx (ser.label)}
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
  {#each xTicks() as i (i)}
    {@const x = padL + i * colW + colW / 2}
    <text x={x} y={H - 8} text-anchor="middle" class="fill-(--color-fg-subtle) text-[9px]">
      {xFormat(i + minCol)}
    </text>
  {/each}
</svg>

<div class="mt-2 flex flex-wrap gap-3 text-[11px]">
  {#each series as ser (ser.label)}
    <span class="flex items-center gap-1.5">
      <span class="h-2 w-2 rounded-sm" style="background:{ser.color}"></span>
      {ser.label}
    </span>
  {/each}
</div>
