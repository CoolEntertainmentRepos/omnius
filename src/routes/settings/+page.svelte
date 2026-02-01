<script lang="ts">
  import { onMount } from "svelte";
  import { browser } from "$app/environment";
  import { getStorageInfo, clearCache, getAppVersion, getSubtitleLanguages, type StorageInfo, type SubtitleLanguage } from "$lib/api/commands";

  let storageInfo = $state<StorageInfo | null>(null);
  let appVersion = $state<string>("");
  let isLoading = $state(true);
  let isClearing = $state(false);
  let error = $state<string | null>(null);
  let successMessage = $state<string | null>(null);

  // Subtitle language settings
  let availableLanguages = $state<SubtitleLanguage[]>([]);
  let preferredLanguage = $state<string>("en");

  onMount(async () => {
    // Load saved preferences
    if (browser) {
      const savedLang = localStorage.getItem("preferredSubtitleLanguage");
      if (savedLang) preferredLanguage = savedLang;
    }
    await loadData();
    if (browser) {
      setTimeout(() => {
        const langSelect = document.getElementById('lang-select');
        if (langSelect) langSelect.focus();
      }, 200);
    }
  });

  async function loadData() {
    isLoading = true;
    error = null;
    try {
      const [storage, version, languages] = await Promise.all([
        getStorageInfo(),
        getAppVersion(),
        getSubtitleLanguages(),
      ]);
      storageInfo = storage;
      appVersion = version;
      availableLanguages = languages;
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to load settings";
    } finally {
      isLoading = false;
    }
  }

  function handleLanguageChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    preferredLanguage = target.value;
    if (browser) {
      localStorage.setItem("preferredSubtitleLanguage", preferredLanguage);
      successMessage = "Subtitle language saved";
      setTimeout(() => successMessage = null, 2000);
    }
  }

  async function handleClearCache() {
    if (!confirm("Clear all downloaded files?")) return;

    isClearing = true;
    successMessage = null;
    error = null;

    try {
      const bytesCleared = await clearCache();
      successMessage = `Cleared ${formatBytes(bytesCleared)}`;
      await loadData();
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to clear cache";
    } finally {
      isClearing = false;
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  }

  function handleBack() {
    history.back();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" || e.key === "Backspace") {
      e.preventDefault();
      handleBack();
    }
  }
</script>

<svelte:head>
  <title>Settings - Streamer</title>
</svelte:head>

<svelte:window onkeydown={handleKeydown} />

<div class="settings-page">
  <header class="header">
    <button class="back-btn" onclick={handleBack} id="back-btn" aria-label="Go back">
      <svg viewBox="0 0 24 24" fill="currentColor">
        <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
      </svg>
    </button>
    <h1>Settings</h1>
  </header>

  <main class="content">
    {#if isLoading}
      <div class="loading">
        <div class="spinner"></div>
      </div>
    {:else if error}
      <div class="error-message">
        <p>{error}</p>
        <button class="retry-btn" onclick={loadData}>Retry</button>
      </div>
    {:else}
      {#if successMessage}
        <div class="success-message">{successMessage}</div>
      {/if}

      <div class="info-section">
        <div class="info-row">
          <span class="info-label">Subtitle Language</span>
          <select
            id="lang-select"
            class="lang-select"
            value={preferredLanguage}
            onchange={handleLanguageChange}
            tabindex="0"
          >
            <option value="">Off (No auto-load)</option>
            {#each availableLanguages as lang (lang.code)}
              <option value={lang.code}>{lang.name}</option>
            {/each}
          </select>
        </div>
        <div class="info-row">
          <span class="info-label">Cache Size</span>
          <span class="info-value">{storageInfo ? formatBytes(storageInfo.used_bytes) : "0 B"} ({storageInfo?.file_count || 0} files)</span>
        </div>
        <div class="info-row">
          <span class="info-label">Free Space</span>
          <span class="info-value">{storageInfo ? formatBytes(storageInfo.free_bytes) : "?"}</span>
        </div>
        <div class="info-row">
          <span class="info-label">Version</span>
          <span class="info-value">{appVersion}</span>
        </div>
      </div>

      <button
        class="clear-btn"
        id="clear-btn"
        onclick={handleClearCache}
        disabled={isClearing || (storageInfo?.used_bytes === 0)}
        data-default-focus
      >
        {#if isClearing}
          <div class="btn-spinner"></div>
          Clearing...
        {:else}
          <svg viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
          </svg>
          Clear Cache
        {/if}
      </button>
    {/if}
  </main>
</div>

<style>
  .settings-page {
    min-height: 100vh;
    background: #141414;
    color: #fff;
    padding: 24px;
  }

  .header {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 32px;
  }

  .header h1 {
    font-size: 1.5rem;
    font-weight: 600;
    margin: 0;
  }

  .back-btn {
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

  .back-btn:focus {
    outline: none;
    box-shadow: 0 0 0 3px #e50914;
    transform: scale(1.1);
  }

  .back-btn svg {
    width: 22px;
    height: 22px;
  }

  .content {
    max-width: 500px;
    margin: 0 auto;
  }

  .loading {
    display: flex;
    justify-content: center;
    padding: 40px;
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid rgba(255, 255, 255, 0.2);
    border-top-color: #e50914;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .error-message {
    background: rgba(229, 9, 20, 0.2);
    border-radius: 8px;
    padding: 16px;
    text-align: center;
  }

  .retry-btn {
    margin-top: 12px;
    padding: 10px 24px;
    background: #e50914;
    border: none;
    border-radius: 6px;
    color: #fff;
    font-weight: 600;
    cursor: pointer;
  }

  .retry-btn:focus {
    outline: none;
    box-shadow: 0 0 0 3px #fff;
  }

  .success-message {
    background: rgba(46, 125, 50, 0.3);
    border-radius: 8px;
    padding: 12px 16px;
    margin-bottom: 20px;
    text-align: center;
    font-size: 0.95rem;
  }

  .info-section {
    background: rgba(255, 255, 255, 0.03);
    border-radius: 12px;
    padding: 8px 0;
    margin-bottom: 24px;
  }

  .info-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 14px 20px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }

  .info-row:last-child {
    border-bottom: none;
  }

  .info-label {
    font-size: 0.95rem;
    color: #888;
  }

  .info-value {
    font-size: 0.95rem;
    color: #fff;
    font-weight: 500;
  }

  .lang-select {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 6px;
    padding: 10px 14px;
    color: #fff;
    font-size: 0.95rem;
    min-width: 160px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .lang-select:hover {
    background: rgba(255, 255, 255, 0.15);
  }

  .lang-select:focus {
    outline: none;
    box-shadow: 0 0 0 3px #e50914;
    background: rgba(229, 9, 20, 0.2);
    border-color: #e50914;
  }

  .lang-select option {
    background: #1a1a1a;
    color: #fff;
  }


  .clear-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    width: 100%;
    padding: 16px 24px;
    background: rgba(229, 9, 20, 0.15);
    border: 2px solid rgba(229, 9, 20, 0.4);
    border-radius: 10px;
    color: #fff;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
  }

  .clear-btn:hover:not(:disabled) {
    background: rgba(229, 9, 20, 0.25);
    border-color: rgba(229, 9, 20, 0.6);
  }

  .clear-btn:focus {
    outline: none;
    box-shadow: 0 0 0 4px #e50914;
    transform: scale(1.02);
    background: rgba(229, 9, 20, 0.35);
    border-color: #e50914;
  }

  .clear-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .clear-btn svg {
    width: 20px;
    height: 20px;
  }

  .btn-spinner {
    width: 18px;
    height: 18px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: #fff;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  /* TV Styles */
  @media (min-width: 960px) {
    .settings-page {
      padding: 40px 60px;
    }

    .header h1 {
      font-size: 2rem;
    }

    .back-btn {
      width: 52px;
      height: 52px;
    }

    .back-btn svg {
      width: 26px;
      height: 26px;
    }

    .content {
      max-width: 550px;
    }

    .info-row {
      padding: 18px 24px;
    }

    .info-label, .info-value {
      font-size: 1.05rem;
    }

    .clear-btn {
      padding: 18px 28px;
      font-size: 1.1rem;
    }
  }
</style>
