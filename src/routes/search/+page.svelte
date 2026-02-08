<script lang="ts">
  import { onMount, tick } from "svelte";
  import { goto } from "$app/navigation";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import MovieGrid from "$lib/components/MovieGrid.svelte";
  import { listMovies } from "$lib/api/commands";
  import { makeFocusable, setFocus } from "$lib/utils/tvNavigation";
  import type { Movie } from "$lib/api/types";

  let searchQuery = $state("");
  let searchResults = $state<Movie[]>([]);
  let searchLoading = $state(false);
  let searchError = $state<string | null>(null);
  let searchDebounceTimer: ReturnType<typeof setTimeout>;

  let selectedGenre = $state<string | null>(null);
  let genreMovies = $state<Movie[]>([]);
  let genreLoading = $state(false);
  let genrePage = $state(1);
  let genreTotal = $state(0);
  let genreLoadingMore = $state(false);

  let searchPage = $state(1);
  let searchTotal = $state(0);
  let searchLoadingMore = $state(false);

  const genres = [
    "Action", "Adventure", "Animation", "Biography", "Comedy",
    "Crime", "Documentary", "Drama", "Family", "Fantasy",
    "History", "Horror", "Music", "Mystery", "Romance",
    "Sci-Fi", "Sport", "Thriller", "War", "Western"
  ];

  // Handle back for sub-states (genre selected -> deselect, search -> clear)
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' || e.key === 'Backspace' || e.key === 'GoBack') {
      if ((e.target as HTMLElement).tagName === 'INPUT') return;
      e.preventDefault();
      e.stopImmediatePropagation();
      if (selectedGenre) {
        selectedGenre = null;
        return;
      }
      if (searchQuery) {
        searchQuery = '';
        searchResults = [];
        return;
      }
      goto('/');
    }
  }

  onMount(() => {
    setTimeout(() => {
      makeFocusable();
      setFocus('.search-input');
    }, 200);

    document.addEventListener('keydown', handleKeydown, true);

    return () => {
      clearTimeout(searchDebounceTimer);
      document.removeEventListener('keydown', handleKeydown, true);
    };
  });

  function handleSearchInput() {
    clearTimeout(searchDebounceTimer);
    if (!searchQuery.trim()) {
      searchResults = [];
      searchError = null;
      return;
    }
    searchDebounceTimer = setTimeout(async () => {
      searchLoading = true;
      searchError = null;
      searchPage = 1;
      try {
        const data = await listMovies({ query_term: searchQuery, limit: 20 });
        searchResults = data.movies || [];
        searchTotal = data.movie_count || 0;
      } catch (err) {
        console.error('Search failed:', err);
        searchError = err instanceof Error ? err.message : 'Search failed. Please try again.';
        searchResults = [];
      } finally {
        searchLoading = false;
      }
    }, 300);
  }

  async function handleSearchSubmit() {
    clearTimeout(searchDebounceTimer);
    if (!searchQuery.trim()) return;
    searchLoading = true;
    searchError = null;
    searchPage = 1;
    try {
      const data = await listMovies({ query_term: searchQuery, limit: 20 });
      searchResults = data.movies || [];
      searchTotal = data.movie_count || 0;
    } catch (err) {
      console.error('Search failed:', err);
      searchError = err instanceof Error ? err.message : 'Search failed. Please try again.';
      searchResults = [];
    } finally {
      searchLoading = false;
    }
  }

  async function loadMoreSearch() {
    if (searchLoadingMore) return;
    searchLoadingMore = true;
    searchPage += 1;
    try {
      const data = await listMovies({ query_term: searchQuery, limit: 20, page: searchPage });
      searchResults = [...searchResults, ...(data.movies || [])];
    } catch (err) {
      console.error(err);
    } finally {
      searchLoadingMore = false;
    }
  }

  async function handleGenreSelect(genre: string) {
    selectedGenre = genre;
    genreLoading = true;
    genreMovies = [];
    genrePage = 1;
    try {
      const data = await listMovies({ genre, sort_by: "rating", limit: 20 });
      genreMovies = data.movies || [];
      genreTotal = data.movie_count || 0;
    } catch (err) {
      console.error(err);
    } finally {
      genreLoading = false;
    }
  }

  async function loadMoreGenre() {
    if (genreLoadingMore || !selectedGenre) return;
    genreLoadingMore = true;
    genrePage += 1;
    try {
      const data = await listMovies({ genre: selectedGenre, sort_by: "rating", limit: 20, page: genrePage });
      genreMovies = [...genreMovies, ...(data.movies || [])];
    } catch (err) {
      console.error(err);
    } finally {
      genreLoadingMore = false;
    }
  }

</script>

<svelte:head>
  <title>Search - Omnius</title>
</svelte:head>

<div class="app">
  <Sidebar />
  <main class="main-content">
    <div class="search-view">
      {#if selectedGenre}
        <div class="genre-header">
          <button
            class="back-btn focusable"
            onclick={() => { selectedGenre = null; }}
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M15 18l-6-6 6-6" />
            </svg>
          </button>
          <h1>{selectedGenre}</h1>
        </div>
        <MovieGrid
          movies={genreMovies}
          loading={genreLoading}
          loadingMore={genreLoadingMore}
          hasMore={genreMovies.length < genreTotal}
          onLoadMore={loadMoreGenre}
        />
      {:else}
        <div class="search-header">
          <h1>Search</h1>
          <input
            class="search-input focusable"
            type="text"
            placeholder="Search movies..."
            bind:value={searchQuery}
            oninput={handleSearchInput}
            onkeydown={(e) => { if (e.key === 'Enter') handleSearchSubmit(); }}
          />
        </div>

        {#if searchError}
          <div class="search-error">
            <p>{searchError}</p>
          </div>
        {/if}

        {#if searchResults.length > 0}
          <div class="search-results">
            <MovieGrid
              movies={searchResults}
              loading={searchLoading}
              loadingMore={searchLoadingMore}
              hasMore={searchResults.length < searchTotal}
              onLoadMore={loadMoreSearch}
            />
          </div>
        {:else if !searchQuery && !searchError}
          <div class="search-genres">
            <h2>Browse by Genre</h2>
            <div class="genre-grid">
              {#each genres as genre}
                <button
                  class="genre-card focusable"
                  onclick={() => handleGenreSelect(genre)}
                >
                  {genre}
                </button>
              {/each}
            </div>
          </div>
        {/if}
      {/if}
    </div>
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

  .search-view {
    padding: 20px 0 0;
  }

  .search-header {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 20px 40px 16px;
  }

  .search-header h1 {
    font-size: 1.3rem;
    font-weight: 600;
    margin: 0;
    color: #fff;
  }

  .search-input {
    flex: 1;
    max-width: 400px;
    padding: 10px 16px;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 8px;
    color: #fff;
    font-size: 0.9rem;
  }

  .search-input:focus {
    outline: none;
    border-color: #e50914;
    box-shadow: 0 0 0 2px rgba(229, 9, 20, 0.3);
  }

  .search-input::placeholder {
    color: #666;
  }

  .search-genres {
    padding: 20px 40px;
  }

  .search-genres h2 {
    font-size: 1.1rem;
    font-weight: 600;
    margin: 0 0 12px;
    color: #fff;
  }

  .genre-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: 8px;
  }

  .genre-card {
    padding: 12px 16px;
    background: rgba(255, 255, 255, 0.06);
    border: none;
    border-radius: 8px;
    color: #ccc;
    font-size: 0.85rem;
    cursor: pointer;
    transition: all 0.2s;
    text-align: center;
  }

  .genre-card:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #fff;
  }

  .genre-card:focus {
    outline: none;
    box-shadow: 0 0 0 2px #e50914;
    background: rgba(229, 9, 20, 0.15);
    color: #fff;
  }

  .genre-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 20px 40px 16px;
  }

  .genre-header h1 {
    font-size: 1.3rem;
    font-weight: 600;
    margin: 0;
    color: #fff;
  }

  .back-btn {
    width: 40px;
    height: 40px;
    background: rgba(255, 255, 255, 0.1);
    border: none;
    border-radius: 50%;
    color: #fff;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .back-btn:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  .back-btn:focus {
    outline: none;
    box-shadow: 0 0 0 2px #e50914;
  }

  .back-btn svg {
    width: 24px;
    height: 24px;
  }

  .search-error {
    padding: 16px 40px;
    color: #ff6b6b;
    background: rgba(229, 9, 20, 0.1);
    border: 1px solid rgba(229, 9, 20, 0.3);
    border-radius: 8px;
    margin: 12px 40px;
    font-size: 0.9rem;
  }

  .search-error p {
    margin: 0;
  }

  .search-results {
    padding: 0 0 20px;
  }
</style>
