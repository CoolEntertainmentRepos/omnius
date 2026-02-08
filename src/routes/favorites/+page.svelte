<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import MovieGrid from "$lib/components/MovieGrid.svelte";
  import SeriesRow from "$lib/components/SeriesRow.svelte";
  import ChannelRow from "$lib/components/ChannelRow.svelte";
  import { favoritesStore } from "$lib/stores/favorites.svelte";
  import { makeFocusable } from "$lib/utils/tvNavigation";

  let totalCount = $derived(favoritesStore.totalCount);

  onMount(() => {
    setTimeout(() => {
      makeFocusable();
    }, 200);
  });
</script>

<svelte:head>
  <title>My List - Omnius</title>
</svelte:head>

<div class="app">
  <Sidebar />

  <main class="main-content">
    <div class="favorites-header">
      <h1 class="category-title">My List</h1>
      {#if totalCount > 0}
        <span class="favorites-count">{totalCount} item{totalCount !== 1 ? 's' : ''}</span>
      {/if}
    </div>

    {#if totalCount === 0}
      <div class="empty-state">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z" />
        </svg>
        <h2>Your list is empty</h2>
        <p>Add movies, shows, and channels to your list to watch them later.</p>
      </div>
    {:else}
      <div class="category-view">
        {#if favoritesStore.favorites.length > 0}
          <MovieGrid movies={favoritesStore.favorites} />
        {/if}

        {#if favoritesStore.favoriteSeries.length > 0}
          <SeriesRow title="My Series" series={favoritesStore.favoriteSeries} />
        {/if}

        {#if favoritesStore.favoriteChannels.length > 0}
          <ChannelRow title="My Channels" channels={favoritesStore.favoriteChannels} />
        {/if}

        {#if favoritesStore.favoriteCountries.length > 0}
          <div class="fav-countries">
            <h3 class="row-title">My Countries</h3>
            <div class="country-list">
              {#each favoritesStore.favoriteCountries as country (country.code)}
                <div class="country-list-item">
                  <button
                    class="country-list-btn focusable"
                    onclick={() => goto('/live')}
                  >
                    <span class="country-name">
                      {#if country.flag}
                        <span>{country.flag}</span>
                      {/if}
                      <span>{country.name}</span>
                    </span>
                  </button>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </main>
</div>

<style>
  .app {
    display: flex;
    min-height: 100vh;
    background: #141414;
  }

  .main-content {
    flex: 1;
    margin-left: 70px;
    min-height: 100vh;
    overflow-x: hidden;
    padding-bottom: 40px;
  }

  .category-view {
    padding: 20px 0 0;
  }

  .category-title {
    font-size: 1.3rem;
    font-weight: 600;
    margin: 0;
    padding: 0 40px;
    color: #fff;
  }

  .favorites-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 20px 40px 16px;
  }

  .favorites-count {
    font-size: 0.85rem;
    color: #888;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 40vh;
    color: #888;
    text-align: center;
    padding: 40px;
  }

  .empty-state svg {
    width: 64px;
    height: 64px;
    color: #333;
    margin-bottom: 16px;
  }

  .empty-state h2 {
    margin: 0 0 8px;
    color: #fff;
  }

  .empty-state p {
    margin: 0;
    font-size: 0.9rem;
  }

  .fav-countries {
    padding: 0 0 20px;
  }

  .row-title {
    font-size: 1.1rem;
    font-weight: 600;
    margin: 0 0 8px;
    padding: 0 40px;
    color: #fff;
  }

  .country-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 0 40px;
  }

  .country-list-item {
    display: flex;
    align-items: center;
  }

  .country-list-btn {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 16px;
    background: rgba(255, 255, 255, 0.05);
    border: none;
    border-radius: 6px;
    color: #e0e0e0;
    font-size: 0.85rem;
    cursor: pointer;
    transition: background 0.2s;
  }

  .country-list-btn:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .country-list-btn:focus {
    outline: none;
    box-shadow: 0 0 0 2px #2ecc71;
    background: rgba(46, 204, 113, 0.1);
  }

  .country-name {
    display: flex;
    align-items: center;
    gap: 8px;
  }
</style>
