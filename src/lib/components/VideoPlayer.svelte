<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import videojs from "video.js";
  import type Player from "video.js/dist/types/player";
  import "video.js/dist/video-js.css";

  interface Props {
    src: string | null;
    poster?: string;
    autoplay?: boolean;
    onready?: (player: Player) => void;
    onerror?: (error: unknown) => void;
  }

  let { src, poster = "", autoplay = false, onready, onerror }: Props = $props();

  let videoElement: HTMLVideoElement;
  let player: Player | null = $state(null);
  let isReady = $state(false);

  onMount(() => {
    if (!videoElement) return;

    const options = {
      controls: true,
      autoplay: autoplay,
      preload: "auto",
      fluid: false,
      responsive: true,
      fill: true,
      playbackRates: [0.5, 1, 1.25, 1.5, 2],
      controlBar: {
        children: [
          "playToggle",
          "volumePanel",
          "currentTimeDisplay",
          "timeDivider",
          "durationDisplay",
          "progressControl",
          "playbackRateMenuButton",
          "fullscreenToggle",
        ],
      },
      userActions: {
        hotkeys: true,
      },
    };

    player = videojs(videoElement, options, function (this: Player) {
      isReady = true;
      onready?.(this);
    });

    player.on("error", (error: unknown) => {
      console.error("Video.js error:", error);
      onerror?.(error);
    });

    // TV-friendly: larger hit areas and focus management
    player.on("ready", function (this: Player) {
      const controlBar = (this as unknown as { controlBar?: { el: () => HTMLElement } }).controlBar;
      if (controlBar) {
        controlBar.el().setAttribute("tabindex", "0");
      }
    });
  });

  onDestroy(() => {
    if (player) {
      player.dispose();
      player = null;
    }
  });

  // Update source when src changes
  $effect(() => {
    if (player && isReady && src) {
      player.src({
        src: src,
        type: "video/mp4",
      });
      if (poster) {
        player.poster(poster);
      }
    }
  });
</script>

<div class="video-container">
  {#if !src}
    <div class="placeholder">
      <svg class="play-icon" viewBox="0 0 24 24" fill="currentColor">
        <path d="M8 5v14l11-7z" />
      </svg>
      <p>Select a quality to start streaming</p>
    </div>
  {/if}

  <!-- svelte-ignore a11y_media_has_caption -->
  <video
    bind:this={videoElement}
    class="video-js vjs-big-play-centered vjs-theme-streamer"
    {poster}
    playsinline
  >
    {#if src}
      <source src={src} type="video/mp4" />
    {/if}
    <p class="vjs-no-js">
      To view this video please enable JavaScript, and consider upgrading to a web browser that
      supports HTML5 video
    </p>
  </video>
</div>

<style>
  .video-container {
    position: relative;
    width: 100%;
    height: 100%;
    min-height: 400px;
    background: #000;
    border-radius: 12px;
    overflow: hidden;
  }

  .placeholder {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 20px;
    background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
    z-index: 1;
  }

  .play-icon {
    width: 80px;
    height: 80px;
    color: #e94560;
    opacity: 0.6;
  }

  .placeholder p {
    font-size: 1.4rem;
    color: #888;
  }

  :global(.video-js) {
    width: 100%;
    height: 100%;
  }

  /* Custom Video.js theme for TV */
  :global(.vjs-theme-streamer) {
    font-family: inherit;
  }

  :global(.vjs-theme-streamer .vjs-control-bar) {
    background: linear-gradient(to top, rgba(0, 0, 0, 0.9), transparent);
    height: 60px;
    padding: 0 20px;
  }

  :global(.vjs-theme-streamer .vjs-button) {
    font-size: 1.4rem;
  }

  :global(.vjs-theme-streamer .vjs-play-control) {
    font-size: 1.8rem;
  }

  :global(.vjs-theme-streamer .vjs-big-play-button) {
    width: 100px;
    height: 100px;
    line-height: 100px;
    font-size: 4rem;
    border: none;
    border-radius: 50%;
    background: rgba(233, 69, 96, 0.9);
    transition: transform 0.2s ease;
  }

  :global(.vjs-theme-streamer .vjs-big-play-button:hover),
  :global(.vjs-theme-streamer .vjs-big-play-button:focus) {
    background: #e94560;
    transform: scale(1.1);
  }

  :global(.vjs-theme-streamer .vjs-progress-control .vjs-play-progress) {
    background: #e94560;
  }

  :global(.vjs-theme-streamer .vjs-volume-level) {
    background: #e94560;
  }

  :global(.vjs-theme-streamer .vjs-time-control) {
    font-size: 1.2rem;
    line-height: 60px;
  }

  :global(.vjs-theme-streamer .vjs-slider:focus) {
    box-shadow: 0 0 0 3px rgba(233, 69, 96, 0.5);
  }

  :global(.vjs-theme-streamer .vjs-button:focus) {
    outline: 2px solid #e94560;
    outline-offset: 2px;
  }

  /* TV-optimized styles */
  @media (min-width: 1920px) {
    .video-container {
      min-height: 600px;
    }

    .play-icon {
      width: 120px;
      height: 120px;
    }

    .placeholder p {
      font-size: 1.8rem;
    }

    :global(.vjs-theme-streamer .vjs-control-bar) {
      height: 80px;
      padding: 0 30px;
    }

    :global(.vjs-theme-streamer .vjs-button) {
      font-size: 1.8rem;
    }

    :global(.vjs-theme-streamer .vjs-play-control) {
      font-size: 2.2rem;
    }

    :global(.vjs-theme-streamer .vjs-big-play-button) {
      width: 140px;
      height: 140px;
      line-height: 140px;
      font-size: 5rem;
    }

    :global(.vjs-theme-streamer .vjs-time-control) {
      font-size: 1.5rem;
      line-height: 80px;
    }
  }
</style>
