<script lang="ts">
  import { goto } from "$app/navigation";
  import { invoke } from "@tauri-apps/api/core";
  import { listMovies, listSeries, API_URL } from "$lib/api/commands";
  import type { Movie, Series, Channel, MovieListData } from "$lib/api/types";
  import MovieGrid from "./MovieGrid.svelte";
  import SeriesGrid from "./SeriesGrid.svelte";

  type SearchType = 'movies' | 'tvshows' | 'channels';

  interface Props {
    type: SearchType;
    channels?: Channel[];
  }

  let { type, channels = [] }: Props = $props();

  let query = $state('');
  let loading = $state(false);
  let movieResults = $state<Movie[]>([]);
  let seriesResults = $state<Series[]>([]);
  let channelResults = $state<Channel[]>([]);
  let hasSearched = $state(false);
  let syncingMovies = $state<Set<string>>(new Set());

  const placeholder = $derived(
    type === 'movies' ? 'Search movies...' :
    type === 'tvshows' ? 'Search TV series...' :
    'Search channels...'
  );

  async function handleSearch() {
    const searchQuery = query.trim();
    if (!searchQuery) return;

    loading = true;
    hasSearched = true;

    try {
      if (type === 'movies') {
        // Search both local DB and YTS in parallel
        const [localData, ytsData] = await Promise.all([
          listMovies({ query_term: searchQuery, limit: 50, page: 1 }).catch(() => ({ movies: [] })),
          invoke<MovieListData>("list_movies", { params: { query_term: searchQuery, limit: 50, page: 1 } }).catch(() => ({ movies: [] }))
        ]);

        const localMovies = localData?.movies || [];
        const ytsMovies = (ytsData?.movies || []) as Movie[];

        // Create a map of local movies by IMDB code for quick lookup
        const localImdbCodes = new Set(localMovies.map(m => m.imdb_code));

        // Mark YTS movies that aren't in local DB
        const ytsOnlyMovies = ytsMovies
          .filter(m => m.imdb_code && !localImdbCodes.has(m.imdb_code))
          .map(m => ({ ...m, _isYtsOnly: true }));

        // Merge: local movies first, then YTS-only movies
        movieResults = [...localMovies, ...ytsOnlyMovies];
        console.log(`[Search] Found ${localMovies.length} local, ${ytsOnlyMovies.length} YTS-only movies`);
      } else if (type === 'tvshows') {
        // Search both local DB and IMDB API in parallel
        const [localData, imdbData] = await Promise.all([
          listSeries({ limit: 200, page: 1 }).catch(() => ({ series: [] })),
          searchImdbSeries(searchQuery).catch(() => [])
        ]);

        // Filter local series by search query (server doesn't support query_term)
        const searchLower = searchQuery.toLowerCase();
        const localSeries = (localData?.series || []).filter(s =>
          s.title.toLowerCase().includes(searchLower)
        );
        const imdbSeries = imdbData || [];

        // Create a map of local series by IMDB code for quick lookup
        const localImdbCodes = new Set(localSeries.map(s => s.imdb_code));

        // Mark IMDB series that aren't in local DB
        const imdbOnlySeries = imdbSeries
          .filter(s => s.imdb_code && !localImdbCodes.has(s.imdb_code))
          .map(s => ({ ...s, _isOmdbOnly: true }));

        // Merge: IMDB matches first (more relevant), then local matches
        seriesResults = [...imdbOnlySeries, ...localSeries];
        console.log(`[Search] Found ${localSeries.length} local, ${imdbOnlySeries.length} IMDB-only series`);
      } else if (type === 'channels') {
        // Filter channels client-side
        const lowerQuery = searchQuery.toLowerCase();
        channelResults = channels.filter(ch =>
          ch.name.toLowerCase().includes(lowerQuery)
        ).slice(0, 50);
      }
    } catch (err) {
      console.error(`[Search] ${type} search failed:`, err);
      movieResults = [];
      seriesResults = [];
      channelResults = [];
    } finally {
      loading = false;
    }
  }

  // IMDB API (api.imdbapi.dev)
  const IMDB_API_BASE = 'https://api.imdbapi.dev';

  // Helper to extract quality from filename
  function extractQuality(filename: string): string {
    const f = filename.toLowerCase();
    if (f.includes('2160p') || f.includes('4k')) return '2160p';
    if (f.includes('1080p')) return '1080p';
    if (f.includes('720p')) return '720p';
    if (f.includes('480p')) return '480p';
    return 'Unknown';
  }

  // Helper to format bytes
  function formatBytes(bytes: number): string {
    if (!bytes || bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  async function searchImdbSeries(query: string): Promise<Series[]> {
    try {
      const response = await fetch(
        `${IMDB_API_BASE}/search/titles?query=${encodeURIComponent(query)}&titleType=tvSeries,tvMiniSeries&limit=20`
      );
      const data = await response.json();

      if (!data.results || data.results.length === 0) {
        return [];
      }

      // Convert IMDB API results to Series format
      return data.results.map((item: any) => ({
        id: 0, // Will be set after sync
        imdb_code: item.id,
        title: item.primaryTitle || item.originalTitle,
        year: item.startYear || 0,
        poster_image: item.primaryImage?.url || '',
        rating: item.averageRating || 0,
        genres: item.genres || [],
        summary: item.plot || '',
        status: item.endYear ? 'ended' : 'ongoing',
        total_seasons: 0,
        total_episodes: 0
      }));
    } catch (err) {
      console.error('[IMDB API] Search failed:', err);
      return [];
    }
  }

  async function syncSeries(series: Series) {
    if (!series.imdb_code) return;

    try {
      console.log(`[Search] Syncing series: ${series.title} (${series.imdb_code})`);

      // Fetch full details from IMDB API
      const [detailsRes, seasonsRes] = await Promise.all([
        fetch(`${IMDB_API_BASE}/titles/${series.imdb_code}`),
        fetch(`${IMDB_API_BASE}/titles/${series.imdb_code}/seasons`)
      ]);

      const details = await detailsRes.json();
      const seasonsData = await seasonsRes.json();

      // Build full series object
      const fullSeries = {
        imdb_code: series.imdb_code,
        title: details.primaryTitle || details.originalTitle || series.title,
        title_slug: (details.primaryTitle || series.title).toLowerCase().replace(/[^a-z0-9]+/g, '-'),
        year: details.startYear || series.year || 0,
        rating: details.averageRating || 0,
        runtime: details.runtime || 0,
        genres: details.genres || [],
        summary: details.plot || '',
        status: details.endYear ? 'ended' : 'ongoing',
        poster_image: details.primaryImage?.url || series.poster_image || '',
        background_image: details.primaryImage?.url || '',
        total_seasons: seasonsData.seasons?.length || 0,
        total_episodes: seasonsData.seasons?.reduce((sum: number, s: any) => sum + (s.episodeCount || 0), 0) || 0
      };

      console.log(`[Search] Got details: ${fullSeries.title}, ${fullSeries.total_seasons} seasons`);

      // Fetch torrents from EZTV (IMDB ID without "tt" prefix)
      const imdbNum = series.imdb_code.replace('tt', '');
      let eztvTorrents: any[] = [];
      try {
        const eztvRes = await fetch(`https://eztvx.to/api/get-torrents?imdb_id=${imdbNum}&limit=100`);
        const eztvData = await eztvRes.json();
        eztvTorrents = eztvData.torrents || [];
        console.log(`[Search] Got ${eztvTorrents.length} torrents from EZTV`);
      } catch (err) {
        console.warn('[Search] EZTV fetch failed:', err);
      }

      // Group torrents by season/episode and build episodes array
      const episodeMap = new Map<string, any>();
      for (const t of eztvTorrents) {
        const key = `${t.season}-${t.episode}`;
        if (!episodeMap.has(key)) {
          episodeMap.set(key, {
            season_number: t.season,
            episode_number: t.episode,
            title: t.title || `Episode ${t.episode}`,
            torrents: []
          });
        }
        episodeMap.get(key).torrents.push({
          hash: t.hash,
          quality: t.quality || extractQuality(t.filename || t.title),
          seeds: t.seeds || 0,
          peers: t.peers || 0,
          size: t.size || formatBytes(t.size_bytes),
          size_bytes: t.size_bytes || 0,
          season_number: t.season,
          episode_number: t.episode
        });
      }

      // Convert to array and add to fullSeries
      const episodes = Array.from(episodeMap.values());
      console.log(`[Search] Grouped into ${episodes.length} episodes`);

      const response = await fetch(`${API_URL}/api/v2/sync_series`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ ...fullSeries, episodes })
      });

      if (response.ok) {
        const data = await response.json();
        const newId = data.data?.id;
        console.log(`[Search] Synced series: ${series.title}, ID: ${newId}`);
        // Update the series in results with the new ID
        seriesResults = seriesResults.map(s =>
          s.imdb_code === series.imdb_code ? { ...s, id: newId, _isOmdbOnly: false } : s
        );
        return newId;
      }
    } catch (err) {
      console.error(`[Search] Failed to sync series:`, err);
    }
    return null;
  }

  async function handleSeriesClick(series: Series) {
    // If it's an OMDB-only series, sync it first
    if ((series as any)._isOmdbOnly) {
      const newId = await syncSeries(series);
      if (newId) {
        goto(`/series/${newId}`);
      }
    } else {
      goto(`/series/${series.id}`);
    }
  }

  async function syncMovie(movie: Movie) {
    if (!movie.imdb_code || syncingMovies.has(movie.imdb_code)) return;

    syncingMovies = new Set([...syncingMovies, movie.imdb_code]);

    try {
      // Call the server to sync this movie from YTS
      const response = await fetch(`${API_URL}/api/v2/sync_movie`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ imdb_code: movie.imdb_code })
      });

      if (response.ok) {
        // Remove the _isYtsOnly flag and refresh
        movieResults = movieResults.map(m =>
          m.imdb_code === movie.imdb_code ? { ...m, _isYtsOnly: false } : m
        );
        console.log(`[Search] Synced movie: ${movie.title}`);
      }
    } catch (err) {
      console.error(`[Search] Failed to sync movie:`, err);
    } finally {
      syncingMovies = new Set([...syncingMovies].filter(c => c !== movie.imdb_code));
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      handleSearch();
    }
  }

  function playChannel(channel: Channel) {
    if (channel.stream_url) {
      goto(`/live?url=${encodeURIComponent(channel.stream_url)}&title=${encodeURIComponent(channel.name)}`);
    }
  }

  const resultCount = $derived(
    type === 'movies' ? movieResults.length :
    type === 'tvshows' ? seriesResults.length :
    channelResults.length
  );

  const noResults = $derived(hasSearched && !loading && resultCount === 0);
</script>

<div class="search-section">
  <form class="search-form" onsubmit={(e) => { e.preventDefault(); handleSearch(); }}>
    <div class="search-input-wrapper">
      <svg class="search-icon" viewBox="0 0 24 24" fill="currentColor">
        <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
      </svg>
      <input
        type="text"
        class="search-input"
        {placeholder}
        bind:value={query}
        onkeydown={handleKeydown}
      />
      {#if query}
        <button
          type="button"
          class="clear-btn"
          onclick={() => { query = ''; movieResults = []; seriesResults = []; channelResults = []; hasSearched = false; }}
        >
          <svg viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
          </svg>
        </button>
      {/if}
    </div>
    <button type="submit" class="search-btn" disabled={loading || !query.trim()}>
      {#if loading}
        <div class="spinner"></div>
      {:else}
        Search
      {/if}
    </button>
  </form>

  {#if loading}
    <div class="loading-state">
      <div class="spinner large"></div>
      <p>Searching...</p>
    </div>
  {:else if noResults}
    <div class="no-results">
      <svg viewBox="0 0 24 24" fill="currentColor">
        <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
      </svg>
      <p>No {type === 'movies' ? 'movies' : type === 'tvshows' ? 'TV series' : 'channels'} found for "{query}"</p>
    </div>
  {:else if resultCount > 0}
    {#if type === 'movies'}
      {@const localCount = movieResults.filter(m => !(m as any)._isYtsOnly).length}
      {@const ytsCount = movieResults.filter(m => (m as any)._isYtsOnly).length}
      <p class="results-count">
        {resultCount} result{resultCount !== 1 ? 's' : ''} for "{query}"
        {#if ytsCount > 0}
          <span class="yts-note">({localCount} in library, {ytsCount} from YTS)</span>
        {/if}
      </p>
      <MovieGrid movies={movieResults} loading={false} error={null} />
    {:else if type === 'tvshows'}
      {@const localCount = seriesResults.filter(s => !(s as any)._isOmdbOnly).length}
      {@const omdbCount = seriesResults.filter(s => (s as any)._isOmdbOnly).length}
      <p class="results-count">
        {resultCount} result{resultCount !== 1 ? 's' : ''} for "{query}"
        {#if omdbCount > 0}
          <span class="yts-note">({localCount} in library, {omdbCount} from IMDB)</span>
        {/if}
      </p>
      <SeriesGrid series={seriesResults} loading={false} error={null} onSeriesClick={handleSeriesClick} />
    {:else if type === 'channels'}
      <div class="channels-list">
        {#each channelResults as channel (channel.id)}
          <button class="channel-item" onclick={() => playChannel(channel)}>
            {#if channel.logo}
              <img src={channel.logo} alt="" class="channel-logo" />
            {:else}
              <div class="channel-logo-placeholder">
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M17 10.5V7c0-.55-.45-1-1-1H4c-.55 0-1 .45-1 1v10c0 .55.45 1 1 1h12c.55 0 1-.45 1-1v-3.5l4 4v-11l-4 4z"/>
                </svg>
              </div>
            {/if}
            <span class="channel-name">{channel.name}</span>
            <svg class="play-icon" viewBox="0 0 24 24" fill="currentColor">
              <path d="M8 5v14l11-7z"/>
            </svg>
          </button>
        {/each}
      </div>
    {/if}
  {:else if !hasSearched}
    <div class="search-hint">
      <svg viewBox="0 0 24 24" fill="currentColor">
        <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
      </svg>
      <p>Enter a search term to find {type === 'movies' ? 'movies' : type === 'tvshows' ? 'TV series' : 'channels'}</p>
    </div>
  {/if}
</div>

<style>
  .search-section {
    padding: 20px 0;
  }

  .search-form {
    display: flex;
    gap: 12px;
    max-width: 600px;
    margin-bottom: 24px;
  }

  .search-input-wrapper {
    flex: 1;
    position: relative;
    display: flex;
    align-items: center;
  }

  .search-icon {
    position: absolute;
    left: 16px;
    width: 20px;
    height: 20px;
    color: rgba(255, 255, 255, 0.5);
    pointer-events: none;
  }

  .search-input {
    width: 100%;
    padding: 14px 44px;
    background: rgba(255, 255, 255, 0.1);
    border: 2px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    color: #fff;
    font-size: 1rem;
    transition: all 0.2s ease;
  }

  .search-input:focus {
    outline: none;
    border-color: #e50914;
    background: rgba(255, 255, 255, 0.15);
  }

  .search-input::placeholder {
    color: rgba(255, 255, 255, 0.5);
  }

  .clear-btn {
    position: absolute;
    right: 12px;
    background: none;
    border: none;
    padding: 4px;
    cursor: pointer;
    color: rgba(255, 255, 255, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: color 0.2s;
  }

  .clear-btn:hover {
    color: #fff;
  }

  .clear-btn svg {
    width: 20px;
    height: 20px;
  }

  .search-btn {
    padding: 14px 28px;
    background: #e50914;
    border: none;
    border-radius: 8px;
    color: #fff;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    transition: background 0.2s ease;
    min-width: 100px;
  }

  .search-btn:hover:not(:disabled) {
    background: #f40612;
  }

  .search-btn:disabled {
    background: #666;
    cursor: not-allowed;
  }

  .spinner {
    width: 20px;
    height: 20px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: #fff;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .spinner.large {
    width: 40px;
    height: 40px;
    border-width: 3px;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .loading-state, .no-results, .search-hint {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 60px 20px;
    color: rgba(255, 255, 255, 0.6);
    text-align: center;
  }

  .loading-state p, .no-results p, .search-hint p {
    margin-top: 16px;
    font-size: 1.1rem;
  }

  .no-results svg, .search-hint svg {
    width: 48px;
    height: 48px;
    opacity: 0.5;
  }

  .results-count {
    color: rgba(255, 255, 255, 0.7);
    margin-bottom: 20px;
    font-size: 0.95rem;
  }

  .yts-note {
    color: #e50914;
    font-size: 0.85rem;
  }

  .channels-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .channel-item {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 12px 16px;
    background: rgba(255, 255, 255, 0.05);
    border: none;
    border-radius: 8px;
    cursor: pointer;
    transition: background 0.2s;
    text-align: left;
    color: #fff;
    width: 100%;
  }

  .channel-item:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .channel-logo {
    width: 48px;
    height: 48px;
    object-fit: contain;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.1);
  }

  .channel-logo-placeholder {
    width: 48px;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    color: rgba(255, 255, 255, 0.5);
  }

  .channel-logo-placeholder svg {
    width: 24px;
    height: 24px;
  }

  .channel-name {
    flex: 1;
    font-size: 1rem;
    font-weight: 500;
  }

  .play-icon {
    width: 24px;
    height: 24px;
    color: rgba(255, 255, 255, 0.5);
    transition: color 0.2s;
  }

  .channel-item:hover .play-icon {
    color: #e50914;
  }
</style>
