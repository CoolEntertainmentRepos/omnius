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
    <!-- Rating badge top right -->
    <span class="rating">
      <svg class="star-icon" viewBox="0 0 24 24" fill="currentColor">
        <path
          d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"
        />
      </svg>
      {movie.rating.toFixed(1)}
    </span>
    <!-- Title overlay at bottom -->
    <div class="title-overlay">
      <h3 class="title">{movie.title}</h3>
      <span class="year">{movie.year}</span>
    </div>
  </div>
</article>

<style>
  .movie-card {
    display: flex;
    flex-direction: column;
    border-radius: 8px;
    overflow: hidden;
    cursor: pointer;
    transition:
      transform 0.3s ease,
      box-shadow 0.3s ease;
  }

  .movie-card:hover,
  .movie-card.focused {
    transform: scale(1.05);
    box-shadow: 0 8px 24px rgba(229, 9, 20, 0.4);
  }

  .movie-card:focus,
  .movie-card:focus-visible {
    outline: none;
    transform: scale(1.05);
    box-shadow: 0 0 0 3px #e50914, 0 8px 24px rgba(229, 9, 20, 0.5);
  }

  .movie-card.focused {
    outline: none;
    box-shadow: 0 0 0 3px #e50914, 0 8px 24px rgba(229, 9, 20, 0.5);
  }

  .poster-container {
    position: relative;
    aspect-ratio: 9 / 16;
    overflow: hidden;
    background: #2a2a2a;
  }

  .poster {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform 0.3s ease;
  }

  .movie-card:hover .poster,
  .movie-card.focused .poster {
    transform: scale(1.05);
  }

  .rating {
    position: absolute;
    top: 8px;
    right: 8px;
    display: flex;
    align-items: center;
    gap: 4px;
    background: rgba(0, 0, 0, 0.75);
    padding: 4px 8px;
    border-radius: 12px;
    font-size: 0.8rem;
    font-weight: 600;
    color: #ffd700;
  }

  .star-icon {
    width: 12px;
    height: 12px;
    color: #ffd700;
  }

  .title-overlay {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    padding: 24px 10px 10px;
    background: linear-gradient(to top, rgba(0, 0, 0, 0.95) 0%, rgba(0, 0, 0, 0.7) 60%, transparent 100%);
  }

  .title {
    font-size: 0.85rem;
    font-weight: 600;
    color: #fff;
    margin: 0;
    line-height: 1.2;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .year {
    font-size: 0.7rem;
    color: #aaa;
  }

  /* TV-optimized styles */
  @media (min-width: 1920px) {
    .title {
      font-size: 1rem;
    }

    .year {
      font-size: 0.85rem;
    }

    .rating {
      font-size: 0.95rem;
      padding: 5px 10px;
    }

    .star-icon {
      width: 14px;
      height: 14px;
    }
  }
</style>
