<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { browser } from "$app/environment";
  import { streamStore } from "$lib/stores/stream.svelte";

  let hash = $derived($page.params.hash || "");
  let title = $derived($page.url.searchParams.get("title") || "Movie");
  let imdbCode = $derived($page.url.searchParams.get("imdb") || "");
  let urlStreamUrl = $derived($page.url.searchParams.get("url") || "");

  let videoElement: HTMLVideoElement;
  let playerContainer: HTMLDivElement;
  let controlsVisible = $state(true);
  let controlsTimeout: ReturnType<typeof setTimeout>;
  let isPlaying = $state(false);
  let currentTime = $state(0);
  let duration = $state(0);
  let volume = $state(1);
  let isMuted = $state(false);
  let isFullscreen = $state(false);
  let isBuffering = $state(true);
  let subtitles = $state<{ label: string; src: string; lang: string }[]>([]);
  let activeSubtitle = $state<string | null>(null);
  let showSubtitleMenu = $state(false);
  let error = $state<string | null>(null);

  // Use URL param first, then store, then fallback
  const streamUrl = $derived(
    urlStreamUrl || streamStore.streamUrl || `http://127.0.0.1:9753/stream/${hash.toLowerCase()}/0`
  );

  onMount(async () => {
    if (!browser) return;

    // Start polling for stream stats
    streamStore.beginStatsPolling(hash.toLowerCase());

    // Fetch subtitles from OpenSubtitles (using IMDB code)
    if (imdbCode) {
      await fetchSubtitles(imdbCode);
    }

    // Auto-hide controls
    resetControlsTimeout();

    // Fullscreen change listener
    document.addEventListener("fullscreenchange", handleFullscreenChange);

    // Keyboard controls
    document.addEventListener("keydown", handleKeydown);
  });

  onDestroy(() => {
    if (browser) {
      document.removeEventListener("fullscreenchange", handleFullscreenChange);
      document.removeEventListener("keydown", handleKeydown);
      clearTimeout(controlsTimeout);
    }
  });

  async function fetchSubtitles(imdb: string) {
    try {
      // Using YIFY subtitles API (or OpenSubtitles)
      const response = await fetch(
        `https://yifysubtitles.ch/movie-imdb/${imdb}`
      );
      // This is a placeholder - in production you'd parse the actual API response
      // For now, we'll set up the structure for subtitles
      subtitles = [];
    } catch (err) {
      console.error("Failed to fetch subtitles:", err);
    }
  }

  function resetControlsTimeout() {
    clearTimeout(controlsTimeout);
    controlsVisible = true;
    controlsTimeout = setTimeout(() => {
      if (isPlaying) {
        controlsVisible = false;
      }
    }, 3000);
  }

  function handleMouseMove() {
    resetControlsTimeout();
  }

  function handleFullscreenChange() {
    isFullscreen = !!document.fullscreenElement;
  }

  function handleKeydown(e: KeyboardEvent) {
    switch (e.key) {
      case " ":
      case "k":
        e.preventDefault();
        togglePlay();
        break;
      case "f":
        toggleFullscreen();
        break;
      case "m":
        toggleMute();
        break;
      case "ArrowLeft":
        e.preventDefault();
        seek(-10);
        break;
      case "ArrowRight":
        e.preventDefault();
        seek(10);
        break;
      case "ArrowUp":
        e.preventDefault();
        adjustVolume(0.1);
        break;
      case "ArrowDown":
        e.preventDefault();
        adjustVolume(-0.1);
        break;
      case "Escape":
        if (!document.fullscreenElement) {
          handleBack();
        }
        break;
    }
    resetControlsTimeout();
  }

  function togglePlay() {
    if (!videoElement) return;
    if (isPlaying) {
      videoElement.pause();
    } else {
      videoElement.play();
    }
  }

  function toggleMute() {
    if (!videoElement) return;
    isMuted = !isMuted;
    videoElement.muted = isMuted;
  }

  function toggleFullscreen() {
    if (!playerContainer) return;
    if (document.fullscreenElement) {
      document.exitFullscreen();
    } else {
      playerContainer.requestFullscreen();
    }
  }

  function seek(seconds: number) {
    if (!videoElement) return;
    videoElement.currentTime = Math.max(0, Math.min(duration, videoElement.currentTime + seconds));
  }

  function adjustVolume(delta: number) {
    if (!videoElement) return;
    volume = Math.max(0, Math.min(1, volume + delta));
    videoElement.volume = volume;
    if (volume > 0) isMuted = false;
  }

  function handleTimeUpdate() {
    currentTime = videoElement?.currentTime || 0;
  }

  function handleLoadedMetadata() {
    duration = videoElement?.duration || 0;
    isBuffering = false;
  }

  function handlePlay() {
    isPlaying = true;
  }

  function handlePause() {
    isPlaying = false;
  }

  function handleWaiting() {
    isBuffering = true;
  }

  function handlePlaying() {
    isBuffering = false;
  }

  function handleError() {
    error = "Failed to load video. Please try again.";
    isBuffering = false;
  }

  function handleProgressClick(e: MouseEvent) {
    if (!videoElement) return;
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const percent = (e.clientX - rect.left) / rect.width;
    videoElement.currentTime = percent * duration;
  }

  function handleVolumeChange(e: Event) {
    const target = e.target as HTMLInputElement;
    volume = parseFloat(target.value);
    if (videoElement) {
      videoElement.volume = volume;
    }
    if (volume > 0) isMuted = false;
  }

  function handleSubtitleSelect(lang: string | null) {
    activeSubtitle = lang;
    showSubtitleMenu = false;

    if (videoElement) {
      // Remove existing tracks
      const tracks = videoElement.querySelectorAll("track");
      tracks.forEach((track) => {
        (track as HTMLTrackElement).track.mode = "hidden";
      });

      // Enable selected track
      if (lang) {
        const track = videoElement.querySelector(`track[srclang="${lang}"]`) as HTMLTrackElement;
        if (track) {
          track.track.mode = "showing";
        }
      }
    }
  }

  function handleBack() {
    // Stop stream and go back
    streamStore.stop();
    history.back();
  }

  function formatTime(seconds: number): string {
    if (!isFinite(seconds)) return "0:00";
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = Math.floor(seconds % 60);
    if (h > 0) {
      return `${h}:${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
    }
    return `${m}:${s.toString().padStart(2, "0")}`;
  }
</script>

<svelte:head>
  <title>{title} - Streamer</title>
</svelte:head>

<div
  class="player-container"
  bind:this={playerContainer}
  onmousemove={handleMouseMove}
  role="application"
  aria-label="Video player"
>
  {#if error}
    <div class="error-overlay">
      <svg viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
      </svg>
      <p>{error}</p>
      <button onclick={handleBack}>Go Back</button>
    </div>
  {:else}
    <!-- Video Element -->
    <!-- svelte-ignore a11y_media_has_caption -->
    <video
      bind:this={videoElement}
      src={streamUrl}
      autoplay
      ontimeupdate={handleTimeUpdate}
      onloadedmetadata={handleLoadedMetadata}
      onplay={handlePlay}
      onpause={handlePause}
      onwaiting={handleWaiting}
      onplaying={handlePlaying}
      onerror={handleError}
      class="video"
    >
      {#each subtitles as sub (sub.lang)}
        <track kind="subtitles" label={sub.label} srclang={sub.lang} src={sub.src} />
      {/each}
    </video>

    <!-- Buffering Indicator -->
    {#if isBuffering}
      <div class="buffering">
        <div class="spinner"></div>
        <p>Loading...</p>
        <div class="stream-info">
          <span>Progress: {streamStore.progressPercent.toFixed(1)}%</span>
          <span>Speed: {streamStore.downloadSpeed}</span>
          <span>Peers: {streamStore.peersConnected}</span>
        </div>
      </div>
    {/if}

    <!-- Controls Overlay -->
    <div class="controls-overlay" class:visible={controlsVisible || !isPlaying}>
      <!-- Top Bar -->
      <div class="top-bar">
        <button class="back-btn" onclick={handleBack} aria-label="Go back">
          <svg viewBox="0 0 24 24" fill="currentColor">
            <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
          </svg>
        </button>
        <h1 class="video-title">{title}</h1>
      </div>

      <!-- Center Play Button -->
      <button class="center-play" onclick={togglePlay} aria-label={isPlaying ? "Pause" : "Play"}>
        {#if isPlaying}
          <svg viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>
          </svg>
        {:else}
          <svg viewBox="0 0 24 24" fill="currentColor">
            <path d="M8 5v14l11-7z"/>
          </svg>
        {/if}
      </button>

      <!-- Bottom Controls -->
      <div class="bottom-controls">
        <!-- Progress Bar -->
        <div class="progress-container" onclick={handleProgressClick} role="slider" aria-label="Seek" tabindex="0">
          <div class="progress-bar">
            <div class="progress-buffered" style="width: {streamStore.progressPercent}%"></div>
            <div class="progress-played" style="width: {(currentTime / duration) * 100}%"></div>
          </div>
          <div class="progress-thumb" style="left: {(currentTime / duration) * 100}%"></div>
        </div>

        <div class="controls-row">
          <!-- Left Controls -->
          <div class="controls-left">
            <button class="control-btn" onclick={togglePlay} aria-label={isPlaying ? "Pause" : "Play"}>
              {#if isPlaying}
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>
                </svg>
              {:else}
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M8 5v14l11-7z"/>
                </svg>
              {/if}
            </button>

            <button class="control-btn" onclick={() => seek(-10)} aria-label="Rewind 10 seconds">
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M11 18V6l-8.5 6 8.5 6zm.5-6l8.5 6V6l-8.5 6z"/>
              </svg>
            </button>

            <button class="control-btn" onclick={() => seek(10)} aria-label="Forward 10 seconds">
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M4 18l8.5-6L4 6v12zm9-12v12l8.5-6L13 6z"/>
              </svg>
            </button>

            <div class="volume-control">
              <button class="control-btn" onclick={toggleMute} aria-label={isMuted ? "Unmute" : "Mute"}>
                {#if isMuted || volume === 0}
                  <svg viewBox="0 0 24 24" fill="currentColor">
                    <path d="M16.5 12c0-1.77-1.02-3.29-2.5-4.03v2.21l2.45 2.45c.03-.2.05-.41.05-.63zm2.5 0c0 .94-.2 1.82-.54 2.64l1.51 1.51C20.63 14.91 21 13.5 21 12c0-4.28-2.99-7.86-7-8.77v2.06c2.89.86 5 3.54 5 6.71zM4.27 3L3 4.27 7.73 9H3v6h4l5 5v-6.73l4.25 4.25c-.67.52-1.42.93-2.25 1.18v2.06c1.38-.31 2.63-.95 3.69-1.81L19.73 21 21 19.73l-9-9L4.27 3zM12 4L9.91 6.09 12 8.18V4z"/>
                  </svg>
                {:else if volume < 0.5}
                  <svg viewBox="0 0 24 24" fill="currentColor">
                    <path d="M18.5 12c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM5 9v6h4l5 5V4L9 9H5z"/>
                  </svg>
                {:else}
                  <svg viewBox="0 0 24 24" fill="currentColor">
                    <path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z"/>
                  </svg>
                {/if}
              </button>
              <input
                type="range"
                min="0"
                max="1"
                step="0.1"
                value={volume}
                oninput={handleVolumeChange}
                class="volume-slider"
                aria-label="Volume"
              />
            </div>

            <span class="time-display">
              {formatTime(currentTime)} / {formatTime(duration)}
            </span>
          </div>

          <!-- Right Controls -->
          <div class="controls-right">
            <!-- Subtitles -->
            <div class="subtitle-menu-container">
              <button
                class="control-btn"
                onclick={() => (showSubtitleMenu = !showSubtitleMenu)}
                aria-label="Subtitles"
                class:active={activeSubtitle !== null}
              >
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zM4 12h4v2H4v-2zm10 6H4v-2h10v2zm6 0h-4v-2h4v2zm0-4H10v-2h10v2z"/>
                </svg>
              </button>
              {#if showSubtitleMenu}
                <div class="subtitle-menu">
                  <button
                    class="subtitle-option"
                    class:selected={activeSubtitle === null}
                    onclick={() => handleSubtitleSelect(null)}
                  >
                    Off
                  </button>
                  {#each subtitles as sub (sub.lang)}
                    <button
                      class="subtitle-option"
                      class:selected={activeSubtitle === sub.lang}
                      onclick={() => handleSubtitleSelect(sub.lang)}
                    >
                      {sub.label}
                    </button>
                  {/each}
                  {#if subtitles.length === 0}
                    <span class="no-subtitles">No subtitles available</span>
                  {/if}
                </div>
              {/if}
            </div>

            <!-- Fullscreen -->
            <button class="control-btn" onclick={toggleFullscreen} aria-label="Fullscreen">
              {#if isFullscreen}
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M5 16h3v3h2v-5H5v2zm3-8H5v2h5V5H8v3zm6 11h2v-3h3v-2h-5v5zm2-11V5h-2v5h5V8h-3z"/>
                </svg>
              {:else}
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M7 14H5v5h5v-2H7v-3zm-2-4h2V7h3V5H5v5zm12 7h-3v2h5v-5h-2v3zM14 5v2h3v3h2V5h-5z"/>
                </svg>
              {/if}
            </button>
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .player-container {
    position: fixed;
    inset: 0;
    background: #000;
    z-index: 9999;
  }

  .video {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  .error-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 20px;
    background: #141414;
    color: #fff;
  }

  .error-overlay svg {
    width: 80px;
    height: 80px;
    color: #e50914;
  }

  .error-overlay p {
    font-size: 1.5rem;
  }

  .error-overlay button {
    padding: 14px 32px;
    background: #e50914;
    border: none;
    border-radius: 6px;
    color: #fff;
    font-size: 1.2rem;
    font-weight: 600;
    cursor: pointer;
  }

  .buffering {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 20px;
    background: rgba(0, 0, 0, 0.7);
    color: #fff;
  }

  .spinner {
    width: 60px;
    height: 60px;
    border: 4px solid rgba(255, 255, 255, 0.2);
    border-top-color: #e50914;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .buffering p {
    font-size: 1.3rem;
  }

  .stream-info {
    display: flex;
    gap: 24px;
    font-size: 1rem;
    color: #888;
  }

  .controls-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    opacity: 0;
    transition: opacity 0.3s ease;
    pointer-events: none;
  }

  .controls-overlay.visible {
    opacity: 1;
    pointer-events: auto;
  }

  .top-bar {
    display: flex;
    align-items: center;
    gap: 20px;
    padding: 30px 40px;
    background: linear-gradient(to bottom, rgba(0, 0, 0, 0.8), transparent);
  }

  .back-btn {
    width: 50px;
    height: 50px;
    background: rgba(255, 255, 255, 0.1);
    border: none;
    border-radius: 50%;
    color: #fff;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.2s ease;
  }

  .back-btn:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  .back-btn:focus {
    outline: none;
    box-shadow: 0 0 0 3px rgba(255, 255, 255, 0.3);
  }

  .back-btn svg {
    width: 28px;
    height: 28px;
  }

  .video-title {
    font-size: 1.8rem;
    font-weight: 600;
    margin: 0;
    text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.8);
  }

  .center-play {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 100px;
    height: 100px;
    background: rgba(0, 0, 0, 0.6);
    border: none;
    border-radius: 50%;
    color: #fff;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
    opacity: 0;
  }

  .controls-overlay.visible .center-play {
    opacity: 1;
  }

  .center-play:hover {
    background: rgba(0, 0, 0, 0.8);
    transform: translate(-50%, -50%) scale(1.1);
  }

  .center-play:focus {
    outline: none;
    box-shadow: 0 0 0 4px rgba(255, 255, 255, 0.3);
  }

  .center-play svg {
    width: 50px;
    height: 50px;
    margin-left: 4px;
  }

  .bottom-controls {
    padding: 20px 40px 30px;
    background: linear-gradient(to top, rgba(0, 0, 0, 0.9), transparent);
  }

  .progress-container {
    position: relative;
    height: 20px;
    cursor: pointer;
    display: flex;
    align-items: center;
    margin-bottom: 16px;
  }

  .progress-bar {
    width: 100%;
    height: 6px;
    background: rgba(255, 255, 255, 0.2);
    border-radius: 3px;
    overflow: hidden;
    position: relative;
  }

  .progress-container:hover .progress-bar {
    height: 8px;
  }

  .progress-buffered {
    position: absolute;
    height: 100%;
    background: rgba(255, 255, 255, 0.4);
    transition: width 0.3s ease;
  }

  .progress-played {
    position: absolute;
    height: 100%;
    background: #e50914;
    transition: width 0.1s linear;
  }

  .progress-thumb {
    position: absolute;
    width: 16px;
    height: 16px;
    background: #e50914;
    border-radius: 50%;
    top: 50%;
    transform: translate(-50%, -50%) scale(0);
    transition: transform 0.2s ease;
  }

  .progress-container:hover .progress-thumb {
    transform: translate(-50%, -50%) scale(1);
  }

  .controls-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .controls-left,
  .controls-right {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .control-btn {
    width: 44px;
    height: 44px;
    background: none;
    border: none;
    color: #fff;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    transition: background 0.2s ease;
  }

  .control-btn:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .control-btn:focus {
    outline: none;
    background: rgba(255, 255, 255, 0.1);
  }

  .control-btn.active {
    color: #e50914;
  }

  .control-btn svg {
    width: 28px;
    height: 28px;
  }

  .volume-control {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .volume-slider {
    width: 80px;
    height: 4px;
    -webkit-appearance: none;
    appearance: none;
    background: rgba(255, 255, 255, 0.3);
    border-radius: 2px;
    cursor: pointer;
  }

  .volume-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 14px;
    height: 14px;
    background: #fff;
    border-radius: 50%;
    cursor: pointer;
  }

  .volume-slider::-moz-range-thumb {
    width: 14px;
    height: 14px;
    background: #fff;
    border-radius: 50%;
    border: none;
    cursor: pointer;
  }

  .time-display {
    font-size: 1rem;
    color: #fff;
    min-width: 120px;
  }

  .subtitle-menu-container {
    position: relative;
  }

  .subtitle-menu {
    position: absolute;
    bottom: 60px;
    right: 0;
    background: rgba(20, 20, 20, 0.95);
    border-radius: 8px;
    padding: 8px 0;
    min-width: 150px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
  }

  .subtitle-option {
    display: block;
    width: 100%;
    padding: 12px 20px;
    background: none;
    border: none;
    color: #fff;
    font-size: 1rem;
    text-align: left;
    cursor: pointer;
    transition: background 0.2s ease;
  }

  .subtitle-option:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .subtitle-option.selected {
    color: #e50914;
  }

  .no-subtitles {
    display: block;
    padding: 12px 20px;
    color: #666;
    font-size: 0.9rem;
  }

  /* TV Styles */
  @media (min-width: 1920px) {
    .top-bar {
      padding: 40px 60px;
    }

    .back-btn {
      width: 60px;
      height: 60px;
    }

    .back-btn svg {
      width: 32px;
      height: 32px;
    }

    .video-title {
      font-size: 2.2rem;
    }

    .center-play {
      width: 120px;
      height: 120px;
    }

    .center-play svg {
      width: 60px;
      height: 60px;
    }

    .bottom-controls {
      padding: 24px 60px 40px;
    }

    .control-btn {
      width: 56px;
      height: 56px;
    }

    .control-btn svg {
      width: 32px;
      height: 32px;
    }

    .time-display {
      font-size: 1.2rem;
    }
  }
</style>
