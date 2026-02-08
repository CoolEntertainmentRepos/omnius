<script lang="ts">
  import { onMount, tick } from "svelte";
  import { goto } from "$app/navigation";
  import { browser } from "$app/environment";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import MovieGrid from "$lib/components/MovieGrid.svelte";
  import SearchSection from "$lib/components/SearchSection.svelte";
  import { listMovies, getCuratedLists, getCuratedList, type CuratedList } from "$lib/api/commands";
  import { makeFocusable, setFocus } from "$lib/utils/tvNavigation";
  import type { Movie } from "$lib/api/types";

  // Browse mode
  let movieBrowseMode = $state<'genre' | 'year' | 'curated' | 'search'>('curated');

  // Curated
  let curatedList = $state<string>('');
  let curatedMovies = $state<Movie[]>([]);
  let curatedLoading = $state(false);
  let curatedLists = $state<CuratedList[]>([]);
  let curatedListsLoading = $state(false);

  // Genre
  let selectedGenre = $state<string | null>(null);
  let genreMovies = $state<Movie[]>([]);
  let genreLoading = $state(false);
  let genrePage = $state(1);
  let genreTotal = $state(0);
  let genreLoadingMore = $state(false);

  // Year
  let selectedYear = $state<number | null>(null);
  let yearMovies = $state<Movie[]>([]);
  let yearLoading = $state(false);

  // Search
  let movieSearchQuery = $state('');
  let movieSearchResults = $state<Movie[]>([]);
  let movieSearchLoading = $state(false);

  const genres = [
    "Action", "Adventure", "Animation", "Biography", "Comedy", "Crime",
    "Documentary", "Drama", "Family", "Fantasy", "History", "Horror",
    "Music", "Mystery", "Romance", "Sci-Fi", "Sport", "Thriller", "War", "Western"
  ];

  const years = Array.from({ length: 30 }, (_, i) => new Date().getFullYear() - i);

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' || e.key === 'Backspace' || e.key === 'GoBack') {
      if ((e.target as HTMLElement).tagName === 'INPUT') return;

      // Genre selected → deselect
      if (movieBrowseMode === 'genre' && selectedGenre) {
        e.preventDefault();
        e.stopImmediatePropagation();
        console.log('[Movies] Back → deselect genre');
        selectedGenre = null;
        tick().then(() => setTimeout(() => { makeFocusable(); setFocus('.genre-card'); }, 100));
        return;
      }

      // Year selected → deselect
      if (movieBrowseMode === 'year' && selectedYear) {
        e.preventDefault();
        e.stopImmediatePropagation();
        console.log('[Movies] Back → deselect year');
        selectedYear = null;
        yearMovies = [];
        tick().then(() => setTimeout(() => { makeFocusable(); setFocus('.year-card'); }, 100));
        return;
      }

      // Non-curated browse mode → back to curated
      if (movieBrowseMode !== 'curated') {
        e.preventDefault();
        e.stopImmediatePropagation();
        console.log('[Movies] Back → switch to curated');
        switchMovieBrowseMode('curated');
        return;
      }

      // At top level → let layout handle (navigate to /)
      console.log('[Movies] Back → letting layout navigate to /');
    }
  }

  onMount(() => {
    loadCuratedLists();

    if (browser) {
      document.addEventListener('keydown', handleKeydown, true);
    }

    let focusTimer = setTimeout(() => {
      makeFocusable();
      setFocus('.curated-tab, .movie-card');
    }, 300);

    return () => {
      clearTimeout(focusTimer);
      if (browser) {
        document.removeEventListener('keydown', handleKeydown, true);
      }
    };
  });

  function switchMovieBrowseMode(mode: 'genre' | 'year' | 'curated' | 'search') {
    movieBrowseMode = mode;
    selectedYear = null;
    selectedGenre = null;
    if (mode === 'curated') {
      loadCuratedLists();
    }
    if (mode === 'search') {
      movieSearchQuery = '';
      movieSearchResults = [];
    }
  }

  async function loadCuratedLists() {
    if (curatedLists.length > 0) return;
    curatedListsLoading = true;
    try {
      curatedLists = await getCuratedLists();
      if (curatedLists.length > 0 && !curatedList) {
        handleCuratedSelect(curatedLists[0].slug);
      }
    } catch (err) {
      console.error('Failed to load curated lists:', err);
    } finally {
      curatedListsLoading = false;
    }
  }

  async function handleCuratedSelect(slug: string) {
    curatedList = slug;
    curatedLoading = true;
    try {
      const list = await getCuratedList(slug);
      curatedMovies = list?.movies || [];
    } catch (err) {
      console.error('Failed to load curated list:', err);
    } finally {
      curatedLoading = false;
      await tick();
      setTimeout(() => { makeFocusable(); setFocus('.movie-card'); }, 100);
    }
  }

  async function handleGenreSelect(genre: string) {
    selectedGenre = genre;
    genreLoading = true;
    genrePage = 1;
    try {
      const data = await listMovies({ genre, sort_by: "rating", limit: 20, page: 1 });
      genreMovies = data.movies || [];
      genreTotal = data.movie_count || 0;
    } catch {
      genreMovies = [];
      genreTotal = 0;
    } finally {
      genreLoading = false;
      await tick();
      setTimeout(() => { makeFocusable(); setFocus('.movie-card'); }, 100);
    }
  }

  async function loadMoreGenre() {
    if (genreLoadingMore || !selectedGenre) return;
    genreLoadingMore = true;
    genrePage += 1;
    try {
      const data = await listMovies({ genre: selectedGenre, sort_by: "rating", limit: 20, page: genrePage });
      genreMovies = [...genreMovies, ...(data.movies || [])];
    } catch {
      genrePage -= 1;
    } finally {
      genreLoadingMore = false;
    }
  }

  async function handleYearSelect(year: number) {
    selectedYear = year;
    yearLoading = true;
    try {
      const data = await listMovies({ year, sort_by: 'rating', limit: 50 });
      yearMovies = data.movies || [];
    } catch (err) {
      console.error('Failed to load year movies:', err);
    } finally {
      yearLoading = false;
      await tick();
      setTimeout(() => { makeFocusable(); setFocus('.movie-card'); }, 100);
    }
  }
</script>

<svelte:head>
  <title>Browse Movies - Omnius</title>
</svelte:head>

<div class="app">
  <Sidebar />

  <main class="main-content">
    <div class="category-view">
      <div class="movies-header">
        <h1 class="category-title">Browse Movies</h1>
        <div class="group-toggle">
          <button
            class="toggle-btn"
            class:active={movieBrowseMode === 'curated'}
            onclick={() => switchMovieBrowseMode('curated')}
          >
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z"/>
            </svg>
            Curated
          </button>
          <button
            class="toggle-btn"
            class:active={movieBrowseMode === 'genre'}
            onclick={() => switchMovieBrowseMode('genre')}
          >
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M4 8h4V4H4v4zm6 12h4v-4h-4v4zm-6 0h4v-4H4v4zm0-6h4v-4H4v4zm6 0h4v-4h-4v4zm6-10v4h4V4h-4zm-6 4h4V4h-4v4zm6 6h4v-4h-4v4zm0 6h4v-4h-4v4z"/>
            </svg>
            Genre
          </button>
          <button
            class="toggle-btn"
            class:active={movieBrowseMode === 'year'}
            onclick={() => switchMovieBrowseMode('year')}
          >
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M19 3h-1V1h-2v2H8V1H6v2H5c-1.11 0-1.99.9-1.99 2L3 19c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H5V8h14v11zM9 10H7v2h2v-2zm4 0h-2v2h2v-2zm4 0h-2v2h2v-2zm-8 4H7v2h2v-2zm4 0h-2v2h2v-2zm4 0h-2v2h2v-2z"/>
            </svg>
            Year
          </button>
          <button
            class="toggle-btn"
            class:active={movieBrowseMode === 'search'}
            onclick={() => switchMovieBrowseMode('search')}
          >
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
            </svg>
            Search
          </button>
        </div>
      </div>

      {#if movieBrowseMode === 'curated'}
        {#if curatedListsLoading}
          <div class="loading-spinner"><div class="spinner"></div></div>
        {:else if curatedLists.length === 0}
          <p style="color: #888; text-align: center;">No curated lists available</p>
        {:else}
          <div class="curated-tabs">
            {#each curatedLists as list (list.slug)}
              <button
                class="curated-tab"
                class:active={curatedList === list.slug}
                onclick={() => handleCuratedSelect(list.slug)}
              >
                {list.name}
              </button>
            {/each}
          </div>
          {#if curatedLoading}
            <div class="loading-spinner"><div class="spinner"></div></div>
          {:else}
            <MovieGrid movies={curatedMovies} loading={false} error={null} />
          {/if}
        {/if}
      {:else if movieBrowseMode === 'genre'}
        {#if selectedGenre}
          <div class="genre-header">
            <button class="back-btn" onclick={async () => { selectedGenre = null; await tick(); setTimeout(() => { makeFocusable(); setFocus('.genre-card'); }, 100); }} aria-label="Go back">
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
              </svg>
            </button>
            <h2>{selectedGenre} Movies</h2>
          </div>
          <MovieGrid
            movies={genreMovies}
            loading={genreLoading}
            error={null}
            hasMore={genreMovies.length < genreTotal}
            loadingMore={genreLoadingMore}
            onLoadMore={loadMoreGenre}
          />
        {:else}
          <div class="genre-grid">
            {#each genres as genre (genre)}
              <button class="genre-card" onclick={() => handleGenreSelect(genre)}>
                {genre}
              </button>
            {/each}
          </div>
        {/if}
      {:else if movieBrowseMode === 'year'}
        {#if selectedYear}
          <div class="genre-header">
            <button class="back-btn" onclick={async () => { selectedYear = null; yearMovies = []; await tick(); setTimeout(() => { makeFocusable(); setFocus('.year-card'); }, 100); }} aria-label="Go back">
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
              </svg>
            </button>
            <h2>{selectedYear} Movies</h2>
          </div>
          {#if yearLoading}
            <div class="loading-spinner"><div class="spinner"></div></div>
          {:else}
            <MovieGrid movies={yearMovies} loading={false} error={null} />
          {/if}
        {:else}
          <div class="year-grid">
            {#each years as year (year)}
              <button class="year-card" onclick={() => handleYearSelect(year)}>
                {year}
              </button>
            {/each}
          </div>
        {/if}
      {:else if movieBrowseMode === 'search'}
        <SearchSection type="movies" />
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

  .movies-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
    flex-wrap: wrap;
    gap: 10px;
  }

  .category-title {
    font-size: 1.3rem;
    font-weight: 600;
    margin: 0;
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

  .toggle-btn.active {
    color: #fff;
    background: #e50914;
  }

  .toggle-btn:focus,
  .toggle-btn:focus-visible {
    outline: none;
    box-shadow: 0 0 0 3px rgba(229, 9, 20, 0.5);
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

  .curated-tab.active {
    background: #e50914;
    border-color: #e50914;
    color: #fff;
  }

  .curated-tab:focus,
  .curated-tab:focus-visible {
    outline: none;
    box-shadow: 0 0 0 3px rgba(229, 9, 20, 0.5);
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
    transform: scale(1.02);
  }

  .genre-card:focus,
  .genre-card:focus-visible {
    outline: none;
    border-color: #e50914;
    box-shadow: 0 0 0 3px rgba(229, 9, 20, 0.5);
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

  .year-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
    gap: 12px;
  }

  .year-card {
    padding: 10px 12px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 8px;
    color: #fff;
    font-size: 0.9rem;
    font-weight: 600;
    font-family: inherit;
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: center;
  }

  .year-card:hover {
    background: rgba(229, 9, 20, 0.3);
    border-color: rgba(229, 9, 20, 0.5);
    transform: scale(1.05);
  }

  .year-card:focus,
  .year-card:focus-visible {
    outline: none;
    border-color: #e50914;
    box-shadow: 0 0 0 3px rgba(229, 9, 20, 0.5);
  }

  .loading-spinner {
    display: flex;
    justify-content: center;
    padding: 30px 0;
  }

  .spinner {
    width: 60px;
    height: 60px;
    border: 4px solid rgba(229, 9, 20, 0.2);
    border-top-color: #e50914;
    border-radius: 50%;
    animation: spin 1s linear infinite;
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
