<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import MovieRow from "$lib/components/MovieRow.svelte";
  import Top10Row from "$lib/components/Top10Row.svelte";
  import SeriesRow from "$lib/components/SeriesRow.svelte";
  import LiveTVFavoritesSection from "$lib/components/LiveTVFavoritesSection.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import { listMovies, getLocalRatings, listSeries, listChannels, getHomeData, getImdbImages, type HomeSection } from "$lib/api/commands";
  import { configStore } from "$lib/stores/config.svelte";
  import { makeFocusable, addSection, setFocus } from "$lib/utils/tvNavigation";
  import type { Movie, Series, Channel } from "$lib/api/types";

  // Hero
  let featured = $state<Movie | null>(null);
  let featuredBackground = $state<string | null>(null);
  let heroBackgrounds = $state<string[]>([]);
  let currentBgIndex = $state(0);
  let prevBgIndex = $state(-1);
  let bgTransitioning = $state(false);
  let heroInterval: ReturnType<typeof setInterval> | null = null;

  // Movie rows
  let trending = $state<Movie[]>([]);
  let topRated = $state<Movie[]>([]);
  let newReleases = $state<Movie[]>([]);
  let action = $state<Movie[]>([]);
  let comedy = $state<Movie[]>([]);
  let thriller = $state<Movie[]>([]);
  let scifi = $state<Movie[]>([]);
  let drama = $state<Movie[]>([]);
  let homeSeries = $state<Series[]>([]);

  let loading = $state(true);
  let loadError = $state<string | null>(null);
  let retryCount = $state(0);
  let retryTimer: ReturnType<typeof setTimeout> | null = null;
  let homeTab = $state<string>('all');

  // Dynamic home sections from API
  let homeSections = $state<HomeSection[]>([]);

  onMount(() => {
    loadHomeContent();

    setTimeout(() => {
      makeFocusable();

      addSection('sidebar', '.sidebar .nav-item', {
        restrict: 'self-first',
        leaveFor: {
          right: '.movie-card, .featured-actions button',
          down: '.movie-card, .featured-actions button'
        }
      });

      addSection('content', '.main-content button, .main-content a, .movie-card', {
        restrict: 'self-first',
        leaveFor: {
          left: '.sidebar .nav-item.active'
        }
      });

      setFocus('.nav-item[aria-label="Home"]');
    }, 300);

    return () => {
      if (retryTimer) clearTimeout(retryTimer);
      stopHeroAutoplay();
    };
  });

  async function loadHomeContent() {
    loading = true;
    loadError = null;

    try {
      if (!configStore.loaded) {
        await configStore.fetchConfig();
      }

      const homeData = await getHomeData();
      homeSections = homeData.sections || [];

      // Get featured movie
      if (homeData.hero_slider?.[0]) {
        featured = homeData.hero_slider[0];
      } else {
        const heroSection = homeSections.find(s => s.display_type === 'hero' || s.display_type === 'banner');
        if (heroSection?.movies?.[0]) {
          featured = heroSection.movies[0];
        } else {
          const topData = await listMovies({ sort_by: "rating", limit: 1, minimum_rating: 8 });
          if (topData.movies?.[0]) {
            featured = topData.movies[0];
          }
        }
      }

      if (featured) {
        if (featured.imdb_code) {
          const images = await getImdbImages(featured.imdb_code);
          const bgImages = images
            .filter(img => img.width > img.height)
            .filter(img => img.type === 'still_frame' || img.type === 'publicity' || img.type === 'production')
            .map(img => img.url)
            .slice(0, 5);

          if (bgImages.length > 0) {
            heroBackgrounds = bgImages;
            featuredBackground = bgImages[0];
            currentBgIndex = 0;
            startHeroAutoplay();
          } else {
            featuredBackground = featured.background_image_original || featured.large_cover_image || null;
          }
        }
      }

      // Map API sections to local arrays
      for (const section of homeSections) {
        if (!section.movies || section.display_type === 'hero' || section.display_type === 'banner') continue;
        switch (section.type || section.id) {
          case 'recent':
          case 'recently_added':
            newReleases = section.movies;
            break;
          case 'top_rated':
            topRated = section.movies;
            break;
        }
        if (section.id === 'trending' || section.id.includes('trending')) {
          trending = section.movies;
        }
      }

      // Fallback
      if (homeSections.length === 0 || !featured) {
        await loadFallbackContent();
      }

      // Genre rows
      if (configStore.isEnabled("movies") && (action.length === 0 || comedy.length === 0)) {
        const [actionData, comedyData, thrillerData, scifiData, dramaData] = await Promise.all([
          listMovies({ genre: "Action", sort_by: "rating", limit: 20 }),
          listMovies({ genre: "Comedy", sort_by: "rating", limit: 20 }),
          listMovies({ genre: "Thriller", sort_by: "rating", limit: 20 }),
          listMovies({ genre: "Sci-Fi", sort_by: "rating", limit: 20 }),
          listMovies({ genre: "Drama", sort_by: "rating", limit: 20 }),
        ]);
        action = actionData.movies || [];
        comedy = comedyData.movies || [];
        thriller = thrillerData.movies || [];
        scifi = scifiData.movies || [];
        drama = dramaData.movies || [];
      }

      // Series row (non-blocking)
      if (configStore.isEnabled("series")) {
        listSeries({ limit: 20, sort_by: 'rating', order_by: 'desc' })
          .then(data => { homeSeries = data.series || []; })
          .catch(err => console.warn('[Home] Failed to load series:', err));
      }

      retryCount = 0;
      loadError = null;
    } catch (err) {
      console.error('[Home] Failed to load content:', err);
      loadError = "Unable to connect to server";
      const delay = Math.min(3000 * Math.pow(2, retryCount), 30000);
      retryCount += 1;
      if (retryTimer) clearTimeout(retryTimer);
      retryTimer = setTimeout(() => { loadHomeContent(); }, delay);
    } finally {
      loading = false;
    }
  }

  async function loadFallbackContent() {
    const [trendingData, topRatedData, newReleasesData] = await Promise.all([
      listMovies({ sort_by: "download_count", limit: 20 }),
      listMovies({ sort_by: "rating", limit: 20, minimum_rating: 8 }),
      listMovies({ sort_by: "date_added", limit: 20 }),
    ]);
    trending = trendingData.movies || [];
    topRated = topRatedData.movies || [];
    newReleases = newReleasesData.movies || [];

    const allMovies = [...trending, ...topRated, ...newReleases];
    const imdbCodes = [...new Set(allMovies.map(m => m.imdb_code).filter(Boolean))];
    if (imdbCodes.length > 0) {
      const localRatings = await getLocalRatings(imdbCodes);
      const enrichMovie = (m: Movie) => {
        const rating = localRatings[m.imdb_code];
        if (rating?.imdb_rating) m.rating = rating.imdb_rating;
        return m;
      };
      trending = trending.map(enrichMovie);
      topRated = topRated.map(enrichMovie);
      newReleases = newReleases.map(enrichMovie);
    }

    if (!featured && topRated.length > 0) {
      featured = topRated[0];
    }
  }

  // Hero background rotation
  function startHeroAutoplay() {
    stopHeroAutoplay();
    if (heroBackgrounds.length <= 1) return;
    heroInterval = setInterval(() => { nextBackground(); }, 5000);
  }

  function stopHeroAutoplay() {
    if (heroInterval) {
      clearInterval(heroInterval);
      heroInterval = null;
    }
  }

  function nextBackground() {
    if (heroBackgrounds.length <= 1) return;
    bgTransitioning = true;
    prevBgIndex = currentBgIndex;
    setTimeout(() => {
      currentBgIndex = (currentBgIndex + 1) % heroBackgrounds.length;
      featuredBackground = heroBackgrounds[currentBgIndex];
      bgTransitioning = false;
    }, 1000);
  }

  function manualRetry() {
    retryCount = 0;
    if (retryTimer) clearTimeout(retryTimer);
    loadHomeContent();
  }

  function handlePlay() {
    if (featured) {
      goto(`/movies/${featured.id}`);
    }
  }

  function handleMoreInfo() {
    if (featured) {
      goto(`/movies/${featured.id}`);
    }
  }
</script>

<svelte:head>
  <title>Omnius - Watch Movies & Live TV</title>
</svelte:head>

<div class="app">
  <Sidebar />

  <main class="main-content">
    {#if loading && !loadError}
      <div class="loading-page">
        <div class="spinner"></div>
      </div>
    {:else if loadError}
      <div class="error-page">
        <svg viewBox="0 0 24 24" fill="currentColor" class="error-icon">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
        </svg>
        <h2>Connection Error</h2>
        <p>{loadError}</p>
        <p class="retry-info">Retrying automatically... (attempt {retryCount})</p>
        <button class="btn-retry" onclick={manualRetry}>
          Retry Now
        </button>
      </div>
    {:else}
      <!-- Home Tab Bar -->
      <div class="home-tabs">
        <div class="group-toggle">
          <button class="toggle-btn" class:active={homeTab === 'all'} onclick={() => homeTab = 'all'}>
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M3 13h8V3H3v10zm0 8h8v-6H3v6zm10 0h8V11h-8v10zm0-18v6h8V3h-8z"/>
            </svg>
            All
          </button>
          {#each configStore.getEnabledServices() as service (service.id)}
            <button class="toggle-btn" class:active={homeTab === service.id}
              onclick={() => homeTab = service.id}>
              {#if service.icon === 'movie' || service.id === 'movies'}
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M18 4l2 4h-3l-2-4h-2l2 4h-3l-2-4H8l2 4H7L5 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V4h-4z"/>
                </svg>
              {:else if service.icon === 'tv' || service.id === 'series'}
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M21 3H3c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h5v2h8v-2h5c1.1 0 1.99-.9 1.99-2L23 5c0-1.1-.9-2-2-2zm0 14H3V5h18v12z"/>
                </svg>
              {:else if service.icon === 'live' || service.id === 'channels'}
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M21 6h-7.59l3.29-3.29L16 2l-4 4-4-4-.71.71L10.59 6H3c-1.1 0-2 .89-2 2v12c0 1.1.9 2 2 2h18c1.1 0 2-.9 2-2V8c0-1.11-.9-2-2-2zm0 14H3V8h18v12z"/>
                </svg>
              {:else}
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
                </svg>
              {/if}
              {service.label}
            </button>
          {/each}
        </div>
      </div>

      {#if featured && (homeTab === 'all' || homeTab === 'movies')}
        <div
          class="hero"
          onmouseenter={stopHeroAutoplay}
          onmouseleave={startHeroAutoplay}
        >
          {#if heroBackgrounds.length > 1}
            <div
              class="hero-bg hero-bg-back"
              style="background-image: url({heroBackgrounds[prevBgIndex >= 0 ? prevBgIndex : 0]})"
            ></div>
            <div
              class="hero-bg hero-bg-front"
              class:visible={!bgTransitioning}
              style="background-image: url({heroBackgrounds[currentBgIndex]})"
            ></div>
          {:else}
            <div
              class="hero-bg"
              style="background-image: url({featuredBackground || featured.background_image_original || featured.large_cover_image})"
            ></div>
          {/if}
          <div class="hero-gradient"></div>
          <div class="hero-content">
            <div class="hero-badge">
              <svg class="badge-icon" viewBox="0 0 24 24" fill="currentColor">
                <path d="M4 6.47L5.76 10H20v8H4V6.47M22 4h-4l2 4h-3l-2-4h-2l2 4h-3l-2-4H8l2 4H7L5 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V4z"/>
              </svg>
              <span>MOVIE</span>
            </div>
            <h1 class="hero-title">{featured.title}</h1>
            <div class="hero-meta">
              <span class="hero-year">{featured.year}</span>
              {#if featured.imdb_rating && featured.imdb_rating > 0}
                <span class="hero-rating">
                  <svg viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
                  </svg>
                  {featured.imdb_rating.toFixed(1)} IMDb
                </span>
              {:else if featured.rating && featured.rating > 0}
                <span class="hero-rating">
                  <svg viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
                  </svg>
                  {featured.rating.toFixed(1)} IMDb
                </span>
              {/if}
              {#if featured.genres && featured.genres.length > 0}
                <span class="hero-genres">{featured.genres.slice(0, 2).join(" / ")}</span>
              {/if}
            </div>
            <p class="hero-description">
              {featured.summary || featured.description_full || featured.synopsis || "A thrilling movie experience awaits you."}
            </p>
            <div class="hero-buttons">
              <button class="btn-play" onclick={handlePlay} onfocus={() => setTimeout(() => window.scrollTo({ top: 0, behavior: 'smooth' }), 50)}>
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M8 5v14l11-7z"/>
                </svg>
                Play
              </button>
              <button class="btn-info" onclick={handleMoreInfo} onfocus={() => setTimeout(() => window.scrollTo({ top: 0, behavior: 'smooth' }), 50)}>
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/>
                </svg>
                More Info
              </button>
            </div>
          </div>
        </div>
      {/if}

      <!-- Content rows -->
      <div class="content">
        {#if homeTab === 'channels'}
          <LiveTVFavoritesSection />
        {:else}
          {#each configStore.getEnabledServices() as service (service.id)}
            {#if service.id === "movies" && (homeTab === 'all' || homeTab === 'movies')}
              {#each homeSections.filter(s => s.display_type !== 'hero' && s.display_type !== 'banner' && s.movies && s.movies.length > 0) as section}
                {#if section.display_type === 'top10'}
                  <Top10Row title={section.title} movies={section.movies || []} sectionId={section.id} />
                {:else}
                  <MovieRow title={section.title} movies={section.movies || []} />
                {/if}
              {/each}

              {#if homeSections.filter(s => s.display_type !== 'hero').length === 0}
                <MovieRow title="Trending Now" movies={trending} />
                <MovieRow title="Top Rated" movies={topRated} />
                <MovieRow title="New Releases" movies={newReleases} />
              {/if}

              {#if action.length > 0}
                <MovieRow title="Action" movies={action} />
              {/if}
              {#if thriller.length > 0}
                <MovieRow title="Thriller" movies={thriller} />
              {/if}
              {#if scifi.length > 0}
                <MovieRow title="Sci-Fi" movies={scifi} />
              {/if}
              {#if comedy.length > 0}
                <MovieRow title="Comedy" movies={comedy} />
              {/if}
              {#if drama.length > 0}
                <MovieRow title="Drama" movies={drama} />
              {/if}
            {:else if service.id === "series" && (homeTab === 'all' || homeTab === 'series')}
              {#if homeSeries.length > 0}
                <SeriesRow title="Popular TV Shows" series={homeSeries} />
              {/if}
            {/if}
          {/each}
        {/if}
      </div>
    {/if}
  </main>
</div>

<style>
  :global(html, body) {
    background: #141414;
    margin: 0;
    padding: 0;
    height: 100%;
    overflow: hidden;
  }

  /* Focus styles handled by +layout.svelte globally */

  .app {
    display: flex;
    height: 100vh;
    height: 100dvh;
    background: #141414;
    color: #fff;
    overflow: hidden;
  }

  .main-content {
    flex: 1;
    margin-left: 70px;
    height: 100vh;
    height: 100dvh;
    overflow-y: auto;
    overflow-x: hidden;
    scroll-behavior: smooth;
  }

  /* Hero */
  .hero {
    position: relative;
    height: 85vh;
    min-height: 400px;
    display: flex;
    align-items: flex-end;
    overflow: hidden;
  }

  .hero-gradient {
    position: absolute;
    inset: 0;
    z-index: 5;
    background: linear-gradient(
      to right,
      rgba(20, 20, 20, 0.95) 0%,
      rgba(20, 20, 20, 0.7) 30%,
      rgba(20, 20, 20, 0.4) 50%,
      rgba(20, 20, 20, 0.2) 70%,
      transparent 100%
    ),
    linear-gradient(
      to top,
      #141414 0%,
      rgba(20, 20, 20, 0.8) 15%,
      transparent 40%
    );
  }

  .hero-content {
    position: relative;
    z-index: 10;
    padding: 0 40px 30px;
    max-width: 550px;
  }

  .hero-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: #e50914;
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 2px;
    margin-bottom: 6px;
  }

  .badge-icon {
    width: 18px;
    height: 18px;
  }

  .hero-title {
    font-size: 1.8rem;
    font-weight: 700;
    margin: 0 0 8px;
    text-shadow: 2px 2px 8px rgba(0, 0, 0, 0.8);
    line-height: 1.1;
  }

  .hero-meta {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 8px;
    font-size: 0.85rem;
  }

  .hero-year {
    color: #fff;
  }

  .hero-rating {
    display: flex;
    align-items: center;
    gap: 6px;
    color: #f5c518;
    font-weight: 600;
  }

  .hero-rating svg {
    width: 14px;
    height: 14px;
  }

  .hero-genres {
    color: #aaa;
  }

  .hero-description {
    font-size: 0.85rem;
    line-height: 1.4;
    color: #ddd;
    margin: 0 0 12px;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    text-shadow: 1px 1px 4px rgba(0, 0, 0, 0.8);
  }

  .hero-buttons {
    display: flex;
    gap: 10px;
    scroll-margin-top: 80vh;
  }

  .btn-play, .btn-info {
    scroll-margin-top: 80vh;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 20px;
    border: none;
    border-radius: 6px;
    font-size: 0.9rem;
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

  .btn-play:focus,
  .btn-play:focus-visible {
    outline: none;
    box-shadow: 0 0 0 4px #e50914;
    transform: scale(1.05);
  }

  .btn-info {
    background: rgba(109, 109, 110, 0.7);
    color: #fff;
  }

  .btn-info:hover {
    background: rgba(109, 109, 110, 0.5);
  }

  .btn-info:focus,
  .btn-info:focus-visible {
    outline: none;
    box-shadow: 0 0 0 4px #e50914;
    background: rgba(229, 9, 20, 0.4);
    transform: scale(1.05);
  }

  .btn-play svg, .btn-info svg {
    width: 20px;
    height: 20px;
  }

  /* Hero background layers */
  .hero-bg {
    position: absolute;
    inset: 0;
    background-size: cover;
    background-position: center center;
  }

  .hero-bg-back {
    z-index: 0;
  }

  .hero-bg-front {
    z-index: 1;
    opacity: 0;
    transition: opacity 1s ease-in-out;
  }

  .hero-bg-front.visible {
    opacity: 1;
  }

  /* Home Tab Bar */
  .home-tabs {
    position: fixed;
    top: 12px;
    right: 20px;
    z-index: 100;
  }

  .group-toggle {
    display: flex;
    gap: 8px;
    background: rgba(255, 255, 255, 0.05);
    padding: 4px;
    border-radius: 8px;
  }

  .toggle-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background: transparent;
    border: none;
    border-radius: 6px;
    color: #888;
    font-size: 0.8rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .toggle-btn svg {
    width: 18px;
    height: 18px;
  }

  .toggle-btn:hover {
    color: #fff;
    background: rgba(255, 255, 255, 0.1);
  }

  .toggle-btn.active {
    color: #fff;
    background: #e50914;
  }

  .toggle-btn:focus,
  .toggle-btn:focus-visible {
    outline: none;
    box-shadow: 0 0 0 3px rgba(229, 9, 20, 0.5);
  }

  /* Content */
  .content {
    position: relative;
    margin-top: -10px;
    padding-bottom: 30px;
    z-index: 10;
  }

  /* Loading */
  .loading-page {
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .spinner {
    width: 60px;
    height: 60px;
    border: 4px solid rgba(229, 9, 20, 0.2);
    border-top-color: #e50914;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* Error page */
  .error-page {
    height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: 40px;
  }

  .error-icon {
    width: 80px;
    height: 80px;
    color: #e50914;
    margin-bottom: 24px;
  }

  .error-page h2 {
    font-size: 1.3rem;
    font-weight: 600;
    margin: 0 0 8px;
  }

  .error-page p {
    color: #888;
    margin: 0 0 8px;
  }

  .retry-info {
    font-size: 0.9rem;
    color: #666;
    margin-bottom: 24px;
  }

  .btn-retry {
    padding: 10px 24px;
    background: #e50914;
    color: #fff;
    border: none;
    border-radius: 6px;
    font-size: 0.9rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn-retry:hover {
    background: #f40d17;
  }

  .btn-retry:focus,
  .btn-retry:focus-visible {
    outline: none;
    box-shadow: 0 0 0 4px rgba(229, 9, 20, 0.5);
    transform: scale(1.05);
  }

  /* Responsive */
  @media (max-width: 900px) {
    .main-content {
      margin-left: 60px;
      height: 100vh;
      height: 100dvh;
    }

    .hero-content {
      padding: 0 32px 24px;
      max-width: 450px;
    }

    .hero-title {
      font-size: 1.6rem;
    }
  }

  @media (max-width: 600px) {
    .main-content {
      margin-left: 0;
    }

    .hero {
      height: 70vh;
      min-height: 300px;
    }

    .hero-content {
      padding: 0 20px 20px;
    }

    .hero-title {
      font-size: 1.4rem;
    }

    .content {
      margin-top: 0;
    }
  }
</style>
