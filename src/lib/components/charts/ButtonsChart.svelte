<script lang="ts">
  type ButtonsCounts = {
    learning: number[];
    young: number[];
    mature: number[];
  };
  let { counts }: { counts: ButtonsCounts } = $props();

  const W = 720;
  const H = 140;
  const padL = 28;
  const padR = 6;
  const padT = 6;
  const padB = 24;
  const innerW = W - padL - padR;
  const innerH = H - padT - padB;

  const groups = $derived([
    { label: "Learning", values: counts.learning },
    { label: "Young", values: counts.young },
    { label: "Mature", values: counts.mature },
  ]);
  const buttonColors = ["#e26d6d", "#f5a623", "#7ed87a", "#3aa050"]; // Again/Hard/Good/Easy

  const maxValue = $derived(
    Math.max(1, ...groups.flatMap((g) => g.values)),
  );
  const groupW = $derived(innerW / groups.length);
  const barW = $derived((groupW * 0.75) / 4);

  function tickValues(): number[] {
    const step = Math.max(1, Math.ceil(maxValue / 4));
    const out: number[] = [];
    for (let v = 0; v <= maxValue; v += step) out.push(v);
    if (out[out.length - 1] !== maxValue) out.push(maxValue);
    return out;
  }
</script>

<svg viewBox="0 0 {W} {H}" class="h-[140px] w-full" preserveAspectRatio="none" aria-label="Answer buttons chart">
  {#each tickValues() as v, ti (ti)}
    {@const y = padT + innerH - (v / maxValue) * innerH}
    <line x1={padL} x2={W - padR} y1={y} y2={y} stroke="currentColor" class="text-(--color-border-default)" stroke-width="0.5" />
    <text x={padL - 6} y={y + 3} text-anchor="end" class="fill-(--color-fg-subtle) text-[9px]">{v}</text>
  {/each}
  {#each groups as g, gi (g.label)}
    {@const groupX = padL + gi * groupW + groupW * 0.125}
    {#each g.values as v, bi (bi)}
      {@const h = (v / maxValue) * innerH}
      <rect
        x={groupX + bi * barW}
        y={padT + innerH - h}
        width={barW - 1}
        height={h}
        fill={buttonColors[bi]}
      />
    {/each}
    <text
      x={padL + gi * groupW + groupW / 2}
      y={H - 16}
      text-anchor="middle"
      class="fill-(--color-fg-muted) text-[10px]"
    >
      {g.label}
    </text>
  {/each}
</svg>

<div class="mt-1 flex flex-wrap gap-3 text-[11px]">
  {#each ["Again", "Hard", "Good", "Easy"] as label, i (label)}
    <span class="flex items-center gap-1.5">
      <span class="h-2 w-2 rounded-sm" style="background:{buttonColors[i]}"></span>
      {label}
    </span>
  {/each}
</div>
