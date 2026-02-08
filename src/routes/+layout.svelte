<script lang="ts">
  import { onMount } from "svelte";
  import UpdateChecker from "$lib/components/UpdateChecker.svelte";
  import IntroSplash from "$lib/components/IntroSplash.svelte";
  import { browser } from "$app/environment";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { initSpatialNavigation } from "$lib/utils/tvNavigation";
  import { configStore } from "$lib/stores/config.svelte";

  let { children } = $props();

  let showIntro = $state(false);
  let introCompleted = $state(true);

  onMount(() => {
    console.log('[Layout] onMount called, browser:', browser);

    // Check if intro was already shown this session
    if (browser && sessionStorage.getItem("introShown")) {
      showIntro = false;
      introCompleted = true;
    }

    // Initialize spatial navigation for TV remote/D-pad
    if (browser) {
      console.log('[Layout] Calling initSpatialNavigation...');
      initSpatialNavigation()
        .then(() => console.log('[Layout] initSpatialNavigation completed'))
        .catch((error) => console.error('[Layout] initSpatialNavigation error:', error));

      // Fetch server config (enabled services) - default URL is already set
      configStore.fetchConfig();

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
      const target = e.target as HTMLElement;
      const path = $page.url.pathname;
      const targetTag = target.tagName;
      const targetClass = target.className?.toString?.().slice(0, 60) || '';
      const targetLabel = target.getAttribute('aria-label') || '';

      console.log(`[Back] key=${e.key} path=${path} target=<${targetTag} class="${targetClass}" aria-label="${targetLabel}">`);
      console.log(`[Back] defaultPrevented=${e.defaultPrevented} phase=${e.eventPhase}`);

      // Don't handle if we're in an input field
      if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') {
        console.log('[Back] SKIP: input field');
        return;
      }

      // If a modal/overlay is open, close it instead of navigating
      const modal = document.querySelector('.modal-overlay, .context-menu-overlay');
      if (modal) {
        e.preventDefault();
        e.stopPropagation();
        console.log('[Back] Modal detected → closing it');
        // Try clicking the close button inside the modal
        const closeBtn = modal.querySelector('.modal-close, [aria-label="Close"]') as HTMLElement;
        if (closeBtn) {
          closeBtn.click();
        } else {
          // Click the overlay itself to dismiss
          (modal as HTMLElement).click();
        }
        return;
      }

      // If the event was already handled by a page-level handler, skip
      if (e.defaultPrevented) {
        console.log('[Back] SKIP: already handled by page');
        return;
      }

      e.preventDefault();
      e.stopPropagation();

      // Deterministic back navigation — no history.back()
      if (path === '/') {
        // At root: if focus is in content, retreat to sidebar
        const inSidebar = target.closest('.sidebar');
        if (!inSidebar) {
          console.log('[Back] At root → focus sidebar Home');
          const homeBtn = document.querySelector('.sidebar .nav-item[aria-label="Home"]') as HTMLElement;
          if (homeBtn) {
            homeBtn.focus();
          }
        } else {
          console.log('[Back] At root + sidebar → do nothing');
        }
        return;
      }

      // Compute parent: /movies/42/play/abc → /movies/42
      const segments = path.split('/').filter(Boolean);
      segments.pop();
      const parent = segments.length > 0 ? '/' + segments.join('/') : '/';
      console.log(`[Back] NAVIGATE: ${path} → ${parent}`);
      goto(parent, { replaceState: true });
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
