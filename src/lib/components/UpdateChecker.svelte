<script lang="ts">
  import { onMount } from "svelte";
  import { checkForUpdates, type UpdateInfo } from "$lib/api/commands";

  let showModal = $state(false);
  let updateInfo = $state<UpdateInfo | null>(null);
  let checking = $state(true);
  let installing = $state(false);
  let downloadProgress = $state(0);
  let error = $state<string | null>(null);
  let isDesktop = $state(false);

  onMount(async () => {
    // Delay check slightly to let app initialize
    await new Promise((r) => setTimeout(r, 2000));

    // Check if we're on desktop (Tauri updater works on desktop only)
    try {
      const { check } = await import("@tauri-apps/plugin-updater");
      isDesktop = true;

      // Use Tauri's built-in updater for desktop
      const update = await check();
      if (update) {
        updateInfo = {
          current_version: update.currentVersion,
          latest_version: update.version,
          update_available: true,
          download_url: null,
          release_notes: update.body || null,
        };
        showModal = true;
      }
      checking = false;
    } catch {
      // Fallback to manual check (for Android TV or if plugin not available)
      isDesktop = false;
      try {
        const info = await checkForUpdates();
        updateInfo = info;
        if (info.update_available) {
          showModal = true;
        }
      } catch (err) {
        console.error("Failed to check for updates:", err);
        error = err instanceof Error ? err.message : "Update check failed";
      } finally {
        checking = false;
      }
    }
  });

  function dismiss() {
    showModal = false;
  }

  async function installUpdate() {
    if (!isDesktop) {
      openDownload();
      return;
    }

    installing = true;
    try {
      const { check } = await import("@tauri-apps/plugin-updater");
      const { relaunch } = await import("@tauri-apps/plugin-process");

      const update = await check();
      if (update) {
        await update.downloadAndInstall((event) => {
          if (event.event === "Started" && event.data.contentLength) {
            downloadProgress = 0;
          } else if (event.event === "Progress") {
            downloadProgress = Math.round((event.data.chunkLength / (event.data.contentLength || 1)) * 100);
          } else if (event.event === "Finished") {
            downloadProgress = 100;
          }
        });

        // Relaunch the app
        await relaunch();
      }
    } catch (err) {
      console.error("Failed to install update:", err);
      error = err instanceof Error ? err.message : "Update failed";
      installing = false;
    }
  }

  function openDownload() {
    if (updateInfo?.download_url) {
      window.open(updateInfo.download_url, "_blank");
    }
  }
</script>

{#if showModal && updateInfo}
  <div class="modal-overlay">
    <div class="modal" role="dialog" aria-modal="true" aria-labelledby="update-title">
      <div class="modal-header">
        <svg viewBox="0 0 24 24" fill="currentColor" class="update-icon">
          <path d="M21 10.12h-6.78l2.74-2.82c-2.73-2.7-7.15-2.8-9.88-.1-2.73 2.71-2.73 7.08 0 9.79s7.15 2.71 9.88 0C18.32 15.65 19 14.08 19 12.1h2c0 1.98-.88 4.55-2.64 6.29-3.51 3.48-9.21 3.48-12.72 0-3.5-3.47-3.53-9.11-.02-12.58s9.14-3.47 12.65 0L21 3v7.12zM12.5 8v4.25l3.5 2.08-.72 1.21L11 13V8h1.5z"/>
        </svg>
        <h2 id="update-title">Update Available</h2>
      </div>

      <div class="modal-body">
        <p class="version-info">
          <span class="current">v{updateInfo.current_version}</span>
          <svg viewBox="0 0 24 24" fill="currentColor" class="arrow">
            <path d="M12 4l-1.41 1.41L16.17 11H4v2h12.17l-5.58 5.59L12 20l8-8z"/>
          </svg>
          <span class="new">v{updateInfo.latest_version}</span>
        </p>

        {#if installing}
          <div class="progress-container">
            <div class="progress-bar" style="width: {downloadProgress}%"></div>
          </div>
          <p class="progress-text">Downloading update... {downloadProgress}%</p>
        {:else if updateInfo.release_notes}
          <div class="release-notes">
            <h3>What's New</h3>
            <p>{updateInfo.release_notes}</p>
          </div>
        {/if}
      </div>

      {#if !installing}
        <div class="modal-actions">
          <button class="btn-secondary" onclick={dismiss}>
            Later
          </button>
          <button class="btn-primary" onclick={installUpdate}>
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z"/>
            </svg>
            {isDesktop ? "Install Update" : "Download Update"}
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.85);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10000;
  }

  .modal {
    background: #1a1a2e;
    border-radius: 16px;
    padding: 32px;
    max-width: 480px;
    width: 90%;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .modal-header {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 24px;
  }

  .update-icon {
    width: 48px;
    height: 48px;
    color: #e50914;
  }

  .modal-header h2 {
    font-size: 1.8rem;
    font-weight: 600;
    margin: 0;
    color: #fff;
  }

  .modal-body {
    margin-bottom: 28px;
  }

  .version-info {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 16px;
    font-size: 1.4rem;
    margin: 0 0 20px;
  }

  .current {
    color: #888;
  }

  .arrow {
    width: 24px;
    height: 24px;
    color: #e50914;
  }

  .new {
    color: #4ade80;
    font-weight: 600;
  }

  .progress-container {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    height: 8px;
    overflow: hidden;
    margin-bottom: 12px;
  }

  .progress-bar {
    background: #e50914;
    height: 100%;
    transition: width 0.3s ease;
  }

  .progress-text {
    text-align: center;
    color: #888;
    font-size: 0.95rem;
    margin: 0;
  }

  .release-notes {
    background: rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    padding: 16px;
    max-height: 150px;
    overflow-y: auto;
  }

  .release-notes h3 {
    font-size: 1rem;
    font-weight: 600;
    margin: 0 0 8px;
    color: #ccc;
  }

  .release-notes p {
    font-size: 0.95rem;
    color: #999;
    margin: 0;
    line-height: 1.5;
    white-space: pre-wrap;
  }

  .modal-actions {
    display: flex;
    gap: 12px;
    justify-content: flex-end;
  }

  .btn-secondary,
  .btn-primary {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 14px 28px;
    border-radius: 8px;
    font-size: 1.1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
    border: none;
  }

  .btn-secondary {
    background: rgba(255, 255, 255, 0.1);
    color: #fff;
  }

  .btn-secondary:hover {
    background: rgba(255, 255, 255, 0.15);
  }

  .btn-secondary:focus,
  .btn-secondary:focus-visible {
    outline: none;
    box-shadow: 0 0 0 4px #e50914;
  }

  .btn-primary {
    background: #e50914;
    color: #fff;
  }

  .btn-primary:hover {
    background: #f40612;
  }

  .btn-primary:focus,
  .btn-primary:focus-visible {
    outline: none;
    box-shadow: 0 0 0 4px rgba(229, 9, 20, 0.5);
  }

  .btn-primary svg {
    width: 22px;
    height: 22px;
  }

  /* TV Styles */
  @media (min-width: 1920px) {
    .modal {
      max-width: 560px;
      padding: 40px;
    }

    .update-icon {
      width: 56px;
      height: 56px;
    }

    .modal-header h2 {
      font-size: 2.2rem;
    }

    .version-info {
      font-size: 1.6rem;
    }

    .btn-secondary,
    .btn-primary {
      padding: 18px 36px;
      font-size: 1.3rem;
    }
  }
</style>
