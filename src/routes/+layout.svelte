<script lang="ts">
  import { onMount } from "svelte";
  import UpdateChecker from "$lib/components/UpdateChecker.svelte";
  import IntroSplash from "$lib/components/IntroSplash.svelte";
  import { browser } from "$app/environment";
  import { goto } from "$app/navigation";

  let { children } = $props();

  let showIntro = $state(true);
  let introCompleted = $state(false);

  onMount(async () => {
    // Check if intro was already shown this session
    if (browser && sessionStorage.getItem("introShown")) {
      showIntro = false;
      introCompleted = true;
    }

    if (browser) {
      document.addEventListener('keydown', handleBackButton);
    }

    return () => {
      if (browser) {
        document.removeEventListener('keydown', handleBackButton);
      }
    };
  });

  function handleBackButton(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      const target = e.target as HTMLElement;
      if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') {
        return;
      }

      e.preventDefault();

      if (window.history.length > 1 && document.referrer) {
        window.history.back();
      } else if (window.location.pathname !== '/') {
        goto('/');
      }
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

  :global(*:focus-visible) {
    outline: 2px solid rgba(229, 9, 20, 0.6);
    outline-offset: 2px;
  }

  :global(button:focus-visible, a:focus-visible) {
    outline: 2px solid #e50914;
    outline-offset: 2px;
  }

  :global(.movie-card:focus-visible, .movie-card:focus) {
    outline: none;
  }
</style>
