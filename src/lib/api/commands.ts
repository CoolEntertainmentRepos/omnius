// Tauri invoke wrappers for all commands
import { invoke } from "@tauri-apps/api/core";
import { cachedFetch, makeCacheKey, CACHE_TTL } from "./cache";
import type {
  ListMoviesParams,
  MovieListData,
  MovieDetails,
  Movie,
  MovieRating,
  StreamInfo,
  StreamStats,
} from "./types";

/**
 * Fetch a list of movies from YTS API (cached)
 */
export async function listMovies(params: ListMoviesParams, forceRefresh: boolean = false): Promise<MovieListData> {
  const cacheKey = makeCacheKey('movies', params as Record<string, unknown>);
  return cachedFetch(
    cacheKey,
    CACHE_TTL.MOVIES_LIST,
    () => invoke<MovieListData>("list_movies", { params }),
    forceRefresh
  );
}

/**
 * Get detailed information about a specific movie (cached)
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
      console.log("[getMovieDetails] Fetching movieId:", movieId);
      const result = await invoke<MovieDetails>("get_movie_details", {
        movieId,
        withCast,
        withImages,
      });
      console.log("[getMovieDetails] Success:", result.title);
      return result;
    },
    forceRefresh
  );
}

/**
 * Get movie suggestions/recommendations based on a movie (cached)
 */
export async function getMovieSuggestions(movieId: number, forceRefresh: boolean = false): Promise<Movie[]> {
  const cacheKey = `suggestions_${movieId}`;
  return cachedFetch(
    cacheKey,
    CACHE_TTL.SUGGESTIONS,
    async () => {
      console.log("[getMovieSuggestions] Fetching movieId:", movieId);
      const result = await invoke<Movie[]>("get_movie_suggestions", { movieId });
      console.log("[getMovieSuggestions] Success:", result.length, "suggestions");
      return result;
    },
    forceRefresh
  );
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
 * Clear all cached/downloaded files
 * Returns the number of bytes cleared
 */
export async function clearCache(): Promise<number> {
  return await invoke<number>("clear_cache");
}

/**
 * Get the current download path
 */
export async function getDownloadPath(): Promise<string> {
  return await invoke<string>("get_download_path");
}
