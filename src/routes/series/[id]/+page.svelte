<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { browser } from "$app/environment";
  import { getSeriesDetails, getSeasonEpisodes } from "$lib/api/commands";
  import type { Series, Season, Episode, EpisodeTorrent } from "$lib/api/types";

  let seriesId = $derived(parseInt($page.params.id || "0", 10));

  let series = $state<Series | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  // Season/Episode state
  let selectedSeason = $state<number | null>(null);
  let episodes = $state<Episode[]>([]);
  let episodesLoading = $state(false);
  let selectedEpisode = $state<Episode | null>(null);
  let selectedTorrent = $state<EpisodeTorrent | null>(null);
  let isStartingStream = $state(false);
  let streamError = $state<string | null>(null);
  let showMore = $state(false);
  let modalBodyEl = $state<HTMLElement | null>(null);
  let unmounted = false;

  async function loadSeries(id: number) {
    if (unmounted) return;
    loading = true;
    error = null;
    selectedSeason = null;
    episodes = [];
    selectedEpisode = null;
    selectedTorrent = null;

    try {
      const data = await getSeriesDetails(id);
      if (unmounted) return;
      if (data) {
        series = data;
        // Auto-select first season
        if (series.total_seasons > 0) {
          await selectSeason(1);
        }
      } else {
        error = "Series not found";
      }
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to load series";
      series = null;
    } finally {
      loading = false;
      // Set focus on play button for TV navigation
      if (browser && !error) {
        setTimeout(() => {
          const playBtn = document.querySelector('.play-btn') as HTMLElement;
          if (playBtn) {
            playBtn.focus();
          }
        }, 100);
      }
    }
  }

  async function selectSeason(seasonNum: number) {
    if (unmounted || selectedSeason === seasonNum) return;

    selectedSeason = seasonNum;
    selectedEpisode = null;
    selectedTorrent = null;
    episodesLoading = true;

    try {
      const data = await getSeasonEpisodes(seriesId, seasonNum);
      if (unmounted) return;
      episodes = data;
      // Auto-select first episode with torrents
      if (episodes.length > 0) {
        const episodeWithTorrents = episodes.find(e => e.torrents && e.torrents.length > 0);
        if (episodeWithTorrents) {
          selectEpisode(episodeWithTorrents);
        } else {
          selectEpisode(episodes[0]);
        }
      }
    } catch (err) {
      console.error('Failed to load episodes:', err);
      episodes = [];
    } finally {
      episodesLoading = false;
    }
  }

  function selectEpisode(episode: Episode) {
    selectedEpisode = episode;
    selectedTorrent = null;

    // Auto-select best quality torrent
    if (episode.torrents && episode.torrents.length > 0) {
      const availableTorrents = episode.torrents.filter(t => t.seeds > 0);
      if (availableTorrents.length > 0) {
        selectedTorrent =
          availableTorrents.find(t => t.quality === "1080p") ||
          availableTorrents.find(t => t.quality === "720p") ||
          availableTorrents[0];
      } else {
        // Fallback to first torrent even if no seeds
        selectedTorrent = episode.torrents[0];
      }
    }
  }

  function selectTorrent(torrent: EpisodeTorrent) {
    selectedTorrent = torrent;
  }

  async function handlePlay() {
    if (!selectedTorrent || !selectedEpisode || !series) return;

    isStartingStream = true;
    streamError = null;

    try {
      const params = new URLSearchParams({
        title: `${series.title} S${String(selectedEpisode.season_number).padStart(2, '0')}E${String(selectedEpisode.episode_number).padStart(2, '0')} - ${selectedEpisode.title}`,
        imdb: series.imdb_code || "",
      });
      // Pass file_index for season pack torrents
      if (selectedTorrent.file_index !== undefined && selectedTorrent.file_index !== null) {
        params.set("fileIndex", String(selectedTorrent.file_index));
      }
      await goto(`/series/${seriesId}/play/${selectedTorrent.hash}?${params.toString()}`);
    } catch (err) {
      console.error("Navigation failed:", err);
      streamError = "Failed to start playback. Please try again.";
      isStartingStream = false;
    }
  }

  onMount(() => {
    loadSeries(seriesId);

    // Add focus listener to scroll focused elements into view
    if (browser) {
      document.addEventListener('focus', handleFocusScroll, true);
    }

    return () => {
      unmounted = true;
      if (browser) {
        document.removeEventListener('focus', handleFocusScroll, true);
      }
    };
  });

  function handleFocusScroll(e: FocusEvent) {
    const target = e.target as HTMLElement;
    if (target && target.scrollIntoView) {
      target.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }
  }

  // Reload when series ID changes
  $effect(() => {
    if (seriesId) {
      loadSeries(seriesId);
    }
  });

  function handleModalKeydown(e: KeyboardEvent) {
    console.log(`[SeriesModal] keydown key=${e.key} showMore=${showMore} target=${(e.target as HTMLElement).className}`);
    if (e.key === 'Escape' || e.key === 'GoBack' || e.key === 'XF86Back' || (e.key === 'Backspace' && !(e.target instanceof HTMLInputElement))) {
      e.preventDefault();
      e.stopPropagation();
      console.log('[SeriesModal] Closing modal, stopped propagation');
      showMore = false;
      return;
    }
    if (modalBodyEl) {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        modalBodyEl.scrollBy({ top: 80, behavior: 'smooth' });
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        modalBodyEl.scrollBy({ top: -80, behavior: 'smooth' });
      }
    }
  }

  $effect(() => {
    if (showMore && browser) {
      setTimeout(() => {
        const closeBtn = document.querySelector('.modal-close') as HTMLElement;
        if (closeBtn) closeBtn.focus();
      }, 50);
    }
  });

  function formatSize(size: string): string {
    return size || "Unknown";
  }

  const yearDisplay = $derived(series?.end_year
    ? `${series.year}-${series.end_year}`
    : series?.status === 'Continuing'
      ? `${series?.year}-`
      : `${series?.year}`);
</script>

<svelte:head>
  <title>{series ? `${series.title} - Streamer` : "Loading... - Streamer"}</title>
</svelte:head>

<div class="page">
  {#if loading}
    <div class="loading-container">
      <div class="spinner"></div>
      <p>Loading series...</p>
    </div>
  {:else if error}
    <div class="error-container">
      <svg viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z" />
      </svg>
      <p>{error}</p>
      <div class="error-buttons">
        <button class="back-button" onclick={() => goto('/series')}>
          <svg viewBox="0 0 24 24" fill="currentColor">
            <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z" />
          </svg>
          Go Back
        </button>
        <button class="retry-button" onclick={() => loadSeries(seriesId)}>Try Again</button>
      </div>
    </div>
  {:else if series}
    <!-- Background -->
    <div
      class="backdrop"
      style="background-image: url({series.background_image || series.poster_image})"
    >
      <div class="backdrop-overlay"></div>
    </div>

    <!-- Back Button -->
    <nav class="nav">
      <button class="back-btn" onclick={() => goto('/series')} aria-label="Go back">
        <svg viewBox="0 0 24 24" fill="currentColor">
          <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z" />
        </svg>
      </button>
    </nav>

    <main class="content">
      <div class="series-info">
        <!-- Left: Season/Episode/Play controls -->
        <div class="controls-column">
          <!-- Season Selector -->
          <div class="season-selector">
            <div class="season-buttons">
              {#each Array.from({ length: series.total_seasons }, (_, i) => i + 1) as seasonNum (seasonNum)}
                <button
                  class="season-btn"
                  class:selected={selectedSeason === seasonNum}
                  onclick={() => selectSeason(seasonNum)}
                >
                  S{seasonNum}
                </button>
              {/each}
            </div>
          </div>

          <!-- Episode List -->
          {#if selectedSeason}
            <div class="episode-selector">
              {#if episodesLoading}
                <div class="loading-text">Loading...</div>
              {:else if episodes.length > 0}
                <div class="episode-list">
                  {#each episodes as episode (episode.id)}
                    <button
                      class="episode-btn"
                      class:selected={selectedEpisode?.id === episode.id}
                      class:has-torrents={episode.torrents && episode.torrents.length > 0}
                      onclick={() => selectEpisode(episode)}
                    >
                      <span class="episode-number">E{String(episode.episode_number).padStart(2, '0')}</span>
                      <span class="episode-title">{episode.title}</span>
                    </button>
                  {/each}
                </div>
              {:else}
                <p class="no-episodes">No episodes found.</p>
              {/if}
            </div>
          {/if}

          <!-- Quality + Play for selected episode -->
          {#if selectedEpisode && selectedEpisode.torrents && selectedEpisode.torrents.length > 0}
            <div class="quality-section">
              <div class="quality-options">
                {#each selectedEpisode.torrents as torrent (torrent.id)}
                  <button
                    class="quality-btn"
                    class:selected={selectedTorrent?.id === torrent.id}
                    class:no-seeds={torrent.seeds === 0}
                    onclick={() => selectTorrent(torrent)}
                  >
                    <span class="quality-label">{torrent.quality}</span>
                    <span class="quality-meta">{formatSize(torrent.size)} &middot; {torrent.seeds} seeds</span>
                  </button>
                {/each}
              </div>
            </div>

            <button
              class="play-btn"
              onclick={handlePlay}
              disabled={!selectedTorrent || isStartingStream}
            >
              {#if isStartingStream}
                <div class="btn-spinner"></div>
                Starting...
              {:else}
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M8 5v14l11-7z"/>
                </svg>
                Play {selectedTorrent?.quality || ""}
              {/if}
            </button>
          {:else if selectedEpisode}
            <div class="no-sources-inline">No sources available</div>
          {/if}

          {#if streamError}
            <div class="stream-error">
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
              </svg>
              {streamError}
            </div>
          {/if}
        </div>

        <!-- Right: Info -->
        <div class="info">
          <h1 class="title">{series.title}</h1>

          <div class="meta">
            <span class="year">{yearDisplay}</span>
            <span class="seasons-count">{series.total_seasons} {series.total_seasons === 1 ? 'Season' : 'Seasons'}</span>
            {#if series.network}
              <span class="network">{series.network}</span>
            {/if}
            {#if series.status === 'Continuing'}
              <span class="status-badge continuing">ONGOING</span>
            {:else}
              <span class="status-badge ended">ENDED</span>
            {/if}
          </div>

          {#if series.rating && series.rating > 0}
            <div class="ratings-row">
              <span class="rating-badge imdb">
                <span class="rating-label">IMDb</span>
                <span class="rating-value">{series.rating.toFixed(1)}</span>
              </span>
              {#if series.rotten_tomatoes}
                <span class="rating-badge rt">
                  <span class="rating-label">RT</span>
                  <span class="rating-value">{series.rotten_tomatoes}%</span>
                </span>
              {/if}
            </div>
          {/if}

          {#if series.genres && series.genres.length > 0}
            <div class="genres">
              {#each series.genres as genre (genre)}
                <span class="genre-tag">{genre}</span>
              {/each}
            </div>
          {/if}

          <p class="description truncated">
            {series.summary || "No description available."}
          </p>

          <!-- Selected episode inline preview -->
          {#if selectedEpisode}
            <div class="episode-preview">
              <span class="episode-tag">
                S{String(selectedEpisode.season_number).padStart(2, '0')}E{String(selectedEpisode.episode_number).padStart(2, '0')}
              </span>
              <span class="episode-preview-title">{selectedEpisode.title}</span>
              {#if selectedEpisode.air_date}
                <span class="episode-preview-date">{selectedEpisode.air_date}</span>
              {/if}
            </div>
          {/if}

          <button class="more-btn" onclick={() => showMore = !showMore}>
            More Info
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M11 17h2v-6h-2v6zm1-15C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8zM11 9h2V7h-2v2z"/>
            </svg>
          </button>
        </div>
      </div>
    </main>

    <!-- More Info Modal -->
    {#if showMore && series}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="modal-overlay" onclick={() => showMore = false} onkeydown={handleModalKeydown}>
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="modal-content" onclick={(e) => e.stopPropagation()}>
          <div class="modal-header">
            <h2>{series.title}</h2>
            <button class="modal-close" onclick={() => showMore = false}>
              <svg viewBox="0 0 24 24" fill="currentColor"><path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/></svg>
            </button>
          </div>
          <div class="modal-body" bind:this={modalBodyEl}>
            <p class="modal-description">{series.summary || ""}</p>

            {#if selectedEpisode}
              <div class="modal-episode">
                <h3>
                  S{String(selectedEpisode.season_number).padStart(2, '0')}E{String(selectedEpisode.episode_number).padStart(2, '0')} - {selectedEpisode.title}
                </h3>
                {#if selectedEpisode.air_date}
                  <span class="modal-episode-date">{selectedEpisode.air_date}</span>
                {/if}
                {#if selectedEpisode.summary}
                  <p class="modal-episode-summary">{selectedEpisode.summary}</p>
                {/if}
              </div>
            {/if}
          </div>
        </div>
      </div>
    {/if}
  {/if}
</div>

<style>
  .page {
    min-height: 100vh;
    background: #141414;
    color: #fff;
    position: relative;
  }

  .backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    height: 80vh;
    background-size: cover;
    background-position: center top;
    z-index: 0;
  }

  .backdrop-overlay {
    position: absolute;
    inset: 0;
    background: linear-gradient(
      to right,
      rgba(20, 20, 20, 0.98) 0%,
      rgba(20, 20, 20, 0.8) 40%,
      rgba(20, 20, 20, 0.4) 100%
    ),
    linear-gradient(
      to top,
      #141414 0%,
      rgba(20, 20, 20, 0.9) 30%,
      transparent 100%
    );
  }

  .nav {
    position: relative;
    z-index: 10;
    padding: 12px 32px;
  }

  .back-btn {
    width: 44px;
    height: 44px;
    background: rgba(0, 0, 0, 0.5);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 50%;
    color: #fff;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
  }

  .back-btn:hover {
    background: rgba(0, 0, 0, 0.8);
    border-color: rgba(255, 255, 255, 0.3);
  }

  .back-btn:focus {
    outline: none;
    box-shadow: 0 0 0 3px rgba(229, 9, 20, 0.5);
  }

  .back-btn svg {
    width: 22px;
    height: 22px;
  }

  .content {
    position: relative;
    z-index: 10;
    padding: 0 32px 24px;
  }

  .series-info {
    display: flex;
    gap: 24px;
  }

  /* Left column: controls */
  .controls-column {
    flex-shrink: 0;
    width: 220px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  /* Season selector */
  .season-selector {
    margin-bottom: 4px;
  }

  .season-buttons {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .season-btn {
    padding: 6px 12px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #fff;
    cursor: pointer;
    transition: all 0.2s ease;
    font-size: 0.8rem;
    font-weight: 600;
  }

  .season-btn:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .season-btn:focus {
    outline: none;
    border-color: #e50914;
    box-shadow: 0 0 0 2px rgba(229, 9, 20, 0.5);
  }

  .season-btn.selected {
    background: rgba(229, 9, 20, 0.15);
    border-color: #e50914;
    color: #e50914;
  }

  /* Episode list */
  .episode-selector {
    margin-bottom: 4px;
  }

  .loading-text {
    font-size: 0.8rem;
    color: #888;
  }

  .episode-list {
    display: flex;
    flex-direction: column;
    gap: 3px;
    max-height: 200px;
    overflow-y: auto;
  }

  .episode-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 8px;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid transparent;
    border-radius: 4px;
    color: #ccc;
    cursor: pointer;
    transition: all 0.2s ease;
    font-size: 0.8rem;
    text-align: left;
  }

  .episode-btn:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .episode-btn:focus {
    outline: none;
    border-color: #e50914;
  }

  .episode-btn.selected {
    background: rgba(229, 9, 20, 0.12);
    border-color: #e50914;
    color: #fff;
  }

  .episode-btn:not(.has-torrents) {
    opacity: 0.4;
  }

  .episode-number {
    font-weight: 700;
    color: #e50914;
    min-width: 28px;
    font-size: 0.75rem;
  }

  .episode-title {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .no-episodes {
    color: #666;
    font-size: 0.8rem;
    margin: 0;
  }

  /* Quality section */
  .quality-section {
    margin-top: 4px;
  }

  .quality-options {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .quality-btn {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 6px 10px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #fff;
    cursor: pointer;
    transition: all 0.2s ease;
    width: 100%;
    font-size: 0.85rem;
  }

  .quality-btn:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .quality-btn:focus {
    outline: none;
    border-color: #e50914;
    box-shadow: 0 0 0 2px rgba(229, 9, 20, 0.5);
  }

  .quality-btn.selected {
    background: rgba(229, 9, 20, 0.15);
    border-color: #e50914;
  }

  .quality-btn.no-seeds {
    opacity: 0.4;
  }

  .quality-label {
    font-size: 1rem;
    font-weight: 700;
  }

  .quality-meta {
    font-size: 0.75rem;
    color: #888;
  }

  /* Play button */
  .play-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    width: 100%;
    padding: 10px 16px;
    background: #e50914;
    border: none;
    border-radius: 6px;
    color: #fff;
    font-size: 0.95rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .play-btn:hover:not(:disabled) {
    background: #f40612;
  }

  .play-btn:focus {
    outline: none;
    box-shadow: 0 0 0 4px #fff, 0 0 0 7px #e50914;
    transform: scale(1.02);
  }

  .play-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .play-btn svg {
    width: 24px;
    height: 24px;
  }

  .btn-spinner {
    width: 20px;
    height: 20px;
    border: 3px solid rgba(255, 255, 255, 0.3);
    border-top-color: #fff;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .no-sources-inline {
    font-size: 0.8rem;
    color: #666;
    text-align: center;
    padding: 8px;
  }

  .stream-error {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    background: rgba(220, 38, 38, 0.15);
    border: 1px solid rgba(220, 38, 38, 0.3);
    border-radius: 6px;
    color: #ef4444;
    font-size: 0.8rem;
  }

  .stream-error svg {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
  }

  /* Right column: info */
  .info {
    flex: 1;
    max-width: 800px;
  }

  .title {
    font-size: 1.6rem;
    font-weight: 700;
    margin: 0 0 6px;
    line-height: 1.1;
  }

  .meta {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px;
    margin-bottom: 8px;
    font-size: 0.85rem;
    color: #aaa;
  }

  .status-badge {
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 0.7rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .status-badge.continuing {
    background: #2ecc71;
    color: #fff;
  }

  .status-badge.ended {
    background: rgba(255, 255, 255, 0.15);
    color: #aaa;
  }

  .network {
    color: #e50914;
  }

  /* Ratings */
  .ratings-row {
    display: flex;
    gap: 8px;
    margin-bottom: 10px;
  }

  .rating-badge {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    background: rgba(255, 255, 255, 0.06);
    border-radius: 4px;
  }

  .rating-badge .rating-label {
    font-size: 0.65rem;
    color: #888;
    text-transform: uppercase;
  }

  .rating-badge .rating-value {
    font-size: 0.9rem;
    font-weight: 700;
    color: #fff;
  }

  .rating-badge.imdb .rating-value { color: #f5c518; }
  .rating-badge.rt .rating-value { color: #fa320a; }

  /* Genres */
  .genres {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 8px;
  }

  .genre-tag {
    padding: 3px 10px;
    background: rgba(255, 255, 255, 0.08);
    border-radius: 12px;
    font-size: 0.75rem;
    color: #ccc;
  }

  /* Description */
  .description {
    font-size: 0.85rem;
    line-height: 1.5;
    color: #999;
    margin: 0 0 6px;
  }

  .description.truncated {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  /* Episode preview inline */
  .episode-preview {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
    font-size: 0.85rem;
  }

  .episode-tag {
    display: inline-block;
    background: #e50914;
    color: #fff;
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 700;
  }

  .episode-preview-title {
    color: #fff;
    font-weight: 500;
  }

  .episode-preview-date {
    color: #666;
    font-size: 0.8rem;
  }

  /* More button */
  .more-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 12px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 6px;
    color: #ccc;
    font-size: 0.8rem;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .more-btn:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #fff;
  }

  .more-btn:focus {
    outline: none;
    border-color: #e50914;
    box-shadow: 0 0 0 2px rgba(229, 9, 20, 0.5);
  }

  .more-btn svg {
    width: 16px;
    height: 16px;
  }

  /* Modal */
  .modal-overlay {
    position: fixed;
    inset: 0;
    z-index: 100;
    background: rgba(0, 0, 0, 0.85);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .modal-content {
    background: #1a1a1a;
    border-radius: 12px;
    width: 90%;
    max-width: 700px;
    max-height: 85vh;
    display: flex;
    flex-direction: column;
    padding: 24px 24px 0;
    border: 1px solid rgba(255, 255, 255, 0.1);
    overflow: hidden;
  }

  .modal-body {
    overflow-y: auto;
    flex: 1;
    padding-bottom: 24px;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .modal-header h2 {
    font-size: 1.3rem;
    font-weight: 700;
    margin: 0;
    color: #fff;
  }

  .modal-close {
    width: 36px;
    height: 36px;
    background: rgba(255, 255, 255, 0.1);
    border: none;
    border-radius: 50%;
    color: #aaa;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
  }

  .modal-close:hover { background: rgba(255, 255, 255, 0.2); color: #fff; }
  .modal-close:focus { outline: none; box-shadow: 0 0 0 2px #e50914; }
  .modal-close svg { width: 20px; height: 20px; }

  .modal-description {
    font-size: 0.85rem;
    line-height: 1.6;
    color: #999;
    margin: 0 0 16px;
  }

  .modal-episode {
    padding: 12px;
    background: rgba(255, 255, 255, 0.04);
    border-radius: 8px;
  }

  .modal-episode h3 {
    font-size: 1rem;
    font-weight: 600;
    margin: 0 0 4px;
    color: #fff;
  }

  .modal-episode-date {
    font-size: 0.8rem;
    color: #666;
  }

  .modal-episode-summary {
    font-size: 0.85rem;
    line-height: 1.5;
    color: #999;
    margin: 8px 0 0;
  }

  /* Loading / Error */
  .loading-container,
  .error-container {
    position: relative;
    z-index: 10;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 80vh;
    gap: 20px;
  }

  .spinner {
    width: 50px;
    height: 50px;
    border: 4px solid rgba(229, 9, 20, 0.2);
    border-top-color: #e50914;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  .error-container > svg {
    width: 60px;
    height: 60px;
    color: #e50914;
  }

  .error-container p {
    font-size: 1.1rem;
    color: #888;
    margin: 0;
  }

  .error-buttons {
    display: flex;
    gap: 12px;
    margin-top: 16px;
  }

  .back-button,
  .retry-button {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 12px 24px;
    border-radius: 8px;
    font-size: 0.95rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .back-button {
    background: transparent;
    border: 2px solid rgba(255, 255, 255, 0.3);
    color: #fff;
  }

  .back-button:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .back-button svg {
    width: 18px;
    height: 18px;
  }

  .retry-button {
    background: #e50914;
    border: 2px solid #e50914;
    color: #fff;
  }

  .retry-button:hover {
    background: #f40612;
  }

  /* No column-breaking media queries for TV */
  @media (max-width: 768px) {
    .nav { padding: 12px 16px; }
    .content { padding: 0 16px 24px; }
    .title { font-size: 1.4rem; }
  }
</style>
