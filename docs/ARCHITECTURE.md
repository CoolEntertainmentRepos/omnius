# Streamer App Architecture

## Overview

Streamer is a Tauri-based desktop/mobile application for streaming movies and TV series. It connects to the Omnius torrent server for metadata and streaming.

---

## Tech Stack

- **Frontend**: Svelte 5 with SvelteKit
- **Desktop Runtime**: Tauri 2.x (Rust)
- **Styling**: CSS with CSS variables
- **State Management**: Svelte 5 runes (`$state`, `$derived`)
- **Build Tool**: Vite

---

## Project Structure

```
Streamer/
├── src/
│   ├── routes/                 # SvelteKit routes (pages)
│   │   ├── +layout.svelte      # Root layout
│   │   ├── +page.svelte        # Home page
│   │   ├── movie/[id]/         # Movie detail page
│   │   ├── series/[id]/        # Series detail page
│   │   ├── player/[hash]/      # Video player
│   │   └── settings/           # Settings page
│   ├── lib/
│   │   ├── api/
│   │   │   ├── commands.ts     # API functions (Tauri + HTTP)
│   │   │   ├── types.ts        # TypeScript interfaces
│   │   │   └── cache.ts        # Client-side caching
│   │   ├── components/
│   │   │   ├── MovieCard.svelte
│   │   │   ├── MovieRow.svelte
│   │   │   ├── SeriesCard.svelte
│   │   │   ├── QualitySelector.svelte
│   │   │   ├── Sidebar.svelte
│   │   │   └── NotificationToast.svelte
│   │   └── stores/
│   │       └── reminders.svelte.ts
│   └── app.html
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs              # Tauri command registration
│   │   ├── commands/           # Rust commands
│   │   │   ├── movies.rs       # Movie-related commands
│   │   │   ├── streaming.rs    # Torrent streaming
│   │   │   └── subtitles.rs    # Subtitle fetching
│   │   ├── yts/                # YTS client (deprecated)
│   │   └── omdb/               # OMDB client
│   ├── Cargo.toml
│   └── tauri.conf.json
├── scripts/
│   ├── release.sh              # Release build script
│   └── build-tv.sh             # TV build script
└── docs/
```

---

## Key Components

### 1. API Layer (`src/lib/api/`)

#### commands.ts
Main API interface. Wraps both Tauri commands and HTTP calls:

```typescript
// Tauri command (Rust backend)
export async function startStream(hash: string): Promise<StreamInfo> {
  return await invoke<StreamInfo>("start_stream", { torrentHash: hash });
}

// HTTP call to torrent-server
export async function listMovies(params: ListMoviesParams): Promise<MovieListData> {
  const response = await fetch(`${API_URL}/api/v2/list_movies.json?...`);
  return response.json();
}
```

#### cache.ts
Client-side caching with localStorage:

```typescript
export const CACHE_TTL = {
  MOVIES_LIST: 15 * 60 * 1000,      // 15 minutes
  MOVIE_DETAILS: 5 * 60 * 1000,     // 5 minutes
  HOME_DATA: 10 * 60 * 1000,        // 10 minutes
  IPTV: 24 * 60 * 60 * 1000,        // 24 hours
};

// Cache versioning for invalidation
const CACHE_VERSION = 4;

// Dev mode cache bypass
if (dev) {
  // Skip cache entirely
}
```

### 2. Stores (`src/lib/stores/`)

Using Svelte 5 runes for reactive state:

```typescript
// reminders.svelte.ts
class RemindersStore {
  reminders = $state<string[]>([]);  // IMDB codes

  add(imdbCode: string) { ... }
  remove(imdbCode: string) { ... }
  isReminded(imdbCode: string): boolean { ... }
}
```

### 3. Components

#### MovieCard
Displays movie poster with hover preview:
- Poster image
- Title, year, rating
- Genre tags
- Play button on hover

#### QualitySelector
Torrent selection with quality options:
- Available qualities (720p, 1080p, 2160p)
- Seed/peer counts
- File size
- Video codec info

#### Sidebar
Navigation component:
- Home
- Movies
- Series
- Live (IPTV)
- Search

### 4. Tauri Commands (Rust)

Located in `src-tauri/src/commands/`:

```rust
#[tauri::command]
async fn start_stream(torrent_hash: String) -> Result<StreamInfo, String> {
    // Start torrent download
    // Return stream URL when ready
}

#[tauri::command]
async fn get_stream_status(info_hash: String) -> Result<StreamStats, String> {
    // Return download progress, speed, peers
}

#[tauri::command]
async fn search_subtitles(imdb_id: String) -> Result<SubtitleSearchResult, String> {
    // Fetch from SubDL API
}
```

---

## Data Flow

### Movie Playback Flow

```
User clicks Play
    ↓
QualitySelector shown
    ↓
User selects torrent
    ↓
startStream(hash) → Tauri Command
    ↓
Rust torrent client starts download
    ↓
Returns stream URL when pieces available
    ↓
Navigate to Player page
    ↓
Video.js plays HLS stream
    ↓
Periodic getStreamStats() for progress
```

### Home Page Load

```
+page.svelte onMount
    ↓
getHomeData() → HTTP to torrent-server
    ↓
Check cache (if valid, return cached)
    ↓
Fetch /api/v2/home.json
    ↓
Cache response
    ↓
Render sections
```

---

## Caching Strategy

### Client-Side Cache

```typescript
// Get from cache if valid
const cached = getCache<T>(key, ttl);
if (cached) return cached;

// Otherwise fetch and cache
const data = await fetchFn();
setCache(key, data);
return data;
```

### Cache Invalidation

- **Version bump**: Increment `CACHE_VERSION` to invalidate all caches
- **TTL expiry**: Each cache type has its own TTL
- **Manual clear**: Settings page has "Refresh Movie Data" button
- **Dev mode**: Cache disabled entirely in development

---

## Running the App

### Development
```bash
# MUST use tauri dev, not npm run dev
npm run tauri dev
```

### Production Build
```bash
npm run tauri build
# or use release script
./scripts/release.sh
```

### Android Build
```bash
npm run tauri android build
```

---

## API Endpoints Used

| Endpoint | Purpose |
|----------|---------|
| `/api/v2/home.json` | Home page sections |
| `/api/v2/list_movies.json` | Movie listings |
| `/api/v2/movie_details.json` | Movie details |
| `/api/v2/list_series.json` | Series listings |
| `/api/v2/series_details.json` | Series details |
| `/api/v2/list_channels.json` | IPTV channels |
| `/api/v2/search.json` | Unified search |
| `/api/v2/franchise_movies.json` | Related movies |
| `/api/v2/check_availability` | Coming soon status |

---

## Key Features

### 1. Unified Search
Searches across movies, series, and channels:
```typescript
const results = await unifiedSearch("batman");
// results.movies, results.series, results.channels
```

### 2. Franchise Grouping
Shows related movies in the same franchise:
```typescript
const franchise = await getFranchiseMovies(movieId);
// All Avengers movies, all Star Wars, etc.
```

### 3. Coming Soon / Reminders
- Movies can be marked as "coming_soon"
- Users click "Remind Me"
- IMDB code stored in localStorage
- On app start, check if now available
- Show notification toast

### 4. Analytics
Records viewing behavior:
```typescript
streamStart({ contentType: 'movie', contentId: 123 });
// Every 30 seconds while playing:
streamHeartbeat();
// When done:
streamEnd();
```

---

## TV Remote Navigation

All components support D-pad navigation:
- Focus states with visible outlines
- Arrow key navigation
- Enter to select
- Back/Escape to go back

```svelte
<button
  class="nav-item"
  class:focused={isFocused}
  onkeydown={handleKeydown}
>
```
