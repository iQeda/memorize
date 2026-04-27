<script lang="ts">
  type Counts = {
    new_cards: number;
    learn: number;
    relearn: number;
    young: number;
    mature: number;
    suspended: number;
    buried: number;
  };
  let { counts }: { counts: Counts } = $props();

  const slices = $derived([
    { label: "New", value: counts.new_cards, color: "#7c8aff" },
    { label: "Learning", value: counts.learn, color: "#f5a623" },
    { label: "Relearning", value: counts.relearn, color: "#e26d6d" },
    { label: "Young", value: counts.young, color: "#7ed87a" },
    { label: "Mature", value: counts.mature, color: "#3aa050" },
    { label: "Suspended", value: counts.suspended, color: "#d4b14a" },
    { label: "Buried", value: counts.buried, color: "#888" },
  ]);
  const total = $derived(slices.reduce((s, x) => s + x.value, 0));

  function arc(cx: number, cy: number, r: number, start: number, end: number) {
    const large = end - start > Math.PI ? 1 : 0;
    const x1 = cx + r * Math.cos(start);
    const y1 = cy + r * Math.sin(start);
    const x2 = cx + r * Math.cos(end);
    const y2 = cy + r * Math.sin(end);
    return `M${cx},${cy} L${x1},${y1} A${r},${r} 0 ${large},1 ${x2},${y2} Z`;
  }

  const paths = $derived.by(() => {
    if (total === 0) return [];
    let acc = -Math.PI / 2;
    return slices
      .filter((s) => s.value > 0)
      .map((s) => {
        const angle = (s.value / total) * Math.PI * 2;
        const d = arc(60, 60, 56, acc, acc + angle);
        acc += angle;
        return { d, color: s.color };
      });
  });
</script>

<div class="flex flex-wrap items-center gap-6">
  <svg viewBox="0 0 120 120" class="h-32 w-32 shrink-0" aria-label="Card counts pie">
    {#if total === 0}
      <circle
        cx="60"
        cy="60"
        r="56"
        class="fill-(--color-bg-overlay)"
      />
    {:else}
      {#each paths as p (p.d)}
        <path d={p.d} fill={p.color} />
      {/each}
    {/if}
  </svg>
  <ul class="grid flex-1 grid-cols-1 gap-y-1 text-xs sm:grid-cols-2">
    {#each slices as s (s.label)}
      <li class="flex items-center justify-between gap-3 pr-2">
        <span class="flex items-center gap-1.5">
          <span class="h-2 w-2 rounded-sm" style="background:{s.color}"></span>
          {s.label}
        </span>
        <span class="number-tabular text-(--color-fg-muted)">
          {s.value}
          {#if total > 0}
            <span class="ml-1 text-[10px] text-(--color-fg-subtle)">
              {((s.value / total) * 100).toFixed(1)}%
            </span>
          {/if}
        </span>
      </li>
    {/each}
    <li class="col-span-full mt-1 flex items-center justify-between border-t border-(--color-border-default) pt-1 text-(--color-fg-default)">
      <span>Total</span>
      <span class="number-tabular">{total}</span>
    </li>
  </ul>
</div>
