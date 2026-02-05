# Streamer API Reference

This document describes the API functions available in Streamer for communicating with the torrent server and Tauri backend.

All functions are exported from `src/lib/api/commands.ts`.

---

## Movies

### listMovies
Fetch a list of movies with filtering and pagination.

```typescript
async function listMovies(
  params: ListMoviesParams,
  forceRefresh?: boolean
): Promise<MovieListData>
```

**Parameters:**
```typescript
interface ListMoviesParams {
  limit?: number;          // Results per page (default: 20)
  page?: number;           // Page number (default: 1)
  quality?: string;        // Filter by quality
  minimum_rating?: number; // Minimum IMDB rating
  query_term?: string;     // Search term
  genre?: string;          // Genre filter
  sort_by?: string;        // Sort field
  order_by?: string;       // asc or desc
  year?: number;           // Filter by year
}
```

### getMovieDetails
Get detailed information about a specific movie.

```typescript
async function getMovieDetails(
  movieId: number,
  withCast?: boolean,
  withImages?: boolean,
  forceRefresh?: boolean
): Promise<MovieDetails>
```

### getMovieSuggestions
Get similar movie recommendations.

```typescript
async function getMovieSuggestions(
  movieId: number,
  forceRefresh?: boolean
): Promise<Movie[]>
```

### getFranchiseMovies
Get all movies in the same franchise.

```typescript
async function getFranchiseMovies(movieId: number): Promise<Movie[]>
```

### searchMovies
Search movies by query term.

```typescript
async function searchMovies(
  query: string,
  page?: number,
  limit?: number
): Promise<MovieListData>
```

### getTopRatedMovies
Get top-rated movies.

```typescript
async function getTopRatedMovies(
  minimumRating?: number,
  page?: number,
  limit?: number
): Promise<MovieListData>
```

### getLatestMovies
Get recently added movies.

```typescript
async function getLatestMovies(
  page?: number,
  limit?: number
): Promise<MovieListData>
```

### getComingSoonMovies
Get movies marked as "coming soon".

```typescript
async function getComingSoonMovies(): Promise<Movie[]>
```

---

## Series

### listSeries
Fetch a list of TV series.

```typescript
async function listSeries(
  params?: ListSeriesParams
): Promise<SeriesListData>
```

**Parameters:**
```typescript
interface ListSeriesParams {
  limit?: number;
  page?: number;
  query_term?: string;
  genre?: string;
  status?: string;         // "Continuing" or "Ended"
  network?: string;        // HBO, Netflix, etc.
  minimum_rating?: number;
  sort_by?: string;
  order_by?: string;
  year?: number;
}
```

### getSeriesDetails
Get detailed series information.

```typescript
async function getSeriesDetails(
  seriesId: number
): Promise<Series | null>
```

### getSeasonEpisodes
Get all episodes for a specific season.

```typescript
async function getSeasonEpisodes(
  seriesId: number,
  season: number
): Promise<Episode[]>
```

### searchSeries
Search series by title.

```typescript
async function searchSeries(
  query: string,
  page?: number,
  limit?: number
): Promise<SeriesListData>
```

### getTopRatedSeries
Get top-rated series.

```typescript
async function getTopRatedSeries(
  page?: number,
  limit?: number
): Promise<SeriesListData>
```

---

## Channels (IPTV)

### listChannels
Fetch IPTV channels.

```typescript
async function listChannels(
  params?: ListChannelsParams
): Promise<ChannelListData>
```

**Parameters:**
```typescript
interface ListChannelsParams {
  limit?: number;
  page?: number;
  country?: string;    // Country code (US, UK, DE)
  category?: string;   // news, sports, movies, etc.
  query_term?: string;
}
```

### getChannelDetails
Get channel information by ID.

```typescript
async function getChannelDetails(
  channelId: string
): Promise<Channel | null>
```

### getChannelCountries
Get list of all countries with channels.

```typescript
async function getChannelCountries(): Promise<ChannelCountry[]>
```

### getChannelCategories
Get list of all channel categories.

```typescript
async function getChannelCategories(): Promise<ChannelCategory[]>
```

### getChannelsByCountry
Get channels for a specific country.

```typescript
async function getChannelsByCountry(
  country: string,
  limit?: number
): Promise<Channel[]>
```

### searchChannels
Search channels by name.

```typescript
async function searchChannels(
  query: string,
  page?: number,
  limit?: number
): Promise<ChannelListData>
```

---

## Unified Search

### unifiedSearch
Search across movies, series, and channels.

```typescript
async function unifiedSearch(
  query: string,
  limit?: number
): Promise<UnifiedSearchResponse>
```

**Response:**
```typescript
interface UnifiedSearchResponse {
  query: string;
  movies: Movie[];
  series: Series[];
  channels: Channel[];
}
```

---

## Streaming (Tauri Commands)

### startStream
Start streaming a torrent.

```typescript
async function startStream(torrentHash: string): Promise<StreamInfo>
```

**Response:**
```typescript
interface StreamInfo {
  info_hash: string;
  stream_url: string;
  file_name: string;
  total_size: number;
}
```

### stopStream
Stop an active stream.

```typescript
async function stopStream(infoHash: string): Promise<void>
```

### getStreamStats
Get current streaming statistics.

```typescript
async function getStreamStats(infoHash: string): Promise<StreamStats>
```

**Response:**
```typescript
interface StreamStats {
  downloaded_bytes: number;
  total_bytes: number;
  download_speed: number;
  upload_speed: number;
  peers_connected: number;
  progress_percent: number;
}
```

---

## Subtitles

### searchSubtitles
Search for subtitles by IMDB ID.

```typescript
async function searchSubtitles(
  imdbId: string,
  apiKey?: string,
  languages?: string
): Promise<SubtitleSearchResult>
```

### downloadSubtitle
Download and convert subtitle to VTT format.

```typescript
async function downloadSubtitle(
  downloadUrl: string
): Promise<SubtitleDownloadResult>
```

### getSubtitleLanguages
Get available subtitle languages.

```typescript
async function getSubtitleLanguages(): Promise<SubtitleLanguage[]>
```

---

## Home Page

### getHomeData
Get home page sections and content.

```typescript
async function getHomeData(): Promise<HomeData>
```

**Response:**
```typescript
interface HomeData {
  hero_slider?: Movie[];
  sections: HomeSection[];
}

interface HomeSection {
  id: string;
  title: string;
  type: string;         // recent, top_rated, genre, curated_list
  display_type: string; // hero, carousel, grid, featured, banner
  movies?: Movie[];
  series?: Series[];
}
```

---

## Analytics

### recordView
Record a content view.

```typescript
async function recordView(params: RecordViewParams): Promise<void>
```

**Parameters:**
```typescript
interface RecordViewParams {
  contentType: 'movie' | 'series' | 'episode';
  contentId: number;
  imdbCode?: string;
  duration?: number;
  completed?: boolean;
  quality?: string;
}
```

### streamStart
Mark stream start for analytics.

```typescript
async function streamStart(params: StreamStartParams): Promise<void>
```

### streamHeartbeat
Send heartbeat during active stream (call every 30-60 seconds).

```typescript
async function streamHeartbeat(): Promise<void>
```

### streamEnd
Mark stream end.

```typescript
async function streamEnd(): Promise<void>
```

### getTopMovies
Get top movies from analytics.

```typescript
async function getTopMovies(
  days?: number,
  genre?: string,
  limit?: number
): Promise<Movie[]>
```

---

## Availability / Reminders

### checkAvailability
Check if "coming soon" movies are now available.

```typescript
async function checkAvailability(
  imdbCodes: string[]
): Promise<Record<string, AvailabilityInfo>>
```

**Response:**
```typescript
interface AvailabilityInfo {
  available: boolean;
  title?: string;
  id?: number;
  poster?: string;
}
```

---

## Sync

### syncMovieToLocal
Sync a movie to the local database.

```typescript
async function syncMovieToLocal(
  movie: Movie | MovieDetails
): Promise<{ synced: boolean; id?: number }>
```

### refreshMovieData
Refresh movie data from external sources.

```typescript
async function refreshMovieData(movieId: number): Promise<void>
```

---

## Torrent Stats

### getTorrentStats
Get real-time seed/peer info for a torrent.

```typescript
async function getTorrentStats(hash: string): Promise<TorrentStats | null>
```

### getMultipleTorrentStats
Get stats for multiple torrents.

```typescript
async function getMultipleTorrentStats(
  hashes: string[]
): Promise<Record<string, TorrentStats>>
```

---

## Utility

### getStorageInfo
Get storage usage information.

```typescript
async function getStorageInfo(): Promise<StorageInfo>
```

### clearDownloadedFiles
Clear cached/downloaded files.

```typescript
async function clearDownloadedFiles(): Promise<number>
```

### getAppVersion
Get current app version.

```typescript
async function getAppVersion(): Promise<string>
```

### checkForUpdates
Check for app updates.

```typescript
async function checkForUpdates(): Promise<UpdateInfo>
```

---

## Cache Functions

### clearAllCaches
Clear all API response caches.

```typescript
function clearAllCaches(): void
```

### clearCache
Clear specific cache key.

```typescript
function clearCache(key: string): void
```

### initCache
Initialize cache (clears old version caches).

```typescript
function initCache(): void
```
