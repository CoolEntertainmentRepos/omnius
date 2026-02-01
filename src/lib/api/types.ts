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
