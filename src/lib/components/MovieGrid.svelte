<script lang="ts">
  import type { Movie } from "$lib/api/types";
  import MovieCard from "./MovieCard.svelte";
  import { goto } from "$app/navigation";

  interface Props {
    movies: Movie[];
    loading?: boolean;
    error?: string | null;
  }

  let { movies, loading = false, error = null }: Props = $props();

  function handleMovieClick(movie: Movie) {
    goto(`/movie/${movie.id}`);
  }
</script>

{#if loading}
  <div class="loading-container">
    <div class="spinner"></div>
    <p class="loading-text">Loading movies...</p>
  </div>
{:else if error}
  <div class="error-container">
    <svg class="error-icon" viewBox="0 0 24 24" fill="currentColor">
      <path
        d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"
      />
    </svg>
    <p class="error-text">{error}</p>
  </div>
{:else if movies.length === 0}
  <div class="empty-container">
    <svg class="empty-icon" viewBox="0 0 24 24" fill="currentColor">
      <path
        d="M18 4l2 4h-3l-2-4h-2l2 4h-3l-2-4H8l2 4H7L5 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V4h-4z"
      />
    </svg>
    <p class="empty-text">No movies found</p>
  </div>
{:else}
  <div class="movie-grid">
    {#each movies as movie (movie.id)}
      <MovieCard {movie} onclick={() => handleMovieClick(movie)} />
    {/each}
  </div>
{/if}

<style>
  .movie-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 24px;
    padding: 20px 0;
  }

  .loading-container,
  .error-container,
  .empty-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 400px;
    gap: 20px;
  }

  .spinner {
    width: 60px;
    height: 60px;
    border: 4px solid rgba(233, 69, 96, 0.2);
    border-top-color: #e94560;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .loading-text {
    font-size: 1.4rem;
    color: #888;
  }

  .error-icon {
    width: 64px;
    height: 64px;
    color: #e94560;
  }

  .error-text {
    font-size: 1.4rem;
    color: #e94560;
    text-align: center;
    max-width: 400px;
  }

  .empty-icon {
    width: 64px;
    height: 64px;
    color: #555;
  }

  .empty-text {
    font-size: 1.4rem;
    color: #888;
  }

  /* TV-optimized styles */
  @media (min-width: 1920px) {
    .movie-grid {
      grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
      gap: 32px;
      padding: 30px 0;
    }

    .spinner {
      width: 80px;
      height: 80px;
      border-width: 6px;
    }

    .loading-text,
    .error-text,
    .empty-text {
      font-size: 1.8rem;
    }

    .error-icon,
    .empty-icon {
      width: 80px;
      height: 80px;
    }
  }

  /* Large TV screens */
  @media (min-width: 2560px) {
    .movie-grid {
      grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
      gap: 40px;
    }
  }
</style>
