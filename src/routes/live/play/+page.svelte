<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { browser } from "$app/environment";
  let streamUrl = $derived($page.url.searchParams.get("url") || "");
  let title = $derived($page.url.searchParams.get("title") || "Channel");
  let channelIndex = $derived(parseInt($page.url.searchParams.get("chIdx") || "-1"));

  let channelList = $state<Array<{ name: string; url: string }>>([]);
  let isAndroid = $state(false);
  let error = $state<string | null>(null);
  let videoElement: HTMLVideoElement;

  async function playLiveVideo(url: string, channelTitle: string) {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke('plugin:videoplayer|play_live_video', {
      payload: { url, title: channelTitle }
    });
  }

  function goBack() {
    goto('/live', { replaceState: true });
  }

  function switchChannel(direction: 1 | -1) {
    if (channelList.length === 0 || channelIndex < 0) return;
    const nextIdx = (channelIndex + direction + channelList.length) % channelList.length;
    const next = channelList[nextIdx];
    if (next) {
      goto(`/live/play?url=${encodeURIComponent(next.url)}&title=${encodeURIComponent(next.name)}&chIdx=${nextIdx}`, { replaceState: true });
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    console.log(`[LivePlayer] keydown key=${e.key} channelList=${channelList.length} chIdx=${channelIndex} isAndroid=${isAndroid}`);
    switch (e.key) {
      case "Escape":
      case "Backspace":
      case "GoBack":
        e.preventDefault();
        e.stopImmediatePropagation();
        console.log('[LivePlayer] Back → goBack()');
        goBack();
        break;
      case "ArrowUp":
        if (channelList.length > 1) {
          e.preventDefault();
          e.stopImmediatePropagation();
          console.log(`[LivePlayer] Up → switchChannel(-1) from idx ${channelIndex}`);
          switchChannel(-1);
        }
        break;
      case "ArrowDown":
        if (channelList.length > 1) {
          e.preventDefault();
          e.stopImmediatePropagation();
          console.log(`[LivePlayer] Down → switchChannel(1) from idx ${channelIndex}`);
          switchChannel(1);
        }
        break;
    }
  }

  onMount(async () => {
    if (!browser || !streamUrl) return;
    console.log(`[LivePlayer] onMount url=${streamUrl.slice(0, 60)}... title=${title} chIdx=${channelIndex}`);

    // Load channel list for switching
    try {
      const stored = sessionStorage.getItem('liveChannelList');
      if (stored) {
        channelList = JSON.parse(stored);
        console.log(`[LivePlayer] Loaded ${channelList.length} channels from sessionStorage`);
      } else {
        console.log('[LivePlayer] No channel list in sessionStorage');
      }
    } catch (e) {
      console.error('[LivePlayer] Failed to parse channel list:', e);
    }

    // Detect platform
    try {
      const { platform } = await import("@tauri-apps/plugin-os");
      const os = await platform();
      isAndroid = os === "android";
      console.log(`[LivePlayer] Platform: ${os}, isAndroid: ${isAndroid}`);
    } catch {
      isAndroid = false;
      console.log('[LivePlayer] Platform detection failed, assuming non-Android');
    }

    document.addEventListener("keydown", handleKeydown, true);
    console.log('[LivePlayer] Keydown listener registered (capture phase)');

    // Android: launch LivePlayerActivity immediately, go back when done
    if (isAndroid) {
      console.log('[LivePlayer] Android: launching native player...');
      try {
        await playLiveVideo(streamUrl, title);
        console.log('[LivePlayer] Native player returned, going back');
      } catch (err) {
        console.error("[LivePlayer] Error:", err);
        error = "Player failed: " + (err instanceof Error ? err.message : String(err));
        return;
      }
      goBack();
      return;
    }
    console.log('[LivePlayer] Desktop: using HTML5 video');
  });

  onDestroy(() => {
    if (browser) {
      document.removeEventListener("keydown", handleKeydown, true);
    }
  });
</script>

<svelte:head>
  <title>{title} - Live</title>
</svelte:head>

<div class="live-player">
  {#if error}
    <div class="live-error">
      <p>{error}</p>
      <button onclick={goBack}>Go Back</button>
    </div>
  {:else if !isAndroid && streamUrl}
    <!-- Desktop: HTML5 video player -->
    <video
      bind:this={videoElement}
      src={streamUrl}
      controls
      autoplay
      class="live-video"
    ></video>
    <div class="live-overlay">
      <h1 class="live-title">{title}</h1>
      <button class="live-close" onclick={goBack} aria-label="Close">
        <svg viewBox="0 0 24 24" fill="currentColor">
          <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
        </svg>
      </button>
    </div>
  {:else}
    <!-- Android: brief splash while LivePlayerActivity launches -->
    <div class="live-launching">
      <h1>{title}</h1>
      <div class="live-spinner"></div>
    </div>
  {/if}
</div>

<style>
  .live-player {
    position: fixed;
    inset: 0;
    background: #000;
    z-index: 9999;
  }

  .live-video {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  .live-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 24px 32px;
    background: linear-gradient(to bottom, rgba(0,0,0,0.7), transparent);
    opacity: 0;
    transition: opacity 0.3s;
  }

  .live-player:hover .live-overlay,
  .live-overlay:focus-within {
    opacity: 1;
  }

  .live-title {
    font-size: 1.4rem;
    font-weight: 600;
    color: #fff;
    margin: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .live-close {
    width: 44px;
    height: 44px;
    background: rgba(255,255,255,0.1);
    border: none;
    border-radius: 50%;
    color: #fff;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .live-close:focus {
    outline: none;
    box-shadow: 0 0 0 3px #e50914;
  }

  .live-close svg {
    width: 24px;
    height: 24px;
  }

  .live-launching {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #fff;
    gap: 24px;
  }

  .live-launching h1 {
    font-size: 1.6rem;
    font-weight: 600;
    margin: 0;
  }

  .live-spinner {
    width: 48px;
    height: 48px;
    border: 3px solid rgba(255,255,255,0.15);
    border-top-color: #e50914;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .live-error {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #fff;
    gap: 16px;
  }

  .live-error p {
    color: #e50914;
    font-size: 1.1rem;
  }

  .live-error button {
    padding: 12px 28px;
    background: rgba(255,255,255,0.1);
    border: 1px solid rgba(255,255,255,0.2);
    border-radius: 8px;
    color: #fff;
    font-size: 1rem;
    cursor: pointer;
  }

  .live-error button:focus {
    outline: none;
    box-shadow: 0 0 0 3px #e50914;
  }
</style>
