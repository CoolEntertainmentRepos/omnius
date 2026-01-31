<script lang="ts">
  import type { Movie } from "$lib/api/types";

  interface Props {
    movie: Movie;
    onclick?: () => void;
  }

  let { movie, onclick }: Props = $props();

  let isFocused = $state(false);

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      onclick?.();
    }
  }
</script>

<article
  class="movie-card"
  class:focused={isFocused}
  tabindex="0"
  role="button"
  aria-label="View details for {movie.title}"
  onclick={onclick}
  onkeydown={handleKeydown}
  onfocus={() => (isFocused = true)}
  onblur={() => (isFocused = false)}
>
  <div class="poster-container">
    <img
      src={movie.medium_cover_image || movie.small_cover_image}
      alt="{movie.title} poster"
      class="poster"
      loading="lazy"
    />
    <div class="overlay">
      <span class="rating">
        <svg class="star-icon" viewBox="0 0 24 24" fill="currentColor">
          <path
            d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"
          />
        </svg>
        {movie.rating.toFixed(1)}
      </span>
    </div>
  </div>

  <div class="info">
    <h3 class="title">{movie.title}</h3>
    <p class="year">{movie.year}</p>
    {#if movie.genres && movie.genres.length > 0}
      <p class="genres">{movie.genres.slice(0, 2).join(" / ")}</p>
    {/if}
  </div>
</article>

<style>
  .movie-card {
    display: flex;
    flex-direction: column;
    background: #1a1a2e;
    border-radius: 12px;
    overflow: hidden;
    cursor: pointer;
    transition:
      transform 0.3s ease,
      box-shadow 0.3s ease,
      outline 0.2s ease;
    outline: 3px solid transparent;
  }

  .movie-card:hover,
  .movie-card.focused {
    transform: scale(1.08);
    box-shadow: 0 12px 40px rgba(229, 9, 20, 0.4);
  }

  .movie-card:focus,
  .movie-card:focus-visible {
    outline: none;
    transform: scale(1.08);
    box-shadow: 0 0 0 4px #e50914, 0 12px 40px rgba(229, 9, 20, 0.5);
  }

  .movie-card.focused {
    outline: none;
    box-shadow: 0 0 0 4px #e50914, 0 12px 40px rgba(229, 9, 20, 0.5);
  }

  .poster-container {
    position: relative;
    aspect-ratio: 2 / 3;
    overflow: hidden;
  }

  .poster {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform 0.3s ease;
  }

  .movie-card:hover .poster,
  .movie-card.focused .poster {
    transform: scale(1.1);
  }

  .overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(
      to top,
      rgba(0, 0, 0, 0.8) 0%,
      transparent 50%,
      rgba(0, 0, 0, 0.3) 100%
    );
    opacity: 0;
    transition: opacity 0.3s ease;
  }

  .movie-card:hover .overlay,
  .movie-card.focused .overlay,
  .movie-card:focus .overlay {
    opacity: 1;
  }

  .rating {
    position: absolute;
    top: 12px;
    right: 12px;
    display: flex;
    align-items: center;
    gap: 6px;
    background: rgba(0, 0, 0, 0.7);
    padding: 6px 12px;
    border-radius: 20px;
    font-size: 1.1rem;
    font-weight: 600;
    color: #ffd700;
  }

  .star-icon {
    width: 18px;
    height: 18px;
    color: #ffd700;
  }

  .info {
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .title {
    font-size: 1.2rem;
    font-weight: 600;
    color: #ffffff;
    margin: 0;
    line-height: 1.3;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .year {
    font-size: 1rem;
    color: #888;
    margin: 0;
  }

  .genres {
    font-size: 0.9rem;
    color: #e94560;
    margin: 0;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  /* TV-optimized styles */
  @media (min-width: 1920px) {
    .title {
      font-size: 1.5rem;
    }

    .year {
      font-size: 1.2rem;
    }

    .genres {
      font-size: 1.1rem;
    }

    .rating {
      font-size: 1.3rem;
      padding: 8px 16px;
    }

    .star-icon {
      width: 22px;
      height: 22px;
    }

    .info {
      padding: 20px;
      gap: 8px;
    }
  }
</style>
