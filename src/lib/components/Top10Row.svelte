<script lang="ts">
  import { goto } from "$app/navigation";
  import type { Movie } from "$lib/api/types";

  interface Props {
    title: string;
    movies: Movie[];
    sectionId?: string;
  }

  let { title, movies, sectionId = "" }: Props = $props();

  function handleClick(movie: Movie) {
    goto(`/movie/${movie.id}`);
  }

  function handleKeydown(e: KeyboardEvent, movie: Movie) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      handleClick(movie);
    }
  }
</script>

<section class="top10-section" id={sectionId}>
  <h2 class="section-title">{title}</h2>
  <div class="top10-scroll">
    <div class="top10-container">
      {#each movies.slice(0, 10) as movie, index (movie.id)}
        <button
          class="top10-item"
          onclick={() => handleClick(movie)}
          onkeydown={(e) => handleKeydown(e, movie)}
          aria-label="{index + 1}. {movie.title}"
        >
          <span class="rank-number">{index + 1}</span>
          <div class="poster-container">
            <img
              src={movie.medium_cover_image || movie.large_cover_image}
              alt={movie.title}
              class="poster"
              loading="lazy"
            />
            <div class="poster-overlay">
              <span class="movie-title">{movie.title}</span>
              {#if movie.rating}
                <span class="movie-rating">{movie.rating.toFixed(1)}</span>
              {/if}
            </div>
          </div>
        </button>
      {/each}
    </div>
  </div>
</section>

<style>
  .top10-section {
    margin-bottom: 20px;
  }

  .section-title {
    font-size: 1.1rem;
    font-weight: 600;
    margin: 0 0 8px 40px;
    color: #fff;
  }

  .top10-scroll {
    overflow-x: auto;
    overflow-y: visible;
    scrollbar-width: none;
    -ms-overflow-style: none;
    padding: 8px 0;
  }

  .top10-scroll::-webkit-scrollbar {
    display: none;
  }

  .top10-container {
    display: flex;
    gap: 8px;
    padding: 0 60px;
  }

  .top10-item {
    position: relative;
    display: flex;
    align-items: flex-end;
    flex-shrink: 0;
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    transition: transform 0.3s ease;
  }

  .top10-item:hover,
  .top10-item:focus {
    transform: scale(1.05);
    z-index: 10;
  }

  .top10-item:focus {
    outline: none;
  }

  .top10-item:focus .poster {
    box-shadow: 0 0 0 3px #e50914, 0 8px 25px rgba(0, 0, 0, 0.5);
  }

  .rank-number {
    font-size: 8rem;
    font-weight: 900;
    color: #141414;
    -webkit-text-stroke: 4px #404040;
    text-stroke: 4px #404040;
    line-height: 1;
    margin-right: -28px;
    z-index: 1;
    font-family: "Netflix Sans", "Helvetica Neue", Helvetica, Arial, sans-serif;
    user-select: none;
  }

  .poster-container {
    position: relative;
    width: 130px;
    height: 195px;
    border-radius: 6px;
    overflow: hidden;
    z-index: 2;
  }

  .poster {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform 0.3s ease;
  }

  .top10-item:hover .poster {
    transform: scale(1.02);
  }

  .poster-overlay {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    padding: 40px 10px 10px;
    background: linear-gradient(to top, rgba(0, 0, 0, 0.9) 0%, transparent 100%);
    opacity: 0;
    transition: opacity 0.3s ease;
  }

  .top10-item:hover .poster-overlay,
  .top10-item:focus .poster-overlay {
    opacity: 1;
  }

  .movie-title {
    display: block;
    font-size: 0.8rem;
    font-weight: 600;
    color: #fff;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .movie-rating {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 0.75rem;
    color: #46d369;
    margin-top: 4px;
  }

  .movie-rating::before {
    content: "★";
  }

  /* TV/Large screens */
  @media (min-width: 1920px) {
    .section-title {
      font-size: 1.8rem;
      margin-left: 80px;
    }

    .top10-container {
      padding: 0 80px;
      gap: 12px;
    }

    .rank-number {
      font-size: 10rem;
      -webkit-text-stroke: 5px #404040;
      margin-right: -35px;
    }

    .poster-container {
      width: 170px;
      height: 255px;
      border-radius: 8px;
    }

    .movie-title {
      font-size: 1rem;
    }
  }

  /* Mobile */
  @media (max-width: 768px) {
    .section-title {
      margin-left: 20px;
      font-size: 1.2rem;
    }

    .top10-container {
      padding: 0 20px;
    }

    .rank-number {
      font-size: 5rem;
      -webkit-text-stroke: 3px #404040;
      margin-right: -18px;
    }

    .poster-container {
      width: 100px;
      height: 150px;
    }
  }
</style>
