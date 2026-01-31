<script lang="ts">
  import UpdateChecker from "$lib/components/UpdateChecker.svelte";
  import { browser } from "$app/environment";

  let { children } = $props();
</script>

{@render children()}

{#if browser}
  <UpdateChecker />
{/if}

<style>
  :global(html, body) {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu, sans-serif;
    background: #141414;
    color: #fff;
    -webkit-font-smoothing: antialiased;
  }

  :global(*) {
    box-sizing: border-box;
  }

  :global(a) {
    color: inherit;
    text-decoration: none;
  }

  /* Global TV focus styles */
  :global(*:focus-visible) {
    outline: 3px solid #e50914;
    outline-offset: 2px;
  }
</style>
