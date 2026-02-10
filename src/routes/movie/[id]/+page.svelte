<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { browser } from "$app/environment";
  import MovieCard from "$lib/components/MovieCard.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import { getMovieDetails, getMovieSuggestions, getMovieRating, startStream, syncMovieToLocal, getFranchiseMovies } from "$lib/api/commands";
  import { streamStore } from "$lib/stores/stream.svelte";
  import { favoritesStore } from "$lib/stores/favorites.svelte";
  import { remindersStore } from "$lib/stores/reminders.svelte";
  import type { MovieDetails, Movie, MovieRating, Torrent } from "$lib/api/types";

  let movieId = $derived(parseInt($page.params.id || "0", 10));

  let movie = $state<MovieDetails | null>(null);
  let suggestions = $state<Movie[]>([]);
  let franchiseMovies = $state<Movie[]>([]);
  let rating = $state<MovieRating | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let selectedTorrent = $state<Torrent | null>(null);
  let isStartingStream = $state(false);
  let streamError = $state<string | null>(null);

  async function loadMovie(id: number) {
    loading = true;
    error = null;
    selectedTorrent = null;
    rating = null;
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

      // Fetch rating from OMDB in background
      if (movie.imdb_code) {
        getMovieRating(movie.imdb_code)
          .then((r) => {
            rating = r;
          })
          .catch((err) => {
            console.warn("Failed to fetch rating:", err);
          });
      }

      // Sync movie to local database if it came from external search (YTS)
      // This grows the DB organically when users discover new movies
      if (movie.provider === 'yts' || !movie.provider) {
        syncMovieToLocal(movie);
      }
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

  // Reload when movie ID changes (skip invalid IDs)
  $effect(() => {
    if (movieId && movieId > 0) {
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
      <!-- Movie Info Section -->
      <div class="movie-info">
        <div class="poster-container">
          <img
            src={movie.large_cover_image || movie.medium_cover_image}
            alt="{movie.title} poster"
            class="poster"
          />
        </div>

        <div class="info">
          <h1 class="title">{movie.title}</h1>

          <div class="meta">
            <span class="year">{movie.year}</span>
            {#if movie.runtime}
              <span class="runtime">{formatRuntime(movie.runtime)}</span>
            {/if}
            {#if rating?.rated || movie.mpa_rating}
              <span class="mpa-rating">{rating?.rated || movie.mpa_rating}</span>
            {/if}
            {#if rating?.language || movie.language}
              <span class="language">{rating?.language?.split(', ')[0] || movie.language}</span>
            {/if}
          </div>

          <!-- Ratings Row -->
          <div class="ratings-row">
            {#if movie.imdb_rating || rating?.imdb_rating}
              <span class="rating-badge imdb">
                <span class="rating-label">IMDb</span>
                <span class="rating-value">{(movie.imdb_rating || rating?.imdb_rating || 0).toFixed(1)}</span>
                {#if movie.imdb_votes || rating?.imdb_votes}
                  <span class="rating-votes">{movie.imdb_votes || rating?.imdb_votes}</span>
                {/if}
              </span>
            {/if}
            {#if movie.rotten_tomatoes || rating?.rotten_tomatoes}
              <span class="rating-badge rt">
                <span class="rating-label">🍅 Rotten</span>
                <span class="rating-value">{movie.rotten_tomatoes || rating?.rotten_tomatoes}%</span>
              </span>
            {/if}
            {#if movie.metacritic || rating?.metascore}
              {@const metaScore = movie.metacritic || rating?.metascore || 0}
              <span class="rating-badge meta" class:green={metaScore >= 61} class:yellow={metaScore >= 40 && metaScore < 61} class:red={metaScore < 40}>
                <span class="rating-label">Metascore</span>
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

          <!-- Director & Cast & Writers -->
          {#if movie.director || rating?.director}
            <div class="crew-info">
              <span class="crew-label">Director</span>
              <span class="crew-value">{movie.director || rating?.director}</span>
            </div>
          {/if}
          {#if movie.writers && movie.writers.length > 0}
            <div class="crew-info">
              <span class="crew-label">Writers</span>
              <span class="crew-value">{movie.writers.slice(0, 3).join(', ')}</span>
            </div>
          {/if}
          {#if rating?.actors || (movie.cast && movie.cast.length > 0)}
            <div class="crew-info">
              <span class="crew-label">Stars</span>
              <span class="crew-value">{rating?.actors || movie.cast?.slice(0, 3).map(c => c.name).join(', ')}</span>
            </div>
          {/if}
          {#if movie.country || rating?.country}
            <div class="crew-info">
              <span class="crew-label">Country</span>
              <span class="crew-value">{movie.country || rating?.country}</span>
            </div>
          {/if}

          <p class="description">
            {rating?.plot || movie.description_full || movie.synopsis || movie.summary || "No description available."}
          </p>

          <!-- Extra Info -->
          {#if (movie.awards && movie.awards !== "N/A") || (rating?.awards && rating.awards !== "N/A")}
            <div class="awards">
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 2C9.24 2 7 4.24 7 7v2H5v2h2v9c0 1.1.9 2 2 2h6c1.1 0 2-.9 2-2v-9h2V9h-2V7c0-2.76-2.24-5-5-5zm-2 5c0-1.1.9-2 2-2s2 .9 2 2v2H10V7zm5 13H9v-8h6v8z"/>
              </svg>
              {movie.awards || rating?.awards}
            </div>
          {/if}
          {#if movie.budget}
            <div class="budget-info">
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M11.8 10.9c-2.27-.59-3-1.2-3-2.15 0-1.09 1.01-1.85 2.7-1.85 1.78 0 2.44.85 2.5 2.1h2.21c-.07-1.72-1.12-3.3-3.21-3.81V3h-3v2.16c-1.94.42-3.5 1.68-3.5 3.61 0 2.31 1.91 3.46 4.7 4.13 2.5.6 3 1.48 3 2.41 0 .69-.49 1.79-2.7 1.79-2.06 0-2.87-.92-2.98-2.1h-2.2c.12 2.19 1.76 3.42 3.68 3.83V21h3v-2.15c1.95-.37 3.5-1.5 3.5-3.55 0-2.84-2.43-3.81-4.7-4.4z"/>
              </svg>
              Budget: {movie.budget}
            </div>
          {/if}
          {#if movie.box_office_gross || rating?.box_office}
            <div class="box-office">
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1.41 16.09V20h-2.67v-1.93c-1.71-.36-3.16-1.46-3.27-3.4h1.96c.1 1.05.82 1.87 2.65 1.87 1.96 0 2.4-.98 2.4-1.59 0-.83-.44-1.61-2.67-2.14-2.48-.6-4.18-1.62-4.18-3.67 0-1.72 1.39-2.84 3.11-3.21V4h2.67v1.95c1.86.45 2.79 1.86 2.85 3.39H14.3c-.05-1.11-.64-1.87-2.22-1.87-1.5 0-2.4.68-2.4 1.64 0 .84.65 1.39 2.67 1.91s4.18 1.39 4.18 3.91c-.01 1.83-1.38 2.83-3.12 3.16z"/>
              </svg>
              Box Office: {movie.box_office_gross || rating?.box_office}
            </div>
          {/if}

          <!-- Coming Soon Section -->
          {#if movie.status === 'coming_soon'}
            <div class="coming-soon-section">
              <div class="coming-soon-badge">Coming Soon</div>
              {#if movie.release_date}
                <p class="release-date">Expected Release: {new Date(movie.release_date).toLocaleDateString('en-US', { year: 'numeric', month: 'long', day: 'numeric' })}</p>
              {/if}
              <div class="remind-section">
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
                {#if movie.yt_trailer_code}
                  <a
                    href={getYouTubeUrl(movie.yt_trailer_code)}
                    target="_blank"
                    rel="noopener noreferrer"
                    class="trailer-btn"
                  >
                    <svg viewBox="0 0 24 24" fill="currentColor">
                      <path d="M10 15l5.19-3L10 9v6m11.56-7.83c.13.47.22 1.1.28 1.9.07.8.1 1.49.1 2.09L22 12c0 2.19-.16 3.8-.44 4.83-.25.9-.83 1.48-1.73 1.73-.47.13-1.33.22-2.65.28-1.3.07-2.49.1-3.59.1L12 19c-4.19 0-6.8-.16-7.83-.44-.9-.25-1.48-.83-1.73-1.73-.13-.47-.22-1.1-.28-1.9-.07-.8-.1-1.49-.1-2.09L2 12c0-2.19.16-3.8.44-4.83.25-.9.83-1.48 1.73-1.73.47-.13 1.33-.22 2.65-.28 1.3-.07 2.49-.1 3.59-.1L12 5c4.19 0 6.8.16 7.83.44.9.25 1.48.83 1.73 1.73z"/>
                    </svg>
                    Watch Trailer
                  </a>
                {/if}
              </div>
            </div>
          {:else if movie.torrents && movie.torrents.filter(t => t.seeds > 0).length > 0}
          <!-- Quality Selector - only show torrents with seeds -->
            <div class="quality-section">
              <h3>Select Quality</h3>
              <div class="quality-options">
                {#each movie.torrents.filter(t => t.seeds > 0) as torrent (torrent.hash)}
                  <button
                    class="quality-btn"
                    class:selected={selectedTorrent?.hash === torrent.hash}
                    onclick={() => selectQuality(torrent)}
                  >
                    <span class="quality-label">{torrent.quality}</span>
                    <span class="quality-type">{torrent.type || "BluRay"}</span>
                    <span class="quality-size">{formatSize(torrent.size)}</span>
                    <span class="quality-seeds">
                      <svg viewBox="0 0 24 24" fill="currentColor">
                        <path d="M16 6l2.29 2.29-4.88 4.88-4-4L2 16.59 3.41 18l6-6 4 4 6.3-6.29L22 12V6z"/>
                      </svg>
                      {torrent.seeds}
                    </span>
                  </button>
                {/each}
              </div>
            </div>

            <!-- Play Button -->
            <div class="play-section">
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

              <button
                class="favorite-btn"
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
                {favoritesStore.isFavorite(movie.id) ? "In My List" : "My List"}
              </button>

              {#if movie.yt_trailer_code}
                <a
                  href={getYouTubeUrl(movie.yt_trailer_code)}
                  target="_blank"
                  rel="noopener noreferrer"
                  class="trailer-btn"
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

          <!-- Cast -->
          {#if movie.cast && movie.cast.length > 0}
            <div class="cast-section">
              <h3>Cast</h3>
              <div class="cast-list">
                {#each movie.cast.slice(0, 6) as actor (actor.imdb_code)}
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

      <!-- Franchise Movies (if part of a franchise) -->
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

      <!-- Similar Movies (only show if no franchise) -->
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
    padding: 30px 50px;
  }

  .back-btn {
    width: 50px;
    height: 50px;
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
    width: 24px;
    height: 24px;
  }

  .content {
    position: relative;
    z-index: 10;
    padding: 0 50px 60px;
  }

  .movie-info {
    display: flex;
    gap: 50px;
    margin-bottom: 60px;
  }

  .poster-container {
    flex-shrink: 0;
  }

  .poster {
    width: 300px;
    border-radius: 12px;
    box-shadow: 0 8px 40px rgba(0, 0, 0, 0.6);
  }

  .info {
    flex: 1;
    max-width: 900px;
  }

  .title {
    font-size: 3rem;
    font-weight: 700;
    margin: 0 0 20px;
    line-height: 1.1;
  }

  .meta {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 16px;
    margin-bottom: 16px;
    font-size: 1.1rem;
    color: #aaa;
  }

  .mpa-rating {
    padding: 4px 10px;
    border: 1px solid #666;
    border-radius: 4px;
    font-size: 0.9rem;
  }

  .language {
    color: #888;
  }

  /* Ratings Row */
  .ratings-row {
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
    margin-bottom: 20px;
  }

  .rating-badge {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 12px 20px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    min-width: 90px;
  }

  .rating-badge .rating-label {
    font-size: 0.75rem;
    color: #888;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 4px;
  }

  .rating-badge .rating-value {
    font-size: 1.4rem;
    font-weight: 700;
    color: #fff;
  }

  .rating-badge .rating-votes {
    font-size: 0.7rem;
    color: #666;
    margin-top: 2px;
  }

  .rating-badge.imdb .rating-value {
    color: #f5c518;
  }

  .rating-badge.rt .rating-value {
    color: #fa320a;
  }

  .rating-badge.meta .rating-value {
    padding: 4px 8px;
    border-radius: 4px;
  }

  .rating-badge.meta.green .rating-value {
    background: #66cc33;
    color: #fff;
  }

  .rating-badge.meta.yellow .rating-value {
    background: #ffcc33;
    color: #000;
  }

  .rating-badge.meta.red .rating-value {
    background: #ff0000;
    color: #fff;
  }

  .genres {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    margin-bottom: 16px;
  }

  .genre-tag {
    padding: 8px 16px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 20px;
    font-size: 0.95rem;
  }

  /* Crew Info */
  .crew-info {
    display: flex;
    gap: 12px;
    margin-bottom: 8px;
    font-size: 1rem;
  }

  .crew-label {
    color: #666;
    min-width: 70px;
  }

  .crew-value {
    color: #ccc;
  }

  .description {
    font-size: 1.1rem;
    line-height: 1.7;
    color: #999;
    margin: 16px 0 20px;
  }

  /* Awards & Box Office */
  .awards, .box-office {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 16px;
    background: rgba(255, 215, 0, 0.1);
    border-radius: 8px;
    margin-bottom: 12px;
    font-size: 0.95rem;
    color: #ffd700;
  }

  .awards svg, .box-office svg {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
  }

  .box-office {
    background: rgba(46, 204, 113, 0.1);
    color: #2ecc71;
  }

  .budget-info {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 16px;
    background: rgba(52, 152, 219, 0.1);
    border-radius: 8px;
    margin-bottom: 12px;
    font-size: 0.95rem;
    color: #3498db;
  }

  .budget-info svg {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
  }

  /* Coming Soon Section */
  .coming-soon-section {
    margin-bottom: 24px;
    padding: 24px;
    background: rgba(229, 9, 20, 0.1);
    border: 1px solid rgba(229, 9, 20, 0.3);
    border-radius: 12px;
  }

  .coming-soon-badge {
    display: inline-block;
    padding: 8px 16px;
    background: #e50914;
    color: #fff;
    font-weight: 600;
    font-size: 0.9rem;
    border-radius: 20px;
    margin-bottom: 12px;
    text-transform: uppercase;
    letter-spacing: 1px;
  }

  .release-date {
    font-size: 1.1rem;
    color: #ccc;
    margin-bottom: 20px;
  }

  .remind-section {
    display: flex;
    gap: 16px;
    flex-wrap: wrap;
  }

  .remind-btn {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 18px 32px;
    background: rgba(255, 255, 255, 0.1);
    border: 2px solid rgba(255, 255, 255, 0.2);
    border-radius: 8px;
    color: #fff;
    font-size: 1.2rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .remind-btn:hover {
    background: rgba(255, 255, 255, 0.15);
    border-color: rgba(255, 255, 255, 0.3);
  }

  .remind-btn:focus {
    outline: none;
    box-shadow: 0 0 0 4px rgba(229, 9, 20, 0.5);
  }

  .remind-btn svg {
    width: 24px;
    height: 24px;
  }

  .remind-btn.active {
    background: rgba(46, 204, 113, 0.15);
    border-color: #2ecc71;
    color: #2ecc71;
  }

  .quality-section {
    margin-bottom: 24px;
  }

  .quality-section h3 {
    font-size: 1.2rem;
    font-weight: 600;
    margin: 0 0 12px;
    color: #fff;
  }

  .quality-options {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    padding-bottom: 8px;
  }

  .quality-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 14px 20px;
    background: rgba(255, 255, 255, 0.05);
    border: 2px solid rgba(255, 255, 255, 0.1);
    border-radius: 10px;
    color: #fff;
    cursor: pointer;
    transition: all 0.2s ease;
    min-width: 100px;
  }

  .quality-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.2);
  }

  .quality-btn:focus,
  .quality-btn:focus-visible {
    outline: none;
    border-color: #e50914;
    box-shadow: 0 0 0 3px rgba(229, 9, 20, 0.5);
    background: rgba(229, 9, 20, 0.2);
    transform: scale(1.05);
  }

  .quality-btn.selected {
    background: rgba(229, 9, 20, 0.15);
    border-color: #e50914;
  }

  .quality-label {
    font-size: 1.3rem;
    font-weight: 700;
  }

  .quality-type {
    font-size: 0.85rem;
    color: #888;
  }

  .quality-size {
    font-size: 0.9rem;
    color: #888;
  }

  .quality-seeds {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 0.85rem;
    color: #46d369;
  }

  .quality-seeds svg {
    width: 14px;
    height: 14px;
  }

  .play-section {
    display: flex;
    gap: 16px;
    margin-bottom: 20px;
  }

  .play-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 18px 48px;
    background: #e50914;
    border: none;
    border-radius: 8px;
    color: #fff;
    font-size: 1.3rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .play-btn:hover:not(:disabled) {
    background: #f40612;
    transform: scale(1.02);
  }

  .play-btn:focus,
  .play-btn:focus-visible {
    outline: none;
    box-shadow: 0 0 0 5px #fff, 0 0 0 8px #e50914;
    transform: scale(1.05);
  }

  .play-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .play-btn svg {
    width: 28px;
    height: 28px;
  }

  .btn-spinner {
    width: 24px;
    height: 24px;
    border: 3px solid rgba(255, 255, 255, 0.3);
    border-top-color: #fff;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .trailer-btn {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 18px 32px;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 8px;
    color: #fff;
    font-size: 1.2rem;
    font-weight: 500;
    text-decoration: none;
    transition: all 0.2s ease;
  }

  .trailer-btn:hover {
    background: rgba(255, 255, 255, 0.15);
  }

  .trailer-btn:focus,
  .trailer-btn:focus-visible {
    outline: none;
    box-shadow: 0 0 0 4px #e50914;
    background: rgba(229, 9, 20, 0.3);
    transform: scale(1.05);
  }

  .trailer-btn svg {
    width: 24px;
    height: 24px;
    color: #ff0000;
  }

  .favorite-btn {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 18px 32px;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 8px;
    color: #fff;
    font-size: 1.2rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .favorite-btn:hover {
    background: rgba(255, 255, 255, 0.15);
  }

  .favorite-btn:focus,
  .favorite-btn:focus-visible {
    outline: none;
    box-shadow: 0 0 0 4px #e50914;
    background: rgba(229, 9, 20, 0.3);
    transform: scale(1.05);
  }

  .favorite-btn svg {
    width: 24px;
    height: 24px;
  }

  .favorite-btn.active {
    background: rgba(46, 204, 113, 0.15);
    border-color: #2ecc71;
    color: #2ecc71;
  }

  .stream-error {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 14px 20px;
    background: rgba(220, 38, 38, 0.15);
    border: 1px solid rgba(220, 38, 38, 0.3);
    border-radius: 8px;
    color: #ef4444;
    font-size: 1rem;
  }

  .stream-error svg {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
  }

  .cast-section {
    margin-top: 30px;
  }

  .cast-section h3 {
    font-size: 1.2rem;
    font-weight: 600;
    margin: 0 0 16px;
    color: #fff;
  }

  .cast-list {
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
  }

  .cast-member {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 16px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 8px;
  }

  .cast-image {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    object-fit: cover;
  }

  .cast-placeholder {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.1);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .cast-placeholder svg {
    width: 24px;
    height: 24px;
    color: #666;
  }

  .cast-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .cast-name {
    font-size: 0.95rem;
    font-weight: 500;
    color: #fff;
  }

  .cast-character {
    font-size: 0.85rem;
    color: #888;
  }

  .suggestions-section {
    margin-top: 40px;
  }

  .suggestions-section h2 {
    font-size: 1.6rem;
    font-weight: 600;
    margin: 0 0 24px;
    color: #fff;
  }

  .suggestions-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 20px;
  }

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
    width: 60px;
    height: 60px;
    border: 4px solid rgba(229, 9, 20, 0.2);
    border-top-color: #e50914;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  .error-container > svg {
    width: 80px;
    height: 80px;
    color: #e50914;
  }

  .error-container p {
    font-size: 1.3rem;
    color: #888;
    margin: 0;
  }

  .error-buttons {
    display: flex;
    gap: 12px;
    margin-top: 20px;
  }

  .back-button,
  .retry-button {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 14px 32px;
    border-radius: 8px;
    font-size: 1rem;
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
    border-color: rgba(255, 255, 255, 0.5);
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
    border-color: #f40612;
  }

  @media (max-width: 1024px) {
    .nav {
      padding: 24px 30px;
    }

    .content {
      padding: 0 30px 50px;
    }

    .movie-info {
      gap: 30px;
    }

    .poster {
      width: 250px;
    }

    .title {
      font-size: 2.4rem;
    }
  }

  @media (max-width: 900px) {
    .page {
      margin-left: 60px;
    }
  }

  @media (max-width: 768px) {
    .nav {
      padding: 20px;
    }

    .content {
      padding: 0 20px 40px;
    }

    .movie-info {
      flex-direction: column;
      align-items: center;
      text-align: center;
    }

    .poster {
      width: 200px;
    }

    .info {
      display: flex;
      flex-direction: column;
      align-items: center;
    }

    .title {
      font-size: 2rem;
    }

    .meta {
      justify-content: center;
    }

    .genres {
      justify-content: center;
    }

    .quality-options {
      justify-content: flex-start;
    }

    .play-section {
      flex-direction: column;
      align-items: center;
    }

    .play-btn {
      width: 100%;
      max-width: 300px;
    }

    .trailer-btn,
    .favorite-btn {
      width: 100%;
      max-width: 300px;
      justify-content: center;
    }

    .cast-list {
      justify-content: center;
    }

    .suggestions-grid {
      grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    }
  }

  @media (max-width: 600px) {
    .page {
      margin-left: 0;
    }
  }
</style>
