# Streamer - Development Notes

## This is a Tauri App

Streamer is a **Tauri desktop application**, not a regular web app.

### Running the App

```bash
# Run the Tauri app (correct way)
npm run tauri dev

# DO NOT use this - it won't work properly
# npm run dev
```

### Building

```bash
# Build for production
npm run tauri build
```

## Project Structure

- `src/` - Svelte 5 frontend (SvelteKit)
- `src-tauri/` - Rust backend (Tauri)
- `src/lib/components/` - Reusable Svelte components
- `src/lib/stores/` - Svelte stores (using runes)
- `src/lib/api/` - API client for torrent-server
- `src/routes/` - SvelteKit routes

## Backend API

The app connects to torrent-server at `http://localhost:8080` for:
- Movie listings and details
- Torrent streaming
- Franchise information

## Tech Stack

- **Frontend**: Svelte 5 with SvelteKit
- **Desktop**: Tauri 2.x (Rust)
- **Styling**: CSS with CSS variables
