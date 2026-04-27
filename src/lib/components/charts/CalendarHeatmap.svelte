<script lang="ts">
  type Props = {
    /** day → review count. day relative to today (0 = today, -1 = yesterday). */
    perDay: { day: number; total: number }[];
  };
  let { perDay }: Props = $props();

  // Build 53 weeks × 7 days grid for the past year (offset by today's weekday).
  const today = new Date();
  const todayDow = today.getDay();
  const cols = 53;
  const cellSize = 11;
  const cellGap = 2;
  const W = cols * (cellSize + cellGap) + 18;
  const H = 7 * (cellSize + cellGap) + 4;

  const map = $derived(new Map(perDay.map((p) => [p.day, p.total])));
  const maxVal = $derived(Math.max(1, ...perDay.map((p) => p.total)));

  function color(v: number): string {
    if (v <= 0) return "var(--color-bg-overlay)";
    const t = Math.min(1, v / maxVal);
    // Blend accent at varying opacities
    const opacity = 0.25 + 0.75 * t;
    return `rgba(124, 138, 255, ${opacity.toFixed(2)})`;
  }

  type Cell = { x: number; y: number; v: number; date: Date };
  const cells = $derived.by(() => {
    const out: Cell[] = [];
    // Right-most column = current week, leftmost = ~52 weeks ago
    for (let c = 0; c < cols; c++) {
      for (let r = 0; r < 7; r++) {
        const dayOffset = -((cols - 1 - c) * 7) - (todayDow - r);
        if (dayOffset > 0) continue; // future days
        const v = map.get(dayOffset) ?? 0;
        const d = new Date(today);
        d.setDate(today.getDate() + dayOffset);
        out.push({
          x: 18 + c * (cellSize + cellGap),
          y: r * (cellSize + cellGap),
          v,
          date: d,
        });
      }
    }
    return out;
  });
</script>

<svg viewBox="0 0 {W} {H}" class="h-auto w-full max-w-full" aria-label="Calendar heatmap">
  {#each ["S", "M", "T", "W", "T", "F", "S"] as label, r (r)}
    <text
      x={12}
      y={r * (cellSize + cellGap) + cellSize - 1}
      text-anchor="end"
      class="fill-(--color-fg-subtle) text-[8px]"
    >
      {label}
    </text>
  {/each}
  {#each cells as c (c.date.toISOString())}
    <rect
      x={c.x}
      y={c.y}
      width={cellSize}
      height={cellSize}
      rx="1.5"
      fill={color(c.v)}
    >
      <title>{c.date.toLocaleDateString()}: {c.v} reviews</title>
    </rect>
  {/each}
</svg>
