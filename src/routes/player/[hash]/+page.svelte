<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { browser } from "$app/environment";
  import { streamStore } from "$lib/stores/stream.svelte";
  import { searchSubtitles, startStream, downloadSubtitle, type Subtitle } from "$lib/api/commands";
  import { makeFocusable } from "$lib/utils/tvNavigation";

  let hash = $derived($page.params.hash || "");
  let title = $derived($page.url.searchParams.get("title") || "Movie");
  let imdbCode = $derived($page.url.searchParams.get("imdb") || "");
  let urlStreamUrl = $derived($page.url.searchParams.get("url") || "");

  let videoElement: HTMLVideoElement;
  let playerContainer: HTMLDivElement;
  let controlsVisible = $state(true);
  let controlsForceHidden = $state(false);
  let controlsTimeout: ReturnType<typeof setTimeout>;
  let isPlaying = $state(false);
  let currentTime = $state(0);
  let duration = $state(0);
  let isBuffering = $state(true);
  let isStartingStream = $state(false);
  let availableSubtitles = $state<Subtitle[]>([]);
  let activeSubtitle = $state<Subtitle | null>(null);
  let showSubtitleMenu = $state(false);
  let subtitlesLoading = $state(false);
  let subtitleUrl = $state<string | null>(null);
  let subtitleDownloading = $state(false);
  let preferredLanguage = $state<string>("en");
  let subdlApiKey = $state<string | undefined>(undefined);
  let error = $state<string | null>(null);
  let actualStreamUrl = $state<string | null>(null);
  let inactivityTimeout: ReturnType<typeof setTimeout>;
  let videoRetryCount = $state(0);
  let readyToPlay = $state(false);

  // Use actual stream URL once we have it, or URL param, or store
  const streamUrl = $derived(
    actualStreamUrl || urlStreamUrl || streamStore.streamUrl || ""
  );

  // Only provide src to video when stream is ready (has some progress)
  const videoSrc = $derived(
    readyToPlay && streamUrl ? streamUrl : ""
  );

  // Watch for stream progress to know when ready to play
  $effect(() => {
    // Ready to play when we have a stream URL and either:
    // 1. Progress is > 0.5% (some data buffered)
    // 2. Or we have direct URL from params (already streaming)
    if (streamUrl && !readyToPlay) {
      if (urlStreamUrl || streamStore.progressPercent > 0.5) {
        console.log("[Player] Stream ready, progress:", streamStore.progressPercent);
        readyToPlay = true;
      }
    }
  });

  onMount(async () => {
    if (!browser) return;

    // Load subtitle settings
    const savedLang = localStorage.getItem("preferredSubtitleLanguage");
    if (savedLang) preferredLanguage = savedLang;
    const savedKey = localStorage.getItem("subdlApiKey");
    if (savedKey) subdlApiKey = savedKey;

    // Stop any existing stream before starting a new one
    if (streamStore.isActive) {
      console.log("[Player] Stopping existing stream before starting new one");
      await streamStore.stop();
    }

    // If no stream URL provided, start the stream with timeout
    if (!urlStreamUrl) {
      isStartingStream = true;
      try {
        console.log("[Player] Starting stream for hash:", hash);

        // Add timeout - 60 seconds max
        const timeoutPromise = new Promise<never>((_, reject) => {
          setTimeout(() => reject(new Error("Stream start timed out. The torrent may not have enough peers.")), 60000);
        });

        const info = await Promise.race([startStream(hash), timeoutPromise]);
        actualStreamUrl = info.stream_url;
        streamStore.streamInfo = info;
        streamStore.isActive = true;
        console.log("[Player] Stream started:", info.stream_url);
      } catch (err) {
        console.error("[Player] Failed to start stream:", err);
        error = err instanceof Error ? err.message : "Failed to start stream";
        isStartingStream = false;
      } finally {
        isStartingStream = false;
      }
    } else {
      actualStreamUrl = urlStreamUrl || streamStore.streamUrl;
    }

    // Start polling for stream stats
    streamStore.beginStatsPolling(hash.toLowerCase());

    // Fetch subtitles from OpenSubtitles (using IMDB code)
    // This runs while torrent is buffering
    if (imdbCode) {
      await fetchSubtitlesAndAutoLoad(imdbCode);
    }

    // Auto-hide controls
    resetControlsTimeout();

    // Keyboard controls
    document.addEventListener("keydown", handleKeydown);

    // Set initial focus - on cancel button if loading, otherwise center play
    setTimeout(() => {
      makeFocusable(); // Refresh focusable elements
      const cancelBtn = document.querySelector('.cancel-btn') as HTMLElement;
      const centerPlay = document.querySelector('.center-play') as HTMLElement;
      if (cancelBtn && (isStartingStream || isBuffering)) {
        cancelBtn.focus();
      } else if (centerPlay) {
        centerPlay.focus();
      }
    }, 500);
  });

  onDestroy(() => {
    if (browser) {
      document.removeEventListener("keydown", handleKeydown);
      clearTimeout(controlsTimeout);
      clearTimeout(inactivityTimeout);
      // Stop the stream when leaving the player
      streamStore.stop();
    }
  });

  async function fetchSubtitlesAndAutoLoad(imdb: string) {
    subtitlesLoading = true;
    try {
      // Build language filter: preferred language + English fallback
      let languages: string | undefined;
      if (preferredLanguage) {
        if (preferredLanguage === "en") {
          languages = "en";
        } else {
          languages = `${preferredLanguage},en`; // e.g., "sq,en"
        }
      }
      console.log(`[Player] Searching subtitles with languages: ${languages}`);

      const result = await searchSubtitles(imdb, subdlApiKey, languages);
      availableSubtitles = result.subtitles;
      console.log(`[Player] Found ${result.total_count} subtitles`);

      // Auto-load subtitle in preferred language if set
      if (preferredLanguage && result.subtitles.length > 0) {
        // Try preferred language first
        let subToLoad = result.subtitles.find(s => s.language === preferredLanguage);

        // Fallback to English if preferred not found
        if (!subToLoad && preferredLanguage !== "en") {
          subToLoad = result.subtitles.find(s => s.language === "en");
          if (subToLoad) {
            console.log(`[Player] Preferred ${preferredLanguage} not found, falling back to English`);
          }
        }

        // Load the subtitle if found
        if (subToLoad) {
          console.log(`[Player] Auto-loading ${subToLoad.language} subtitle:`, subToLoad.release_name);
          await loadSubtitle(subToLoad);
        } else {
          console.log(`[Player] No subtitle found for ${preferredLanguage} or English`);
        }
      }
    } catch (err) {
      console.error("Failed to fetch subtitles:", err);
    } finally {
      subtitlesLoading = false;
    }
  }

  async function loadSubtitle(sub: Subtitle) {
    subtitleDownloading = true;
    try {
      console.log(`[Player] Downloading subtitle: ${sub.release_name}`);
      const result = await downloadSubtitle(sub.download_url);
      subtitleUrl = result.vtt_data_url;
      activeSubtitle = sub;
      console.log("[Player] Subtitle loaded successfully");
    } catch (err) {
      console.error("[Player] Failed to download subtitle:", err);
    } finally {
      subtitleDownloading = false;
    }
  }

  function clearSubtitle() {
    subtitleUrl = null;
    activeSubtitle = null;
  }

  // Group subtitles by language for the menu
  const subtitlesByLanguage = $derived(() => {
    const grouped: Record<string, Subtitle[]> = {};
    for (const sub of availableSubtitles) {
      if (!grouped[sub.language_name]) {
        grouped[sub.language_name] = [];
      }
      grouped[sub.language_name].push(sub);
    }
    return grouped;
  });

  function resetControlsTimeout() {
    clearTimeout(controlsTimeout);
    clearTimeout(inactivityTimeout);
    controlsVisible = true;
    controlsForceHidden = false;
    controlsTimeout = setTimeout(() => {
      if (isPlaying) {
        controlsVisible = false;
      }
    }, 3000);
    // Force hide controls after 5 seconds of inactivity
    inactivityTimeout = setTimeout(() => {
      if (isPlaying) {
        showSubtitleMenu = false;
        controlsForceHidden = true;
        controlsVisible = false;
        // Blur any focused element
        if (document.activeElement instanceof HTMLElement) {
          document.activeElement.blur();
        }
      }
    }, 5000);
  }

  function handleMouseMove() {
    resetControlsTimeout();
  }

  function handleKeydown(e: KeyboardEvent) {
    const target = e.target as HTMLElement;
    const isOnFocusable = target.tagName === 'BUTTON' || target.tagName === 'INPUT' ||
                          target.classList.contains('progress-container') || target.closest('button');

    // Show controls on any key press
    controlsVisible = true;
    resetControlsTimeout();

    switch (e.key) {
      case "Escape":
      case "Backspace":
        e.preventDefault();
        handleBack();
        break;
      case " ":
      case "k":
        if (!isOnFocusable || target.classList.contains('progress-container')) {
          e.preventDefault();
          togglePlay();
        }
        break;
      case "Enter":
        if (!isOnFocusable) {
          e.preventDefault();
          togglePlay();
        }
        break;
      case "ArrowLeft":
        // If on progress bar, seek. Otherwise let spatial navigation handle it.
        if (target.classList.contains('progress-container') || target.classList.contains('seek-track')) {
          e.preventDefault();
          seek(-10);
        }
        break;
      case "ArrowRight":
        // If on progress bar, seek. Otherwise let spatial navigation handle it.
        if (target.classList.contains('progress-container') || target.classList.contains('seek-track')) {
          e.preventDefault();
          seek(10);
        }
        break;
      case "ArrowUp":
        // Let spatial navigation handle it
        break;
      case "ArrowDown":
        // Let spatial navigation handle it
        break;
    }
  }

  function togglePlay() {
    if (!videoElement) return;
    if (isPlaying) {
      videoElement.pause();
    } else {
      videoElement.play();
    }
  }

  function seek(seconds: number) {
    if (!videoElement) return;
    videoElement.currentTime = Math.max(0, Math.min(duration, videoElement.currentTime + seconds));
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
    // Allow retries - torrent streaming may need time to buffer
    videoRetryCount += 1;
    console.log("[Player] Video error, retry count:", videoRetryCount);

    if (videoRetryCount < 5) {
      // Retry after a short delay
      isBuffering = true;
      setTimeout(() => {
        if (videoElement && videoSrc) {
          console.log("[Player] Retrying video load...");
          videoElement.load();
        }
      }, 2000);
    } else {
      error = "Failed to load video. The torrent may not have enough peers.";
      isBuffering = false;
    }
  }

  function handleProgressClick(e: MouseEvent) {
    if (!videoElement) return;
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const percent = (e.clientX - rect.left) / rect.width;
    videoElement.currentTime = percent * duration;
  }

  function handleProgressKeydown(e: KeyboardEvent) {
    // When progress bar is focused, left/right seek the video
    if (e.key === 'ArrowLeft') {
      e.preventDefault();
      e.stopPropagation();
      seek(-10);
    } else if (e.key === 'ArrowRight') {
      e.preventDefault();
      e.stopPropagation();
      seek(10);
    } else if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      togglePlay();
    }
  }

  // Note: Subtitle download and display would require downloading the VTT/SRT file
  // and either serving it locally or converting it. For now, we show available subtitles
  // and could integrate with a subtitle download service in the future.

  function handleBack() {
    // Stop stream and go back
    streamStore.stop();
    history.back();
  }

  function openSubtitleMenu() {
    showSubtitleMenu = true;
    // Focus the first option after modal renders
    setTimeout(() => {
      const firstOption = document.getElementById('subtitle-off-btn') as HTMLElement;
      if (firstOption) {
        firstOption.focus();
      }
    }, 50);
  }

  function closeSubtitleMenu() {
    showSubtitleMenu = false;
    // Return focus to the subtitle button
    setTimeout(() => {
      const subtitleBtn = document.querySelector('.subtitle-btn') as HTMLElement;
      if (subtitleBtn) {
        subtitleBtn.focus();
      }
    }, 50);
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
  {:else if !readyToPlay}
    <!-- Loading Splash - shown until video is ready -->
    <div class="loading-splash">
      <div class="splash-content">
        <h1 class="splash-title">{title}</h1>
        <div class="splash-spinner"></div>
        <p class="splash-status">{isStartingStream ? "Starting stream..." : "Buffering..."}</p>
        <div class="splash-stats">
          <span>Progress: {streamStore.progressPercent.toFixed(1)}%</span>
          <span>Speed: {streamStore.downloadSpeed}</span>
          <span>Peers: {streamStore.peersConnected}</span>
        </div>
        {#if subtitlesLoading || subtitleDownloading}
          <p class="splash-subtitle-status">Loading subtitles...</p>
        {:else if activeSubtitle}
          <p class="splash-subtitle-status">Subtitles: {activeSubtitle.language_name}</p>
        {:else if preferredLanguage}
          <p class="splash-subtitle-status">Searching for {preferredLanguage} subtitles...</p>
        {/if}
        <button class="splash-cancel" onclick={handleBack}>Cancel</button>
      </div>
    </div>
  {:else}
    <!-- Loading overlay until video is ready -->
    {#if !readyToPlay}
      <div class="video-loading">
        <div class="loading-spinner"></div>
        <p>Loading video...</p>
      </div>
    {/if}

    <!-- Video Element - hidden until ready -->
    <video
      bind:this={videoElement}
      src={videoSrc}
      autoplay
      ontimeupdate={handleTimeUpdate}
      onloadedmetadata={handleLoadedMetadata}
      onplay={handlePlay}
      onpause={handlePause}
      onwaiting={handleWaiting}
      onplaying={handlePlaying}
      onerror={handleError}
      class="video"
      class:video-hidden={!readyToPlay}
    >
      {#if subtitleUrl}
        <track
          kind="subtitles"
          src={subtitleUrl}
          srclang={activeSubtitle?.language || "en"}
          label={activeSubtitle?.language_name || "Subtitles"}
          default
        />
      {/if}
    </video>

    <!-- Error State -->
    {#if error}
      <div class="error-overlay">
        <div class="error-content">
          <svg viewBox="0 0 24 24" fill="currentColor" class="error-icon">
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
          </svg>
          <h2>Stream Failed</h2>
          <p>{error}</p>
          <div class="error-buttons">
            <button class="error-btn" onclick={handleBack}>
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
              </svg>
              Go Back
            </button>
            <button class="error-btn primary" onclick={() => location.reload()}>
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
              </svg>
              Retry
            </button>
          </div>
        </div>
      </div>
    {:else if isStartingStream || isBuffering}
      <!-- Buffering/Loading Indicator -->
      <div class="buffering">
        <div class="spinner"></div>
        <p>{isStartingStream ? "Starting stream..." : "Buffering..."}</p>
        <div class="stream-info">
          {#if isStartingStream}
            <span>Connecting to peers...</span>
          {:else}
            <span>Progress: {streamStore.progressPercent.toFixed(1)}%</span>
            <span>Speed: {streamStore.downloadSpeed}</span>
            <span>Peers: {streamStore.peersConnected}</span>
          {/if}
        </div>
        <button class="cancel-btn" onclick={handleBack}>Cancel</button>
      </div>
    {/if}

    <!-- Controls Overlay -->
    <div class="controls-overlay" class:visible={controlsVisible || !isPlaying} class:force-hidden={controlsForceHidden}>
      <!-- Top Bar -->
      <div class="top-bar">
        <button class="back-btn" onclick={handleBack} aria-label="Go back" tabindex="0">
          <svg viewBox="0 0 24 24" fill="currentColor">
            <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
          </svg>
        </button>
        <h1 class="video-title">{title}</h1>
      </div>

      <!-- Center Play Button -->
      <button class="center-play" onclick={togglePlay} aria-label={isPlaying ? "Pause" : "Play"} tabindex="0">
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
        <div
          class="progress-container"
          onclick={handleProgressClick}
          onkeydown={handleProgressKeydown}
          role="slider"
          aria-label="Seek"
          aria-valuemin="0"
          aria-valuemax={duration}
          aria-valuenow={currentTime}
          tabindex="0"
        >
          <div class="progress-bar">
            <div class="progress-buffered" style="width: {streamStore.progressPercent}%"></div>
            <div class="progress-played" style="width: {(currentTime / duration) * 100}%"></div>
          </div>
          <div class="progress-thumb" style="left: {(currentTime / duration) * 100}%"></div>
        </div>

        <div class="controls-row">
          <!-- Left Controls -->
          <div class="controls-left">
            <button class="control-btn" onclick={togglePlay} aria-label={isPlaying ? "Pause" : "Play"} tabindex="0">
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

            <button class="control-btn" onclick={() => seek(-10)} aria-label="Rewind 10 seconds" tabindex="0">
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M11 18V6l-8.5 6 8.5 6zm.5-6l8.5 6V6l-8.5 6z"/>
              </svg>
            </button>

            <button class="control-btn" onclick={() => seek(10)} aria-label="Forward 10 seconds" tabindex="0">
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M4 18l8.5-6L4 6v12zm9-12v12l8.5-6L13 6z"/>
              </svg>
            </button>

            <span class="time-display">
              {formatTime(currentTime)} / {formatTime(duration)}
            </span>
          </div>

          <!-- Right Controls -->
          <div class="controls-right">
            <!-- Subtitles -->
            <div class="subtitle-menu-container">
              <button
                class="control-btn subtitle-btn"
                onclick={openSubtitleMenu}
                aria-label="Subtitles"
                class:active={activeSubtitle !== null}
                tabindex="0"
              >
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zM4 12h4v2H4v-2zm10 6H4v-2h10v2zm6 0h-4v-2h4v2zm0-4H10v-2h10v2z"/>
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Subtitle Modal -->
    {#if showSubtitleMenu}
      <div class="subtitle-modal-overlay" onclick={closeSubtitleMenu}>
        <div class="subtitle-modal" onclick={(e) => e.stopPropagation()}>
          <div class="subtitle-modal-header">
            <h2>Subtitles</h2>
            <button class="subtitle-close-btn" onclick={closeSubtitleMenu} tabindex="0">
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
              </svg>
            </button>
          </div>
          <div class="subtitle-modal-content">
            <button
              class="subtitle-modal-option"
              class:selected={activeSubtitle === null}
              onclick={() => { clearSubtitle(); closeSubtitleMenu(); }}
              tabindex="0"
              id="subtitle-off-btn"
            >
              <span class="sub-lang-badge">OFF</span>
              <span>No Subtitles</span>
            </button>
            {#if subtitlesLoading || subtitleDownloading}
              <div class="subtitle-loading">
                <div class="loading-spinner-small"></div>
                <span>Loading subtitles...</span>
              </div>
            {:else if availableSubtitles.length === 0}
              <p class="no-subtitles-msg">No subtitles found for this movie</p>
            {:else}
              {#each Object.entries(subtitlesByLanguage()) as [lang, subs] (lang)}
                {#each subs.slice(0, 5) as sub (sub.id)}
                  <button
                    class="subtitle-modal-option"
                    class:selected={activeSubtitle?.id === sub.id}
                    onclick={() => { loadSubtitle(sub); closeSubtitleMenu(); }}
                    tabindex="0"
                  >
                    <span class="sub-lang-badge">{sub.language.toUpperCase()}</span>
                    <span class="sub-release-name">{sub.release_name || lang}</span>
                    {#if sub.hearing_impaired}
                      <span class="sub-hi-badge">CC</span>
                    {/if}
                  </button>
                {/each}
              {/each}
            {/if}
          </div>
        </div>
      </div>
    {/if}
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

  .video-hidden {
    opacity: 0;
  }

  .video-loading {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    background: #000;
    z-index: 5;
  }

  .video-loading p {
    margin-top: 20px;
    color: #888;
    font-size: 1.1rem;
  }

  .loading-spinner {
    width: 60px;
    height: 60px;
    border: 4px solid rgba(255, 255, 255, 0.1);
    border-top-color: #e50914;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  /* Subtitle styling */
  .video::cue {
    background: rgba(0, 0, 0, 0.8);
    color: #fff;
    font-size: 1.4rem;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    line-height: 1.4;
    padding: 4px 8px;
    border-radius: 4px;
  }

  @media (min-width: 1920px) {
    .video::cue {
      font-size: 1.8rem;
    }
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

  /* Loading Splash */
  .loading-splash {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, #0a0a0a 0%, #1a1a2e 50%, #0a0a0a 100%);
    color: #fff;
  }

  .splash-content {
    text-align: center;
    max-width: 500px;
    padding: 40px;
  }

  .splash-title {
    font-size: 2.2rem;
    font-weight: 700;
    margin: 0 0 40px;
    color: #fff;
    text-shadow: 0 2px 20px rgba(229, 9, 20, 0.3);
  }

  .splash-spinner {
    width: 80px;
    height: 80px;
    border: 5px solid rgba(229, 9, 20, 0.2);
    border-top-color: #e50914;
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin: 0 auto 30px;
  }

  .splash-status {
    font-size: 1.3rem;
    color: #aaa;
    margin: 0 0 20px;
  }

  .splash-stats {
    display: flex;
    justify-content: center;
    gap: 24px;
    font-size: 0.95rem;
    color: #666;
    margin-bottom: 16px;
  }

  .splash-subtitle-status {
    font-size: 0.9rem;
    color: #4caf50;
    margin: 0 0 20px;
  }

  .splash-cancel {
    padding: 14px 40px;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 8px;
    color: #fff;
    font-size: 1rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .splash-cancel:hover {
    background: rgba(255, 255, 255, 0.15);
  }

  .splash-cancel:focus {
    outline: none;
    box-shadow: 0 0 0 3px #e50914;
  }

  .error-content {
    text-align: center;
    max-width: 500px;
    padding: 40px;
  }

  .error-icon {
    width: 80px;
    height: 80px;
    color: #e50914;
    margin-bottom: 20px;
  }

  .error-content h2 {
    font-size: 2rem;
    margin: 0 0 16px;
  }

  .error-content p {
    font-size: 1.2rem;
    color: #888;
    margin: 0 0 30px;
  }

  .error-buttons {
    display: flex;
    gap: 16px;
    justify-content: center;
  }

  .error-btn {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 14px 28px;
    background: rgba(255, 255, 255, 0.1);
    border: none;
    border-radius: 6px;
    color: #fff;
    font-size: 1.1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .error-btn:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  .error-btn:focus {
    outline: none;
    box-shadow: 0 0 0 3px #e50914;
  }

  .error-btn svg {
    width: 22px;
    height: 22px;
  }

  .error-btn.primary {
    background: #e50914;
  }

  .error-btn.primary:hover {
    background: #f40612;
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

  .cancel-btn {
    margin-top: 20px;
    padding: 12px 32px;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.3);
    border-radius: 6px;
    color: #fff;
    font-size: 1rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .cancel-btn:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  .cancel-btn:focus {
    outline: none;
    box-shadow: 0 0 0 3px #e50914;
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
    /* Always allow pointer events for TV navigation */
    pointer-events: auto;
  }

  .controls-overlay.visible {
    opacity: 1;
  }

  /* Show controls when any button is focused (for TV), but not when force-hidden */
  .controls-overlay:not(.force-hidden):focus-within {
    opacity: 1;
  }

  .controls-overlay.force-hidden {
    opacity: 0 !important;
    pointer-events: none;
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

  .progress-container:focus,
  .progress-container:focus-visible {
    outline: none;
  }

  .progress-container:focus .progress-bar,
  .progress-container:focus-visible .progress-bar {
    height: 10px;
    box-shadow: 0 0 0 3px #e50914;
  }

  .progress-container:focus .progress-thumb,
  .progress-container:focus-visible .progress-thumb {
    transform: translate(-50%, -50%) scale(1.2);
    box-shadow: 0 0 8px #e50914;
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

  .control-btn:focus,
  .control-btn:focus-visible {
    outline: none;
    background: rgba(229, 9, 20, 0.4);
    box-shadow: 0 0 0 3px #e50914;
    transform: scale(1.15);
  }

  .control-btn.active {
    color: #e50914;
  }

  .control-btn.active:focus {
    background: rgba(229, 9, 20, 0.5);
  }

  .control-btn svg {
    width: 28px;
    height: 28px;
  }

  .time-display {
    font-size: 1rem;
    color: #fff;
    min-width: 120px;
  }

  .subtitle-menu-container {
    position: relative;
  }

  /* Subtitle Modal */
  .subtitle-modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.85);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .subtitle-modal {
    background: #1a1a1a;
    border-radius: 16px;
    width: 90%;
    max-width: 500px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.8);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .subtitle-modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px 24px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .subtitle-modal-header h2 {
    margin: 0;
    font-size: 1.4rem;
    font-weight: 600;
  }

  .subtitle-close-btn {
    width: 44px;
    height: 44px;
    background: rgba(255, 255, 255, 0.1);
    border: none;
    border-radius: 50%;
    color: #fff;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
  }

  .subtitle-close-btn:focus {
    outline: none;
    box-shadow: 0 0 0 3px #e50914;
    background: rgba(229, 9, 20, 0.3);
  }

  .subtitle-close-btn svg {
    width: 24px;
    height: 24px;
  }

  .subtitle-modal-content {
    flex: 1;
    overflow-y: auto;
    padding: 8px 0;
  }

  .subtitle-modal-option {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 16px 24px;
    background: none;
    border: none;
    color: #fff;
    font-size: 1rem;
    text-align: left;
    cursor: pointer;
    transition: background 0.15s;
  }

  .subtitle-modal-option:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .subtitle-modal-option:focus {
    outline: none;
    background: rgba(229, 9, 20, 0.3);
  }

  .subtitle-modal-option.selected {
    background: rgba(229, 9, 20, 0.2);
  }

  .subtitle-modal-option.selected .sub-lang-badge {
    background: #e50914;
  }

  .sub-lang-badge {
    background: rgba(255, 255, 255, 0.2);
    padding: 4px 10px;
    border-radius: 4px;
    font-size: 0.8rem;
    font-weight: 600;
    min-width: 40px;
    text-align: center;
  }

  .sub-release-name {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: #aaa;
    font-size: 0.9rem;
  }

  .sub-hi-badge {
    background: #4a4a00;
    color: #ffd700;
    padding: 2px 6px;
    border-radius: 3px;
    font-size: 0.7rem;
    font-weight: 600;
  }

  .subtitle-loading {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 40px;
    color: #888;
  }

  .loading-spinner-small {
    width: 24px;
    height: 24px;
    border: 3px solid rgba(255, 255, 255, 0.1);
    border-top-color: #e50914;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  .no-subtitles-msg {
    text-align: center;
    padding: 40px;
    color: #666;
  }

  .sub-release {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 200px;
  }

  .sub-hi {
    font-size: 0.7rem;
    padding: 2px 6px;
    background: rgba(255, 255, 255, 0.2);
    border-radius: 3px;
    color: #fff;
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
