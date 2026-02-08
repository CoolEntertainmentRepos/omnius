<script lang="ts">
  import type { Series } from "$lib/api/types";

  interface Props {
    series: Series;
    onclick?: () => void;
    onlongpress?: (target: HTMLElement) => void;
  }

  let { series, onclick, onlongpress }: Props = $props();

  let isFocused = $state(false);
  let longPressTimer: ReturnType<typeof setTimeout> | null = null;
  let longPressed = false;

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter' && !event.repeat) {
      event.preventDefault();
      event.stopPropagation();
      longPressed = false;
      longPressTimer = setTimeout(() => {
        longPressed = true;
        onlongpress?.(event.currentTarget as HTMLElement);
      }, 600);
      return;
    }
    if (event.key === " ") {
      event.preventDefault();
      onclick?.();
    }
  }

  function handleKeyup(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      event.preventDefault();
      event.stopPropagation();
      if (longPressTimer) { clearTimeout(longPressTimer); longPressTimer = null; }
      if (!longPressed) {
        onclick?.();
      }
    }
  }

  function handlePointerDown(e: PointerEvent) {
    longPressed = false;
    longPressTimer = setTimeout(() => {
      longPressed = true;
      onlongpress?.(e.currentTarget as HTMLElement);
    }, 600);
  }

  function handlePointerUp() {
    if (longPressTimer) { clearTimeout(longPressTimer); longPressTimer = null; }
    if (!longPressed) {
      onclick?.();
    }
  }

  function handlePointerCancel() {
    if (longPressTimer) { clearTimeout(longPressTimer); longPressTimer = null; }
  }

  const yearDisplay = series.end_year
    ? `${series.year}-${series.end_year}`
    : series.status === 'Continuing'
      ? `${series.year}-`
      : `${series.year}`;
</script>

<article
  class="series-card"
  class:focused={isFocused}
  tabindex="0"
  role="button"
  aria-label="View details for {series.title}"
  onkeydown={handleKeydown}
  onkeyup={handleKeyup}
  onpointerdown={handlePointerDown}
  onpointerup={handlePointerUp}
  onpointercancel={handlePointerCancel}
  onfocus={(e) => { isFocused = true; e.currentTarget.scrollIntoView({ behavior: 'instant', block: 'nearest', inline: 'center' }); }}
  onblur={() => (isFocused = false)}
>
  <div class="poster-container">
    <img
      src={series.poster_image}
      alt="{series.title} poster"
      class="poster"
      loading="lazy"
      onerror={(e) => { (e.target as HTMLImageElement).src = 'https://via.placeholder.com/300x450?text=No+Image'; }}
    />
    {#if series.status === 'Continuing'}
      <span class="status continuing">ONGOING</span>
    {/if}
    {#if series.rating && series.rating > 0}
      <span class="rating">
        <svg class="star-icon" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
        </svg>
        {series.rating.toFixed(1)}
      </span>
    {/if}
  </div>
  <div class="card-info">
    <h3 class="title">{series.title}</h3>
    <div class="meta">
      <span class="year">{yearDisplay}</span>
      <span class="dot">·</span>
      <span class="seasons">{series.total_seasons}S</span>
    </div>
  </div>
</article>

<style>
  .series-card {
    display: flex;
    flex-direction: column;
    border-radius: 8px;
    overflow: hidden;
    cursor: pointer;
    transition:
      transform 0.3s ease,
      box-shadow 0.3s ease;
  }

  .series-card:hover,
  .series-card.focused {
    transform: scale(1.05);
    box-shadow: 0 8px 24px rgba(46, 204, 113, 0.4);
  }

  .series-card:focus,
  .series-card:focus-visible {
    outline: none;
    transform: scale(1.05);
    box-shadow: 0 0 0 3px #2ecc71, 0 8px 24px rgba(46, 204, 113, 0.5);
  }

  .series-card.focused {
    outline: none;
    box-shadow: 0 0 0 3px #2ecc71, 0 8px 24px rgba(46, 204, 113, 0.5);
  }

  .poster-container {
    position: relative;
    aspect-ratio: 2 / 3;
    overflow: hidden;
    background: #2a2a2a;
    border-radius: 8px;
  }

  .poster {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform 0.3s ease;
  }

  .series-card:hover .poster,
  .series-card.focused .poster {
    transform: scale(1.05);
  }

  .rating {
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

  .star-icon {
    width: 10px;
    height: 10px;
    color: #ffd700;
  }

  .status {
    position: absolute;
    top: 6px;
    left: 6px;
    padding: 2px 6px;
    border-radius: 3px;
    font-size: 0.6rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .status.continuing {
    background: #2ecc71;
    color: #fff;
  }

  .card-info {
    padding: 6px 2px 0;
  }

  .title {
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

  .meta {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-top: 2px;
  }

  .year {
    font-size: 0.65rem;
    color: #888;
  }

  .dot {
    font-size: 0.6rem;
    color: #555;
  }

  .seasons {
    font-size: 0.65rem;
    color: #888;
  }

  /* TV-optimized styles */
  @media (min-width: 1920px) {

    .title {
      font-size: 0.95rem;
    }

    .year, .seasons {
      font-size: 0.8rem;
    }

    .rating {
      font-size: 0.85rem;
      padding: 4px 8px;
    }

    .star-icon {
      width: 12px;
      height: 12px;
    }
  }
</style>
