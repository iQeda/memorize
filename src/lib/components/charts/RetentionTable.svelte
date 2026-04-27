<script lang="ts">
  type R = {
    young_passed: number;
    young_failed: number;
    mature_passed: number;
    mature_failed: number;
  };
  type Props = {
    today: R;
    yesterday: R;
    week: R;
    month: R;
    year: R;
    all_time: R;
  };
  let stats: Props = $props();

  const rows = $derived([
    { label: "Today", r: stats.today },
    { label: "Yesterday", r: stats.yesterday },
    { label: "Last week", r: stats.week },
    { label: "Last month", r: stats.month },
    { label: "Last year", r: stats.year },
    { label: "All time", r: stats.all_time },
  ]);

  function pct(passed: number, failed: number): string {
    const total = passed + failed;
    if (total === 0) return "N/A";
    return ((passed / total) * 100).toFixed(1) + "%";
  }
</script>

<table class="w-full text-xs">
  <thead class="text-[10px] tracking-wider text-(--color-fg-subtle) uppercase">
    <tr>
      <th class="py-1.5 text-left font-medium"></th>
      <th class="py-1.5 text-right font-medium text-(--color-success)">Young</th>
      <th class="py-1.5 text-right font-medium text-(--color-success)">Mature</th>
      <th class="py-1.5 text-right font-medium">Total</th>
      <th class="py-1.5 text-right font-medium">Count</th>
    </tr>
  </thead>
  <tbody>
    {#each rows as row (row.label)}
      {@const total = row.r.young_passed + row.r.young_failed + row.r.mature_passed + row.r.mature_failed}
      <tr class="border-t border-(--color-border-default)">
        <td class="py-1.5 font-medium text-(--color-fg-default)">{row.label}</td>
        <td class="py-1.5 text-right text-(--color-success) tabular-nums">
          {pct(row.r.young_passed, row.r.young_failed)}
        </td>
        <td class="py-1.5 text-right text-(--color-success) tabular-nums">
          {pct(row.r.mature_passed, row.r.mature_failed)}
        </td>
        <td class="py-1.5 text-right tabular-nums">
          {pct(
            row.r.young_passed + row.r.mature_passed,
            row.r.young_failed + row.r.mature_failed,
          )}
        </td>
        <td class="py-1.5 text-right text-(--color-fg-subtle) tabular-nums">{total}</td>
      </tr>
    {/each}
  </tbody>
</table>
