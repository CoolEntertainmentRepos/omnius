<script lang="ts">
  import { goto } from "$app/navigation";
  import { favoritesStore } from "$lib/stores/favorites.svelte";
  import { configStore } from "$lib/stores/config.svelte";

  interface Props {
    activeNav?: string;
    onNavClick?: (nav: string) => void;
  }

  let { activeNav = "", onNavClick }: Props = $props();

  let showMovies = $derived(configStore.isEnabled("movies"));
  let showSeries = $derived(configStore.isEnabled("series"));
  let showLive = $derived(configStore.isEnabled("channels"));

  function handleClick(nav: string) {
    if (onNavClick) {
      onNavClick(nav);
    } else {
      // Default behavior: navigate to home with nav param
      if (nav === "settings") {
        goto("/settings");
      } else {
        goto(`/?nav=${nav}`);
      }
    }
  }
</script>

<nav class="sidebar">
  <div class="sidebar-top">
    <!-- Omnius Logo -->
    <div class="logo-container">
      <svg viewBox="0 0 7 7" fill="none" class="omnius-logo">
        <path d="M3.336 6.69596C2.696 6.69596 2.124 6.55196 1.62 6.26396C1.116 5.97596 0.72 5.57996 0.432 5.07596C0.144 4.57196 0 3.99996 0 3.35996C0 2.71196 0.144 2.13596 0.432 1.63196C0.72 1.12796 1.116 0.731963 1.62 0.443963C2.124 0.155963 2.696 0.0119629 3.336 0.0119629C3.976 0.0119629 4.544 0.155963 5.04 0.443963C5.544 0.731963 5.94 1.12796 6.228 1.63196C6.516 2.13596 6.664 2.71196 6.672 3.35996C6.672 3.99996 6.524 4.57196 6.228 5.07596C5.94 5.57996 5.544 5.97596 5.04 6.26396C4.544 6.55196 3.976 6.69596 3.336 6.69596ZM3.336 5.85596C3.8 5.85596 4.216 5.74796 4.584 5.53196C4.952 5.31596 5.24 5.01996 5.448 4.64396C5.656 4.26796 5.76 3.83996 5.76 3.35996C5.76 2.87996 5.656 2.45196 5.448 2.07596C5.24 1.69196 4.952 1.39196 4.584 1.17596C4.216 0.959963 3.8 0.851963 3.336 0.851963C2.872 0.851963 2.456 0.959963 2.088 1.17596C1.72 1.39196 1.428 1.69196 1.212 2.07596C1.004 2.45196 0.9 2.87996 0.9 3.35996C0.9 3.83996 1.004 4.26796 1.212 4.64396C1.428 5.01996 1.72 5.31596 2.088 5.53196C2.456 5.74796 2.872 5.85596 3.336 5.85596Z" fill="#FF2D55"/>
      </svg>
    </div>
    <button
      class="nav-item"
      class:active={activeNav === "search"}
      onclick={() => handleClick("search")}
      aria-label="Search"
    >
      <svg viewBox="0 0 24 24" fill="currentColor">
        <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
      </svg>
    </button>
    <button
      class="nav-item"
      class:active={activeNav === "home"}
      onclick={() => handleClick("home")}
      aria-label="Home"
      autofocus
    >
      <svg viewBox="0 0 24 24" fill="currentColor">
        <path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z"/>
      </svg>
    </button>
    {#if showMovies}
    <button
      class="nav-item"
      class:active={activeNav === "movies"}
      onclick={() => handleClick("movies")}
      aria-label="Movies"
    >
      <svg viewBox="0 0 24 24" fill="currentColor">
        <path d="M18 4l2 4h-3l-2-4h-2l2 4h-3l-2-4H8l2 4H7L5 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V4h-4z"/>
      </svg>
    </button>
    {/if}
    {#if showSeries}
    <button
      class="nav-item"
      class:active={activeNav === "tv"}
      onclick={() => handleClick("tv")}
      aria-label="TV Series"
    >
      <svg viewBox="0 0 24 24" fill="currentColor">
        <path d="M21 3H3c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h5v2h8v-2h5c1.1 0 1.99-.9 1.99-2L23 5c0-1.1-.9-2-2-2zm0 14H3V5h18v12z"/>
      </svg>
    </button>
    {/if}
    {#if showLive}
    <button
      class="nav-item"
      class:active={activeNav === "live"}
      onclick={() => handleClick("live")}
      aria-label="Live Channels"
    >
      <svg viewBox="0 0 24 24" fill="currentColor">
        <path d="M17 10.5V7c0-.55-.45-1-1-1H4c-.55 0-1 .45-1 1v10c0 .55.45 1 1 1h12c.55 0 1-.45 1-1v-3.5l4 4v-11l-4 4z"/>
      </svg>
    </button>
    {/if}
  </div>
  <div class="sidebar-bottom">
    <button
      class="nav-item"
      class:active={activeNav === "favorites"}
      onclick={() => handleClick("favorites")}
      aria-label="My List"
    >
      <svg viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/>
      </svg>
      {#if favoritesStore.totalCount > 0}
        <span class="nav-badge">{favoritesStore.totalCount}</span>
      {/if}
    </button>
    <button
      class="nav-item"
      class:active={activeNav === "settings"}
      onclick={() => handleClick("settings")}
      aria-label="Settings"
    >
      <svg viewBox="0 0 24 24" fill="currentColor">
        <path d="M19.14 12.94c.04-.31.06-.63.06-.94 0-.31-.02-.63-.06-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.04.31-.06.63-.06.94s.02.63.06.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z"/>
      </svg>
    </button>
  </div>
</nav>

<style>
  /* Sidebar - truly fixed, never scrolls */
  .sidebar {
    position: fixed;
    left: 0;
    top: 0;
    width: 70px;
    height: 100vh;
    height: 100dvh;
    background: rgba(0, 0, 0, 0.95);
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    padding: 20px 0;
    z-index: 1000;
    overflow: hidden;
    box-sizing: border-box;
  }

  .sidebar-top,
  .sidebar-bottom {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }

  .logo-container {
    width: 50px;
    height: 50px;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 16px;
  }

  .omnius-logo {
    width: 32px;
    height: 32px;
  }

  .nav-item {
    width: 50px;
    height: 50px;
    background: none;
    border: none;
    border-radius: 8px;
    color: #808080;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
    position: relative;
  }

  .nav-item::before {
    content: '';
    position: absolute;
    left: 0;
    top: 50%;
    transform: translateY(-50%);
    width: 4px;
    height: 0;
    background: #e50914;
    border-radius: 0 2px 2px 0;
    transition: height 0.2s ease;
  }

  .nav-item:hover {
    color: #fff;
    background: rgba(255, 255, 255, 0.1);
  }

  .nav-item:focus {
    outline: none;
    color: #fff;
    background: rgba(229, 9, 20, 0.3);
    box-shadow: inset 0 0 0 3px #e50914;
  }

  .nav-item:focus-visible {
    outline: none;
    color: #fff;
    background: rgba(229, 9, 20, 0.3);
    box-shadow: inset 0 0 0 3px #e50914;
  }

  .nav-item.active {
    color: #fff;
  }

  .nav-item.active::before {
    height: 24px;
  }

  .nav-item.active:focus {
    box-shadow: inset 0 0 0 3px #e50914;
  }

  .nav-item svg {
    width: 26px;
    height: 26px;
  }

  .nav-badge {
    position: absolute;
    top: 4px;
    right: 4px;
    background: #e50914;
    color: #fff;
    font-size: 0.7rem;
    font-weight: 600;
    min-width: 18px;
    height: 18px;
    border-radius: 9px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0 4px;
  }

  @media (max-width: 900px) {
    .sidebar {
      width: 60px;
    }

    .nav-item {
      width: 44px;
      height: 44px;
    }
  }

  @media (max-width: 600px) {
    .sidebar {
      display: none;
    }
  }
</style>
