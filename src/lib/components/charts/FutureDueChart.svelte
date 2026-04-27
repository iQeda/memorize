<script lang="ts">
  type Bucket = { day: number; count: number };

  type Props = {
    buckets: Bucket[];
    days: number;
  };
  let { buckets, days }: Props = $props();

  const W = 720;
  const H = 140;
  const padL = 28;
  const padR = 28;
  const padT = 6;
  const padB = 20;
  const innerW = W - padL - padR;
  const innerH = H - padT - padB;

  const maxCount = $derived(Math.max(1, ...buckets.map((b) => b.count)));
  const barW = $derived(innerW / Math.max(1, days));

  // Cumulative line (right axis): running total of buckets
  const cumPoints = $derived.by(() => {
    const sorted = [...buckets].sort((a, b) => a.day - b.day);
    let acc = 0;
    const total = sorted.reduce((s, b) => s + b.count, 0) || 1;
    const pts: Array<{ x: number; y: number }> = [];
    pts.push({ x: padL, y: padT + innerH });
    for (let i = 0; i < days; i++) {
      const bucket = sorted.find((b) => b.day === i);
      if (bucket) acc += bucket.count;
      const x = padL + (i + 1) * barW;
      const y = padT + innerH - (acc / total) * innerH;
      pts.push({ x, y });
    }
    return pts;
  });

  const cumPath = $derived(
    cumPoints.map((p, i) => `${i === 0 ? "M" : "L"}${p.x},${p.y}`).join(" "),
  );

  function tickValues(): number[] {
    const step = Math.max(1, Math.ceil(maxCount / 4));
    const out: number[] = [];
    for (let v = 0; v <= maxCount; v += step) out.push(v);
    if (out[out.length - 1] !== maxCount) out.push(maxCount);
    return out;
  }
</script>

<svg
  viewBox="0 0 {W} {H}"
  class="h-[140px] w-full"
  preserveAspectRatio="none"
  aria-label="Future due chart"
>
  <!-- left axis ticks -->
  {#each tickValues() as v, i (i)}
    {@const y = padT + innerH - (v / maxCount) * innerH}
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

  <!-- bars -->
  {#each Array.from({ length: days }) as _, i (i)}
    {@const bucket = buckets.find((b) => b.day === i)}
    {@const c = bucket?.count ?? 0}
    {@const h = (c / maxCount) * innerH}
    <rect
      x={padL + i * barW + 1}
      y={padT + innerH - h}
      width={Math.max(1, barW - 2)}
      height={h}
      class="fill-(--color-success)"
      opacity="0.85"
    />
  {/each}

  <!-- cumulative line -->
  <path
    d={cumPath}
    fill="none"
    stroke="currentColor"
    stroke-width="1.2"
    class="text-(--color-fg-muted)"
  />

  <!-- x axis ticks (every Nth day) -->
  {#each [0, Math.floor(days / 4), Math.floor(days / 2), Math.floor((days * 3) / 4), days - 1] as i, idx (idx)}
    {@const x = padL + i * barW + barW / 2}
    <text
      x={x}
      y={H - 8}
      text-anchor="middle"
      class="fill-(--color-fg-subtle) text-[9px]"
    >
      {i}
    </text>
  {/each}
</svg>
