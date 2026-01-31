<script lang="ts">
  import type { Movie } from "$lib/api/types";
  import { goto } from "$app/navigation";
  import { favoritesStore } from "$lib/stores/favorites.svelte";

  interface Props {
    title: string;
    movies: Movie[];
    loading?: boolean;
  }

  let { title, movies, loading = false }: Props = $props();

  function handleFavoriteClick(e: MouseEvent, movie: Movie) {
    e.stopPropagation();
    favoritesStore.toggle(movie);
  }

  let scrollContainer: HTMLDivElement;
  let showLeftArrow = $state(false);
  let showRightArrow = $state(true);

  function handleScroll() {
    if (!scrollContainer) return;
    showLeftArrow = scrollContainer.scrollLeft > 0;
    showRightArrow =
      scrollContainer.scrollLeft <
      scrollContainer.scrollWidth - scrollContainer.clientWidth - 10;
  }

  function scrollLeft() {
    scrollContainer?.scrollBy({ left: -900, behavior: "smooth" });
  }

  function scrollRight() {
    scrollContainer?.scrollBy({ left: 900, behavior: "smooth" });
  }

  function handleMovieClick(movie: Movie) {
    goto(`/movie/${movie.id}`);
  }
</script>

<section class="movie-row">
  <h2 class="row-title">{title}</h2>

  <div class="row-container">
    {#if showLeftArrow}
      <button class="scroll-btn scroll-left" onclick={scrollLeft} aria-label="Scroll left">
        <svg viewBox="0 0 24 24" fill="currentColor">
          <path d="M15.41 7.41L14 6l-6 6 6 6 1.41-1.41L10.83 12z"/>
        </svg>
      </button>
    {/if}

    <div
      class="movies-scroll"
      bind:this={scrollContainer}
      onscroll={handleScroll}
    >
      {#if loading}
        {#each Array(6) as _, i (i)}
          <div class="movie-card-skeleton">
            <div class="skeleton-poster"></div>
          </div>
        {/each}
      {:else}
        {#each movies as movie (movie.id)}
          <div
            class="movie-card"
            onclick={() => handleMovieClick(movie)}
            onkeydown={(e) => e.key === 'Enter' && handleMovieClick(movie)}
            role="button"
            tabindex="0"
            aria-label="View {movie.title}"
          >
            <div class="card-poster">
              <img
                src={movie.medium_cover_image || movie.small_cover_image}
                alt={movie.title}
                loading="lazy"
              />
              <div class="card-overlay">
                <button
                  class="card-favorite"
                  class:active={favoritesStore.isFavorite(movie.id)}
                  onclick={(e) => handleFavoriteClick(e, movie)}
                  aria-label={favoritesStore.isFavorite(movie.id) ? "Remove from My List" : "Add to My List"}
                >
                  <svg viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/>
                  </svg>
                </button>
                <div class="card-play">
                  <svg viewBox="0 0 24 24" fill="currentColor">
                    <path d="M8 5v14l11-7z"/>
                  </svg>
                </div>
              </div>
            </div>
            <div class="card-info">
              <h3 class="card-title">{movie.title}</h3>
              <div class="card-meta">
                <span class="card-year">{movie.year}</span>
                <span class="card-rating">
                  <svg viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
                  </svg>
                  {movie.rating ? movie.rating.toFixed(1) : "N/A"}
                </span>
              </div>
            </div>
          </div>
        {/each}
      {/if}
    </div>

    {#if showRightArrow && movies.length > 4}
      <button class="scroll-btn scroll-right" onclick={scrollRight} aria-label="Scroll right">
        <svg viewBox="0 0 24 24" fill="currentColor">
          <path d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z"/>
        </svg>
      </button>
    {/if}
  </div>
</section>

<style>
  .movie-row {
    margin-bottom: 50px;
  }

  .row-title {
    font-size: 1.6rem;
    font-weight: 600;
    margin: 0 0 20px;
    padding: 0 80px;
    color: #fff;
  }

  .row-container {
    position: relative;
  }

  .movies-scroll {
    display: flex;
    gap: 12px;
    overflow-x: auto;
    scroll-behavior: smooth;
    padding: 10px 80px 30px;
    scrollbar-width: none;
  }

  .movies-scroll::-webkit-scrollbar {
    display: none;
  }

  .scroll-btn {
    position: absolute;
    top: 10px;
    bottom: 30px;
    width: 70px;
    background: rgba(20, 20, 20, 0.8);
    border: none;
    color: #fff;
    cursor: pointer;
    z-index: 10;
    opacity: 0;
    transition: opacity 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .row-container:hover .scroll-btn {
    opacity: 1;
  }

  .scroll-btn:hover {
    background: rgba(20, 20, 20, 0.95);
  }

  .scroll-btn:focus {
    outline: none;
    opacity: 1;
    background: rgba(20, 20, 20, 0.95);
  }

  .scroll-btn svg {
    width: 48px;
    height: 48px;
  }

  .scroll-left {
    left: 0;
  }

  .scroll-right {
    right: 0;
  }

  .movie-card {
    flex-shrink: 0;
    width: 220px;
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    text-align: left;
    transition: transform 0.3s ease;
  }

  .movie-card:hover {
    transform: scale(1.08);
    z-index: 20;
  }

  .movie-card:focus {
    outline: none;
    transform: scale(1.08);
    z-index: 20;
  }

  .movie-card:focus .card-poster {
    box-shadow: 0 0 0 4px #fff;
  }

  .card-poster {
    position: relative;
    aspect-ratio: 2/3;
    border-radius: 6px;
    overflow: hidden;
    background: #2a2a2a;
  }

  .card-poster img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform 0.3s ease;
  }

  .movie-card:hover .card-poster img {
    transform: scale(1.05);
  }

  .card-overlay {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    opacity: 0;
    transition: opacity 0.3s ease;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .movie-card:hover .card-overlay,
  .movie-card:focus .card-overlay {
    opacity: 1;
  }

  .card-favorite {
    position: absolute;
    top: 10px;
    right: 10px;
    width: 36px;
    height: 36px;
    background: rgba(0, 0, 0, 0.6);
    border: none;
    border-radius: 50%;
    color: #fff;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0;
    transform: scale(0.8);
    transition: all 0.2s ease;
  }

  .movie-card:hover .card-favorite,
  .movie-card:focus .card-favorite {
    opacity: 1;
    transform: scale(1);
  }

  .card-favorite:hover {
    background: rgba(0, 0, 0, 0.8);
    transform: scale(1.1);
  }

  .card-favorite.active {
    color: #e50914;
    opacity: 1;
  }

  .card-favorite svg {
    width: 20px;
    height: 20px;
  }

  .card-play {
    width: 60px;
    height: 60px;
    background: rgba(255, 255, 255, 0.95);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    transform: scale(0);
    transition: transform 0.3s ease;
  }

  .movie-card:hover .card-play,
  .movie-card:focus .card-play {
    transform: scale(1);
  }

  .card-play svg {
    width: 30px;
    height: 30px;
    color: #141414;
    margin-left: 4px;
  }

  .card-info {
    padding: 12px 4px;
  }

  .card-title {
    font-size: 1rem;
    font-weight: 500;
    margin: 0 0 6px;
    color: #fff;
    line-height: 1.3;
    display: -webkit-box;
    -webkit-line-clamp: 1;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .card-meta {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 0.9rem;
    color: #808080;
  }

  .card-rating {
    display: flex;
    align-items: center;
    gap: 4px;
    color: #46d369;
  }

  .card-rating svg {
    width: 14px;
    height: 14px;
  }

  /* Skeleton loading */
  .movie-card-skeleton {
    flex-shrink: 0;
    width: 220px;
  }

  .skeleton-poster {
    aspect-ratio: 2/3;
    background: linear-gradient(
      90deg,
      #2a2a2a 25%,
      #3a3a3a 50%,
      #2a2a2a 75%
    );
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
    border-radius: 6px;
  }

  @keyframes shimmer {
    0% {
      background-position: 200% 0;
    }
    100% {
      background-position: -200% 0;
    }
  }

  @media (max-width: 1200px) {
    .row-title {
      padding: 0 60px;
      font-size: 1.4rem;
    }

    .movies-scroll {
      padding: 10px 60px 24px;
    }

    .scroll-btn {
      width: 60px;
    }

    .scroll-btn svg {
      width: 40px;
      height: 40px;
    }

    .movie-card {
      width: 180px;
    }
  }

  @media (max-width: 900px) {
    .row-title {
      padding: 0 40px;
      font-size: 1.3rem;
    }

    .movies-scroll {
      padding: 8px 40px 20px;
      gap: 10px;
    }

    .scroll-btn {
      width: 50px;
    }

    .scroll-btn svg {
      width: 32px;
      height: 32px;
    }

    .movie-card {
      width: 160px;
    }

    .card-play {
      width: 50px;
      height: 50px;
    }

    .card-play svg {
      width: 24px;
      height: 24px;
    }
  }

  @media (max-width: 600px) {
    .row-title {
      padding: 0 24px;
      font-size: 1.2rem;
    }

    .movies-scroll {
      padding: 8px 24px 16px;
      gap: 10px;
    }

    .scroll-btn {
      display: none;
    }

    .movie-card {
      width: 140px;
    }

    .movie-card:hover {
      transform: none;
    }

    .card-title {
      font-size: 0.9rem;
    }

    .card-meta {
      font-size: 0.8rem;
    }
  }
</style>
