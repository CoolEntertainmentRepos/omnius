<script lang="ts">
  import type { Series } from "$lib/api/types";
  import { goto } from "$app/navigation";
  import SeriesCard from "./SeriesCard.svelte";

  interface Props {
    title: string;
    series: Series[];
    loading?: boolean;
  }

  let { title, series, loading = false }: Props = $props();

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

  function handleSeriesClick(s: Series) {
    goto(`/series/${s.id}`);
  }
</script>

<section class="series-row">
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
      class="series-scroll"
      bind:this={scrollContainer}
      onscroll={handleScroll}
    >
      {#if loading}
        {#each Array(6) as _, i (i)}
          <div class="series-card-skeleton">
            <div class="skeleton-poster"></div>
          </div>
        {/each}
      {:else}
        {#each series as s (s.id)}
          <SeriesCard series={s} onclick={() => handleSeriesClick(s)} />
        {/each}
      {/if}
    </div>

    {#if showRightArrow && series.length > 4}
      <button class="scroll-btn scroll-right" onclick={scrollRight} tabindex="-1" aria-label="Scroll right">
        <svg viewBox="0 0 24 24" fill="currentColor">
          <path d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z"/>
        </svg>
      </button>
    {/if}
  </div>
</section>

<style>
  .series-row {
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

  .series-scroll {
    display: flex;
    gap: 12px;
    overflow-x: auto;
    scroll-behavior: smooth;
    padding: 20px 80px 30px;
    scrollbar-width: none;
  }

  .series-scroll::-webkit-scrollbar {
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

  /* Skeleton loading */
  .series-card-skeleton {
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

    .series-scroll {
      padding: 10px 60px 24px;
    }

    .scroll-btn {
      width: 60px;
    }

    .scroll-btn svg {
      width: 40px;
      height: 40px;
    }

    .series-card-skeleton {
      width: 180px;
    }
  }

  @media (max-width: 900px) {
    .row-title {
      padding: 0 40px;
      font-size: 1.3rem;
    }

    .series-scroll {
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

    .series-card-skeleton {
      width: 160px;
    }
  }

  @media (max-width: 600px) {
    .row-title {
      padding: 0 24px;
      font-size: 1.2rem;
    }

    .series-scroll {
      padding: 8px 24px 16px;
      gap: 10px;
    }

    .scroll-btn {
      display: none;
    }

    .series-card-skeleton {
      width: 140px;
    }
  }
</style>
