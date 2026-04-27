<script lang="ts">
  type Bucket = { key: number; value: number };

  type Props = {
    buckets: Bucket[];
    /** Tick formatter for x axis. */
    xFormat?: (v: number) => string;
    /** Width of each bin in domain units (1 = group consecutive integers). */
    binSize?: number;
    color?: string;
    /** Override domain */
    minKey?: number;
    maxKey?: number;
  };
  let {
    buckets,
    xFormat = (v) => `${v}`,
    binSize = 1,
    color = "var(--color-accent-500)",
    minKey,
    maxKey,
  }: Props = $props();

  const W = 720;
  const H = 200;
  const padL = 36;
  const padR = 36;
  const padT = 8;
  const padB = 28;
  const innerW = W - padL - padR;
  const innerH = H - padT - padB;

  const min = $derived(
    minKey ?? (buckets.length ? Math.min(...buckets.map((b) => b.key)) : 0),
  );
  const max = $derived(
    maxKey ?? (buckets.length ? Math.max(...buckets.map((b) => b.key)) : 1),
  );

  const binned = $derived.by(() => {
    if (binSize <= 1) return buckets;
    const map = new Map<number, number>();
    for (const b of buckets) {
      const bucket = Math.floor((b.key - min) / binSize) * binSize + min;
      map.set(bucket, (map.get(bucket) ?? 0) + b.value);
    }
    return Array.from(map.entries())
      .map(([key, value]) => ({ key, value }))
      .sort((a, b) => a.key - b.key);
  });

  const maxValue = $derived(Math.max(1, ...binned.map((b) => b.value)));
  const span = $derived(Math.max(1, max - min));
  const barW = $derived((innerW / span) * binSize);

  function tickValues(): number[] {
    const step = Math.max(1, Math.ceil(maxValue / 4));
    const out: number[] = [];
    for (let v = 0; v <= maxValue; v += step) out.push(v);
    if (out[out.length - 1] !== maxValue) out.push(maxValue);
    return out;
  }

  function xTicks(): number[] {
    return [
      min,
      min + Math.floor(span / 4),
      min + Math.floor(span / 2),
      min + Math.floor((span * 3) / 4),
      max,
    ];
  }
</script>

<svg viewBox="0 0 {W} {H}" class="h-[200px] w-full" aria-label="Histogram">
  {#each tickValues() as v (v)}
    {@const y = padT + innerH - (v / maxValue) * innerH}
    <line
      x1={padL}
      x2={W - padR}
      y1={y}
      y2={y}
      stroke="currentColor"
      class="text-(--color-border-default)"
      stroke-width="0.5"
    />
    <text
      x={padL - 6}
      y={y + 3}
      text-anchor="end"
      class="fill-(--color-fg-subtle) text-[9px]"
    >
      {v}
    </text>
  {/each}
  {#each binned as b (b.key)}
    {@const x = padL + ((b.key - min) / span) * innerW}
    {@const h = (b.value / maxValue) * innerH}
    <rect
      x={x + 1}
      y={padT + innerH - h}
      width={Math.max(1, barW - 2)}
      height={h}
      fill={color}
      opacity="0.85"
    />
  {/each}
  {#each xTicks() as v (v)}
    {@const x = padL + ((v - min) / span) * innerW}
    <text
      x={x}
      y={H - 8}
      text-anchor="middle"
      class="fill-(--color-fg-subtle) text-[9px]"
    >
      {xFormat(v)}
    </text>
  {/each}
</svg>
