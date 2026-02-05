// TypeScript interfaces matching Rust types from src-tauri

export interface ListMoviesParams {
  limit?: number;
  page?: number;
  quality?: string;
  minimum_rating?: number;
  query_term?: string;
  genre?: string;
  sort_by?: string;
  order_by?: string;
  with_rt_ratings?: boolean;
  year?: number;
}

export interface Movie {
  id: number;
  url: string;
  imdb_code: string;
  title: string;
  title_english?: string;
  title_long: string;
  slug: string;
  year: number;
  rating: number;
  runtime: number;
  genres: string[];
  summary: string;
  description_full: string;
  synopsis: string;
  yt_trailer_code: string;
  language: string;
  background_image: string;
  background_image_original: string;
  small_cover_image: string;
  medium_cover_image: string;
  large_cover_image: string;
  torrents: Torrent[];
  // Coming soon status
  status?: string;        // "available" or "coming_soon"
  release_date?: string;  // YYYY-MM-DD format
  // Franchise/collection
  franchise?: string;     // e.g., "Lord of the Rings", "Star Wars"
}

export interface Torrent {
  url: string;
  hash: string;
  quality: string;
  type?: string;
  torrent_type?: string;
  is_repack?: string;
  video_codec?: string;
  bit_depth?: string;
  audio_channels?: string;
  seeds: number;
  peers: number;
  size: string;
  size_bytes: number;
  date_uploaded: string;
  date_uploaded_unix: number;
}

export interface MovieListData {
  movie_count: number;
  limit: number;
  page_number: number;
  movies: Movie[];
}

export interface MovieDetails {
  id: number;
  url: string;
  imdb_code: string;
  title: string;
  title_english?: string;
  title_long: string;
  slug: string;
  year: number;
  rating: number;
  runtime: number;
  genres: string[];
  summary: string;
  description_full: string;
  synopsis: string;
  yt_trailer_code: string;
  language: string;
  mpa_rating?: string;
  background_image: string;
  background_image_original: string;
  small_cover_image: string;
  medium_cover_image: string;
  large_cover_image: string;
  torrents: Torrent[];
  cast: Cast[];
  like_count?: number;
  download_count?: number;
  // Rich data fields from IMDB/OMDB
  provider?: string;
  director?: string;
  writers?: string[];
  country?: string;
  awards?: string;
  budget?: string;
  box_office_gross?: string;
  imdb_rating?: number;
  imdb_votes?: string;
  rotten_tomatoes?: number;
  metacritic?: number;
  all_images?: string[];
  // Coming soon status
  status?: string;        // "available" or "coming_soon"
  release_date?: string;  // YYYY-MM-DD format
  // Franchise/collection
  franchise?: string;     // e.g., "Lord of the Rings", "Star Wars"
}

export interface Cast {
  name: string;
  character_name: string;
  url_small_image?: string;
  imdb_code: string;
}

export interface StreamInfo {
  info_hash: string;
  stream_url: string;
  file_name: string;
  total_size: number;
}

export interface StreamStats {
  downloaded_bytes: number;
  total_bytes: number;
  download_speed: number;
  upload_speed: number;
  peers_connected: number;
  progress_percent: number;
}

export interface TorrentFile {
  index: number;
  name: string;
  size: number;
}

export interface MovieRating {
  imdb_rating: number | null;
  imdb_votes: string | null;
  metascore: number | null;
  rotten_tomatoes: number | null;
  director: string | null;
  actors: string | null;
  plot: string | null;
  language: string | null;
  country: string | null;
  awards: string | null;
  box_office: string | null;
  rated: string | null;
}

// TV Series types
export interface Series {
  id: number;
  imdb_code: string;
  tvdb_id?: number;
  title: string;
  title_slug: string;
  year: number;
  end_year?: number;
  rating: number;
  runtime: number;
  genres: string[];
  summary: string;
  status: string;
  network?: string;
  poster_image: string;
  background_image: string;
  total_seasons: number;
  total_episodes: number;
  seasons?: Season[];
  date_added: string;
  date_added_unix: number;
  imdb_rating?: number;
  rotten_tomatoes?: number;
}

export interface Season {
  id: number;
  series_id: number;
  season_number: number;
  episode_count: number;
  air_date?: string;
  poster_image?: string;
  episodes?: Episode[];
}

export interface Episode {
  id: number;
  series_id: number;
  season_number: number;
  episode_number: number;
  title: string;
  summary?: string;
  air_date?: string;
  runtime?: number;
  still_image?: string;
  torrents?: EpisodeTorrent[];
}

export interface EpisodeTorrent {
  id: number;
  episode_id: number;
  series_id: number;
  season_number: number;
  episode_number: number;
  hash: string;
  quality: string;
  video_codec?: string;
  seeds: number;
  peers: number;
  size: string;
  size_bytes: number;
  release_group?: string;
  date_uploaded: string;
  date_uploaded_unix: number;
}

export interface ListSeriesParams {
  limit?: number;
  page?: number;
  query_term?: string;
  genre?: string;
  status?: string;
  network?: string;
  minimum_rating?: number;
  maximum_year?: number;
  sort_by?: string;
  order_by?: string;
  year?: number;
}

export interface SeriesListData {
  series_count: number;
  limit: number;
  page_number: number;
  series: Series[];
}

// Availability check for "Remind Me" feature
export interface AvailabilityInfo {
  available: boolean;
  title?: string;
  id?: number;
  poster?: string;
}

// IPTV Channel types
export interface Channel {
  id: string;
  name: string;
  country?: string;
  languages?: string[];
  categories?: string[];
  logo?: string;
  stream_url?: string;
}

export interface ChannelCountry {
  code: string;
  name: string;
  flag?: string;
}

export interface ChannelCategory {
  id: string;
  name: string;
}

export interface ChannelListData {
  channel_count: number;
  limit: number;
  page_number: number;
  channels: Channel[];
}

export interface ListChannelsParams {
  limit?: number;
  page?: number;
  country?: string;
  category?: string;
  query_term?: string;
}
