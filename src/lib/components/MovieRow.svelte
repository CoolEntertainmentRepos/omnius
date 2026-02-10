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
    scrollContainer?.scrollBy({ left: -650, behavior: "smooth" });
  }

  function scrollRight() {
    scrollContainer?.scrollBy({ left: 650, behavior: "smooth" });
  }

  function handleMovieClick(movie: Movie) {
    goto(`/movie/${movie.id}`);
  }

  // Handle keyboard navigation for arrow keys
  function handleCardKeydown(e: KeyboardEvent, index: number) {
    const cards = scrollContainer?.querySelectorAll<HTMLElement>('.movie-card');
    if (!cards) return;

    switch (e.key) {
      case 'ArrowLeft':
        e.preventDefault();
        e.stopPropagation();
        if (index > 0) {
          cards[index - 1].focus();
        }
        break;
      case 'ArrowRight':
        e.preventDefault();
        e.stopPropagation();
        if (index < cards.length - 1) {
          cards[index + 1].focus();
        }
        break;
      case 'ArrowUp':
      case 'ArrowDown':
        e.stopPropagation();
        // Find the closest row and navigate to it
        const currentRow = scrollContainer?.closest('.movie-row');
        const allRows = document.querySelectorAll('.movie-row');
        const rowIndex = Array.from(allRows).indexOf(currentRow as Element);

        if (e.key === 'ArrowUp') {
          e.preventDefault();
          if (rowIndex > 0) {
            const prevRow = allRows[rowIndex - 1];
            const prevCards = prevRow.querySelectorAll<HTMLElement>('.movie-card');
            if (prevCards.length > 0) {
              // Focus the same index or last card if index is out of bounds
              const targetIndex = Math.min(index, prevCards.length - 1);
              prevCards[targetIndex].focus();
            }
          } else {
            // First row - go to hero buttons
            const heroBtn = document.querySelector<HTMLElement>('.btn-play');
            heroBtn?.focus();
          }
        } else if (e.key === 'ArrowDown' && rowIndex < allRows.length - 1) {
          e.preventDefault();
          const nextRow = allRows[rowIndex + 1];
          const nextCards = nextRow.querySelectorAll<HTMLElement>('.movie-card');
          if (nextCards.length > 0) {
            // Focus the same index or last card if index is out of bounds
            const targetIndex = Math.min(index, nextCards.length - 1);
            nextCards[targetIndex].focus();
          }
        }
        break;
    }
  }
</script>

<section class="movie-row">
  <h2 class="row-title">{title}</h2>

  <div class="row-container">
    {#if showLeftArrow}
      <button class="scroll-btn scroll-left" onclick={scrollLeft} tabindex="-1" aria-label="Scroll left">
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
        {#each movies as movie, index (movie.id)}
          <div
            class="movie-card"
            onclick={() => handleMovieClick(movie)}
            onkeydown={(e) => { if (e.key === 'Enter') handleMovieClick(movie); else handleCardKeydown(e, index); }}
            onfocus={(e) => e.currentTarget.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'center' })}
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
                <div class="card-play">
                  <svg viewBox="0 0 24 24" fill="currentColor">
                    <path d="M8 5v14l11-7z"/>
                  </svg>
                </div>
              </div>
              <!-- Title overlay at bottom -->
              <div class="card-title-overlay">
                {#if movie.rating && movie.rating > 0}
                  <span class="card-rating-badge">
                    <svg viewBox="0 0 24 24" fill="currentColor">
                      <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
                    </svg>
                    {movie.rating.toFixed(1)}
                  </span>
                {/if}
                <h3 class="card-title">{movie.title}</h3>
                <span class="card-year">{movie.year}</span>
              </div>
            </div>
          </div>
        {/each}
      {/if}
    </div>

    {#if showRightArrow && movies.length > 4}
      <button class="scroll-btn scroll-right" onclick={scrollRight} tabindex="-1" aria-label="Scroll right">
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
    padding: 20px 80px 30px;
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
    width: 150px;
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    text-align: left;
    transition: transform 0.2s ease;
  }

  .movie-card:hover {
    transform: scale(1.05);
    z-index: 20;
  }

  .movie-card:focus,
  .movie-card:focus-visible {
    outline: none;
    transform: scale(1.08);
    z-index: 20;
  }

  .movie-card:focus .card-poster,
  .movie-card:focus-visible .card-poster {
    box-shadow: 0 0 0 3px #e50914, 0 8px 24px rgba(229, 9, 20, 0.5);
  }

  .card-poster {
    position: relative;
    aspect-ratio: 2/3;
    border-radius: 8px;
    overflow: hidden;
    background: #2a2a2a;
  }

  .card-poster img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .card-overlay {
    position: absolute;
    inset: 0;
    background: transparent;
    opacity: 0;
    transition: opacity 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .movie-card:hover .card-overlay,
  .movie-card:focus .card-overlay {
    opacity: 1;
    background: rgba(0, 0, 0, 0.3);
  }

  .card-title-overlay {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    padding: 35px 8px 8px;
    background: linear-gradient(to top, rgba(0,0,0,0.95) 0%, rgba(0,0,0,0.7) 50%, transparent 100%);
  }

  .card-rating-badge {
    display: flex;
    align-items: center;
    gap: 3px;
    font-size: 0.7rem;
    font-weight: 600;
    color: #ffd700;
    margin-bottom: 2px;
  }

  .card-rating-badge svg {
    width: 10px;
    height: 10px;
  }

  .card-title {
    font-size: 0.8rem;
    font-weight: 600;
    color: #fff;
    margin: 0;
    line-height: 1.2;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .card-year {
    font-size: 0.65rem;
    color: #aaa;
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


  /* Skeleton loading */
  .movie-card-skeleton {
    flex-shrink: 0;
    width: 150px;
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

    .movie-card,
    .movie-card-skeleton {
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

    .movie-card,
    .movie-card-skeleton {
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

    .movie-card,
    .movie-card-skeleton {
      width: 140px;
    }

    .movie-card:hover {
      transform: none;
    }

    .card-title {
      font-size: 0.9rem;
    }
  }
</style>
