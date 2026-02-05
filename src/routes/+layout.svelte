<script lang="ts">
  import { onMount } from "svelte";
  import UpdateChecker from "$lib/components/UpdateChecker.svelte";
  import IntroSplash from "$lib/components/IntroSplash.svelte";
  import { browser } from "$app/environment";
  import { goto } from "$app/navigation";
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

      // Handle Android TV back button
      document.addEventListener('keydown', handleBackButton);
    }

    return () => {
      if (browser) {
        document.removeEventListener('keydown', handleBackButton);
      }
    };
  });

  function handleBackButton(e: KeyboardEvent) {
    // Android TV back button sends Escape or Backspace
    if (e.key === 'Escape' || e.key === 'Backspace' || e.key === 'GoBack') {
      // Don't handle if we're in an input field
      const target = e.target as HTMLElement;
      if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') {
        return;
      }

      e.preventDefault();
      e.stopPropagation();

      // Check if we can go back in history
      if (window.history.length > 1 && document.referrer) {
        window.history.back();
      } else if (window.location.pathname !== '/') {
        // Navigate to home if we're not already there
        goto('/');
      }
      // If we're at home with no history, do nothing (don't close app)
    }
  }

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
    font-family: 'JetBrains Mono', monospace;
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

  /* Global TV focus styles - exclude movie cards which have their own styles */
  :global(*:focus-visible:not(.movie-card)) {
    outline: 3px solid #e50914;
    outline-offset: 2px;
  }

  :global(.movie-card:focus-visible) {
    outline: none !important;
  }
</style>
