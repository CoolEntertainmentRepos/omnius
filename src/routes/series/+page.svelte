<script lang="ts">
  import { onMount, tick } from "svelte";
  import { goto } from "$app/navigation";
  import { browser } from "$app/environment";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import SeriesGrid from "$lib/components/SeriesGrid.svelte";
  import SearchSection from "$lib/components/SearchSection.svelte";
  import { listSeries } from "$lib/api/commands";
  import { makeFocusable, setFocus } from "$lib/utils/tvNavigation";
  import type { Series } from "$lib/api/types";

  // Browse mode
  let tvBrowseMode = $state<'curated' | 'genre' | 'network' | 'search'>('curated');

  // Main data
  let tvSeries = $state<Series[]>([]);
  let tvSeriesTopRated = $state<Series[]>([]);
  let tvSeriesOngoing = $state<Series[]>([]);
  let tvSeriesLoading = $state(false);
  let tvSeriesPage = $state(1);
  let tvSeriesTotal = $state(0);
  let tvSeriesLoadingMore = $state(false);

  // Curated
  let tvCuratedList = $state<string>('top');
  let tvCuratedSeries = $state<Series[]>([]);
  let tvCuratedLoading = $state(false);

  // Genre
  let tvSelectedGenre = $state<string | null>(null);
  let tvGenreSeries = $state<Series[]>([]);
  let tvGenreLoading = $state(false);

  // Network
  let tvSelectedNetwork = $state<string | null>(null);
  let tvNetworkSeries = $state<Series[]>([]);
  let tvNetworkLoading = $state(false);

  const tvCuratedLists = [
    { id: 'top', name: 'Top Rated', filter: { sort_by: 'rating', order_by: 'desc', minimum_rating: 8.5 } },
    { id: 'popular', name: 'Most Popular', filter: { sort_by: 'date_added', order_by: 'desc' } },
    { id: 'ongoing', name: 'Ongoing', filter: { status: 'Continuing' } },
    { id: 'classics', name: 'Classics', filter: { sort_by: 'rating', order_by: 'desc', maximum_year: 2010 } },
  ];

  const tvGenres = ['Drama', 'Comedy', 'Action', 'Adventure', 'Crime', 'Thriller', 'Sci-Fi', 'Fantasy', 'Horror', 'Mystery', 'Animation', 'Documentary', 'Romance', 'Biography', 'History', 'War'];
  const tvNetworks = ['HBO', 'Netflix', 'AMC', 'BBC', 'NBC', 'CBS', 'ABC', 'FOX', 'Disney+', 'Showtime', 'Comedy Central', 'Adult Swim', 'The CW', 'PBS', 'Nickelodeon'];

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' || e.key === 'Backspace' || e.key === 'GoBack') {
      if ((e.target as HTMLElement).tagName === 'INPUT') return;

      // Genre selected → deselect
      if (tvBrowseMode === 'genre' && tvSelectedGenre) {
        e.preventDefault();
        e.stopImmediatePropagation();
        console.log('[Series] Back → deselect genre');
        tvSelectedGenre = null;
        tvGenreSeries = [];
        return;
      }

      // Network selected → deselect
      if (tvBrowseMode === 'network' && tvSelectedNetwork) {
        e.preventDefault();
        e.stopImmediatePropagation();
        console.log('[Series] Back → deselect network');
        tvSelectedNetwork = null;
        tvNetworkSeries = [];
        return;
      }

      // Non-curated browse mode → back to curated
      if (tvBrowseMode !== 'curated') {
        e.preventDefault();
        e.stopImmediatePropagation();
        console.log('[Series] Back → switch to curated');
        switchTVBrowseMode('curated');
        return;
      }

      // At top level → let layout handle (navigate to /)
      console.log('[Series] Back → letting layout navigate to /');
    }
  }

  onMount(() => {
    loadTVSeries();

    if (browser) {
      document.addEventListener('keydown', handleKeydown, true);
    }

    let focusTimer = setTimeout(() => {
      makeFocusable();
      setFocus('.toggle-btn.tv-toggle, .series-card, .movie-card');
    }, 300);

    return () => {
      clearTimeout(focusTimer);
      if (browser) {
        document.removeEventListener('keydown', handleKeydown, true);
      }
    };
  });

  async function loadTVSeries() {
    if (tvSeries.length > 0) return;
    tvSeriesLoading = true;
    try {
      const [allData, topData, ongoingData] = await Promise.all([
        listSeries({ limit: 50, sort_by: 'date_added', order_by: 'desc' }),
        listSeries({ limit: 20, sort_by: 'rating', order_by: 'desc', minimum_rating: 8 }),
        listSeries({ limit: 20, status: 'Continuing', sort_by: 'rating', order_by: 'desc' }),
      ]);
      tvSeries = allData.series || [];
      tvSeriesTotal = allData.series_count || 0;
      tvSeriesTopRated = topData.series || [];
      tvSeriesOngoing = ongoingData.series || [];
    } catch (err) {
      console.error('[TV] Failed to load series:', err);
      tvSeries = [];
    } finally {
      tvSeriesLoading = false;
    }
  }

  async function loadMoreTVSeries() {
    if (tvSeriesLoadingMore) return;
    tvSeriesLoadingMore = true;
    tvSeriesPage += 1;
    try {
      const data = await listSeries({ limit: 50, page: tvSeriesPage, sort_by: 'date_added', order_by: 'desc' });
      tvSeries = [...tvSeries, ...(data.series || [])];
    } catch {
      tvSeriesPage -= 1;
    } finally {
      tvSeriesLoadingMore = false;
    }
  }

  function switchTVBrowseMode(mode: 'curated' | 'genre' | 'network' | 'search') {
    tvBrowseMode = mode;
  }

  async function handleTVCuratedSelect(listId: string) {
    tvCuratedList = listId;
    tvCuratedLoading = true;
    tvCuratedSeries = [];
    const list = tvCuratedLists.find(l => l.id === listId);
    if (!list) {
      tvCuratedLoading = false;
      return;
    }
    try {
      const data = await listSeries({ limit: 50, ...list.filter });
      tvCuratedSeries = data.series || [];
    } catch (err) {
      console.error('Failed to load curated series:', err);
    } finally {
      tvCuratedLoading = false;
    }
  }

  async function handleTVGenreSelect(genre: string) {
    tvSelectedGenre = genre;
    tvGenreLoading = true;
    tvGenreSeries = [];
    try {
      const data = await listSeries({ limit: 50, genre: genre });
      tvGenreSeries = data.series || [];
    } catch (err) {
      console.error('Failed to load genre series:', err);
    } finally {
      tvGenreLoading = false;
    }
  }

  async function handleTVNetworkSelect(network: string) {
    tvSelectedNetwork = network;
    tvNetworkLoading = true;
    tvNetworkSeries = [];
    try {
      const data = await listSeries({ limit: 50, network: network });
      tvNetworkSeries = data.series || [];
    } catch (err) {
      console.error('Failed to load network series:', err);
    } finally {
      tvNetworkLoading = false;
    }
  }
</script>

<svelte:head>
  <title>Browse TV Series - Omnius</title>
</svelte:head>

<div class="app">
  <Sidebar />

  <main class="main-content">
    <div class="category-view">
      <div class="tv-header">
        <h1 class="category-title">Browse TV Series</h1>
        <div class="group-toggle">
          <button
            class="toggle-btn tv-toggle"
            class:active={tvBrowseMode === 'curated'}
            onclick={() => switchTVBrowseMode('curated')}
          >
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z"/>
            </svg>
            Curated
          </button>
          <button
            class="toggle-btn tv-toggle"
            class:active={tvBrowseMode === 'genre'}
            onclick={() => switchTVBrowseMode('genre')}
          >
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M4 8h4V4H4v4zm6 12h4v-4h-4v4zm-6 0h4v-4H4v4zm0-6h4v-4H4v4zm6 0h4v-4h-4v4zm6-10v4h4V4h-4zm-6 4h4V4h-4v4zm6 6h4v-4h-4v4zm0 6h4v-4h-4v4z"/>
            </svg>
            Genre
          </button>
          <button
            class="toggle-btn tv-toggle"
            class:active={tvBrowseMode === 'network'}
            onclick={() => switchTVBrowseMode('network')}
          >
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M17 10.5V7c0-.55-.45-1-1-1H4c-.55 0-1 .45-1 1v10c0 .55.45 1 1 1h12c.55 0 1-.45 1-1v-3.5l4 4v-11l-4 4z"/>
            </svg>
            Network
          </button>
          <button
            class="toggle-btn tv-toggle"
            class:active={tvBrowseMode === 'search'}
            onclick={() => switchTVBrowseMode('search')}
          >
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
            </svg>
            Search
          </button>
        </div>
      </div>

      {#if tvBrowseMode === 'curated'}
        <div class="curated-tabs tv-curated">
          {#each tvCuratedLists as list (list.id)}
            <button
              class="curated-tab tv-tab"
              class:active={tvCuratedList === list.id}
              onclick={() => handleTVCuratedSelect(list.id)}
            >
              {list.name}
            </button>
          {/each}
        </div>
        {#if tvCuratedLoading}
          <div class="loading-spinner"><div class="spinner tv-spinner"></div></div>
        {:else if tvCuratedSeries.length > 0}
          <SeriesGrid series={tvCuratedSeries} loading={false} error={null} />
        {:else if tvSeriesLoading}
          <div class="loading-inline">
            <div class="spinner tv-spinner"></div>
            <p>Loading series...</p>
          </div>
        {:else}
          <SeriesGrid
            series={tvCuratedList === 'top' ? tvSeriesTopRated : tvCuratedList === 'ongoing' ? tvSeriesOngoing : tvSeries}
            loading={false}
            error={null}
            hasMore={tvCuratedList !== 'top' && tvCuratedList !== 'ongoing' && tvSeries.length < tvSeriesTotal}
            loadingMore={tvSeriesLoadingMore}
            onLoadMore={tvCuratedList !== 'top' && tvCuratedList !== 'ongoing' ? loadMoreTVSeries : undefined}
          />
        {/if}
      {:else if tvBrowseMode === 'genre'}
        {#if tvSelectedGenre}
          <div class="genre-header">
            <button class="back-btn" onclick={() => { tvSelectedGenre = null; tvGenreSeries = []; }} aria-label="Go back">
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
              </svg>
            </button>
            <h2>{tvSelectedGenre} Series</h2>
          </div>
          {#if tvGenreLoading}
            <div class="loading-spinner"><div class="spinner tv-spinner"></div></div>
          {:else}
            <SeriesGrid series={tvGenreSeries} loading={false} error={null} />
          {/if}
        {:else}
          <div class="genre-grid tv-genre-grid">
            {#each tvGenres as genre (genre)}
              <button class="genre-card tv-genre-card" onclick={() => handleTVGenreSelect(genre)}>
                {genre}
              </button>
            {/each}
          </div>
        {/if}
      {:else if tvBrowseMode === 'network'}
        {#if tvSelectedNetwork}
          <div class="genre-header">
            <button class="back-btn" onclick={() => { tvSelectedNetwork = null; tvNetworkSeries = []; }} aria-label="Go back">
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
              </svg>
            </button>
            <h2>{tvSelectedNetwork} Series</h2>
          </div>
          {#if tvNetworkLoading}
            <div class="loading-spinner"><div class="spinner tv-spinner"></div></div>
          {:else}
            <SeriesGrid series={tvNetworkSeries} loading={false} error={null} />
          {/if}
        {:else}
          <div class="network-grid">
            {#each tvNetworks as network (network)}
              <button class="network-card" onclick={() => handleTVNetworkSelect(network)}>
                {network}
              </button>
            {/each}
          </div>
        {/if}
      {:else if tvBrowseMode === 'search'}
        <SearchSection type="tvshows" />
      {/if}
    </div>
  </main>
</div>

<style>
  .app {
    display: flex;
    height: 100vh;
    height: 100dvh;
    background: #141414;
    color: #fff;
    overflow: hidden;
  }

  .main-content {
    flex: 1;
    margin-left: 70px;
    height: 100vh;
    height: 100dvh;
    overflow-y: auto;
    overflow-x: hidden;
    scroll-behavior: smooth;
  }

  .category-view {
    padding: 20px 40px;
  }

  .tv-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
    flex-wrap: wrap;
    gap: 10px;
  }

  .tv-header .category-title {
    margin: 0;
  }

  .category-title {
    font-size: 1.3rem;
    font-weight: 600;
    margin: 0 0 16px;
  }

  .group-toggle {
    display: flex;
    gap: 8px;
    background: rgba(255, 255, 255, 0.05);
    padding: 4px;
    border-radius: 8px;
  }

  .toggle-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background: transparent;
    border: none;
    border-radius: 6px;
    color: #888;
    font-size: 0.8rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .toggle-btn svg {
    width: 18px;
    height: 18px;
  }

  .toggle-btn:hover {
    color: #fff;
    background: rgba(255, 255, 255, 0.1);
  }

  .toggle-btn.tv-toggle.active {
    color: #fff;
    background: #2ecc71;
  }

  .toggle-btn:focus,
  .toggle-btn:focus-visible {
    outline: none;
  }

  .toggle-btn.tv-toggle:focus,
  .toggle-btn.tv-toggle:focus-visible {
    box-shadow: 0 0 0 3px rgba(46, 204, 113, 0.5);
  }

  .curated-tabs {
    display: flex;
    gap: 8px;
    margin-bottom: 12px;
    flex-wrap: wrap;
  }

  .curated-tab {
    padding: 6px 14px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 20px;
    color: #aaa;
    font-size: 0.8rem;
    font-weight: 500;
    font-family: inherit;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .curated-tab:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #fff;
  }

  .curated-tab.tv-tab.active {
    color: #fff;
    background: #2ecc71;
    border-color: #2ecc71;
  }

  .curated-tab.tv-tab:focus,
  .curated-tab.tv-tab:focus-visible {
    outline: none;
    box-shadow: 0 0 0 3px rgba(46, 204, 113, 0.5);
    border-color: #2ecc71;
  }

  .genre-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: 10px;
  }

  .genre-card {
    padding: 14px 12px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    color: #e0e0e0;
    font-size: 0.85rem;
    font-weight: 500;
    font-family: inherit;
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: center;
  }

  .genre-card:hover {
    background: rgba(255, 255, 255, 0.12);
    border-color: rgba(255, 255, 255, 0.2);
  }

  .genre-card.tv-genre-card:focus,
  .genre-card.tv-genre-card:focus-visible {
    outline: none;
    border-color: #2ecc71;
    box-shadow: 0 0 0 3px rgba(46, 204, 113, 0.5);
    background: rgba(255, 255, 255, 0.12);
    transform: scale(1.05);
  }

  .genre-header {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 16px;
  }

  .genre-header h2 {
    font-size: 1.3rem;
    font-weight: 600;
    margin: 0;
  }

  .back-btn {
    width: 44px;
    height: 44px;
    background: rgba(255, 255, 255, 0.1);
    border: none;
    border-radius: 50%;
    color: #fff;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.2s ease;
  }

  .back-btn:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  .back-btn:focus {
    outline: none;
    box-shadow: 0 0 0 3px rgba(255, 255, 255, 0.2);
  }

  .back-btn svg {
    width: 24px;
    height: 24px;
  }

  .network-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: 10px;
    margin-top: 12px;
  }

  .network-card {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 12px 10px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    color: #fff;
    font-size: 0.85rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .network-card:hover {
    background: rgba(46, 204, 113, 0.15);
    border-color: #2ecc71;
  }

  .network-card:focus,
  .network-card:focus-visible {
    outline: none;
    border-color: #2ecc71;
    box-shadow: 0 0 0 3px rgba(46, 204, 113, 0.5);
  }

  .loading-spinner {
    display: flex;
    justify-content: center;
    padding: 30px 0;
  }

  .loading-inline {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 30px 20px;
    color: #888;
  }

  .spinner {
    width: 60px;
    height: 60px;
    border: 4px solid rgba(229, 9, 20, 0.2);
    border-top-color: #e50914;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  .spinner.tv-spinner {
    border-color: rgba(46, 204, 113, 0.2);
    border-top-color: #2ecc71;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  @media (max-width: 900px) {
    .main-content {
      margin-left: 60px;
    }
    .category-view {
      padding: 16px 32px;
    }
  }

  @media (max-width: 600px) {
    .main-content {
      margin-left: 0;
    }
    .category-view {
      padding: 16px 20px;
    }
    .genre-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }
</style>
