<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { browser } from "$app/environment";
  import MovieCard from "$lib/components/MovieCard.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import { getMovieDetails, getMovieSuggestions, startStream, getFranchiseMovies } from "$lib/api/commands";
  import { streamStore } from "$lib/stores/stream.svelte";
  import { favoritesStore } from "$lib/stores/favorites.svelte";
  import { remindersStore } from "$lib/stores/reminders.svelte";
  import type { MovieDetails, Movie, Torrent } from "$lib/api/types";

  let movieId = $derived(parseInt($page.params.id || "0", 10));

  let movie = $state<MovieDetails | null>(null);
  let suggestions = $state<Movie[]>([]);
  let franchiseMovies = $state<Movie[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let selectedTorrent = $state<Torrent | null>(null);
  let isStartingStream = $state(false);
  let streamError = $state<string | null>(null);
  let showMore = $state(false);
  let modalBodyEl = $state<HTMLElement | null>(null);

  async function loadMovie(id: number) {
    loading = true;
    error = null;
    selectedTorrent = null;
    franchiseMovies = [];

    try {
      const [movieData, suggestionsData] = await Promise.all([
        getMovieDetails(id, true, true),
        getMovieSuggestions(id),
      ]);

      movie = movieData;
      suggestions = suggestionsData;

      console.log("[Movie] Loaded movie:", movie.title, "franchise:", movie.franchise);

      // If movie has a franchise, fetch other movies in the franchise
      if (movie.franchise) {
        console.log("[Movie] Fetching franchise movies for:", movie.franchise);
        const movies = await getFranchiseMovies(id);
        franchiseMovies = movies;
        console.log("[Movie] Franchise movies loaded:", movies.length);
      } else {
        console.log("[Movie] No franchise for this movie");
      }

      // Auto-select best quality (only torrents with seeds > 0)
      if (movie.torrents && movie.torrents.length > 0) {
        // Filter out torrents with no seeds
        const availableTorrents = movie.torrents.filter((t) => t.seeds > 0);
        if (availableTorrents.length > 0) {
          // Prefer 1080p, then 720p
          selectedTorrent =
            availableTorrents.find((t) => t.quality === "1080p") ||
            availableTorrents.find((t) => t.quality === "720p") ||
            availableTorrents[0];
        }
      }

      // Ratings come from the movie details object already (imdb_rating, rotten_tomatoes, metacritic)
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to load movie";
      movie = null;
    } finally {
      loading = false;
      // Set focus on play button for TV navigation
      if (browser && !error) {
        setTimeout(() => {
          const playBtn = document.querySelector('.play-btn') as HTMLElement;
          if (playBtn) {
            playBtn.focus();
          }
        }, 100);
      }
    }
  }

  onMount(() => {
    loadMovie(movieId);

    // Add focus listener to scroll focused elements into view
    if (browser) {
      document.addEventListener('focus', handleFocusScroll, true);
    }

    return () => {
      if (browser) {
        document.removeEventListener('focus', handleFocusScroll, true);
      }
    };
  });

  function handleFocusScroll(e: FocusEvent) {
    const target = e.target as HTMLElement;
    if (target && target.scrollIntoView) {
      target.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }
  }

  // Reload when movie ID changes
  $effect(() => {
    if (movieId) {
      loadMovie(movieId);
    }
  });

  async function handlePlay() {
    if (!selectedTorrent || !movie) return;

    isStartingStream = true;
    streamError = null;

    try {
      // Navigate immediately with hash - player will start stream
      const params = new URLSearchParams({
        title: movie.title,
        imdb: movie.imdb_code || "",
        movie_id: movie.id.toString(),
        quality: selectedTorrent.quality || "",
      });
      await goto(`/player/${selectedTorrent.hash}?${params.toString()}`);
    } catch (err) {
      console.error("Navigation failed:", err);
      streamError = "Failed to start playback. Please try again.";
      isStartingStream = false;
    }
  }

  function selectQuality(torrent: Torrent) {
    selectedTorrent = torrent;
  }

  function handleModalKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' || e.key === 'GoBack' || e.key === 'XF86Back' || (e.key === 'Backspace' && !(e.target instanceof HTMLInputElement))) {
      e.preventDefault();
      e.stopPropagation();
      showMore = false;
      return;
    }
    // D-pad up/down scrolls modal body
    if (modalBodyEl) {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        modalBodyEl.scrollBy({ top: 80, behavior: 'smooth' });
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        modalBodyEl.scrollBy({ top: -80, behavior: 'smooth' });
      }
    }
  }

  // Focus modal close button when modal opens
  $effect(() => {
    if (showMore && browser) {
      setTimeout(() => {
        const closeBtn = document.querySelector('.modal-close') as HTMLElement;
        if (closeBtn) closeBtn.focus();
      }, 50);
    }
  });

  function handleSuggestionClick(suggestedMovie: Movie) {
    goto(`/movie/${suggestedMovie.id}`);
  }

  function formatRuntime(minutes: number): string {
    if (!minutes) return "";
    const hours = Math.floor(minutes / 60);
    const mins = minutes % 60;
    return hours > 0 ? `${hours}h ${mins}m` : `${mins}m`;
  }

  function formatSize(size: string): string {
    return size || "Unknown";
  }

  function getYouTubeUrl(code: string): string {
    return `https://www.youtube.com/watch?v=${code}`;
  }
</script>

<svelte:head>
  <title>{movie ? `${movie.title} - Streamer` : "Loading... - Streamer"}</title>
</svelte:head>

<Sidebar />

<div class="page">
  {#if loading}
    <div class="loading-container">
      <div class="spinner"></div>
      <p>Loading movie...</p>
    </div>
  {:else if error}
    <div class="error-container">
      <svg viewBox="0 0 24 24" fill="currentColor">
        <path
          d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"
        />
      </svg>
      <p>{error}</p>
      <div class="error-buttons">
        <button class="back-button" onclick={() => goto('/?tab=movies')}>
          <svg viewBox="0 0 24 24" fill="currentColor">
            <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z" />
          </svg>
          Go Back
        </button>
        <button class="retry-button" onclick={() => loadMovie(movieId)}>Try Again</button>
      </div>
    </div>
  {:else if movie}
    <!-- Background -->
    <div
      class="backdrop"
      style="background-image: url({movie.background_image_original || movie.background_image || movie.large_cover_image})"
    >
      <div class="backdrop-overlay"></div>
    </div>

    <!-- Back Button -->
    <nav class="nav">
      <button class="back-btn" onclick={() => goto('/?tab=movies')} aria-label="Go back">
        <svg viewBox="0 0 24 24" fill="currentColor">
          <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z" />
        </svg>
      </button>
    </nav>

    <main class="content">
      <div class="movie-info">
        <!-- Left: Actions -->
        <div class="poster-column">
          {#if movie.status === 'coming_soon'}
            <div class="coming-soon-section">
              <div class="coming-soon-badge">Coming Soon</div>
              {#if movie.release_date}
                <p class="release-date">{new Date(movie.release_date).toLocaleDateString('en-US', { year: 'numeric', month: 'long', day: 'numeric' })}</p>
              {/if}
              <button
                class="remind-btn"
                class:active={remindersStore.isReminded(movie.imdb_code)}
                onclick={() => movie && remindersStore.toggle(movie.imdb_code, movie.title, movie.medium_cover_image)}
              >
                <svg viewBox="0 0 24 24" fill="currentColor">
                  {#if remindersStore.isReminded(movie.imdb_code)}
                    <path d="M12 22c1.1 0 2-.9 2-2h-4c0 1.1.89 2 2 2zm6-6v-5c0-3.07-1.64-5.64-4.5-6.32V4c0-.83-.67-1.5-1.5-1.5s-1.5.67-1.5 1.5v.68C7.63 5.36 6 7.92 6 11v5l-2 2v1h16v-1l-2-2z"/>
                  {:else}
                    <path d="M12 22c1.1 0 2-.9 2-2h-4c0 1.1.89 2 2 2zm6-6v-5c0-3.07-1.64-5.64-4.5-6.32V4c0-.83-.67-1.5-1.5-1.5s-1.5.67-1.5 1.5v.68C7.63 5.36 6 7.92 6 11v5l-2 2v1h16v-1l-2-2zm-2 1H8v-6c0-2.48 1.51-4.5 4-4.5s4 2.02 4 4.5v6z"/>
                  {/if}
                </svg>
                {remindersStore.isReminded(movie.imdb_code) ? "Reminder Set" : "Remind Me"}
              </button>
            </div>
          {:else if movie.torrents && movie.torrents.filter(t => t.seeds > 0).length > 0}
            <!-- Quality Selector -->
            <div class="quality-section">
              <div class="quality-options">
                {#each movie.torrents.filter(t => t.seeds > 0) as torrent (torrent.hash)}
                  <button
                    class="quality-btn"
                    class:selected={selectedTorrent?.hash === torrent.hash}
                    onclick={() => selectQuality(torrent)}
                  >
                    <span class="quality-label">{torrent.quality}</span>
                    <span class="quality-meta">{formatSize(torrent.size)} &middot; {torrent.seeds} seeds</span>
                  </button>
                {/each}
              </div>
            </div>

            <!-- Play Button -->
            <button
              class="play-btn"
              onclick={handlePlay}
              disabled={!selectedTorrent || isStartingStream}
            >
              {#if isStartingStream}
                <div class="btn-spinner"></div>
                Starting...
              {:else}
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M8 5v14l11-7z"/>
                </svg>
                Play {selectedTorrent?.quality || ""}
              {/if}
            </button>

            <div class="secondary-actions">
              <button
                class="action-btn"
                class:active={favoritesStore.isFavorite(movie.id)}
                onclick={() => movie && favoritesStore.toggle(movie)}
                aria-label={favoritesStore.isFavorite(movie.id) ? "Remove from My List" : "Add to My List"}
              >
                <svg viewBox="0 0 24 24" fill="currentColor">
                  {#if favoritesStore.isFavorite(movie.id)}
                    <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
                  {:else}
                    <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
                  {/if}
                </svg>
                {favoritesStore.isFavorite(movie.id) ? "Listed" : "My List"}
              </button>

              {#if movie.yt_trailer_code}
                <a
                  href={getYouTubeUrl(movie.yt_trailer_code)}
                  target="_blank"
                  rel="noopener noreferrer"
                  class="action-btn"
                >
                  <svg viewBox="0 0 24 24" fill="currentColor">
                    <path d="M10 15l5.19-3L10 9v6m11.56-7.83c.13.47.22 1.1.28 1.9.07.8.1 1.49.1 2.09L22 12c0 2.19-.16 3.8-.44 4.83-.25.9-.83 1.48-1.73 1.73-.47.13-1.33.22-2.65.28-1.3.07-2.49.1-3.59.1L12 19c-4.19 0-6.8-.16-7.83-.44-.9-.25-1.48-.83-1.73-1.73-.13-.47-.22-1.1-.28-1.9-.07-.8-.1-1.49-.1-2.09L2 12c0-2.19.16-3.8.44-4.83.25-.9.83-1.48 1.73-1.73.47-.13 1.33-.22 2.65-.28 1.3-.07 2.49-.1 3.59-.1L12 5c4.19 0 6.8.16 7.83.44.9.25 1.48.83 1.73 1.73z"/>
                  </svg>
                  Trailer
                </a>
              {/if}
            </div>

            {#if streamError}
              <div class="stream-error">
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
                </svg>
                {streamError}
              </div>
            {/if}
          {/if}
        </div>

        <!-- Right: Info -->
        <div class="info">
          <h1 class="title">{movie.title}</h1>

          <div class="meta">
            <span class="year">{movie.year}</span>
            {#if movie.runtime}
              <span class="runtime">{formatRuntime(movie.runtime)}</span>
            {/if}
            {#if movie.mpa_rating}
              <span class="mpa-rating">{movie.mpa_rating}</span>
            {/if}
          </div>

          <div class="ratings-row">
            {#if movie.imdb_rating}
              <span class="rating-badge imdb">
                <span class="rating-label">IMDb</span>
                <span class="rating-value">{(movie.imdb_rating || 0).toFixed(1)}</span>
              </span>
            {/if}
            {#if movie.rotten_tomatoes}
              <span class="rating-badge rt">
                <span class="rating-label">RT</span>
                <span class="rating-value">{movie.rotten_tomatoes}%</span>
              </span>
            {/if}
            {#if movie.metacritic}
              {@const metaScore = movie.metacritic || 0}
              <span class="rating-badge meta" class:green={metaScore >= 61} class:yellow={metaScore >= 40 && metaScore < 61} class:red={metaScore < 40}>
                <span class="rating-value">{metaScore}</span>
              </span>
            {/if}
          </div>

          {#if movie.genres && movie.genres.length > 0}
            <div class="genres">
              {#each movie.genres as genre (genre)}
                <span class="genre-tag">{genre}</span>
              {/each}
            </div>
          {/if}

          <p class="description" class:truncated={!showMore}>
            {movie.description_full || movie.synopsis || movie.summary || "No description available."}
          </p>

          <button class="more-btn" onclick={() => showMore = !showMore}>
            More Info
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M11 17h2v-6h-2v6zm1-15C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8zM11 9h2V7h-2v2z"/>
            </svg>
          </button>
        </div>
      </div>

      <!-- Franchise Movies -->
      {#if franchiseMovies.length > 0 && movie?.franchise}
        <section class="suggestions-section franchise-section">
          <h2>{movie.franchise}</h2>
          <div class="suggestions-grid">
            {#each franchiseMovies as franchiseMovie (franchiseMovie.id)}
              <MovieCard movie={franchiseMovie} onclick={() => handleSuggestionClick(franchiseMovie)} />
            {/each}
          </div>
        </section>
      {/if}

      <!-- Similar Movies -->
      {#if suggestions.length > 0 && franchiseMovies.length === 0}
        <section class="suggestions-section">
          <h2>Similar Movies</h2>
          <div class="suggestions-grid">
            {#each suggestions as suggestion (suggestion.id)}
              <MovieCard movie={suggestion} onclick={() => handleSuggestionClick(suggestion)} />
            {/each}
          </div>
        </section>
      {/if}
    </main>

    <!-- More Info Modal -->
    {#if showMore}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="modal-overlay" onclick={() => showMore = false} onkeydown={handleModalKeydown}>
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="modal-content" onclick={(e) => e.stopPropagation()}>
          <div class="modal-header">
            <h2>{movie.title}</h2>
            <button class="modal-close" onclick={() => showMore = false}>
              <svg viewBox="0 0 24 24" fill="currentColor"><path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/></svg>
            </button>
          </div>

          <div class="modal-body" bind:this={modalBodyEl}>
            <p class="modal-description">
              {movie.description_full || movie.synopsis || movie.summary || ""}
            </p>

            <div class="modal-details">
              {#if movie.director}
                <div class="crew-info">
                  <span class="crew-label">Director</span>
                  <span class="crew-value">{movie.director}</span>
                </div>
              {/if}
              {#if movie.writers && movie.writers.length > 0}
                <div class="crew-info">
                  <span class="crew-label">Writers</span>
                  <span class="crew-value">{movie.writers.slice(0, 3).join(', ')}</span>
                </div>
              {/if}
              {#if movie.cast && movie.cast.length > 0}
                <div class="crew-info">
                  <span class="crew-label">Stars</span>
                  <span class="crew-value">{movie.cast.slice(0, 3).map(c => c.name).join(', ')}</span>
                </div>
              {/if}
              {#if movie.country}
                <div class="crew-info">
                  <span class="crew-label">Country</span>
                  <span class="crew-value">{movie.country}</span>
                </div>
              {/if}
              {#if movie.awards && movie.awards !== "N/A"}
                <div class="crew-info">
                  <span class="crew-label">Awards</span>
                  <span class="crew-value">{movie.awards}</span>
                </div>
              {/if}
              {#if movie.budget}
                <div class="crew-info">
                  <span class="crew-label">Budget</span>
                  <span class="crew-value">{movie.budget}</span>
                </div>
              {/if}
              {#if movie.box_office_gross}
                <div class="crew-info">
                  <span class="crew-label">Box Office</span>
                  <span class="crew-value">{movie.box_office_gross}</span>
                </div>
              {/if}
            </div>

            {#if movie.cast && movie.cast.length > 0}
              <div class="cast-section">
                <h3>Cast</h3>
                <div class="cast-list">
                  {#each movie.cast.slice(0, 8) as actor (actor.imdb_code)}
                    <div class="cast-member">
                      {#if actor.url_small_image}
                        <img src={actor.url_small_image} alt={actor.name} class="cast-image" />
                      {:else}
                        <div class="cast-placeholder">
                          <svg viewBox="0 0 24 24" fill="currentColor">
                            <path d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"/>
                          </svg>
                        </div>
                      {/if}
                      <div class="cast-info">
                        <span class="cast-name">{actor.name}</span>
                        <span class="cast-character">{actor.character_name}</span>
                      </div>
                    </div>
                  {/each}
                </div>
              </div>
            {/if}
          </div>
        </div>
      </div>
    {/if}
  {/if}
</div>

<style>
  .page {
    min-height: 100vh;
    background: #141414;
    color: #fff;
    position: relative;
    margin-left: 70px;
  }

  .backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    height: 80vh;
    background-size: cover;
    background-position: center top;
    z-index: 0;
  }

  .backdrop-overlay {
    position: absolute;
    inset: 0;
    background: linear-gradient(
      to right,
      rgba(20, 20, 20, 0.98) 0%,
      rgba(20, 20, 20, 0.8) 40%,
      rgba(20, 20, 20, 0.4) 100%
    ),
    linear-gradient(
      to top,
      #141414 0%,
      rgba(20, 20, 20, 0.9) 30%,
      transparent 100%
    );
  }

  .nav {
    position: relative;
    z-index: 10;
    padding: 12px 32px;
  }

  .back-btn {
    width: 44px;
    height: 44px;
    background: rgba(0, 0, 0, 0.5);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 50%;
    color: #fff;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
  }

  .back-btn:hover {
    background: rgba(0, 0, 0, 0.8);
    border-color: rgba(255, 255, 255, 0.3);
  }

  .back-btn:focus {
    outline: none;
    box-shadow: 0 0 0 3px rgba(255, 255, 255, 0.3);
  }

  .back-btn svg {
    width: 22px;
    height: 22px;
  }

  .content {
    position: relative;
    z-index: 10;
    padding: 0 32px 24px;
  }

  /* Main layout: poster-column + info side by side */
  .movie-info {
    display: flex;
    gap: 24px;
  }

  .poster-column {
    flex-shrink: 0;
    width: 220px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  /* Quality selector under poster */
  .quality-section {
    margin-top: 4px;
  }

  .quality-options {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .quality-btn {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 6px 10px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #fff;
    cursor: pointer;
    transition: all 0.2s ease;
    width: 100%;
    font-size: 0.85rem;
  }

  .quality-btn:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .quality-btn:focus,
  .quality-btn:focus-visible {
    outline: none;
    border-color: #e50914;
    box-shadow: 0 0 0 2px rgba(229, 9, 20, 0.5);
  }

  .quality-btn.selected {
    background: rgba(229, 9, 20, 0.15);
    border-color: #e50914;
  }

  .quality-label {
    font-size: 1rem;
    font-weight: 700;
  }

  .quality-meta {
    font-size: 0.8rem;
    color: #888;
  }

  /* Play button */
  .play-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    width: 100%;
    padding: 10px 16px;
    background: #e50914;
    border: none;
    border-radius: 6px;
    color: #fff;
    font-size: 0.95rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .play-btn:hover:not(:disabled) {
    background: #f40612;
  }

  .play-btn:focus,
  .play-btn:focus-visible {
    outline: none;
    box-shadow: 0 0 0 4px #fff, 0 0 0 7px #e50914;
    transform: scale(1.02);
  }

  .play-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .play-btn svg {
    width: 24px;
    height: 24px;
  }

  .btn-spinner {
    width: 20px;
    height: 20px;
    border: 3px solid rgba(255, 255, 255, 0.3);
    border-top-color: #fff;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* Secondary action buttons */
  .secondary-actions {
    display: flex;
    gap: 8px;
  }

  .action-btn {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 10px 8px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    color: #aaa;
    font-size: 0.75rem;
    cursor: pointer;
    text-decoration: none;
    transition: all 0.2s ease;
  }

  .action-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #fff;
  }

  .action-btn:focus,
  .action-btn:focus-visible {
    outline: none;
    border-color: #e50914;
    box-shadow: 0 0 0 2px rgba(229, 9, 20, 0.5);
  }

  .action-btn svg {
    width: 22px;
    height: 22px;
  }

  .action-btn.active {
    color: #2ecc71;
    border-color: rgba(46, 204, 113, 0.3);
  }

  .stream-error {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    background: rgba(220, 38, 38, 0.15);
    border: 1px solid rgba(220, 38, 38, 0.3);
    border-radius: 8px;
    color: #ef4444;
    font-size: 0.85rem;
  }

  .stream-error svg {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
  }

  /* Right column: Info */
  .info {
    flex: 1;
    max-width: 800px;
  }

  .title {
    font-size: 1.6rem;
    font-weight: 700;
    margin: 0 0 6px;
    line-height: 1.1;
  }

  .meta {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px;
    margin-bottom: 8px;
    font-size: 0.85rem;
    color: #aaa;
  }

  .mpa-rating {
    padding: 2px 8px;
    border: 1px solid #666;
    border-radius: 4px;
    font-size: 0.85rem;
  }

  /* Ratings */
  .ratings-row {
    display: flex;
    gap: 8px;
    margin-bottom: 10px;
  }

  .rating-badge {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    background: rgba(255, 255, 255, 0.06);
    border-radius: 4px;
  }

  .rating-badge .rating-label {
    font-size: 0.65rem;
    color: #888;
    text-transform: uppercase;
  }

  .rating-badge .rating-value {
    font-size: 0.9rem;
    font-weight: 700;
    color: #fff;
  }

  .rating-badge.imdb .rating-value { color: #f5c518; }
  .rating-badge.rt .rating-value { color: #fa320a; }

  .rating-badge.meta .rating-value {
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 0.9rem;
  }

  .rating-badge.meta.green .rating-value { background: #66cc33; color: #fff; }
  .rating-badge.meta.yellow .rating-value { background: #ffcc33; color: #000; }
  .rating-badge.meta.red .rating-value { background: #ff0000; color: #fff; }

  /* Genres */
  .genres {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 8px;
  }

  .genre-tag {
    padding: 3px 10px;
    background: rgba(255, 255, 255, 0.08);
    border-radius: 12px;
    font-size: 0.75rem;
    color: #ccc;
  }

  /* Description */
  .description {
    font-size: 0.85rem;
    line-height: 1.5;
    color: #999;
    margin: 0 0 6px;
  }

  .description.truncated {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  /* More button */
  .more-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 12px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 6px;
    color: #ccc;
    font-size: 0.8rem;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .more-btn:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #fff;
  }

  .more-btn:focus,
  .more-btn:focus-visible {
    outline: none;
    border-color: #e50914;
    box-shadow: 0 0 0 2px rgba(229, 9, 20, 0.5);
  }

  .more-btn svg {
    width: 16px;
    height: 16px;
  }

  /* Modal overlay */
  .modal-overlay {
    position: fixed;
    inset: 0;
    z-index: 100;
    background: rgba(0, 0, 0, 0.85);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .modal-content {
    background: #1a1a1a;
    border-radius: 12px;
    width: 90%;
    max-width: 700px;
    max-height: 85vh;
    display: flex;
    flex-direction: column;
    padding: 24px 24px 0;
    border: 1px solid rgba(255, 255, 255, 0.1);
    overflow: hidden;
  }

  .modal-body {
    overflow-y: auto;
    flex: 1;
    padding-bottom: 24px;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .modal-header h2 {
    font-size: 1.3rem;
    font-weight: 700;
    margin: 0;
    color: #fff;
  }

  .modal-close {
    width: 36px;
    height: 36px;
    background: rgba(255, 255, 255, 0.1);
    border: none;
    border-radius: 50%;
    color: #aaa;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
  }

  .modal-close:hover { background: rgba(255, 255, 255, 0.2); color: #fff; }
  .modal-close:focus { outline: none; box-shadow: 0 0 0 2px #e50914; }
  .modal-close svg { width: 20px; height: 20px; }

  .modal-description {
    font-size: 0.85rem;
    line-height: 1.6;
    color: #999;
    margin: 0 0 16px;
  }

  .modal-details {
    margin-bottom: 16px;
  }

  .crew-info {
    display: flex;
    gap: 10px;
    margin-bottom: 6px;
    font-size: 0.85rem;
  }

  .crew-label {
    color: #666;
    min-width: 70px;
    flex-shrink: 0;
  }

  .crew-value {
    color: #ccc;
  }

  /* Cast */
  .cast-section {
    margin-top: 16px;
  }

  .cast-section h3 {
    font-size: 1rem;
    font-weight: 600;
    margin: 0 0 10px;
    color: #fff;
  }

  .cast-list {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
  }

  .cast-member {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 6px;
  }

  .cast-image {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    object-fit: cover;
  }

  .cast-placeholder {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.1);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .cast-placeholder svg {
    width: 18px;
    height: 18px;
    color: #666;
  }

  .cast-info {
    display: flex;
    flex-direction: column;
  }

  .cast-name {
    font-size: 0.85rem;
    font-weight: 500;
    color: #fff;
  }

  .cast-character {
    font-size: 0.75rem;
    color: #888;
  }

  /* Coming soon */
  .coming-soon-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .coming-soon-badge {
    display: inline-block;
    padding: 6px 12px;
    background: #e50914;
    color: #fff;
    font-weight: 600;
    font-size: 0.8rem;
    border-radius: 16px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    text-align: center;
  }

  .release-date {
    font-size: 0.85rem;
    color: #ccc;
    margin: 0;
  }

  .remind-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 12px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 8px;
    color: #fff;
    font-size: 0.9rem;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .remind-btn:hover { background: rgba(255, 255, 255, 0.12); }

  .remind-btn:focus {
    outline: none;
    box-shadow: 0 0 0 3px rgba(229, 9, 20, 0.5);
  }

  .remind-btn svg { width: 20px; height: 20px; }

  .remind-btn.active {
    background: rgba(46, 204, 113, 0.15);
    border-color: #2ecc71;
    color: #2ecc71;
  }

  /* Suggestions / Franchise */
  .suggestions-section {
    margin-top: 32px;
  }

  .suggestions-section h2 {
    font-size: 1.3rem;
    font-weight: 600;
    margin: 0 0 16px;
    color: #fff;
  }

  .suggestions-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 16px;
  }

  /* Loading / Error */
  .loading-container,
  .error-container {
    position: relative;
    z-index: 10;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 80vh;
    gap: 20px;
  }

  .spinner {
    width: 50px;
    height: 50px;
    border: 4px solid rgba(229, 9, 20, 0.2);
    border-top-color: #e50914;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  .error-container > svg {
    width: 60px;
    height: 60px;
    color: #e50914;
  }

  .error-container p {
    font-size: 1.1rem;
    color: #888;
    margin: 0;
  }

  .error-buttons {
    display: flex;
    gap: 12px;
    margin-top: 16px;
  }

  .back-button,
  .retry-button {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 12px 24px;
    border-radius: 8px;
    font-size: 0.95rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .back-button {
    background: transparent;
    border: 2px solid rgba(255, 255, 255, 0.3);
    color: #fff;
  }

  .back-button:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .back-button svg {
    width: 18px;
    height: 18px;
  }

  .retry-button {
    background: #e50914;
    border: 2px solid #e50914;
    color: #fff;
  }

  .retry-button:hover {
    background: #f40612;
  }

  /* Responsive */
  @media (max-width: 900px) {
    .page { margin-left: 60px; }
  }

  @media (max-width: 768px) {
    .nav { padding: 16px; }
    .content { padding: 0 16px 32px; }
    .title { font-size: 1.8rem; }
    .suggestions-grid { grid-template-columns: repeat(auto-fill, minmax(130px, 1fr)); }
  }

  @media (max-width: 600px) {
    .page { margin-left: 0; }
  }
</style>
