<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { browser } from "$app/environment";
  import { streamStore } from "$lib/stores/stream.svelte";
  import { searchSubtitles, startStream, downloadSubtitle, checkStreamReady, serveSubtitle, recordView, streamStart, streamHeartbeat, streamEnd, type Subtitle } from "$lib/api/commands";
  import { makeFocusable } from "$lib/utils/tvNavigation";
  import { platform } from "@tauri-apps/plugin-os";
  import { playVideo } from "tauri-plugin-videoplayer-api";

  let hash = $derived($page.params.hash || "");
  let title = $derived($page.url.searchParams.get("title") || "Movie");
  let imdbCode = $derived($page.url.searchParams.get("imdb") || "");
  let urlStreamUrl = $derived($page.url.searchParams.get("url") || "");
  let movieId = $derived(parseInt($page.url.searchParams.get("movie_id") || "0"));
  let quality = $derived($page.url.searchParams.get("quality") || "");
  let fileIndex = $derived($page.url.searchParams.get("fileIndex") ? parseInt($page.url.searchParams.get("fileIndex")!) : undefined);

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
  let servedSubtitleUrl = $state<string | null>(null);
  let preferredLanguage = $state<string>("en");
  let subdlApiKey = $state<string | undefined>(undefined);
  let error = $state<string | null>(null);
  let actualStreamUrl = $state<string | null>(null);
  let isAndroid = $state(false);
  let nativePlayerLaunched = $state(false);
  let inactivityTimeout: ReturnType<typeof setTimeout>;
  let videoRetryCount = $state(0);
  let readyToPlay = $state(false);
  let bufferingStartTime = $state<number | null>(null);
  let lastProgressCheck = $state(0);
  let stuckDetected = $state(false);

  // Analytics tracking
  let viewRecorded = $state(false);
  let watchStartTime = $state<number | null>(null);
  let analyticsInterval: ReturnType<typeof setInterval> | null = null;
  let heartbeatInterval: ReturnType<typeof setInterval> | null = null;

  // Use actual stream URL once we have it, or URL param, or store
  const streamUrl = $derived(
    actualStreamUrl || urlStreamUrl || streamStore.streamUrl || ""
  );

  // Only provide src to video when stream is ready (has some progress)
  const videoSrc = $derived(
    readyToPlay && streamUrl ? streamUrl : ""
  );

  // Calculate if we have enough buffer to start playing
  // More aggressive - start early if we have good speed/peers
  const hasEnoughBuffer = $derived.by(() => {
    // If we have direct URL, we're ready immediately
    if (urlStreamUrl) return true;

    const progress = streamStore.progressPercent;
    const peers = streamStore.peersConnected;
    const speed = streamStore.downloadSpeed;

    // Need at least minimal progress
    if (progress < 0.1) return false;

    // Parse download speed (e.g., "2.5 MB/s" -> 2.5)
    const speedMatch = speed.match(/([\d.]+)\s*(MB|KB|GB)/i);
    const speedMB = speedMatch
      ? parseFloat(speedMatch[1]) * (speedMatch[2].toUpperCase() === 'GB' ? 1024 : speedMatch[2].toUpperCase() === 'KB' ? 0.001 : 1)
      : 0;

    // Ready conditions (ordered by priority):
    // 1. Progress >= 1% with decent speed (>0.5 MB/s) - most common case
    // 2. Progress >= 0.3% with good speed (>2 MB/s) and peers (>5) - fast start
    // 3. Progress >= 2% regardless of speed - fallback
    if (progress >= 1 && speedMB > 0.5) return true;
    if (progress >= 0.3 && speedMB > 2 && peers > 5) return true;
    if (progress >= 2) return true;

    return false;
  });

  // Watch for stream progress to know when ready to play
  $effect(() => {
    const progress = streamStore.progressPercent;
    const url = streamUrl;
    const ready = readyToPlay;
    const buffer = hasEnoughBuffer;  // Now a value, not a function

    console.log("[Player] Buffer check - streamUrl:", !!url, "readyToPlay:", ready,
                "hasEnoughBuffer:", buffer, "progress:", progress.toFixed(1) + "%");

    if (url && !ready && buffer) {
      console.log("[Player] Starting playback! progress:", progress.toFixed(1) + "%",
                  "speed:", streamStore.downloadSpeed, "peers:", streamStore.peersConnected);
      readyToPlay = true;
    }
  });

  // Track buffering time and detect stuck streams
  $effect(() => {
    if (!readyToPlay && streamUrl && !urlStreamUrl) {
      if (bufferingStartTime === null) {
        bufferingStartTime = Date.now();
        lastProgressCheck = streamStore.progressPercent;
      }

      // Check if stuck (no progress for 15 seconds with low speed)
      const elapsed = Date.now() - bufferingStartTime;
      if (elapsed > 15000) {
        const progressDelta = streamStore.progressPercent - lastProgressCheck;
        if (progressDelta < 0.1 && streamStore.peersConnected < 2) {
          stuckDetected = true;
        }
        lastProgressCheck = streamStore.progressPercent;
        bufferingStartTime = Date.now();
      }
    }
  });

  onMount(async () => {
    if (!browser) return;

    // Detect platform
    try {
      const os = await platform();
      isAndroid = os === "android";
      console.log("[Player] Platform detected:", os, "isAndroid:", isAndroid);
    } catch (e) {
      console.log("[Player] Platform detection failed, assuming desktop");
      isAndroid = false;
    }

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

        const info = await Promise.race([startStream(hash, fileIndex), timeoutPromise]);
        actualStreamUrl = info.stream_url;
        streamStore.streamInfo = info;
        streamStore.isActive = true;
        console.log("[Player] Stream started:", info.stream_url);
      } catch (err) {
        console.error("[Player] Failed to start stream:", err);
        error = err instanceof Error ? err.message : "Failed to start stream";
        isStartingStream = false;
        return;
      } finally {
        isStartingStream = false;
      }
    } else {
      actualStreamUrl = urlStreamUrl || streamStore.streamUrl;
    }

    // Start polling for stream stats
    streamStore.beginStatsPolling(hash.toLowerCase());

    // On Android, use native ExoPlayer for full codec support (HEVC/x265)
    // 2-phase buffer check + parallel subtitle loading
    if (isAndroid && actualStreamUrl) {
      console.log("[Player] Android detected - 2-phase buffer + parallel subs");

      // Start subtitle fetch in parallel (don't await yet)
      let subtitlePromise: Promise<string | null> = Promise.resolve(null);
      if (imdbCode) {
        subtitlePromise = fetchAndServeSubtitle(imdbCode);
      }

      // Phase 1: Wait for torrent buffer threshold
      const waitForBuffer = async (): Promise<boolean> => {
        const maxWait = 60000;
        const startTime = Date.now();

        while (Date.now() - startTime < maxWait) {
          const progress = streamStore.progressPercent;
          const peers = streamStore.peersConnected;

          console.log("[Player] Phase 1 buffer - progress:", progress.toFixed(2) + "%", "peers:", peers);

          if (progress >= 0.5 || (progress >= 0.1 && peers >= 2)) {
            return true;
          }

          await new Promise(resolve => setTimeout(resolve, 500));
        }
        return false;
      };

      const bufferReady = await waitForBuffer();

      if (!bufferReady) {
        error = "Stream buffering timed out. The torrent may not have enough peers.";
        return;
      }

      // Phase 2: Verify stream is actually serving data via HTTP
      console.log("[Player] Phase 2 - verifying stream serves data...");
      let streamReady = false;
      for (let i = 0; i < 10; i++) {
        try {
          streamReady = await checkStreamReady(actualStreamUrl);
          if (streamReady) break;
        } catch (e) {
          console.log("[Player] Stream ready check error:", e);
        }
        await new Promise(resolve => setTimeout(resolve, 1000));
      }

      if (!streamReady) {
        error = "Stream server not responding. Try again.";
        return;
      }

      // Wait for subtitle fetch (max 10s timeout, don't block playback forever)
      console.log("[Player] Waiting for subtitle fetch...");
      let nativeSubtitleUrl: string | null = null;
      try {
        nativeSubtitleUrl = await Promise.race([
          subtitlePromise,
          new Promise<null>(resolve => setTimeout(() => resolve(null), 10000))
        ]);
      } catch (e) {
        console.log("[Player] Subtitle fetch failed, continuing without subs:", e);
      }

      console.log("[Player] Launching ExoPlayer - video:", actualStreamUrl, "subtitle:", nativeSubtitleUrl);
      try {
        await playVideo(actualStreamUrl, nativeSubtitleUrl ?? undefined);
        nativePlayerLaunched = true;
        console.log("[Player] Native ExoPlayer launched - stream kept alive");
        return;
      } catch (err) {
        console.error("[Player] Native player failed:", err);
        error = "Native player failed: " + (err instanceof Error ? err.message : String(err));
      }
    }

    // Desktop: Fetch subtitles from OpenSubtitles (using IMDB code)
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

      // Clear heartbeat interval
      if (heartbeatInterval) {
        clearInterval(heartbeatInterval);
        heartbeatInterval = null;
      }

      // Send final analytics with watch duration and end stream tracking
      if (viewRecorded && movieId > 0 && watchStartTime) {
        const watchDuration = Math.floor((Date.now() - watchStartTime) / 1000);
        const completed = duration > 0 && currentTime > duration * 0.9;
        recordView({
          contentType: 'movie',
          contentId: movieId,
          imdbCode: imdbCode,
          duration: watchDuration,
          completed: completed,
          quality: quality,
        });
        // End stream tracking
        streamEnd();
        console.log('[Analytics] Final duration:', watchDuration, 'seconds, completed:', completed);
      }

      // Stop the stream when leaving the player
      streamStore.stop();
    }
  });

  /**
   * Search for subtitles and serve the best match via HTTP (for native player).
   * Returns the HTTP URL of the VTT file, or null if none found.
   */
  async function fetchAndServeSubtitle(imdb: string): Promise<string | null> {
    try {
      let languages: string | undefined;
      if (preferredLanguage) {
        languages = preferredLanguage === "en" ? "en" : `${preferredLanguage},en`;
      }
      console.log("[Player] Searching subtitles for native player, languages:", languages);

      const result = await searchSubtitles(imdb, subdlApiKey, languages);
      if (result.subtitles.length === 0) {
        console.log("[Player] No subtitles found for native player");
        return null;
      }

      // Find best subtitle (preferred language, then English fallback)
      let sub = result.subtitles.find(s => s.language === preferredLanguage);
      if (!sub && preferredLanguage !== "en") {
        sub = result.subtitles.find(s => s.language === "en");
      }
      if (!sub) {
        sub = result.subtitles[0];
      }

      console.log("[Player] Serving subtitle for native player:", sub.release_name);
      const url = await serveSubtitle(sub.download_url);
      servedSubtitleUrl = url;
      activeSubtitle = sub;
      return url;
    } catch (err) {
      console.error("[Player] Failed to fetch/serve subtitle:", err);
      return null;
    }
  }

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

    // Record view and start stream tracking on first play
    if (!viewRecorded && movieId > 0) {
      viewRecorded = true;
      watchStartTime = Date.now();

      // Record the view
      recordView({
        contentType: 'movie',
        contentId: movieId,
        imdbCode: imdbCode,
        quality: quality,
      });

      // Start stream tracking (for "Active Now" count)
      streamStart({
        contentType: 'movie',
        contentId: movieId,
        imdbCode: imdbCode,
        quality: quality,
      });

      // Start heartbeat every 30 seconds to keep stream marked as active
      if (!heartbeatInterval) {
        heartbeatInterval = setInterval(() => {
          streamHeartbeat();
        }, 30000);
      }

      console.log('[Analytics] View recorded and stream started for movie:', movieId);
    }
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
    console.log("[Player] Video error, retry count:", videoRetryCount, "progress:", streamStore.progressPercent);

    // Parse current download speed
    const speedMatch = streamStore.downloadSpeed.match(/([\d.]+)\s*(MB|KB|GB)/i);
    const speedMB = speedMatch
      ? parseFloat(speedMatch[1]) * (speedMatch[2].toUpperCase() === 'GB' ? 1024 : speedMatch[2].toUpperCase() === 'KB' ? 0.001 : 1)
      : 0;

    // More retries if we have good conditions
    const maxRetries = speedMB > 0.5 || streamStore.peersConnected > 5 ? 10 : 5;
    const retryDelay = speedMB > 1 ? 1500 : speedMB > 0.3 ? 2500 : 4000;

    if (videoRetryCount < maxRetries) {
      // Need more buffer - wait for more data
      isBuffering = true;
      readyToPlay = false;

      // Wait for more buffer before retrying
      const checkAndRetry = () => {
        if (streamStore.progressPercent > videoRetryCount * 0.5 || streamStore.progressPercent > 3) {
          console.log("[Player] Retrying video load with", streamStore.progressPercent.toFixed(1) + "% buffered");
          readyToPlay = true;
          setTimeout(() => {
            if (videoElement) {
              videoElement.load();
            }
          }, 500);
        } else {
          // Not enough buffer yet, wait more
          setTimeout(checkAndRetry, retryDelay);
        }
      };

      setTimeout(checkAndRetry, retryDelay);
    } else {
      // Show error with helpful info
      if (streamStore.peersConnected < 2) {
        error = "Not enough peers to stream. Try a different torrent with more seeders.";
      } else if (speedMB < 0.1) {
        error = "Download speed too slow. The torrent may have limited availability.";
      } else {
        error = "Unable to play this stream. The video format may not be supported.";
      }
      isBuffering = false;
    }
  }

  function retryStream() {
    error = null;
    videoRetryCount = 0;
    readyToPlay = false;
    isBuffering = true;
    bufferingStartTime = null;
    stuckDetected = false;

    // Reset and retry
    if (videoElement) {
      setTimeout(() => {
        readyToPlay = true;
        videoElement.load();
      }, 1000);
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

  async function replayNative() {
    if (!actualStreamUrl) return;
    try {
      await playVideo(actualStreamUrl, servedSubtitleUrl ?? undefined);
    } catch (err) {
      console.error("[Player] Native replay failed:", err);
      error = "Native player failed: " + (err instanceof Error ? err.message : String(err));
    }
  }

  async function openExternal() {
    if (!actualStreamUrl) return;
    try {
      const { open } = await import("@tauri-apps/plugin-opener");
      await open(actualStreamUrl);
      console.log("[Player] Opened stream in external player");
    } catch (err) {
      console.error("[Player] Failed to open external player:", err);
    }
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
  {#if nativePlayerLaunched && !error}
    <!-- Native player return screen -->
    <div class="native-return-overlay">
      <div class="native-return-content">
        <h1 class="native-return-title">{title}</h1>
        <div class="native-return-stats">
          <span>{streamStore.progressPercent.toFixed(1)}% buffered</span>
          <span>{streamStore.downloadSpeed}</span>
          <span>{streamStore.peersConnected} peers</span>
        </div>
        <div class="native-return-buttons">
          <button class="error-btn primary" onclick={replayNative}>
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M8 5v14l11-7z"/>
            </svg>
            Play Again
          </button>
          <button class="error-btn secondary" onclick={handleBack}>
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
            </svg>
            Go Back
          </button>
        </div>
      </div>
    </div>
  {:else if error}
    <div class="error-overlay">
      <div class="error-content">
        <div class="error-icon-container">
          <svg viewBox="0 0 24 24" fill="currentColor" class="error-icon">
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
          </svg>
          <div class="error-icon-ring"></div>
        </div>
        <h2>Stream Unavailable</h2>
        <p class="error-message">{error}</p>
        <div class="error-stats">
          <div class="error-stat">
            <span class="stat-value">{streamStore.progressPercent.toFixed(1)}%</span>
            <span class="stat-label">Buffered</span>
          </div>
          <div class="error-stat">
            <span class="stat-value">{streamStore.peersConnected}</span>
            <span class="stat-label">Peers</span>
          </div>
          <div class="error-stat">
            <span class="stat-value">{streamStore.downloadSpeed}</span>
            <span class="stat-label">Speed</span>
          </div>
        </div>
        <div class="error-buttons">
          <button class="error-btn secondary" onclick={handleBack}>
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
            </svg>
            Go Back
          </button>
          <button class="error-btn primary" onclick={retryStream}>
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
            </svg>
            Try Again
          </button>
          {#if !isAndroid && actualStreamUrl}
            <button class="error-btn external" onclick={openExternal}>
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 19H5V5h7V3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2v-7h-2v7zM14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3h-7z"/>
              </svg>
              Open in External Player
            </button>
          {/if}
        </div>
      </div>
    </div>
  {:else if !readyToPlay}
    <!-- Loading Splash - shown until video is ready -->
    <div class="loading-splash">
      <div class="splash-content">
        <h1 class="splash-title">{title}</h1>
        <div class="splash-spinner" class:spinner-slow={stuckDetected}></div>
        <p class="splash-status">{isStartingStream ? "Connecting to peers..." : "Buffering video..."}</p>

        <div class="splash-progress-bar">
          <div class="splash-progress-fill" style="width: {Math.min(streamStore.progressPercent * 50, 100)}%"></div>
        </div>

        <div class="splash-stats">
          <div class="splash-stat">
            <span class="splash-stat-value">{streamStore.progressPercent.toFixed(1)}%</span>
            <span class="splash-stat-label">Buffered</span>
          </div>
          <div class="splash-stat">
            <span class="splash-stat-value">{streamStore.downloadSpeed}</span>
            <span class="splash-stat-label">Speed</span>
          </div>
          <div class="splash-stat">
            <span class="splash-stat-value">{streamStore.peersConnected}</span>
            <span class="splash-stat-label">Peers</span>
          </div>
        </div>

        {#if stuckDetected}
          <p class="splash-warning">
            <svg viewBox="0 0 24 24" fill="currentColor" width="16" height="16">
              <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
            </svg>
            Low peer count - this may take longer
          </p>
        {/if}

        {#if subtitlesLoading || subtitleDownloading}
          <p class="splash-subtitle-status">Loading subtitles...</p>
        {:else if activeSubtitle}
          <p class="splash-subtitle-status">Subtitles: {activeSubtitle.language_name}</p>
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
          <div class="error-icon-container">
            <svg viewBox="0 0 24 24" fill="currentColor" class="error-icon">
              <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
            </svg>
            <div class="error-icon-ring"></div>
          </div>
          <h2>Stream Unavailable</h2>
          <p class="error-message">{error}</p>
          <div class="error-stats">
            <div class="error-stat">
              <span class="stat-value">{streamStore.progressPercent.toFixed(1)}%</span>
              <span class="stat-label">Buffered</span>
            </div>
            <div class="error-stat">
              <span class="stat-value">{streamStore.peersConnected}</span>
              <span class="stat-label">Peers</span>
            </div>
            <div class="error-stat">
              <span class="stat-value">{streamStore.downloadSpeed}</span>
              <span class="stat-label">Speed</span>
            </div>
          </div>
          <div class="error-buttons">
            <button class="error-btn secondary" onclick={handleBack}>
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
              </svg>
              Go Back
            </button>
            <button class="error-btn primary" onclick={retryStream}>
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
              </svg>
              Try Again
            </button>
          </div>
        </div>
      </div>
    {:else if isStartingStream || isBuffering}
      <!-- Buffering/Loading Indicator -->
      <div class="buffering">
        <div class="buffering-content">
          <div class="spinner"></div>
          <p class="buffering-status">{isStartingStream ? "Connecting..." : "Buffering..."}</p>
          <div class="buffering-stats">
            {#if isStartingStream}
              <span>Finding peers</span>
            {:else}
              <span>{streamStore.progressPercent.toFixed(1)}%</span>
              <span class="stat-divider">|</span>
              <span>{streamStore.downloadSpeed}</span>
              <span class="stat-divider">|</span>
              <span>{streamStore.peersConnected} peers</span>
            {/if}
          </div>
        </div>
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
    background: linear-gradient(135deg, #0a0a0a 0%, #1a1a2e 50%, #0a0a0a 100%);
    color: #fff;
    z-index: 10;
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
    max-width: 450px;
    padding: 40px;
  }

  .splash-title {
    font-size: 1.8rem;
    font-weight: 600;
    margin: 0 0 32px;
    color: #fff;
    line-height: 1.3;
  }

  .splash-spinner {
    width: 70px;
    height: 70px;
    border: 4px solid rgba(229, 9, 20, 0.15);
    border-top-color: #e50914;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    margin: 0 auto 24px;
  }

  .splash-spinner.spinner-slow {
    animation-duration: 2s;
    border-top-color: #ff9800;
    border-color: rgba(255, 152, 0, 0.15);
  }

  .splash-status {
    font-size: 1.1rem;
    color: #888;
    margin: 0 0 20px;
  }

  .splash-progress-bar {
    width: 100%;
    height: 4px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 2px;
    margin-bottom: 24px;
    overflow: hidden;
  }

  .splash-progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #e50914, #ff6b6b);
    border-radius: 2px;
    transition: width 0.3s ease;
  }

  .splash-stats {
    display: flex;
    justify-content: center;
    gap: 40px;
    margin-bottom: 20px;
  }

  .splash-stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
  }

  .splash-stat-value {
    font-size: 1.2rem;
    font-weight: 600;
    color: #fff;
  }

  .splash-stat-label {
    font-size: 0.75rem;
    color: #666;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .splash-warning {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    font-size: 0.9rem;
    color: #ff9800;
    margin: 0 0 16px;
    padding: 10px 16px;
    background: rgba(255, 152, 0, 0.1);
    border-radius: 8px;
  }

  .splash-warning svg {
    width: 18px;
    height: 18px;
    flex-shrink: 0;
  }

  .splash-subtitle-status {
    font-size: 0.85rem;
    color: #4caf50;
    margin: 0 0 20px;
  }

  .splash-cancel {
    padding: 12px 32px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 8px;
    color: #999;
    font-size: 0.95rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .splash-cancel:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #fff;
  }

  .splash-cancel:focus {
    outline: none;
    box-shadow: 0 0 0 3px rgba(229, 9, 20, 0.4);
  }

  .error-content {
    text-align: center;
    max-width: 500px;
    padding: 40px;
  }

  .error-icon-container {
    position: relative;
    width: 100px;
    height: 100px;
    margin: 0 auto 24px;
  }

  .error-icon {
    width: 60px;
    height: 60px;
    color: #e50914;
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    z-index: 2;
  }

  .error-icon-ring {
    position: absolute;
    inset: 0;
    border: 3px solid rgba(229, 9, 20, 0.3);
    border-radius: 50%;
    animation: pulse-ring 2s ease-out infinite;
  }

  @keyframes pulse-ring {
    0% {
      transform: scale(0.8);
      opacity: 1;
    }
    100% {
      transform: scale(1.3);
      opacity: 0;
    }
  }

  .error-content h2 {
    font-size: 1.8rem;
    font-weight: 600;
    margin: 0 0 12px;
    color: #fff;
  }

  .error-message {
    font-size: 1.1rem;
    color: #999;
    margin: 0 0 24px;
    line-height: 1.5;
  }

  .error-stats {
    display: flex;
    justify-content: center;
    gap: 32px;
    margin-bottom: 32px;
    padding: 16px 24px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 12px;
  }

  .error-stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
  }

  .stat-value {
    font-size: 1.3rem;
    font-weight: 600;
    color: #fff;
  }

  .stat-label {
    font-size: 0.8rem;
    color: #666;
    text-transform: uppercase;
    letter-spacing: 0.5px;
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
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 8px;
    color: #fff;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .error-btn:hover {
    background: rgba(255, 255, 255, 0.15);
    border-color: rgba(255, 255, 255, 0.25);
  }

  .error-btn:focus {
    outline: none;
    box-shadow: 0 0 0 3px rgba(229, 9, 20, 0.5);
  }

  .error-btn svg {
    width: 20px;
    height: 20px;
  }

  .error-btn.secondary {
    background: transparent;
  }

  .error-btn.primary {
    background: #e50914;
    border-color: #e50914;
  }

  .error-btn.primary:hover {
    background: #f40612;
    border-color: #f40612;
  }

  .native-return-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, #0a0a0a 0%, #1a1a2e 50%, #0a0a0a 100%);
    color: #fff;
    z-index: 10;
  }

  .native-return-content {
    text-align: center;
    max-width: 500px;
    padding: 40px;
  }

  .native-return-title {
    font-size: 1.8rem;
    font-weight: 600;
    margin: 0 0 24px;
  }

  .native-return-stats {
    display: flex;
    justify-content: center;
    gap: 24px;
    font-size: 0.9rem;
    color: #888;
    margin-bottom: 32px;
  }

  .native-return-buttons {
    display: flex;
    gap: 16px;
    justify-content: center;
  }

  .error-btn.external {
    background: #1a73e8;
    border-color: #1a73e8;
  }

  .error-btn.external:hover {
    background: #1565c0;
    border-color: #1565c0;
  }

  .buffering {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.75);
    color: #fff;
    z-index: 5;
  }

  .buffering-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 24px 40px;
    background: rgba(0, 0, 0, 0.6);
    border-radius: 16px;
    backdrop-filter: blur(10px);
  }

  .spinner {
    width: 48px;
    height: 48px;
    border: 3px solid rgba(255, 255, 255, 0.15);
    border-top-color: #e50914;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    margin-bottom: 16px;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .buffering-status {
    font-size: 1rem;
    font-weight: 500;
    margin: 0 0 8px;
    color: #fff;
  }

  .buffering-stats {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.85rem;
    color: #888;
  }

  .stat-divider {
    color: #444;
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
