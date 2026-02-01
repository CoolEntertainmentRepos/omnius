<script lang="ts">
  import type { Torrent } from "$lib/api/types";

  interface Props {
    torrents: Torrent[];
    selectedQuality?: string | null;
    onselect?: (torrent: Torrent) => void;
    disabled?: boolean;
  }

  let {
    torrents,
    selectedQuality = null,
    onselect,
    disabled = false,
  }: Props = $props();

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
  <h3 class="title">Select Quality</h3>

  <div class="quality-options">
    {#each sortedTorrents as torrent (torrent.hash)}
      {@const isSelected = selectedQuality === torrent.quality}
      <button
        class="quality-button"
        class:selected={isSelected}
        class:disabled={disabled}
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
          <span class="seeds" title="Seeds">
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M7 14l5-5 5 5z" />
            </svg>
            {torrent.seeds}
          </span>
          <span class="peers" title="Peers">
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M7 10l5 5 5-5z" />
            </svg>
            {torrent.peers}
          </span>
        </span>
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

  .title {
    font-size: 1.3rem;
    font-weight: 600;
    color: #ffffff;
    margin: 0;
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
  }

  .peers {
    color: #f59e0b;
  }

  .seeds svg,
  .peers svg {
    width: 16px;
    height: 16px;
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
