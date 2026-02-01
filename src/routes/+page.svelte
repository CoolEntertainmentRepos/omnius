<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { browser } from "$app/environment";
  import MovieRow from "$lib/components/MovieRow.svelte";
  import MovieGrid from "$lib/components/MovieGrid.svelte";
  import { listMovies } from "$lib/api/commands";
  import { favoritesStore } from "$lib/stores/favorites.svelte";
  import { makeFocusable } from "$lib/utils/tvNavigation";
  import type { Movie } from "$lib/api/types";

  // Category rows data
  let featured = $state<Movie | null>(null);
  let trending = $state<Movie[]>([]);
  let topRated = $state<Movie[]>([]);
  let newReleases = $state<Movie[]>([]);
  let action = $state<Movie[]>([]);
  let comedy = $state<Movie[]>([]);
  let thriller = $state<Movie[]>([]);
  let scifi = $state<Movie[]>([]);
  let drama = $state<Movie[]>([]);

  let loading = $state(true);
  let activeNav = $state("home");

  // Search state
  let searchQuery = $state("");
  let searchResults = $state<Movie[]>([]);
  let searchSuggestions = $state<Movie[]>([]);
  let searchLoading = $state(false);
  let showSearch = $state(false);
  let showSuggestions = $state(false);
  let searchDebounceTimer: ReturnType<typeof setTimeout>;

  // Genre browsing
  let selectedGenre = $state<string | null>(null);
  let genreMovies = $state<Movie[]>([]);
  let genreLoading = $state(false);
  let genrePage = $state(1);
  let genreTotal = $state(0);
  let genreLoadingMore = $state(false);

  // Pagination for category views
  let trendingPage = $state(1);
  let trendingTotal = $state(0);
  let trendingLoadingMore = $state(false);

  let newReleasesPage = $state(1);
  let newReleasesTotal = $state(0);
  let newReleasesLoadingMore = $state(false);

  let searchPage = $state(1);
  let searchTotal = $state(0);
  let searchLoadingMore = $state(false);

  const genres = [
    "Action", "Adventure", "Animation", "Biography", "Comedy", "Crime",
    "Documentary", "Drama", "Family", "Fantasy", "History", "Horror",
    "Music", "Mystery", "Romance", "Sci-Fi", "Sport", "Thriller", "War", "Western"
  ];

  onMount(async () => {
    await loadHomeContent();
    // Refresh focusable elements and set initial focus
    setTimeout(() => {
      makeFocusable();
      // Focus on first nav item (home) by default
      const homeNav = document.querySelector<HTMLElement>('.nav-item.active');
      homeNav?.focus();
    }, 200);
  });

  async function loadHomeContent() {
    loading = true;

    const [
      trendingData,
      topRatedData,
      newReleasesData,
      actionData,
      comedyData,
      thrillerData,
      scifiData,
      dramaData,
    ] = await Promise.all([
      listMovies({ sort_by: "download_count", limit: 20 }),
      listMovies({ sort_by: "rating", limit: 20, minimum_rating: 8 }),
      listMovies({ sort_by: "date_added", limit: 20 }),
      listMovies({ genre: "Action", sort_by: "rating", limit: 20 }),
      listMovies({ genre: "Comedy", sort_by: "rating", limit: 20 }),
      listMovies({ genre: "Thriller", sort_by: "rating", limit: 20 }),
      listMovies({ genre: "Sci-Fi", sort_by: "rating", limit: 20 }),
      listMovies({ genre: "Drama", sort_by: "rating", limit: 20 }),
    ]);

    trending = trendingData.movies || [];
    trendingTotal = trendingData.movie_count || 0;
    topRated = topRatedData.movies || [];
    newReleases = newReleasesData.movies || [];
    newReleasesTotal = newReleasesData.movie_count || 0;
    action = actionData.movies || [];
    comedy = comedyData.movies || [];
    thriller = thrillerData.movies || [];
    scifi = scifiData.movies || [];
    drama = dramaData.movies || [];

    // Pick a random featured movie from top rated
    if (topRated.length > 0) {
      featured = topRated[Math.floor(Math.random() * Math.min(5, topRated.length))];
    }

    loading = false;
  }

  function handleNavClick(nav: string) {
    activeNav = nav;
    showSearch = nav === "search";
    selectedGenre = null;

    if (nav === "home") {
      searchQuery = "";
      searchResults = [];
      // Scroll to top when going home
      window.scrollTo({ top: 0, behavior: 'smooth' });
    }

    // Auto-focus search input when entering search
    if (nav === "search") {
      setTimeout(() => {
        const searchInput = document.querySelector<HTMLInputElement>(".search-input");
        searchInput?.focus();
      }, 100);
    }
  }

  function handleSearchInput() {
    // Clear previous timer
    clearTimeout(searchDebounceTimer);

    if (!searchQuery.trim()) {
      searchSuggestions = [];
      searchResults = [];
      showSuggestions = false;
      return;
    }

    // Show suggestions after short delay (debounce)
    searchDebounceTimer = setTimeout(async () => {
      searchLoading = true;
      showSuggestions = true;
      try {
        const data = await listMovies({ query_term: searchQuery, limit: 8 });
        searchSuggestions = data.movies || [];
      } catch {
        searchSuggestions = [];
      } finally {
        searchLoading = false;
      }
    }, 300);
  }

  async function handleSearchSubmit() {
    clearTimeout(searchDebounceTimer);
    showSuggestions = false;

    if (!searchQuery.trim()) {
      searchResults = [];
      return;
    }

    searchLoading = true;
    searchPage = 1;
    try {
      const data = await listMovies({ query_term: searchQuery, limit: 20, page: 1 });
      searchResults = data.movies || [];
      searchTotal = data.movie_count || 0;
    } catch {
      searchResults = [];
      searchTotal = 0;
    } finally {
      searchLoading = false;
    }
  }

  function selectSuggestion(movie: Movie) {
    showSuggestions = false;
    goto(`/movie/${movie.id}`);
  }

  async function handleGenreSelect(genre: string) {
    selectedGenre = genre;
    activeNav = "movies";
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

  async function loadMoreTrending() {
    if (trendingLoadingMore) return;
    trendingLoadingMore = true;
    trendingPage += 1;

    try {
      const data = await listMovies({ sort_by: "download_count", limit: 20, page: trendingPage });
      trending = [...trending, ...(data.movies || [])];
    } catch {
      trendingPage -= 1;
    } finally {
      trendingLoadingMore = false;
    }
  }

  async function loadMoreNewReleases() {
    if (newReleasesLoadingMore) return;
    newReleasesLoadingMore = true;
    newReleasesPage += 1;

    try {
      const data = await listMovies({ sort_by: "date_added", limit: 20, page: newReleasesPage });
      newReleases = [...newReleases, ...(data.movies || [])];
    } catch {
      newReleasesPage -= 1;
    } finally {
      newReleasesLoadingMore = false;
    }
  }

  async function loadMoreSearch() {
    if (searchLoadingMore || !searchQuery.trim()) return;
    searchLoadingMore = true;
    searchPage += 1;

    try {
      const data = await listMovies({ query_term: searchQuery, limit: 20, page: searchPage });
      searchResults = [...searchResults, ...(data.movies || [])];
    } catch {
      searchPage -= 1;
    } finally {
      searchLoadingMore = false;
    }
  }

  function handlePlay() {
    if (featured) {
      goto(`/movie/${featured.id}`);
    }
  }

  function handleMoreInfo() {
    if (featured) {
      goto(`/movie/${featured.id}`);
    }
  }

  function getRatingDisplay(rating: number): string {
    if (!rating || rating === 0) return "N/A";
    return rating.toFixed(1);
  }

  // Special handling for search input - go to sidebar on left arrow at start
  function handleSearchInputKeydown(e: KeyboardEvent) {
    const target = e.target as HTMLInputElement;
    if (e.key === "ArrowLeft" && target.selectionStart === 0) {
      e.preventDefault();
      const activeNavItem = document.querySelector<HTMLElement>(".nav-item.active");
      activeNavItem?.focus();
    }
  }
</script>

<svelte:head>
  <title>Streamer - Watch Movies</title>
</svelte:head>

<div class="app">
  <!-- Left Sidebar -->
  <nav class="sidebar">
    <div class="sidebar-top">
      <button
        class="nav-item"
        class:active={activeNav === "search"}
        onclick={() => handleNavClick("search")}
        aria-label="Search"
      >
        <svg viewBox="0 0 24 24" fill="currentColor">
          <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
        </svg>
      </button>
      <button
        class="nav-item"
        class:active={activeNav === "home"}
        onclick={() => handleNavClick("home")}
        aria-label="Home"
      >
        <svg viewBox="0 0 24 24" fill="currentColor">
          <path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z"/>
        </svg>
      </button>
      <button
        class="nav-item"
        class:active={activeNav === "trending"}
        onclick={() => handleNavClick("trending")}
        aria-label="Trending"
      >
        <svg viewBox="0 0 24 24" fill="currentColor">
          <path d="M16 6l2.29 2.29-4.88 4.88-4-4L2 16.59 3.41 18l6-6 4 4 6.3-6.29L22 12V6z"/>
        </svg>
      </button>
    </div>
    <div class="sidebar-bottom">
      <button
        class="nav-item"
        class:active={activeNav === "movies"}
        onclick={() => handleNavClick("movies")}
        aria-label="Movies"
      >
        <svg viewBox="0 0 24 24" fill="currentColor">
          <path d="M18 4l2 4h-3l-2-4h-2l2 4h-3l-2-4H8l2 4H7L5 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V4h-4z"/>
        </svg>
      </button>
      <button
        class="nav-item"
        class:active={activeNav === "favorites"}
        onclick={() => handleNavClick("favorites")}
        aria-label="My List"
      >
        <svg viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/>
        </svg>
        {#if favoritesStore.count > 0}
          <span class="nav-badge">{favoritesStore.count}</span>
        {/if}
      </button>
      <button
        class="nav-item"
        class:active={activeNav === "new"}
        onclick={() => handleNavClick("new")}
        aria-label="New Releases"
      >
        <svg viewBox="0 0 24 24" fill="currentColor">
          <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
        </svg>
      </button>
      <button
        class="nav-item"
        onclick={() => goto('/settings')}
        aria-label="Settings"
      >
        <svg viewBox="0 0 24 24" fill="currentColor">
          <path d="M19.14,12.94c0.04-0.3,0.06-0.61,0.06-0.94c0-0.32-0.02-0.64-0.07-0.94l2.03-1.58c0.18-0.14,0.23-0.41,0.12-0.61 l-1.92-3.32c-0.12-0.22-0.37-0.29-0.59-0.22l-2.39,0.96c-0.5-0.38-1.03-0.7-1.62-0.94L14.4,2.81c-0.04-0.24-0.24-0.41-0.48-0.41 h-3.84c-0.24,0-0.43,0.17-0.47,0.41L9.25,5.35C8.66,5.59,8.12,5.92,7.63,6.29L5.24,5.33c-0.22-0.08-0.47,0-0.59,0.22L2.74,8.87 C2.62,9.08,2.66,9.34,2.86,9.48l2.03,1.58C4.84,11.36,4.8,11.69,4.8,12s0.02,0.64,0.07,0.94l-2.03,1.58 c-0.18,0.14-0.23,0.41-0.12,0.61l1.92,3.32c0.12,0.22,0.37,0.29,0.59,0.22l2.39-0.96c0.5,0.38,1.03,0.7,1.62,0.94l0.36,2.54 c0.05,0.24,0.24,0.41,0.48,0.41h3.84c0.24,0,0.44-0.17,0.47-0.41l0.36-2.54c0.59-0.24,1.13-0.56,1.62-0.94l2.39,0.96 c0.22,0.08,0.47,0,0.59-0.22l1.92-3.32c0.12-0.22,0.07-0.47-0.12-0.61L19.14,12.94z M12,15.6c-1.98,0-3.6-1.62-3.6-3.6 s1.62-3.6,3.6-3.6s3.6,1.62,3.6,3.6S13.98,15.6,12,15.6z"/>
        </svg>
      </button>
    </div>
  </nav>

  <!-- Main Content -->
  <main class="main-content">
    {#if loading}
      <div class="loading-page">
        <div class="spinner"></div>
      </div>
    {:else if showSearch}
      <!-- Search View -->
      <div class="search-view">
        <div class="search-header">
          <h1>Search</h1>
          <div class="search-box">
            <div class="search-input-container">
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
              </svg>
              <input
                type="text"
                placeholder="Search movies..."
                bind:value={searchQuery}
                oninput={handleSearchInput}
                onkeydown={(e) => { handleSearchInputKeydown(e); if (e.key === 'Enter') handleSearchSubmit(); }}
                onfocus={() => searchQuery && (showSuggestions = true)}
                class="search-input"
              />
              {#if searchQuery}
                <button class="clear-btn" onclick={() => { searchQuery = ""; searchResults = []; searchSuggestions = []; showSuggestions = false; }} aria-label="Clear search">
                  <svg viewBox="0 0 24 24" fill="currentColor">
                    <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
                  </svg>
                </button>
              {/if}
            </div>

            <!-- Live Suggestions Dropdown -->
            {#if showSuggestions && searchQuery}
              <div class="search-suggestions">
                {#if searchLoading}
                  <div class="suggestion-loading">Searching...</div>
                {:else if searchSuggestions.length === 0}
                  <div class="suggestion-empty">No results for "{searchQuery}"</div>
                {:else}
                  {#each searchSuggestions as movie (movie.id)}
                    <button
                      class="suggestion-item"
                      onclick={() => selectSuggestion(movie)}
                      tabindex="0"
                    >
                      <img
                        src={movie.small_cover_image}
                        alt=""
                        class="suggestion-poster"
                      />
                      <div class="suggestion-info">
                        <span class="suggestion-title">{movie.title}</span>
                        <span class="suggestion-meta">{movie.year} • {movie.rating ? movie.rating.toFixed(1) : 'N/A'}</span>
                      </div>
                    </button>
                  {/each}
                  <button class="suggestion-more" onclick={handleSearchSubmit} tabindex="0">
                    See all results for "{searchQuery}"
                  </button>
                {/if}
              </div>
            {/if}
          </div>
        </div>

        {#if searchResults.length > 0}
          <div class="search-results">
            <p class="results-count">{searchResults.length} of {searchTotal} results for "{searchQuery}"</p>
            <MovieGrid
              movies={searchResults}
              loading={false}
              error={null}
              hasMore={searchResults.length < searchTotal}
              loadingMore={searchLoadingMore}
              onLoadMore={loadMoreSearch}
            />
          </div>
        {:else if !searchQuery}
          <div class="search-genres">
            <h2>Browse by Genre</h2>
            <div class="genre-grid">
              {#each genres as genre (genre)}
                <button class="genre-card" onclick={() => handleGenreSelect(genre)}>
                  {genre}
                </button>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {:else if selectedGenre}
      <!-- Genre View -->
      <div class="genre-view">
        <div class="genre-header">
          <button class="back-btn" onclick={() => { selectedGenre = null; activeNav = "home"; }} aria-label="Go back">
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
            </svg>
          </button>
          <h1>{selectedGenre} Movies</h1>
        </div>
        <MovieGrid
          movies={genreMovies}
          loading={genreLoading}
          error={null}
          hasMore={genreMovies.length < genreTotal}
          loadingMore={genreLoadingMore}
          onLoadMore={loadMoreGenre}
        />
      </div>
    {:else if activeNav === "trending"}
      <!-- Trending View -->
      <div class="category-view">
        <h1 class="category-title">Trending Now</h1>
        <MovieGrid
          movies={trending}
          loading={false}
          error={null}
          hasMore={trending.length < trendingTotal}
          loadingMore={trendingLoadingMore}
          onLoadMore={loadMoreTrending}
        />
      </div>
    {:else if activeNav === "new"}
      <!-- New Releases View -->
      <div class="category-view">
        <h1 class="category-title">New Releases</h1>
        <MovieGrid
          movies={newReleases}
          loading={false}
          error={null}
          hasMore={newReleases.length < newReleasesTotal}
          loadingMore={newReleasesLoadingMore}
          onLoadMore={loadMoreNewReleases}
        />
      </div>
    {:else if activeNav === "movies"}
      <!-- Movies/Genres View -->
      <div class="search-view">
        <h1 class="category-title">Browse by Genre</h1>
        <div class="genre-grid">
          {#each genres as genre (genre)}
            <button class="genre-card" onclick={() => handleGenreSelect(genre)}>
              {genre}
            </button>
          {/each}
        </div>
      </div>
    {:else if activeNav === "favorites"}
      <!-- Favorites View -->
      <div class="category-view">
        <div class="favorites-header">
          <h1 class="category-title">My List</h1>
          {#if favoritesStore.count > 0}
            <span class="favorites-count">{favoritesStore.count} movies</span>
          {/if}
        </div>
        {#if favoritesStore.favorites.length === 0}
          <div class="empty-state">
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/>
            </svg>
            <h2>Your list is empty</h2>
            <p>Add movies to your list to watch them later</p>
          </div>
        {:else}
          <MovieGrid movies={favoritesStore.favorites} loading={false} error={null} />
        {/if}
      </div>
    {:else}
      <!-- Home View -->
      {#if featured}
        <div
          class="hero"
          style="background-image: url({featured.background_image_original || featured.large_cover_image})"
        >
          <div class="hero-gradient"></div>
          <div class="hero-content">
            <div class="hero-badge">
              <svg class="badge-icon" viewBox="0 0 24 24" fill="currentColor">
                <path d="M4 6.47L5.76 10H20v8H4V6.47M22 4h-4l2 4h-3l-2-4h-2l2 4h-3l-2-4H8l2 4H7L5 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V4z"/>
              </svg>
              <span>MOVIE</span>
            </div>
            <h1 class="hero-title">{featured.title}</h1>
            <div class="hero-meta">
              <span class="hero-year">{featured.year}</span>
              <span class="hero-rating">
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
                </svg>
                {getRatingDisplay(featured.rating)} IMDb
              </span>
              {#if featured.genres && featured.genres.length > 0}
                <span class="hero-genres">{featured.genres.slice(0, 2).join(" / ")}</span>
              {/if}
            </div>
            <p class="hero-description">
              {featured.summary || featured.description_full || featured.synopsis || "A thrilling movie experience awaits you."}
            </p>
            <div class="hero-buttons">
              <button class="btn-play" onclick={handlePlay} onfocus={() => window.scrollTo({ top: 0, behavior: 'smooth' })}>
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M8 5v14l11-7z"/>
                </svg>
                Play
              </button>
              <button class="btn-info" onclick={handleMoreInfo} onfocus={() => window.scrollTo({ top: 0, behavior: 'smooth' })}>
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/>
                </svg>
                More Info
              </button>
            </div>
          </div>
        </div>
      {/if}

      <!-- Movie Rows -->
      <div class="content">
        <MovieRow title="Trending Now" movies={trending} />
        <MovieRow title="Top Rated" movies={topRated} />
        <MovieRow title="New Releases" movies={newReleases} />
        <MovieRow title="Action" movies={action} />
        <MovieRow title="Thriller" movies={thriller} />
        <MovieRow title="Sci-Fi" movies={scifi} />
        <MovieRow title="Comedy" movies={comedy} />
        <MovieRow title="Drama" movies={drama} />
      </div>
    {/if}
  </main>
</div>

<style>
  :global(html, body) {
    background: #141414;
    margin: 0;
    padding: 0;
    height: 100%;
    overflow: hidden;
  }

  /* TV Remote focus styles */
  :global(*:focus) {
    outline: none;
  }

  :global(*:focus-visible) {
    outline: 3px solid #e50914 !important;
    outline-offset: 2px;
  }

  .app {
    display: flex;
    height: 100vh;
    height: 100dvh;
    background: #141414;
    color: #fff;
    overflow: hidden;
  }

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

  /* Main Content - scrollable area */
  .main-content {
    flex: 1;
    margin-left: 70px;
    height: 100vh;
    height: 100dvh;
    overflow-y: auto;
    overflow-x: hidden;
    scroll-behavior: smooth;
  }

  /* Hero */
  .hero {
    position: relative;
    height: 85vh;
    min-height: 550px;
    max-height: 900px;
    background-size: cover;
    background-position: center top;
    display: flex;
    align-items: flex-end;
  }

  .hero-gradient {
    position: absolute;
    inset: 0;
    background: linear-gradient(
      to right,
      rgba(20, 20, 20, 0.95) 0%,
      rgba(20, 20, 20, 0.7) 30%,
      rgba(20, 20, 20, 0.4) 50%,
      rgba(20, 20, 20, 0.2) 70%,
      transparent 100%
    ),
    linear-gradient(
      to top,
      #141414 0%,
      rgba(20, 20, 20, 0.8) 15%,
      transparent 40%
    );
  }

  .hero-content {
    position: relative;
    z-index: 10;
    padding: 0 80px 100px;
    max-width: 650px;
  }

  .hero-badge {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    color: #e50914;
    font-size: 0.9rem;
    font-weight: 600;
    letter-spacing: 3px;
    margin-bottom: 12px;
  }

  .badge-icon {
    width: 24px;
    height: 24px;
  }

  .hero-title {
    font-size: 3.5rem;
    font-weight: 700;
    margin: 0 0 16px;
    text-shadow: 2px 2px 8px rgba(0, 0, 0, 0.8);
    line-height: 1.1;
  }

  .hero-meta {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 16px;
    font-size: 1rem;
  }

  .hero-year {
    color: #fff;
    font-weight: 500;
  }

  .hero-rating {
    display: flex;
    align-items: center;
    gap: 6px;
    color: #f5c518;
    font-weight: 600;
  }

  .hero-rating svg {
    width: 18px;
    height: 18px;
  }

  .hero-genres {
    color: #aaa;
  }

  .hero-description {
    font-size: 1.1rem;
    line-height: 1.5;
    color: #ddd;
    margin: 0 0 24px;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
    text-shadow: 1px 1px 4px rgba(0, 0, 0, 0.8);
  }

  .hero-buttons {
    display: flex;
    gap: 16px;
  }

  .btn-play, .btn-info {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 14px 32px;
    border: none;
    border-radius: 6px;
    font-size: 1.1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn-play {
    background: #fff;
    color: #141414;
  }

  .btn-play:hover {
    background: rgba(255, 255, 255, 0.85);
  }

  .btn-play:focus,
  .btn-play:focus-visible {
    outline: none;
    box-shadow: 0 0 0 4px #e50914;
    transform: scale(1.05);
  }

  .btn-info {
    background: rgba(109, 109, 110, 0.7);
    color: #fff;
  }

  .btn-info:hover {
    background: rgba(109, 109, 110, 0.5);
  }

  .btn-info:focus,
  .btn-info:focus-visible {
    outline: none;
    box-shadow: 0 0 0 4px #e50914;
    background: rgba(229, 9, 20, 0.4);
    transform: scale(1.05);
  }

  .btn-play svg, .btn-info svg {
    width: 28px;
    height: 28px;
  }

  /* Content */
  .content {
    position: relative;
    margin-top: 20px;
    padding-bottom: 60px;
    z-index: 10;
  }

  /* Search View */
  .search-view, .genre-view, .category-view {
    padding: 40px 80px;
  }

  .search-header {
    margin-bottom: 40px;
  }

  .search-header h1 {
    font-size: 2rem;
    font-weight: 600;
    margin: 0 0 24px;
  }

  .search-input-container {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 16px 24px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    max-width: 600px;
  }

  .search-input-container svg {
    width: 24px;
    height: 24px;
    color: #888;
    flex-shrink: 0;
  }

  .search-input {
    flex: 1;
    background: none;
    border: none;
    color: #fff;
    font-size: 1.2rem;
    font-family: inherit;
    outline: none;
  }

  .search-input:focus {
    outline: none;
  }

  .search-input-container:focus-within {
    box-shadow: 0 0 0 3px #e50914;
    background: rgba(255, 255, 255, 0.15);
  }

  .search-input::placeholder {
    color: #666;
  }

  .clear-btn {
    background: none;
    border: none;
    color: #888;
    cursor: pointer;
    padding: 4px;
    display: flex;
  }

  .clear-btn:hover {
    color: #fff;
  }

  .clear-btn svg {
    width: 20px;
    height: 20px;
  }

  /* Search Suggestions */
  .search-box {
    position: relative;
    max-width: 600px;
  }

  .search-suggestions {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    background: rgba(30, 30, 30, 0.98);
    border-radius: 0 0 12px 12px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-top: none;
    max-height: 350px;
    overflow-y: auto;
    z-index: 100;
  }

  .suggestion-loading,
  .suggestion-empty {
    padding: 16px 20px;
    color: #888;
    font-size: 0.95rem;
  }

  .suggestion-item {
    display: flex;
    align-items: center;
    gap: 14px;
    width: 100%;
    padding: 10px 16px;
    background: none;
    border: none;
    color: #fff;
    cursor: pointer;
    text-align: left;
    transition: background 0.15s;
  }

  .suggestion-item:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .suggestion-item:focus,
  .suggestion-item:focus-visible {
    outline: none;
    background: rgba(229, 9, 20, 0.4);
    box-shadow: inset 0 0 0 2px #e50914;
  }

  .suggestion-poster {
    width: 40px;
    height: 60px;
    object-fit: cover;
    border-radius: 4px;
    background: #333;
  }

  .suggestion-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .suggestion-title {
    font-size: 1rem;
    font-weight: 500;
  }

  .suggestion-meta {
    font-size: 0.85rem;
    color: #888;
  }

  .suggestion-more {
    display: block;
    width: 100%;
    padding: 14px 16px;
    background: none;
    border: none;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
    color: #e50914;
    font-size: 0.95rem;
    font-weight: 500;
    cursor: pointer;
    text-align: center;
    transition: background 0.15s;
  }

  .suggestion-more:hover {
    background: rgba(229, 9, 20, 0.1);
  }

  .suggestion-more:focus,
  .suggestion-more:focus-visible {
    outline: none;
    background: rgba(229, 9, 20, 0.3);
    box-shadow: inset 0 0 0 2px #e50914;
  }

  .results-count {
    font-size: 1rem;
    color: #888;
    margin: 0 0 24px;
  }

  .search-genres {
    margin-top: 40px;
  }

  .search-genres h2 {
    font-size: 1.4rem;
    font-weight: 600;
    margin: 0 0 24px;
  }

  .genre-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 16px;
  }

  .genre-card {
    padding: 24px 16px;
    background: linear-gradient(135deg, rgba(229, 9, 20, 0.3), rgba(229, 9, 20, 0.1));
    border: 1px solid rgba(229, 9, 20, 0.3);
    border-radius: 8px;
    color: #fff;
    font-size: 1rem;
    font-weight: 500;
    font-family: inherit;
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: center;
  }

  .genre-card:hover {
    background: linear-gradient(135deg, rgba(229, 9, 20, 0.5), rgba(229, 9, 20, 0.2));
    border-color: #e50914;
    transform: scale(1.02);
  }

  .genre-card:focus,
  .genre-card:focus-visible {
    outline: none;
    border-color: #e50914;
    box-shadow: 0 0 0 4px #e50914;
    background: linear-gradient(135deg, rgba(229, 9, 20, 0.6), rgba(229, 9, 20, 0.3));
    transform: scale(1.05);
  }

  .genre-header {
    display: flex;
    align-items: center;
    gap: 20px;
    margin-bottom: 32px;
  }

  .genre-header h1 {
    font-size: 2rem;
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

  .category-title {
    font-size: 2rem;
    font-weight: 600;
    margin: 0 0 32px;
  }

  .favorites-header {
    display: flex;
    align-items: baseline;
    gap: 16px;
    margin-bottom: 32px;
  }

  .favorites-header .category-title {
    margin: 0;
  }

  .favorites-count {
    font-size: 1rem;
    color: #888;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 80px 40px;
    text-align: center;
  }

  .empty-state svg {
    width: 80px;
    height: 80px;
    color: #333;
    margin-bottom: 24px;
  }

  .empty-state h2 {
    font-size: 1.5rem;
    font-weight: 600;
    margin: 0 0 12px;
    color: #fff;
  }

  .empty-state p {
    font-size: 1rem;
    color: #888;
    margin: 0;
  }

  /* Loading */
  .loading-page {
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
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
    to {
      transform: rotate(360deg);
    }
  }

  /* Responsive */
  @media (max-width: 1200px) {
    .hero-content {
      padding: 0 60px 80px;
    }

    .hero-title {
      font-size: 3rem;
    }

    .search-view, .genre-view, .category-view {
      padding: 30px 60px;
    }
  }

  @media (max-width: 900px) {
    .sidebar {
      width: 60px;
    }

    .main-content {
      margin-left: 60px;
      height: 100vh;
      height: 100dvh;
    }

    .nav-item {
      width: 44px;
      height: 44px;
    }

    .nav-item svg {
      width: 22px;
      height: 22px;
    }

    .hero-content {
      padding: 0 40px 60px;
      max-width: 500px;
    }

    .hero-title {
      font-size: 2.5rem;
    }

    .hero-description {
      font-size: 1rem;
      -webkit-line-clamp: 2;
    }

    .btn-play, .btn-info {
      padding: 12px 24px;
      font-size: 1rem;
    }

    .search-view, .genre-view, .category-view {
      padding: 24px 40px;
    }
  }

  @media (max-width: 600px) {
    .sidebar {
      display: none;
    }

    .main-content {
      margin-left: 0;
    }

    .hero {
      height: 70vh;
      min-height: 400px;
    }

    .hero-content {
      padding: 0 24px 50px;
    }

    .hero-title {
      font-size: 2rem;
    }

    .btn-play, .btn-info {
      padding: 12px 20px;
      font-size: 1rem;
      gap: 8px;
    }

    .btn-play svg, .btn-info svg {
      width: 24px;
      height: 24px;
    }

    .content {
      margin-top: 10px;
    }

    .search-view, .genre-view, .category-view {
      padding: 20px 24px;
    }

    .genre-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }
</style>
