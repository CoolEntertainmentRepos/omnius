// Svelte 5 store for active stream state
import type { StreamInfo, StreamStats, Torrent } from "$lib/api/types";
import { startStream, stopStream, getStreamStats } from "$lib/api/commands";

class StreamStore {
  isActive = $state(false);
  isLoading = $state(false);
  error = $state<string | null>(null);
  streamInfo = $state<StreamInfo | null>(null);
  stats = $state<StreamStats | null>(null);
  selectedTorrent = $state<Torrent | null>(null);
  selectedQuality = $state<string | null>(null);

  private statsInterval: ReturnType<typeof setInterval> | null = null;

  get streamUrl() {
    return this.streamInfo?.stream_url ?? null;
  }

  get progressPercent() {
    return this.stats?.progress_percent ?? 0;
  }

  get downloadSpeed() {
    return this.stats ? this.formatSpeed(this.stats.download_speed) : "0 B/s";
  }

  get uploadSpeed() {
    return this.stats ? this.formatSpeed(this.stats.upload_speed) : "0 B/s";
  }

  get peersConnected() {
    return this.stats?.peers_connected ?? 0;
  }

  get downloadedSize() {
    return this.stats ? this.formatBytes(this.stats.downloaded_bytes) : "0 B";
  }

  get totalSize() {
    return this.stats ? this.formatBytes(this.stats.total_bytes) : "0 B";
  }

  async start(torrent: Torrent) {
    this.isLoading = true;
    this.error = null;
    this.selectedTorrent = torrent;
    this.selectedQuality = torrent.quality;

    try {
      const info = await startStream(torrent.hash);
      this.streamInfo = info;
      this.isActive = true;

      // Start polling for stats
      this.startStatsPolling(info.info_hash);
    } catch (err) {
      this.error = err instanceof Error ? err.message : "Failed to start stream";
      this.isActive = false;
      this.streamInfo = null;
    } finally {
      this.isLoading = false;
    }
  }

  // Start polling for a specific info hash (used by player page)
  beginStatsPolling(infoHash: string) {
    this.startStatsPolling(infoHash);
  }

  async stop() {
    this.stopStatsPolling();

    if (this.streamInfo) {
      try {
        await stopStream(this.streamInfo.info_hash);
      } catch (err) {
        console.error("Failed to stop stream:", err);
      }
    }

    this.isActive = false;
    this.streamInfo = null;
    this.stats = null;
    this.selectedTorrent = null;
    this.selectedQuality = null;
    this.error = null;
  }

  private startStatsPolling(infoHash: string) {
    this.stopStatsPolling();

    // Immediate first fetch
    this.fetchStats(infoHash);

    this.statsInterval = setInterval(() => {
      this.fetchStats(infoHash);
    }, 1000);
  }

  private async fetchStats(infoHash: string) {
    try {
      console.log("[StreamStore] Fetching stats for:", infoHash);
      const stats = await getStreamStats(infoHash);
      console.log("[StreamStore] Stats received:", stats);
      this.stats = stats;
    } catch (err) {
      console.error("[StreamStore] Failed to get stream stats:", err);
    }
  }

  private stopStatsPolling() {
    if (this.statsInterval) {
      clearInterval(this.statsInterval);
      this.statsInterval = null;
    }
  }

  selectQuality(quality: string, torrents: Torrent[]) {
    const torrent = torrents.find((t) => t.quality === quality);
    if (torrent) {
      this.selectedTorrent = torrent;
      this.selectedQuality = quality;
    }
  }

  formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  }

  formatSpeed(bytesPerSecond: number): string {
    return this.formatBytes(bytesPerSecond) + "/s";
  }

  reset() {
    this.stopStatsPolling();
    this.isActive = false;
    this.isLoading = false;
    this.error = null;
    this.streamInfo = null;
    this.stats = null;
    this.selectedTorrent = null;
    this.selectedQuality = null;
  }
}

export const streamStore = new StreamStore();
