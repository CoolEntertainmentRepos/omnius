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
      await goto(`/player/${selectedTorrent.hash}?${params.toString()}`);
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
        <button class="back-button" onclick={() => goto('/?tab=tvshows')}>
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
      <button class="back-btn" onclick={() => goto('/?tab=tvshows')} aria-label="Go back">
        <svg viewBox="0 0 24 24" fill="currentColor">
          <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z" />
        </svg>
      </button>
    </nav>

    <main class="content">
      <!-- Series Info Section -->
      <div class="series-info">
        <div class="poster-container">
          <img
            src={series.poster_image}
            alt="{series.title} poster"
            class="poster"
          />
        </div>

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

          <!-- Ratings Row -->
          {#if series.rating && series.rating > 0}
            <div class="ratings-row">
              <span class="rating-badge imdb">
                <span class="rating-label">IMDb</span>
                <span class="rating-value">{series.rating.toFixed(1)}</span>
              </span>
              {#if series.rotten_tomatoes}
                <span class="rating-badge rt">
                  <span class="rating-label">🍅 Rotten</span>
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

          <p class="description">
            {series.summary || "No description available."}
          </p>

          <!-- Season Selector -->
          <div class="season-selector">
            <h3>Select Season</h3>
            <div class="season-buttons">
              {#each Array.from({ length: series.total_seasons }, (_, i) => i + 1) as seasonNum (seasonNum)}
                <button
                  class="season-btn"
                  class:selected={selectedSeason === seasonNum}
                  onclick={() => selectSeason(seasonNum)}
                >
                  Season {seasonNum}
                </button>
              {/each}
            </div>
          </div>

          <!-- Episode Selector -->
          {#if selectedSeason}
            <div class="episode-selector">
              <h3>Episodes {#if episodesLoading}<span class="loading-text">Loading...</span>{/if}</h3>
              {#if episodes.length > 0}
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
                      {#if episode.torrents && episode.torrents.length > 0}
                        <span class="episode-sources">{episode.torrents.length}</span>
                      {/if}
                    </button>
                  {/each}
                </div>
              {:else if !episodesLoading}
                <p class="no-episodes">No episodes found for this season.</p>
              {/if}
            </div>
          {/if}

          <!-- Selected Episode Details & Torrent Selection -->
          {#if selectedEpisode}
            <div class="selected-episode">
              <div class="episode-info">
                <span class="episode-tag">
                  S{String(selectedEpisode.season_number).padStart(2, '0')}E{String(selectedEpisode.episode_number).padStart(2, '0')}
                </span>
                <h4>{selectedEpisode.title}</h4>
                {#if selectedEpisode.air_date}
                  <span class="episode-date">{selectedEpisode.air_date}</span>
                {/if}
              </div>

              {#if selectedEpisode.summary}
                <p class="episode-summary">{selectedEpisode.summary}</p>
              {/if}

              {#if selectedEpisode.torrents && selectedEpisode.torrents.length > 0}
                <div class="quality-section">
                  <h3>Select Quality</h3>
                  <div class="quality-options">
                    {#each selectedEpisode.torrents as torrent (torrent.id)}
                      <button
                        class="quality-btn"
                        class:selected={selectedTorrent?.id === torrent.id}
                        class:no-seeds={torrent.seeds === 0}
                        onclick={() => selectTorrent(torrent)}
                      >
                        <span class="quality-label">{torrent.quality}</span>
                        <span class="quality-size">{formatSize(torrent.size)}</span>
                        <span class="quality-seeds" class:warning={torrent.seeds === 0}>
                          <svg viewBox="0 0 24 24" fill="currentColor">
                            <path d="M16 6l2.29 2.29-4.88 4.88-4-4L2 16.59 3.41 18l6-6 4 4 6.3-6.29L22 12V6z"/>
                          </svg>
                          {torrent.seeds}
                        </span>
                      </button>
                    {/each}
                  </div>
                </div>

                <!-- Play Button -->
                <div class="play-section">
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
                </div>

                {#if streamError}
                  <div class="stream-error">
                    <svg viewBox="0 0 24 24" fill="currentColor">
                      <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
                    </svg>
                    {streamError}
                  </div>
                {/if}
              {:else}
                <div class="no-sources">
                  <svg viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
                  </svg>
                  <span>No sources available for this episode</span>
                </div>
              {/if}
            </div>
          {/if}
        </div>
      </div>
    </main>
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
    padding: 30px 50px;
  }

  .back-btn {
    width: 50px;
    height: 50px;
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
    box-shadow: 0 0 0 3px rgba(46, 204, 113, 0.5);
  }

  .back-btn svg {
    width: 24px;
    height: 24px;
  }

  .content {
    position: relative;
    z-index: 10;
    padding: 0 50px 60px;
  }

  .series-info {
    display: flex;
    gap: 50px;
    margin-bottom: 60px;
  }

  .poster-container {
    flex-shrink: 0;
  }

  .poster {
    width: 300px;
    border-radius: 12px;
    box-shadow: 0 8px 40px rgba(0, 0, 0, 0.6);
  }

  .info {
    flex: 1;
    max-width: 900px;
  }

  .title {
    font-size: 3rem;
    font-weight: 700;
    margin: 0 0 20px;
    line-height: 1.1;
  }

  .meta {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 16px;
    margin-bottom: 16px;
    font-size: 1.1rem;
    color: #aaa;
  }

  .status-badge {
    padding: 4px 12px;
    border-radius: 4px;
    font-size: 0.75rem;
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
    color: #2ecc71;
  }

  /* Ratings Row */
  .ratings-row {
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
    margin-bottom: 20px;
  }

  .rating-badge {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 12px 20px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    min-width: 90px;
  }

  .rating-badge .rating-label {
    font-size: 0.75rem;
    color: #888;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 4px;
  }

  .rating-badge .rating-value {
    font-size: 1.4rem;
    font-weight: 700;
    color: #fff;
  }

  .rating-badge.imdb .rating-value {
    color: #f5c518;
  }

  .rating-badge.rt .rating-value {
    color: #fa320a;
  }

  .genres {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    margin-bottom: 16px;
  }

  .genre-tag {
    padding: 8px 16px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 20px;
    font-size: 0.95rem;
  }

  .description {
    font-size: 1.1rem;
    line-height: 1.7;
    color: #999;
    margin: 0 0 30px;
  }

  /* Season Selector */
  .season-selector {
    margin-bottom: 24px;
  }

  .season-selector h3,
  .episode-selector h3,
  .quality-section h3 {
    font-size: 1.2rem;
    font-weight: 600;
    margin: 0 0 12px;
    color: #fff;
  }

  .loading-text {
    font-size: 0.9rem;
    color: #888;
    font-weight: 400;
    margin-left: 8px;
  }

  .season-buttons {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
  }

  .season-btn {
    padding: 12px 20px;
    background: rgba(255, 255, 255, 0.05);
    border: 2px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    color: #fff;
    cursor: pointer;
    transition: all 0.2s ease;
    font-size: 1rem;
    font-weight: 500;
  }

  .season-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.2);
  }

  .season-btn:focus {
    outline: none;
    border-color: #2ecc71;
    box-shadow: 0 0 0 3px rgba(46, 204, 113, 0.3);
  }

  .season-btn.selected {
    background: rgba(46, 204, 113, 0.15);
    border-color: #2ecc71;
    color: #2ecc71;
  }

  /* Episode Selector */
  .episode-selector {
    margin-bottom: 24px;
  }

  .episode-list {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    max-height: 200px;
    overflow-y: auto;
    padding-right: 8px;
  }

  .episode-btn {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 16px;
    background: rgba(255, 255, 255, 0.05);
    border: 2px solid transparent;
    border-radius: 8px;
    color: #ccc;
    cursor: pointer;
    transition: all 0.2s ease;
    font-size: 0.95rem;
  }

  .episode-btn:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .episode-btn:focus {
    outline: none;
    border-color: #2ecc71;
  }

  .episode-btn.selected {
    background: rgba(46, 204, 113, 0.15);
    border-color: #2ecc71;
    color: #fff;
  }

  .episode-btn:not(.has-torrents) {
    opacity: 0.4;
  }

  .episode-number {
    font-weight: 700;
    color: #2ecc71;
    min-width: 36px;
  }

  .episode-title {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 180px;
  }

  .episode-sources {
    background: rgba(46, 204, 113, 0.2);
    color: #2ecc71;
    padding: 2px 8px;
    border-radius: 8px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .no-episodes {
    color: #666;
    font-size: 0.95rem;
  }

  /* Selected Episode */
  .selected-episode {
    padding: 24px;
    background: rgba(255, 255, 255, 0.03);
    border-radius: 12px;
    margin-bottom: 24px;
  }

  .episode-info {
    margin-bottom: 16px;
  }

  .episode-tag {
    display: inline-block;
    background: #2ecc71;
    color: #fff;
    padding: 4px 10px;
    border-radius: 4px;
    font-size: 0.8rem;
    font-weight: 700;
    margin-bottom: 10px;
  }

  .episode-info h4 {
    font-size: 1.3rem;
    font-weight: 600;
    margin: 0 0 6px;
    color: #fff;
  }

  .episode-date {
    font-size: 0.9rem;
    color: #888;
  }

  .episode-summary {
    font-size: 0.95rem;
    line-height: 1.6;
    color: #999;
    margin: 0 0 20px;
  }

  /* Quality Section */
  .quality-section {
    margin-bottom: 24px;
  }

  .quality-options {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
  }

  .quality-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 14px 20px;
    background: rgba(255, 255, 255, 0.05);
    border: 2px solid rgba(255, 255, 255, 0.1);
    border-radius: 10px;
    color: #fff;
    cursor: pointer;
    transition: all 0.2s ease;
    min-width: 100px;
  }

  .quality-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.2);
  }

  .quality-btn:focus {
    outline: none;
    border-color: #2ecc71;
    box-shadow: 0 0 0 3px rgba(46, 204, 113, 0.3);
  }

  .quality-btn.selected {
    background: rgba(46, 204, 113, 0.15);
    border-color: #2ecc71;
  }

  .quality-btn.no-seeds {
    opacity: 0.5;
  }

  .quality-label {
    font-size: 1.3rem;
    font-weight: 700;
  }

  .quality-size {
    font-size: 0.9rem;
    color: #888;
  }

  .quality-seeds {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 0.85rem;
    color: #2ecc71;
  }

  .quality-seeds.warning {
    color: #e74c3c;
  }

  .quality-seeds svg {
    width: 14px;
    height: 14px;
  }

  /* Play Section */
  .play-section {
    display: flex;
    gap: 16px;
  }

  .play-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 18px 48px;
    background: #2ecc71;
    border: none;
    border-radius: 8px;
    color: #fff;
    font-size: 1.3rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .play-btn:hover:not(:disabled) {
    background: #27ae60;
    transform: scale(1.02);
  }

  .play-btn:focus {
    outline: none;
    box-shadow: 0 0 0 5px #fff, 0 0 0 8px #2ecc71;
    transform: scale(1.05);
  }

  .play-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .play-btn svg {
    width: 28px;
    height: 28px;
  }

  .btn-spinner {
    width: 24px;
    height: 24px;
    border: 3px solid rgba(255, 255, 255, 0.3);
    border-top-color: #fff;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .stream-error {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 14px 20px;
    background: rgba(220, 38, 38, 0.15);
    border: 1px solid rgba(220, 38, 38, 0.3);
    border-radius: 8px;
    color: #ef4444;
    font-size: 1rem;
    margin-top: 16px;
  }

  .stream-error svg {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
  }

  .no-sources {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 24px;
    color: #666;
  }

  .no-sources svg {
    width: 24px;
    height: 24px;
  }

  /* Loading & Error states */
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
    width: 60px;
    height: 60px;
    border: 4px solid rgba(46, 204, 113, 0.2);
    border-top-color: #2ecc71;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  .error-container > svg {
    width: 80px;
    height: 80px;
    color: #2ecc71;
  }

  .error-container p {
    font-size: 1.3rem;
    color: #888;
    margin: 0;
  }

  .error-buttons {
    display: flex;
    gap: 12px;
    margin-top: 20px;
  }

  .back-button,
  .retry-button {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 14px 32px;
    border-radius: 8px;
    font-size: 1rem;
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
    border-color: rgba(255, 255, 255, 0.5);
  }

  .back-button svg {
    width: 18px;
    height: 18px;
  }

  .retry-button {
    background: #2ecc71;
    border: 2px solid #2ecc71;
    color: #fff;
  }

  .retry-button:hover {
    background: #27ae60;
    border-color: #27ae60;
  }

  /* Responsive */
  @media (max-width: 1024px) {
    .nav {
      padding: 24px 30px;
    }

    .content {
      padding: 0 30px 50px;
    }

    .series-info {
      gap: 30px;
    }

    .poster {
      width: 250px;
    }

    .title {
      font-size: 2.4rem;
    }
  }

  @media (max-width: 768px) {
    .nav {
      padding: 20px;
    }

    .content {
      padding: 0 20px 40px;
    }

    .series-info {
      flex-direction: column;
      align-items: center;
      text-align: center;
    }

    .poster {
      width: 200px;
    }

    .info {
      display: flex;
      flex-direction: column;
      align-items: center;
    }

    .title {
      font-size: 2rem;
    }

    .meta {
      justify-content: center;
    }

    .genres {
      justify-content: center;
    }

    .season-buttons {
      justify-content: center;
    }

    .episode-list {
      justify-content: center;
    }

    .quality-options {
      justify-content: center;
    }

    .play-section {
      flex-direction: column;
      align-items: center;
    }

    .play-btn {
      width: 100%;
      max-width: 300px;
    }
  }
</style>
