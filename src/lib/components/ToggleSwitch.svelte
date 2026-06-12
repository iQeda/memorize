<script lang="ts">
  // settings の 6 箇所に重複していた h-5 w-9 トグルスイッチの共通化。
  // class は旧実装から逐語コピー (disabled:opacity-50 は autostart 由来で、
  // disabled を渡さない限り効かない)。
  type Props = {
    checked: boolean;
    onToggle: (next: boolean) => void;
    /** aria-label。視覚ラベルは呼び出し側 (SettingRow) が持つ。 */
    label: string;
    disabled?: boolean;
  };
  let { checked, onToggle, label, disabled = false }: Props = $props();
</script>

<button
  type="button"
  onclick={() => onToggle(!checked)}
  {disabled}
  aria-pressed={checked}
  aria-label={label}
  class="relative h-5 w-9 shrink-0 rounded-full transition-colors disabled:opacity-50 {checked
    ? 'bg-(--color-accent-500)'
    : 'bg-(--color-bg-overlay)'}"
>
  <span
    class="absolute top-0.5 h-4 w-4 rounded-full bg-white shadow-(--shadow-subtle) transition-all {checked
      ? 'left-[18px]'
      : 'left-0.5'}"
  ></span>
</button>
