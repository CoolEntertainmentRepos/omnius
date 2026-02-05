// Tauri invoke wrappers for all commands
import { invoke } from "@tauri-apps/api/core";
import { cachedFetch, makeCacheKey, CACHE_TTL, initCache, clearCache, clearAllCaches } from "./cache";

// Initialize cache on module load - clears old version caches
initCache();
import type {
  ListMoviesParams,
  MovieListData,
  MovieDetails,
  Movie,
  MovieRating,
  StreamInfo,
  StreamStats,
} from "./types";

// Local torrent-server API
const API_URL = 'http://localhost:8080';

/**
 * Fetch a list of movies from LOCAL torrent-server (cached)
 */
export async function listMovies(params: ListMoviesParams, forceRefresh: boolean = false): Promise<MovieListData> {
  const cacheKey = makeCacheKey('movies', params as Record<string, unknown>);
  return cachedFetch(
    cacheKey,
    CACHE_TTL.MOVIES_LIST,
    async () => {
      const queryParams = new URLSearchParams();
      if (params.limit) queryParams.set('limit', String(params.limit));
      if (params.page) queryParams.set('page', String(params.page));
      if (params.quality) queryParams.set('quality', params.quality);
      if (params.minimum_rating) queryParams.set('minimum_rating', String(params.minimum_rating));
      if (params.query_term) queryParams.set('query_term', params.query_term);
      if (params.genre) queryParams.set('genre', params.genre);
      if (params.sort_by) queryParams.set('sort_by', params.sort_by);
      if (params.order_by) queryParams.set('order_by', params.order_by);
      if (params.year) queryParams.set('year', String(params.year));

      const response = await fetch(`${API_URL}/api/v2/list_movies.json?${queryParams}`);
      const data = await response.json();
      return data.data as MovieListData;
    },
    forceRefresh
  );
}

/**
 * Get detailed information about a specific movie from LOCAL torrent-server (cached)
 */
export async function getMovieDetails(
  movieId: number,
  withCast: boolean = true,
  withImages: boolean = true,
  forceRefresh: boolean = false
): Promise<MovieDetails> {
  const cacheKey = `movie_details_${movieId}_${withCast}_${withImages}`;
  return cachedFetch(
    cacheKey,
    CACHE_TTL.MOVIE_DETAILS,
    async () => {
      console.log("[getMovieDetails] Fetching from LOCAL server, movieId:", movieId);
      const response = await fetch(`${API_URL}/api/v2/movie_details.json?movie_id=${movieId}&with_cast=${withCast}&with_images=${withImages}`);
      if (!response.ok) {
        throw new Error(`Failed to fetch movie details: ${response.status}`);
      }
      const data = await response.json();
      if (!data.data?.movie) {
        throw new Error(`Movie not found: ${movieId}`);
      }
      console.log("[getMovieDetails] Success:", data.data.movie.title);
      return data.data.movie as MovieDetails;
    },
    forceRefresh
  );
}

/**
 * Get movie suggestions/recommendations from LOCAL torrent-server (cached)
 */
export async function getMovieSuggestions(movieId: number, forceRefresh: boolean = false): Promise<Movie[]> {
  const cacheKey = `suggestions_${movieId}`;
  return cachedFetch(
    cacheKey,
    CACHE_TTL.SUGGESTIONS,
    async () => {
      console.log("[getMovieSuggestions] Fetching from LOCAL server, movieId:", movieId);
      const response = await fetch(`${API_URL}/api/v2/movie_suggestions.json?movie_id=${movieId}`);
      if (!response.ok) {
        console.warn("[getMovieSuggestions] Failed:", response.status);
        return [];
      }
      const data = await response.json();
      const movies = data.data?.movies || [];
      console.log("[getMovieSuggestions] Success:", movies.length, "suggestions");
      return movies as Movie[];
    },
    forceRefresh
  );
}

/**
 * Get franchise movies (other movies in the same franchise) from LOCAL torrent-server
 */
export async function getFranchiseMovies(movieId: number): Promise<Movie[]> {
  try {
    console.log("[getFranchiseMovies] Fetching for movieId:", movieId);
    const response = await fetch(`${API_URL}/api/v2/franchise_movies.json?movie_id=${movieId}`);
    if (!response.ok) {
      console.warn("[getFranchiseMovies] Failed:", response.status);
      return [];
    }
    const data = await response.json();
    const movies = data.data?.movies || [];
    console.log("[getFranchiseMovies] Success:", movies.length, "franchise movies");
    return movies as Movie[];
  } catch (err) {
    console.warn("[getFranchiseMovies] Error:", err);
    return [];
  }
}

/**
 * Start streaming a torrent and get the stream info
 */
export async function startStream(torrentHash: string): Promise<StreamInfo> {
  return await invoke<StreamInfo>("start_stream", { torrentHash });
}

/**
 * Stop an active stream
 */
export async function stopStream(infoHash: string): Promise<void> {
  return await invoke<void>("stop_stream", { infoHash });
}

/**
 * Get current streaming statistics
 */
export async function getStreamStats(infoHash: string): Promise<StreamStats> {
  return await invoke<StreamStats>("get_stream_status", { infoHash });
}

/**
 * Get movie rating from OMDB (IMDb, Rotten Tomatoes, Metacritic) - cached
 */
export async function getMovieRating(imdbCode: string): Promise<MovieRating> {
  const cacheKey = `rating_${imdbCode}`;
  return cachedFetch(
    cacheKey,
    CACHE_TTL.MOVIE_DETAILS, // Same TTL as movie details
    () => invoke<MovieRating>("get_movie_rating", { imdbCode })
  );
}

/**
 * IMDB image from imdbapi.dev
 */
export interface ImdbImage {
  url: string;
  width: number;
  height: number;
  type: string; // poster, still_frame, publicity, event, product, behind_the_scenes
}

/**
 * Get high-quality images from IMDB API
 * Returns images sorted by type preference for backgrounds
 */
export async function getImdbImages(imdbCode: string): Promise<ImdbImage[]> {
  const cacheKey = `imdb_images_${imdbCode}`;
  return cachedFetch(
    cacheKey,
    CACHE_TTL.MOVIE_DETAILS,
    async () => {
      try {
        const response = await fetch(`https://api.imdbapi.dev/titles/${imdbCode}/images`);
        if (!response.ok) return [];
        const data = await response.json();
        return data.images || [];
      } catch (err) {
        console.warn('[getImdbImages] Failed:', err);
        return [];
      }
    }
  );
}

/**
 * Get the best background image for a movie
 * Prefers horizontal still_frame or publicity images
 */
export async function getBestBackgroundImage(imdbCode: string): Promise<string | null> {
  const images = await getImdbImages(imdbCode);
  if (images.length === 0) return null;

  // Prefer horizontal images (width > height) for backgrounds
  const horizontalImages = images.filter(img => img.width > img.height);

  // Priority: still_frame > publicity > behind_the_scenes > any horizontal
  const priorities = ['still_frame', 'publicity', 'behind_the_scenes'];

  for (const type of priorities) {
    const match = horizontalImages.find(img => img.type === type);
    if (match) return match.url;
  }

  // Fallback to any horizontal image
  if (horizontalImages.length > 0) {
    return horizontalImages[0].url;
  }

  // Last resort: largest image
  const sorted = [...images].sort((a, b) => (b.width * b.height) - (a.width * a.height));
  return sorted[0]?.url || null;
}

/**
 * Search movies by query term
 */
export async function searchMovies(
  query: string,
  page: number = 1,
  limit: number = 20
): Promise<MovieListData> {
  return await listMovies({
    query_term: query,
    page,
    limit,
  });
}

/**
 * Get movies by genre
 */
export async function getMoviesByGenre(
  genre: string,
  page: number = 1,
  limit: number = 20
): Promise<MovieListData> {
  return await listMovies({
    genre,
    page,
    limit,
  });
}

/**
 * Get top rated movies
 */
export async function getTopRatedMovies(
  minimumRating: number = 8,
  page: number = 1,
  limit: number = 20
): Promise<MovieListData> {
  return await listMovies({
    minimum_rating: minimumRating,
    sort_by: "rating",
    order_by: "desc",
    page,
    limit,
  });
}

/**
 * Get latest movies
 */
export async function getLatestMovies(
  page: number = 1,
  limit: number = 20
): Promise<MovieListData> {
  return await listMovies({
    sort_by: "date_added",
    order_by: "desc",
    page,
    limit,
  });
}

/**
 * Update info from GitHub releases
 */
export interface UpdateInfo {
  current_version: string;
  latest_version: string;
  update_available: boolean;
  download_url: string | null;
  release_notes: string | null;
}

/**
 * Check for app updates
 */
export async function checkForUpdates(): Promise<UpdateInfo> {
  return await invoke<UpdateInfo>("check_for_updates");
}

/**
 * Get current app version
 */
export async function getAppVersion(): Promise<string> {
  return await invoke<string>("get_app_version");
}

/**
 * Subtitle types
 */
export interface Subtitle {
  id: string;
  language: string;
  language_name: string;
  download_url: string;
  release_name: string | null;
  uploader: string | null;
  download_count: number;
  hearing_impaired: boolean;
  fps: number | null;
}

export interface SubtitleSearchResult {
  subtitles: Subtitle[];
  total_count: number;
}

export interface SubtitleLanguage {
  code: string;
  name: string;
}

/**
 * Search for subtitles by IMDB ID
 * @param imdbId - The IMDB ID to search for
 * @param apiKey - Optional SubDL API key for better results
 * @param languages - Optional comma-separated language codes (e.g., "en,sq")
 */
export async function searchSubtitles(imdbId: string, apiKey?: string, languages?: string): Promise<SubtitleSearchResult> {
  return await invoke<SubtitleSearchResult>("search_subtitles", { imdbId, apiKey, languages });
}

/**
 * Get available subtitle languages
 */
export async function getSubtitleLanguages(): Promise<SubtitleLanguage[]> {
  return await invoke<SubtitleLanguage[]>("get_subtitle_languages");
}

export interface SubtitleDownloadResult {
  vtt_data_url: string;
}

/**
 * Download a subtitle file and convert to VTT data URL
 */
export async function downloadSubtitle(downloadUrl: string): Promise<SubtitleDownloadResult> {
  return await invoke<SubtitleDownloadResult>("download_subtitle", { downloadUrl });
}

/**
 * Storage info type
 */
export interface StorageInfo {
  download_path: string;
  used_bytes: number;
  free_bytes: number;
  file_count: number;
}

/**
 * Get storage information (cache size, free space)
 */
export async function getStorageInfo(): Promise<StorageInfo> {
  return await invoke<StorageInfo>("get_storage_info");
}

/**
 * Clear all cached/downloaded files (Tauri command)
 * Returns the number of bytes cleared
 */
export async function clearDownloadedFiles(): Promise<number> {
  return await invoke<number>("clear_cache");
}

/**
 * Get the current download path
 */
export async function getDownloadPath(): Promise<string> {
  return await invoke<string>("get_download_path");
}

/**
 * Rating info from local database
 */
export interface LocalRating {
  imdb_rating?: number;
  rotten_tomatoes?: number;
  metacritic?: number;
}

/**
 * Get ratings for movies from local database
 * Returns a map of IMDB code -> rating info
 */
export async function getLocalRatings(imdbCodes: string[]): Promise<Record<string, LocalRating>> {
  if (imdbCodes.length === 0) return {};

  try {
    const response = await fetch(`${API_URL}/api/v2/get_ratings`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(imdbCodes),
    });

    if (!response.ok) return {};

    const data = await response.json();
    return data.data || {};
  } catch (err) {
    console.warn('[getLocalRatings] Failed:', err);
    return {};
  }
}

/**
 * Sync a movie to the local database by IMDB code
 * Server fetches full data from OMDB + torrent providers
 */
export async function syncMovieToLocal(movie: Movie | MovieDetails): Promise<{ synced: boolean; id?: number }> {
  if (!movie.imdb_code) {
    console.log('[syncMovieToLocal] No IMDB code, skipping');
    return { synced: false };
  }

  try {
    const response = await fetch(`${API_URL}/api/v2/sync_movie`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ imdb_code: movie.imdb_code }),
    });
    const data = await response.json();
    if (data.data?.exists) {
      console.log('[syncMovieToLocal] Already exists:', movie.title);
      // Refresh movie data from source to get latest info
      refreshMovieData(data.data.id);
      return { synced: false };
    } else if (data.data?.synced) {
      console.log('[syncMovieToLocal] Synced:', movie.title, 'ID:', data.data.id);
      // Refresh movie data from source to get full details
      refreshMovieData(data.data.id);
      // Clear caches so newly synced movie appears
      clearAllCaches();
      console.log('[syncMovieToLocal] Cleared caches for refresh');
      return { synced: true, id: data.data.id };
    }
    return { synced: false };
  } catch (err) {
    // Silently fail - sync is best-effort
    console.warn('[syncMovieToLocal] Failed to sync:', err);
    return { synced: false };
  }
}

/**
 * Refresh movie data from external source (YTS/IMDB)
 */
export async function refreshMovieData(movieId: number): Promise<void> {
  try {
    const response = await fetch(`${API_URL}/api/v2/refresh_movie`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ movie_id: movieId }),
    });
    const data = await response.json();
    if (data.status === 'ok') {
      console.log('[refreshMovieData] Refreshed movie ID:', movieId);
    }
  } catch (err) {
    console.warn('[refreshMovieData] Failed to refresh:', err);
  }
}

/**
 * Sync multiple movies to local database (batch)
 */
export async function syncMoviesToLocal(movies: Movie[]): Promise<void> {
  try {
    await fetch(`${API_URL}/api/v2/sync_movies`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(movies),
    });
    console.log('[syncMoviesToLocal] Synced batch:', movies.length, 'movies');
  } catch (err) {
    console.warn('[syncMoviesToLocal] Failed to sync batch:', err);
  }
}

// ============ Curated Lists API ============

/**
 * Curated list from local database
 */
export interface CuratedList {
  id: number;
  name: string;
  slug: string;
  description?: string;
  sort_by: string;
  order_by: string;
  minimum_rating?: number;
  maximum_rating?: number;
  minimum_year?: number;
  maximum_year?: number;
  genre?: string;
  limit: number;
  is_active: boolean;
  display_order: number;
  movies?: Movie[];
}

/**
 * Get all curated lists from local database
 */
export async function getCuratedLists(): Promise<CuratedList[]> {
  try {
    const response = await fetch(`${API_URL}/api/v2/curated_lists.json`);
    if (!response.ok) return [];

    const data = await response.json();
    return data.data?.lists || [];
  } catch (err) {
    console.warn('[getCuratedLists] Failed:', err);
    return [];
  }
}

/**
 * Get a curated list with its movies
 */
export async function getCuratedList(slug: string): Promise<CuratedList | null> {
  try {
    const response = await fetch(`${API_URL}/api/v2/curated_list.json?slug=${slug}`);
    if (!response.ok) return null;

    const data = await response.json();
    return data.data?.list || null;
  } catch (err) {
    console.warn('[getCuratedList] Failed:', err);
    return null;
  }
}

// ============ TV Series API ============

import type { Series, SeriesListData, ListSeriesParams, Episode } from "./types";

/**
 * Get list of TV series from local database
 */
export async function listSeries(params: ListSeriesParams = {}): Promise<SeriesListData> {
  try {
    const queryParams = new URLSearchParams();
    if (params.limit) queryParams.set('limit', params.limit.toString());
    if (params.page) queryParams.set('page', params.page.toString());
    if (params.query_term) queryParams.set('query_term', params.query_term);
    if (params.genre) queryParams.set('genre', params.genre);
    if (params.status) queryParams.set('status', params.status);
    if (params.network) queryParams.set('network', params.network);
    if (params.minimum_rating) queryParams.set('minimum_rating', params.minimum_rating.toString());
    if (params.sort_by) queryParams.set('sort_by', params.sort_by);
    if (params.order_by) queryParams.set('order_by', params.order_by);
    if (params.year) queryParams.set('year', params.year.toString());
    if (params.maximum_year) queryParams.set('maximum_year', params.maximum_year.toString());

    const response = await fetch(`${API_URL}/api/v2/list_series.json?${queryParams}`);
    if (!response.ok) throw new Error('Failed to fetch series');

    const data = await response.json();
    return data.data;
  } catch (err) {
    console.warn('[listSeries] Failed:', err);
    return { series_count: 0, limit: 20, page_number: 1, series: [] };
  }
}

/**
 * Get series details by ID
 */
export async function getSeriesDetails(seriesId: number): Promise<Series | null> {
  try {
    const response = await fetch(`${API_URL}/api/v2/series_details.json?series_id=${seriesId}`);
    if (!response.ok) return null;

    const data = await response.json();
    return data.data.series;
  } catch (err) {
    console.warn('[getSeriesDetails] Failed:', err);
    return null;
  }
}

/**
 * Get episodes for a season
 */
export async function getSeasonEpisodes(seriesId: number, season: number): Promise<Episode[]> {
  try {
    const response = await fetch(`${API_URL}/api/v2/season_episodes.json?series_id=${seriesId}&season=${season}`);
    if (!response.ok) return [];

    const data = await response.json();
    return data.data || [];
  } catch (err) {
    console.warn('[getSeasonEpisodes] Failed:', err);
    return [];
  }
}

/**
 * Search TV series
 */
export async function searchSeries(query: string, page: number = 1, limit: number = 20): Promise<SeriesListData> {
  return listSeries({ query_term: query, page, limit });
}

/**
 * Get series by genre
 */
export async function getSeriesByGenre(genre: string, page: number = 1, limit: number = 20): Promise<SeriesListData> {
  return listSeries({ genre, page, limit });
}

/**
 * Get continuing series
 */
export async function getContinuingSeries(page: number = 1, limit: number = 20): Promise<SeriesListData> {
  return listSeries({ status: 'Continuing', sort_by: 'rating', order_by: 'desc', page, limit });
}

/**
 * Get top rated series
 */
export async function getTopRatedSeries(page: number = 1, limit: number = 20): Promise<SeriesListData> {
  return listSeries({ sort_by: 'rating', order_by: 'desc', minimum_rating: 8, page, limit });
}

// ============ Home API ============

/**
 * Home section from server - dynamically configured via admin
 */
export interface HomeSection {
  id: string;
  title: string;
  type: string;         // data source: recent, top_rated, genre, curated_list
  display_type: string; // layout: hero, carousel, grid, featured, banner
  movies?: Movie[];
  series?: Series[];
}

/**
 * Home data response
 */
export interface HomeData {
  hero_slider?: Movie[];
  sections: HomeSection[];
}

/**
 * Get home page data from server (always fresh, no cache)
 * This fetches the admin-configured sections for the home page
 */
export async function getHomeData(): Promise<HomeData> {
  try {
    const response = await fetch(`${API_URL}/api/v2/home.json`);
    if (!response.ok) throw new Error('Failed to fetch home data');

    const data = await response.json();
    return data.data as HomeData;
  } catch (err) {
    console.warn('[getHomeData] Failed:', err);
    return { sections: [], hero_slider: [] };
  }
}

/**
 * Torrent stats (real-time seed/peer info)
 */
export interface TorrentStats {
  hash: string;
  seeds: number;
  peers: number;
  name?: string;
  found: boolean;
}

/**
 * Get real-time torrent stats (seeds/peers) by hash
 * This fetches current data from TPB to get accurate availability info
 */
export async function getTorrentStats(hash: string): Promise<TorrentStats | null> {
  try {
    const response = await fetch(`${API_URL}/api/v2/torrent_stats?hash=${hash}`);
    if (!response.ok) return null;

    const data = await response.json();
    return data.data as TorrentStats;
  } catch (err) {
    console.warn('[getTorrentStats] Failed:', err);
    return null;
  }
}

/**
 * Get real-time torrent stats for multiple hashes
 */
export async function getMultipleTorrentStats(hashes: string[]): Promise<Record<string, TorrentStats>> {
  if (hashes.length === 0) return {};

  try {
    const params = hashes.map(h => `hashes=${h}`).join('&');
    const response = await fetch(`${API_URL}/api/v2/torrent_stats?${params}`);
    if (!response.ok) return {};

    const data = await response.json();
    return data.data as Record<string, TorrentStats>;
  } catch (err) {
    console.warn('[getMultipleTorrentStats] Failed:', err);
    return {};
  }
}

// Analytics - Record a view
export interface RecordViewParams {
  contentType: 'movie' | 'series' | 'episode';
  contentId: number;
  imdbCode?: string;
  duration?: number;  // seconds watched
  completed?: boolean;
  quality?: string;
}

export async function recordView(params: RecordViewParams): Promise<void> {
  try {
    await fetch(`${API_URL}/api/v2/analytics/view`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        content_type: params.contentType,
        content_id: params.contentId,
        imdb_code: params.imdbCode || '',
        device_id: getDeviceId(),
        duration: params.duration || 0,
        completed: params.completed || false,
        quality: params.quality || '',
      }),
    });
  } catch (err) {
    // Silent fail - analytics shouldn't break the app
    console.warn('[recordView] Failed:', err);
  }
}

// Helper to get or create device ID
function getDeviceId(): string {
  let deviceId = localStorage.getItem('analytics_device_id');
  if (!deviceId) {
    deviceId = 'dev_' + Math.random().toString(36).substring(2, 15);
    localStorage.setItem('analytics_device_id', deviceId);
  }
  return deviceId;
}

// Stream tracking - Start
export interface StreamStartParams {
  contentType: 'movie' | 'series' | 'episode';
  contentId: number;
  imdbCode?: string;
  quality?: string;
}

export async function streamStart(params: StreamStartParams): Promise<void> {
  try {
    await fetch(`${API_URL}/api/v2/analytics/stream/start`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        device_id: getDeviceId(),
        content_type: params.contentType,
        content_id: params.contentId,
        imdb_code: params.imdbCode || '',
        quality: params.quality || '',
      }),
    });
  } catch (err) {
    console.warn('[streamStart] Failed:', err);
  }
}

// Stream tracking - Heartbeat (call every 30-60 seconds while streaming)
export async function streamHeartbeat(): Promise<void> {
  try {
    await fetch(`${API_URL}/api/v2/analytics/stream/heartbeat`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        device_id: getDeviceId(),
      }),
    });
  } catch (err) {
    console.warn('[streamHeartbeat] Failed:', err);
  }
}

// Stream tracking - End
export async function streamEnd(): Promise<void> {
  try {
    await fetch(`${API_URL}/api/v2/analytics/stream/end`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        device_id: getDeviceId(),
      }),
    });
  } catch (err) {
    console.warn('[streamEnd] Failed:', err);
  }
}

// Get top movies from analytics
export async function getTopMovies(days: number = 7, genre?: string, limit: number = 10): Promise<Movie[]> {
  try {
    const params = new URLSearchParams({
      days: days.toString(),
      limit: limit.toString(),
    });
    if (genre) params.set('genre', genre);

    const response = await fetch(`${API_URL}/api/v2/analytics/top-movies?${params}`);
    if (!response.ok) return [];

    return await response.json();
  } catch (err) {
    console.warn('[getTopMovies] Failed:', err);
    return [];
  }
}

// ============ Coming Soon / Reminders API ============

/**
 * Get coming soon movies from local server
 */
export async function getComingSoonMovies(): Promise<Movie[]> {
  try {
    const response = await fetch(`${API_URL}/api/v2/list_movies.json?status=coming_soon&limit=20`);
    if (!response.ok) return [];
    const data = await response.json();
    return data.data?.movies || [];
  } catch (err) {
    console.warn('[getComingSoonMovies] Failed:', err);
    return [];
  }
}

/**
 * Check availability of multiple movies by IMDB codes
 * Used to notify users when their reminded movies become available
 */
export async function checkAvailability(imdbCodes: string[]): Promise<Record<string, { available: boolean; title?: string; id?: number; poster?: string }>> {
  if (imdbCodes.length === 0) return {};

  try {
    const response = await fetch(`${API_URL}/api/v2/check_availability?imdb_codes=${imdbCodes.join(',')}`);
    if (!response.ok) return {};
    const data = await response.json();
    return data.data || {};
  } catch (err) {
    console.warn('[checkAvailability] Failed:', err);
    return {};
  }
}

// ============ Channels (IPTV) API ============

import type { Channel, ChannelCountry, ChannelCategory, ChannelListData, ListChannelsParams } from "./types";

/**
 * Get list of IPTV channels
 */
export async function listChannels(params: ListChannelsParams = {}): Promise<ChannelListData> {
  try {
    const queryParams = new URLSearchParams();
    if (params.limit) queryParams.set('limit', params.limit.toString());
    if (params.page) queryParams.set('page', params.page.toString());
    if (params.country) queryParams.set('country', params.country);
    if (params.category) queryParams.set('category', params.category);
    if (params.query_term) queryParams.set('query_term', params.query_term);

    const response = await fetch(`${API_URL}/api/v2/list_channels.json?${queryParams}`);
    if (!response.ok) throw new Error('Failed to fetch channels');

    const data = await response.json();
    return data.data;
  } catch (err) {
    console.warn('[listChannels] Failed:', err);
    return { channel_count: 0, limit: 50, page_number: 1, channels: [] };
  }
}

/**
 * Get channel details by ID
 */
export async function getChannelDetails(channelId: string): Promise<Channel | null> {
  try {
    const response = await fetch(`${API_URL}/api/v2/channel_details.json?channel_id=${channelId}`);
    if (!response.ok) return null;

    const data = await response.json();
    return data.data.channel;
  } catch (err) {
    console.warn('[getChannelDetails] Failed:', err);
    return null;
  }
}

/**
 * Get list of channel countries
 */
export async function getChannelCountries(): Promise<ChannelCountry[]> {
  try {
    const response = await fetch(`${API_URL}/api/v2/channel_countries.json`);
    if (!response.ok) return [];

    const data = await response.json();
    return data.data.countries || [];
  } catch (err) {
    console.warn('[getChannelCountries] Failed:', err);
    return [];
  }
}

/**
 * Get list of channel categories
 */
export async function getChannelCategories(): Promise<ChannelCategory[]> {
  try {
    const response = await fetch(`${API_URL}/api/v2/channel_categories.json`);
    if (!response.ok) return [];

    const data = await response.json();
    return data.data.categories || [];
  } catch (err) {
    console.warn('[getChannelCategories] Failed:', err);
    return [];
  }
}

/**
 * Get channels by country
 */
export async function getChannelsByCountry(country: string, limit: number = 50): Promise<Channel[]> {
  try {
    const response = await fetch(`${API_URL}/api/v2/channels_by_country.json?country=${country}&limit=${limit}`);
    if (!response.ok) return [];

    const data = await response.json();
    return data.data.channels || [];
  } catch (err) {
    console.warn('[getChannelsByCountry] Failed:', err);
    return [];
  }
}

/**
 * Search channels by name
 */
export async function searchChannels(query: string, page: number = 1, limit: number = 50): Promise<ChannelListData> {
  return listChannels({ query_term: query, page, limit });
}

// ============ Unified Search API ============

/**
 * Unified search response
 */
export interface UnifiedSearchResponse {
  query: string;
  movies: Movie[];
  series: Series[];
  channels: Channel[];
}

/**
 * Search across movies, series, and channels
 */
export async function unifiedSearch(query: string, limit: number = 10): Promise<UnifiedSearchResponse> {
  try {
    const response = await fetch(`${API_URL}/api/v2/search.json?query=${encodeURIComponent(query)}&limit=${limit}`);
    if (!response.ok) throw new Error('Search failed');

    const data = await response.json();
    return data.data as UnifiedSearchResponse;
  } catch (err) {
    console.warn('[unifiedSearch] Failed:', err);
    return { query, movies: [], series: [], channels: [] };
  }
}
