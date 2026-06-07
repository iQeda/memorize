<script lang="ts">
  import { FileText, Loader2 } from "lucide-svelte";
  import { pkg } from "$lib/stores/package.svelte";
  import { importFlow } from "$lib/stores/import-flow.svelte";
  import { t } from "$lib/i18n/index.svelte";
</script>

{#if importFlow.active && importFlow.csvPreview}
  {@const preview = importFlow.csvPreview}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4 backdrop-blur-sm"
    onclick={(e) => {
      if (e.target === e.currentTarget) importFlow.cancel();
    }}
    onkeydown={(e) => {
      if (e.key === "Escape") importFlow.cancel();
    }}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <div
      class="flex max-h-[80vh] w-full max-w-lg flex-col overflow-hidden rounded-(--radius-xl) border border-(--color-border-default) bg-(--color-bg-elevated) shadow-(--shadow-card)"
    >
      <header
        class="flex items-center gap-2 border-b border-(--color-border-default) px-5 py-3"
      >
        <FileText size={16} class="text-(--color-accent-500)" />
        <h2 class="text-sm font-semibold tracking-wide">{t("io.tsvDialogTitle")}</h2>
      </header>

      <div class="flex-1 space-y-4 overflow-y-auto px-5 py-4">
        <dl class="grid grid-cols-2 gap-x-4 gap-y-2 text-xs">
          <div>
            <dt class="text-[10px] tracking-wider text-(--color-fg-subtle) uppercase">
              {t("io.tsvDeck")}
            </dt>
            <dd class="mt-0.5 text-(--color-fg-default)">{preview.deck || "—"}</dd>
          </div>
          <div>
            <dt class="text-[10px] tracking-wider text-(--color-fg-subtle) uppercase">
              {t("io.tsvNotetype")}
            </dt>
            <dd class="mt-0.5 text-(--color-fg-default)">{preview.notetype || "—"}</dd>
          </div>
          <div>
            <dt class="text-[10px] tracking-wider text-(--color-fg-subtle) uppercase">
              {t("io.tsvDelimiter")}
            </dt>
            <dd class="mt-0.5 text-(--color-fg-default)">{preview.delimiter}</dd>
          </div>
          <div>
            <dt class="text-[10px] tracking-wider text-(--color-fg-subtle) uppercase">
              {t("io.tsvColumns")}
            </dt>
            <dd class="mt-0.5 text-(--color-fg-default)">{preview.columns}</dd>
          </div>
        </dl>

        <label class="block">
          <span class="mb-1 block text-[10px] tracking-wider text-(--color-fg-subtle) uppercase">
            {t("io.tsvDupe")}
          </span>
          <select
            bind:value={importFlow.csvDupe}
            class="w-full rounded-(--radius-md) border border-(--color-border-default) bg-(--color-bg-base) px-3 py-1.5 text-sm shadow-(--shadow-subtle) outline-none focus:border-(--color-accent-500)"
          >
            <option value="update">{t("io.tsvDupeUpdate")}</option>
            <option value="preserve">{t("io.tsvDupePreserve")}</option>
            <option value="duplicate">{t("io.tsvDupeDuplicate")}</option>
          </select>
        </label>

        {#if preview.preview_rows.length > 0}
          <div>
            <p class="mb-1 text-[10px] tracking-wider text-(--color-fg-subtle) uppercase">
              {t("io.tsvPreview")}
            </p>
            <div
              class="overflow-x-auto rounded-(--radius-md) border border-(--color-border-default)"
            >
              <table class="w-full border-collapse text-xs">
                <tbody>
                  {#each preview.preview_rows as row, ri (ri)}
                    <tr class="border-b border-(--color-border-default) last:border-0">
                      {#each row as cell, ci (ci)}
                        <td
                          class="max-w-[16rem] truncate px-2 py-1 align-top text-(--color-fg-muted)"
                          >{cell}</td
                        >
                      {/each}
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>
        {/if}
      </div>

      <footer
        class="flex items-center justify-end gap-2 border-t border-(--color-border-default) px-5 py-3"
      >
        <button
          type="button"
          onclick={() => importFlow.cancel()}
          class="rounded-(--radius-md) border border-(--color-border-strong) px-3 py-1.5 text-xs text-(--color-fg-default) transition-colors hover:bg-(--color-bg-overlay) active:scale-[0.98]"
        >
          {t("io.tsvCancel")}
        </button>
        <button
          type="button"
          onclick={() => importFlow.confirm()}
          disabled={pkg.busy}
          class="flex items-center gap-1.5 rounded-(--radius-md) bg-(--color-accent-500) px-3 py-1.5 text-xs font-medium text-(--color-fg-onAccent) shadow-(--shadow-subtle) transition-all hover:bg-(--color-accent-600) active:scale-[0.97] disabled:cursor-not-allowed disabled:opacity-50"
        >
          {#if pkg.busy}
            <Loader2 size={12} class="animate-spin" />
          {/if}
          {t("io.tsvImportConfirm")}
        </button>
      </footer>
    </div>
  </div>
{/if}
