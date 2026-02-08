<script lang="ts">
  import type { Movie } from "$lib/api/types";
  import { goto } from "$app/navigation";
  import { favoritesStore } from "$lib/stores/favorites.svelte";
  import ContextMenu from "./ContextMenu.svelte";

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

  // Long-press context menu state
  let longPressTimer: ReturnType<typeof setTimeout> | null = null;
  let longPressed = false;
  let contextMenu = $state({ visible: false, x: 0, y: 0, movie: null as Movie | null });

  function showContextMenuFor(target: HTMLElement, movie: Movie) {
    const rect = target.getBoundingClientRect();
    contextMenu = {
      visible: true,
      x: rect.left + rect.width / 2 - 100,
      y: rect.top + rect.height / 2 - 24,
      movie,
    };
  }

  function closeContextMenu() {
    contextMenu = { visible: false, x: 0, y: 0, movie: null };
  }

  function getContextMenuItems(movie: Movie) {
    const isFav = favoritesStore.isFavorite(movie.id);
    return [{
      label: isFav ? 'Remove from My List' : 'Add to My List',
      action: () => { favoritesStore.toggle(movie); },
    }];
  }

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
    goto(`/movies/${movie.id}`);
  }

  // Handle keyboard navigation for D-pad/remote
  function handleCardKeydown(e: KeyboardEvent, index: number) {
    const cards = scrollContainer?.querySelectorAll<HTMLElement>('.movie-card');
    if (!cards) return;

    if (e.key === 'Enter' && !e.repeat) {
      e.preventDefault();
      e.stopPropagation();
      longPressed = false;
      longPressTimer = setTimeout(() => {
        longPressed = true;
        showContextMenuFor(e.currentTarget as HTMLElement, movies[index]);
      }, 600);
      return;
    }

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
        const currentRow = scrollContainer?.closest('.movie-row');
        const allRows = document.querySelectorAll('.movie-row');
        const rowIndex = Array.from(allRows).indexOf(currentRow as Element);

        if (e.key === 'ArrowUp') {
          e.preventDefault();
          if (rowIndex > 0) {
            const prevRow = allRows[rowIndex - 1];
            const prevCards = prevRow.querySelectorAll<HTMLElement>('.movie-card');
            if (prevCards.length > 0) {
              const targetIndex = Math.min(index, prevCards.length - 1);
              prevCards[targetIndex].focus();
            }
          } else {
            const heroBtn = document.querySelector<HTMLElement>('.btn-play');
            heroBtn?.focus();
          }
        } else if (e.key === 'ArrowDown' && rowIndex < allRows.length - 1) {
          e.preventDefault();
          const nextRow = allRows[rowIndex + 1];
          const nextCards = nextRow.querySelectorAll<HTMLElement>('.movie-card');
          if (nextCards.length > 0) {
            const targetIndex = Math.min(index, nextCards.length - 1);
            nextCards[targetIndex].focus();
          }
        }
        break;
    }
  }

  function handleCardKeyup(e: KeyboardEvent, index: number) {
    if (e.key === 'Enter') {
      e.preventDefault();
      e.stopPropagation();
      if (longPressTimer) { clearTimeout(longPressTimer); longPressTimer = null; }
      if (!longPressed) {
        handleMovieClick(movies[index]);
      }
    }
  }

  // Pointer-based long press for touch/mouse
  function handlePointerDown(e: PointerEvent, index: number) {
    longPressed = false;
    longPressTimer = setTimeout(() => {
      longPressed = true;
      showContextMenuFor(e.currentTarget as HTMLElement, movies[index]);
    }, 600);
  }

  function handlePointerUp(e: PointerEvent, index: number) {
    if (longPressTimer) { clearTimeout(longPressTimer); longPressTimer = null; }
    if (!longPressed) {
      handleMovieClick(movies[index]);
    }
  }

  function handlePointerCancel() {
    if (longPressTimer) { clearTimeout(longPressTimer); longPressTimer = null; }
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
            onkeydown={(e) => handleCardKeydown(e, index)}
            onkeyup={(e) => handleCardKeyup(e, index)}
            onpointerdown={(e) => handlePointerDown(e, index)}
            onpointerup={(e) => handlePointerUp(e, index)}
            onpointercancel={handlePointerCancel}
            onfocus={(e) => e.currentTarget.scrollIntoView({ behavior: 'instant', block: 'nearest', inline: 'center' })}
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
              {#if movie.rating && movie.rating > 0}
                <span class="card-rating-badge">
                  <svg viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
                  </svg>
                  {movie.rating.toFixed(1)}
                </span>
              {/if}
              <button
                class="menu-btn"
                tabindex="-1"
                aria-label="More options"
                onclick={(e) => { e.stopPropagation(); showContextMenuFor(e.currentTarget.closest('.movie-card') as HTMLElement, movie); }}
                onpointerdown={(e) => e.stopPropagation()}
              >
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <circle cx="12" cy="5" r="2"/><circle cx="12" cy="12" r="2"/><circle cx="12" cy="19" r="2"/>
                </svg>
              </button>
            </div>
            <div class="card-info">
              <h3 class="card-title">{movie.title}</h3>
              <span class="card-year">{movie.year}</span>
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

{#if contextMenu.visible && contextMenu.movie}
  <ContextMenu
    visible={contextMenu.visible}
    x={contextMenu.x}
    y={contextMenu.y}
    items={getContextMenuItems(contextMenu.movie)}
    onclose={closeContextMenu}
  />
{/if}

<style>
  .movie-row {
    margin-bottom: 20px;
  }

  .row-title {
    font-size: 1.1rem;
    font-weight: 600;
    margin: 0 0 8px;
    padding: 0 40px;
    color: #fff;
  }

  .row-container {
    position: relative;
  }

  .movies-scroll {
    display: flex;
    gap: 10px;
    overflow-x: auto;
    scroll-behavior: smooth;
    padding: 8px 40px 12px;
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

  .card-rating-badge {
    position: absolute;
    bottom: 6px;
    left: 6px;
    display: flex;
    align-items: center;
    gap: 3px;
    background: rgba(0, 0, 0, 0.7);
    padding: 3px 6px;
    border-radius: 4px;
    font-size: 0.7rem;
    font-weight: 600;
    color: #ffd700;
  }

  .card-rating-badge svg {
    width: 10px;
    height: 10px;
  }

  .menu-btn {
    position: absolute;
    top: 4px;
    right: 4px;
    width: 28px;
    height: 28px;
    background: rgba(0, 0, 0, 0.5);
    border: none;
    border-radius: 50%;
    color: rgba(255, 255, 255, 0.7);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 1;
    transition: background 0.2s ease;
    padding: 0;
    z-index: 5;
  }

  .menu-btn:hover {
    background: rgba(0, 0, 0, 0.85);
    color: #fff;
  }

  .menu-btn svg {
    width: 16px;
    height: 16px;
  }

  .card-info {
    padding: 6px 2px 0;
  }

  .card-title {
    font-size: 0.78rem;
    font-weight: 500;
    color: #e0e0e0;
    margin: 0;
    line-height: 1.3;
    display: -webkit-box;
    -webkit-line-clamp: 1;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .card-year {
    font-size: 0.65rem;
    color: #888;
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
      padding: 0 40px;
      font-size: 1rem;
    }

    .movies-scroll {
      padding: 8px 40px 12px;
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
      padding: 0 32px;
      font-size: 0.95rem;
    }

    .movies-scroll {
      padding: 6px 32px 10px;
      gap: 8px;
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
  }

  @media (max-width: 600px) {
    .row-title {
      padding: 0 20px;
      font-size: 0.9rem;
    }

    .movies-scroll {
      padding: 6px 20px 10px;
      gap: 8px;
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
