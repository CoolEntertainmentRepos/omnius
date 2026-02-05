<script lang="ts">
  import type { Series } from "$lib/api/types";

  interface Props {
    series: Series;
    onclick?: () => void;
  }

  let { series, onclick }: Props = $props();

  let isFocused = $state(false);

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      onclick?.();
    }
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
  onclick={onclick}
  onkeydown={handleKeydown}
  onfocus={() => (isFocused = true)}
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
    <!-- Rating badge top right -->
    {#if series.rating && series.rating > 0}
      <span class="rating">
        <svg class="star-icon" viewBox="0 0 24 24" fill="currentColor">
          <path
            d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"
          />
        </svg>
        {series.rating.toFixed(1)}
      </span>
    {/if}
    <!-- Status badge top left -->
    {#if series.status === 'Continuing'}
      <span class="status continuing">ONGOING</span>
    {/if}
    <!-- Title overlay at bottom -->
    <div class="title-overlay">
      <h3 class="title">{series.title}</h3>
      <div class="meta">
        <span class="year">{yearDisplay}</span>
        <span class="seasons">{series.total_seasons} {series.total_seasons === 1 ? 'Season' : 'Seasons'}</span>
      </div>
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

  .status {
    position: absolute;
    top: 8px;
    left: 8px;
    padding: 3px 8px;
    border-radius: 4px;
    font-size: 0.65rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .status.continuing {
    background: #2ecc71;
    color: #fff;
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

  .meta {
    display: flex;
    gap: 8px;
    margin-top: 4px;
  }

  .year {
    font-size: 0.7rem;
    color: #aaa;
  }

  .seasons {
    font-size: 0.7rem;
    color: #2ecc71;
  }

  /* TV-optimized styles */
  @media (min-width: 1920px) {
    .title {
      font-size: 1rem;
    }

    .year, .seasons {
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
