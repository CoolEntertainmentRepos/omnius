<script lang="ts">
  import { onMount } from "svelte";
  import type { Torrent } from "$lib/api/types";
  import { getMultipleTorrentStats, type TorrentStats } from "$lib/api/commands";

  interface Props {
    torrents: Torrent[];
    selectedQuality?: string | null;
    onselect?: (torrent: Torrent) => void;
    disabled?: boolean;
    fetchLiveStats?: boolean;
  }

  let {
    torrents,
    selectedQuality = null,
    onselect,
    disabled = false,
    fetchLiveStats = true,
  }: Props = $props();

  // Live stats from TPB
  let liveStats = $state<Record<string, TorrentStats>>({});
  let loadingStats = $state(false);
  let statsError = $state(false);

  // Fetch live stats on mount
  onMount(() => {
    if (fetchLiveStats && torrents.length > 0) {
      fetchStats();
    }
  });

  async function fetchStats() {
    // Get unique hashes
    const hashes = [...new Set(torrents.map(t => t.hash))];
    if (hashes.length === 0) return;

    loadingStats = true;
    statsError = false;

    try {
      liveStats = await getMultipleTorrentStats(hashes);
    } catch (err) {
      console.error('[QualitySelector] Failed to fetch live stats:', err);
      statsError = true;
    } finally {
      loadingStats = false;
    }
  }

  // Get seeds for a torrent (live stats or fallback to stored)
  function getSeeds(torrent: Torrent): number {
    const live = liveStats[torrent.hash];
    if (live?.found) return live.seeds;
    return torrent.seeds;
  }

  // Get peers for a torrent (live stats or fallback to stored)
  function getPeers(torrent: Torrent): number {
    const live = liveStats[torrent.hash];
    if (live?.found) return live.peers;
    return torrent.peers;
  }

  // Check if we have live stats for a torrent
  function hasLiveStats(torrent: Torrent): boolean {
    return liveStats[torrent.hash]?.found ?? false;
  }

  // Sort torrents by quality (ascending)
  let sortedTorrents = $derived(
    [...torrents].sort((a, b) => {
      const qualityOrder: Record<string, number> = {
        "720p": 1,
        "1080p": 2,
        "2160p": 3,
        "4K": 3,
      };
      return (qualityOrder[a.quality] || 0) - (qualityOrder[b.quality] || 0);
    })
  );

  function formatSize(size: string): string {
    return size;
  }

  function getQualityLabel(quality: string): string {
    if (quality === "2160p") return "4K";
    return quality;
  }

  function handleSelect(torrent: Torrent) {
    if (!disabled) {
      onselect?.(torrent);
    }
  }

  function handleKeydown(event: KeyboardEvent, torrent: Torrent) {
    if ((event.key === "Enter" || event.key === " ") && !disabled) {
      event.preventDefault();
      handleSelect(torrent);
    }
  }
</script>

<div class="quality-selector">
  <div class="title-row">
    <h3 class="title">Select Quality</h3>
    {#if fetchLiveStats}
      <button
        class="refresh-btn"
        onclick={fetchStats}
        disabled={loadingStats}
        title="Refresh seed counts"
      >
        {#if loadingStats}
          <svg class="spinning" viewBox="0 0 24 24" fill="currentColor">
            <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
          </svg>
        {:else}
          <svg viewBox="0 0 24 24" fill="currentColor">
            <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
          </svg>
        {/if}
      </button>
    {/if}
  </div>

  <div class="quality-options">
    {#each sortedTorrents as torrent (torrent.hash + torrent.quality)}
      {@const isSelected = selectedQuality === torrent.quality}
      {@const seeds = getSeeds(torrent)}
      {@const peers = getPeers(torrent)}
      {@const isLive = hasLiveStats(torrent)}
      <button
        class="quality-button"
        class:selected={isSelected}
        class:disabled={disabled}
        class:no-seeds={seeds === 0}
        {disabled}
        onclick={() => handleSelect(torrent)}
        onkeydown={(e) => handleKeydown(e, torrent)}
        aria-pressed={isSelected}
        aria-label="{torrent.quality} - {torrent.size}"
      >
        <span class="quality-label">{getQualityLabel(torrent.quality)}</span>
        <span class="quality-info">
          <span class="size">{formatSize(torrent.size)}</span>
          <span class="codec">{torrent.video_codec || "x264"}</span>
        </span>
        <span class="peers-info">
          <span class="seeds" class:live={isLive} class:zero={seeds === 0} title={isLive ? "Live seed count" : "Cached seed count"}>
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M7 14l5-5 5 5z" />
            </svg>
            {seeds}
            {#if isLive}
              <span class="live-indicator"></span>
            {/if}
          </span>
          <span class="peers" class:zero={peers === 0} title="Peers">
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M7 10l5 5 5-5z" />
            </svg>
            {peers}
          </span>
        </span>
        {#if seeds === 0 && !loadingStats}
          <span class="unavailable-badge">May be unavailable</span>
        {/if}
      </button>
    {/each}
  </div>

  {#if sortedTorrents.length === 0}
    <p class="no-torrents">No torrents available</p>
  {/if}
</div>

<style>
  .quality-selector {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .title-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .title {
    font-size: 1.3rem;
    font-weight: 600;
    color: #ffffff;
    margin: 0;
  }

  .refresh-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 8px;
    color: #888;
    cursor: pointer;
    transition: all 0.2s;
  }

  .refresh-btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.12);
    color: #fff;
    border-color: rgba(255, 255, 255, 0.25);
  }

  .refresh-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .refresh-btn svg {
    width: 18px;
    height: 18px;
  }

  .refresh-btn svg.spinning {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .quality-options {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
  }

  .quality-button {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
    padding: 16px 24px;
    background: #1a1a2e;
    border: 2px solid #2a2a4e;
    border-radius: 12px;
    cursor: pointer;
    transition:
      all 0.2s ease,
      transform 0.1s ease;
    min-width: 140px;
  }

  .quality-button:hover:not(.disabled) {
    border-color: #e94560;
    background: #1f1f3a;
    transform: translateY(-2px);
  }

  .quality-button:focus {
    outline: none;
    border-color: #e94560;
    box-shadow: 0 0 0 3px rgba(233, 69, 96, 0.3);
  }

  .quality-button.selected {
    border-color: #e94560;
    background: linear-gradient(135deg, rgba(233, 69, 96, 0.2), rgba(233, 69, 96, 0.1));
  }

  .quality-button.disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .quality-label {
    font-size: 1.4rem;
    font-weight: 700;
    color: #ffffff;
  }

  .quality-info {
    display: flex;
    gap: 12px;
    font-size: 0.95rem;
    color: #888;
  }

  .size {
    color: #aaa;
  }

  .codec {
    text-transform: uppercase;
    font-size: 0.85rem;
    padding: 2px 6px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 4px;
  }

  .peers-info {
    display: flex;
    gap: 16px;
    font-size: 0.9rem;
  }

  .seeds,
  .peers {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .seeds {
    color: #4ade80;
    position: relative;
  }

  .seeds.zero {
    color: #ef4444;
  }

  .seeds.live {
    color: #22c55e;
  }

  .live-indicator {
    position: absolute;
    top: -2px;
    right: -6px;
    width: 6px;
    height: 6px;
    background: #22c55e;
    border-radius: 50%;
    animation: pulse 2s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .peers {
    color: #f59e0b;
  }

  .peers.zero {
    color: #666;
  }

  .seeds svg,
  .peers svg {
    width: 16px;
    height: 16px;
  }

  .quality-button.no-seeds {
    border-color: rgba(239, 68, 68, 0.3);
    opacity: 0.7;
  }

  .quality-button.no-seeds:hover:not(.disabled) {
    border-color: rgba(239, 68, 68, 0.5);
  }

  .unavailable-badge {
    font-size: 0.75rem;
    color: #ef4444;
    background: rgba(239, 68, 68, 0.15);
    padding: 3px 8px;
    border-radius: 4px;
    margin-top: 4px;
  }

  .no-torrents {
    font-size: 1.1rem;
    color: #666;
    text-align: center;
    padding: 24px;
  }

  /* TV-optimized styles */
  @media (min-width: 1920px) {
    .title {
      font-size: 1.6rem;
    }

    .quality-options {
      gap: 16px;
    }

    .quality-button {
      padding: 20px 32px;
      border-radius: 16px;
      min-width: 180px;
      gap: 12px;
    }

    .quality-label {
      font-size: 1.8rem;
    }

    .quality-info {
      font-size: 1.1rem;
      gap: 16px;
    }

    .codec {
      font-size: 1rem;
      padding: 4px 10px;
    }

    .peers-info {
      font-size: 1.05rem;
      gap: 20px;
    }

    .seeds svg,
    .peers svg {
      width: 20px;
      height: 20px;
    }

    .no-torrents {
      font-size: 1.4rem;
      padding: 32px;
    }
  }
</style>
