<script lang="ts">
  import { CHART_W, CHART_H, inner, tickValues } from "./chart-utils";
  type ButtonsCounts = {
    learning: number[];
    young: number[];
    mature: number[];
  };
  let { counts }: { counts: ButtonsCounts } = $props();

  const W = CHART_W;
  const H = CHART_H;
  const { w: innerW, h: innerH, pad } = inner({ r: 6, b: 24 });
  const padL = pad.l;
  const padR = pad.r;
  const padT = pad.t;

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

</script>

<svg viewBox="0 0 {W} {H}" class="h-[140px] w-full" preserveAspectRatio="none" aria-label="Answer buttons chart">
  {#each tickValues(maxValue) as v, ti (ti)}
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
