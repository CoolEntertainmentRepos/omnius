<script lang="ts">
  import type { Movie } from "$lib/api/types";
  import { goto } from "$app/navigation";

  interface Props {
    movie: Movie | null;
  }

  let { movie }: Props = $props();

  function handlePlay() {
    if (movie) {
      goto(`/movie/${movie.id}`);
    }
  }

  function handleMoreInfo() {
    if (movie) {
      goto(`/movie/${movie.id}`);
    }
  }
</script>

{#if movie}
  <div
    class="hero"
    style="background-image: url({movie.background_image_original || movie.large_cover_image})"
  >
    <div class="hero-gradient"></div>
    <div class="hero-content">
      <h1 class="hero-title">{movie.title}</h1>
      <div class="hero-meta">
        <span class="hero-rating">
          <svg viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
          </svg>
          {movie.rating.toFixed(1)}
        </span>
        <span class="hero-year">{movie.year}</span>
        {#if movie.genres && movie.genres.length > 0}
          <span class="hero-genres">{movie.genres.slice(0, 3).join(" / ")}</span>
        {/if}
      </div>
      <p class="hero-description">
        {movie.summary || movie.description_full || movie.synopsis || "No description available."}
      </p>
      <div class="hero-buttons">
        <button class="btn-play" onclick={handlePlay}>
          <svg viewBox="0 0 24 24" fill="currentColor">
            <path d="M8 5v14l11-7z"/>
          </svg>
          Play
        </button>
        <button class="btn-info" onclick={handleMoreInfo}>
          <svg viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/>
          </svg>
          More Info
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .hero {
    position: relative;
    height: 85vh;
    min-height: 500px;
    max-height: 900px;
    background-size: cover;
    background-position: center top;
    display: flex;
    align-items: flex-end;
  }

  .hero-gradient {
    position: absolute;
    inset: 0;
    background: linear-gradient(
      to top,
      #141414 0%,
      rgba(20, 20, 20, 0.9) 15%,
      rgba(20, 20, 20, 0.4) 40%,
      rgba(20, 20, 20, 0.2) 60%,
      rgba(20, 20, 20, 0.4) 100%
    );
  }

  .hero-content {
    position: relative;
    z-index: 10;
    padding: 0 60px 80px;
    max-width: 700px;
  }

  .hero-title {
    font-size: 4rem;
    font-weight: 700;
    margin: 0 0 16px;
    text-shadow: 2px 2px 8px rgba(0, 0, 0, 0.8);
    line-height: 1.1;
  }

  .hero-meta {
    display: flex;
    align-items: center;
    gap: 20px;
    margin-bottom: 20px;
    font-size: 1.1rem;
  }

  .hero-rating {
    display: flex;
    align-items: center;
    gap: 6px;
    color: #46d369;
    font-weight: 600;
  }

  .hero-rating svg {
    width: 20px;
    height: 20px;
  }

  .hero-year {
    color: #fff;
    font-weight: 500;
  }

  .hero-genres {
    color: #aaa;
  }

  .hero-description {
    font-size: 1.25rem;
    line-height: 1.5;
    color: #ddd;
    margin: 0 0 28px;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
    text-shadow: 1px 1px 4px rgba(0, 0, 0, 0.8);
  }

  .hero-buttons {
    display: flex;
    gap: 16px;
  }

  .btn-play, .btn-info {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 14px 32px;
    border: none;
    border-radius: 6px;
    font-size: 1.2rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn-play {
    background: #fff;
    color: #141414;
  }

  .btn-play:hover {
    background: rgba(255, 255, 255, 0.85);
  }

  .btn-play:focus {
    outline: none;
    box-shadow: 0 0 0 3px rgba(255, 255, 255, 0.5);
  }

  .btn-info {
    background: rgba(109, 109, 110, 0.7);
    color: #fff;
  }

  .btn-info:hover {
    background: rgba(109, 109, 110, 0.5);
  }

  .btn-info:focus {
    outline: none;
    box-shadow: 0 0 0 3px rgba(255, 255, 255, 0.3);
  }

  .btn-play svg, .btn-info svg {
    width: 28px;
    height: 28px;
  }

  @media (max-width: 1024px) {
    .hero-content {
      padding: 0 40px 60px;
    }

    .hero-title {
      font-size: 3rem;
    }

    .hero-description {
      font-size: 1.1rem;
    }
  }

  @media (max-width: 768px) {
    .hero {
      height: 70vh;
    }

    .hero-content {
      padding: 0 24px 40px;
    }

    .hero-title {
      font-size: 2rem;
    }

    .hero-meta {
      flex-wrap: wrap;
      gap: 12px;
      font-size: 1rem;
    }

    .hero-description {
      font-size: 1rem;
      -webkit-line-clamp: 2;
    }

    .btn-play, .btn-info {
      padding: 12px 24px;
      font-size: 1rem;
    }

    .btn-play svg, .btn-info svg {
      width: 24px;
      height: 24px;
    }
  }
</style>
