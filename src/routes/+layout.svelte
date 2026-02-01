<script lang="ts">
  import { onMount } from "svelte";
  import UpdateChecker from "$lib/components/UpdateChecker.svelte";
  import IntroSplash from "$lib/components/IntroSplash.svelte";
  import { browser } from "$app/environment";
  import { initSpatialNavigation } from "$lib/utils/tvNavigation";

  let { children } = $props();

  let showIntro = $state(true);
  let introCompleted = $state(false);

  onMount(async () => {
    console.log('[Layout] onMount called, browser:', browser);

    // Check if intro was already shown this session
    if (browser && sessionStorage.getItem("introShown")) {
      showIntro = false;
      introCompleted = true;
    }

    // Initialize spatial navigation for TV remote/D-pad
    if (browser) {
      console.log('[Layout] Calling initSpatialNavigation...');
      try {
        await initSpatialNavigation();
        console.log('[Layout] initSpatialNavigation completed');
      } catch (error) {
        console.error('[Layout] initSpatialNavigation error:', error);
      }
    }
  });

  function handleIntroComplete() {
    showIntro = false;
    introCompleted = true;
    if (browser) {
      sessionStorage.setItem("introShown", "true");
    }
  }
</script>

{#if showIntro && browser}
  <IntroSplash onComplete={handleIntroComplete} />
{/if}

<div class:hidden={showIntro}>
  {@render children()}
</div>

{#if browser && introCompleted}
  <UpdateChecker />
{/if}

<style>
  .hidden {
    display: none;
  }

  :global(html, body) {
    margin: 0;
    padding: 0;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu, sans-serif;
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
