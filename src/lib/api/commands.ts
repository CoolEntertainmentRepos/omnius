// Tauri invoke wrappers for all commands
import { invoke } from "@tauri-apps/api/core";
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
 * Fetch a list of movies from YTS API
 */
export async function listMovies(params: ListMoviesParams): Promise<MovieListData> {
  return await invoke<MovieListData>("list_movies", { params });
}

/**
 * Get detailed information about a specific movie
 */
export async function getMovieDetails(
  movieId: number,
  withCast: boolean = true,
  withImages: boolean = true
): Promise<MovieDetails> {
  console.log("[getMovieDetails] Calling with movieId:", movieId);
  try {
    const result = await invoke<MovieDetails>("get_movie_details", {
      movieId,
      withCast,
      withImages,
    });
    console.log("[getMovieDetails] Success:", result.title);
    return result;
  } catch (error) {
    console.error("[getMovieDetails] Error:", error);
    throw error;
  }
}

/**
 * Get movie suggestions/recommendations based on a movie
 */
export async function getMovieSuggestions(movieId: number): Promise<Movie[]> {
  console.log("[getMovieSuggestions] Calling with movieId:", movieId);
  try {
    const result = await invoke<Movie[]>("get_movie_suggestions", { movieId });
    console.log("[getMovieSuggestions] Success:", result.length, "suggestions");
    return result;
  } catch (error) {
    console.error("[getMovieSuggestions] Error:", error);
    throw error;
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
 * Get movie rating from OMDB (IMDb, Rotten Tomatoes, Metacritic)
 */
export async function getMovieRating(imdbCode: string): Promise<MovieRating> {
  return await invoke<MovieRating>("get_movie_rating", { imdbCode });
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
