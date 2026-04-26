<script lang="ts">
  import { fade } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import { page } from "$app/stores";

  let { children } = $props();
  const key = $derived($page.url.pathname);
</script>

{#key key}
  <div
    class="h-full"
    in:fade={{ duration: 180, easing: cubicOut, delay: 60 }}
    out:fade={{ duration: 80, easing: cubicOut }}
  >
    {@render children()}
  </div>
{/key}
