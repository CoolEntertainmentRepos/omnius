<script lang="ts">
  import { onMount } from "svelte";
  import { browser } from "$app/environment";
  import { getStorageInfo, clearDownloadedFiles, getAppVersion, getSubtitleLanguages, getApiUrl, setApiUrl, getDefaultApiUrl, type StorageInfo, type SubtitleLanguage } from "$lib/api/commands";
  import { clearAllCaches } from "$lib/api/cache";

  let storageInfo = $state<StorageInfo | null>(null);
  let appVersion = $state<string>("");
  let isLoading = $state(true);
  let isClearing = $state(false);
  let error = $state<string | null>(null);
  let successMessage = $state<string | null>(null);

  // Server URL settings
  let serverUrl = $state<string>(getApiUrl());
  let serverStatus = $state<'idle' | 'testing' | 'success' | 'error'>('idle');
  let serverStatusMessage = $state<string>('');

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
      const bytesCleared = await clearDownloadedFiles();
      successMessage = `Cleared ${formatBytes(bytesCleared)}`;
      await loadData();
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to clear cache";
    } finally {
      isClearing = false;
    }
  }

  function handleClearApiCache() {
    clearAllCaches();
    successMessage = "API cache cleared! Movie data will refresh.";
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  }

  async function handleTestConnection() {
    const url = serverUrl.trim().replace(/\/+$/, '');
    if (!url) return;
    serverStatus = 'testing';
    serverStatusMessage = '';
    try {
      const response = await fetch(`${url}/api/v2/home.json`, { signal: AbortSignal.timeout(5000) });
      if (response.ok) {
        serverStatus = 'success';
        serverStatusMessage = 'Connected';
      } else {
        serverStatus = 'error';
        serverStatusMessage = `HTTP ${response.status}`;
      }
    } catch (err) {
      serverStatus = 'error';
      serverStatusMessage = 'Connection failed';
    }
  }

  function handleSaveServer() {
    setApiUrl(serverUrl);
    clearAllCaches();
    successMessage = "Server URL saved. Data will refresh.";
    setTimeout(() => successMessage = null, 3000);
  }

  function handleResetServer() {
    serverUrl = getDefaultApiUrl();
    setApiUrl(serverUrl);
    clearAllCaches();
    serverStatus = 'idle';
    serverStatusMessage = '';
    successMessage = "Server URL reset to default.";
    setTimeout(() => successMessage = null, 3000);
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

      <!-- Server -->
      <h2 class="section-title">Server</h2>
      <input
        type="url"
        class="server-input"
        bind:value={serverUrl}
        placeholder={getDefaultApiUrl()}
        tabindex="0"
      />
      <div class="server-actions">
        <button class="server-btn primary" onclick={handleTestConnection} disabled={serverStatus === 'testing'}>
          {#if serverStatus === 'testing'}
            <div class="btn-spinner"></div>
          {:else}
            Test Connection
          {/if}
        </button>
        <button class="server-btn accent" onclick={handleSaveServer}>Save</button>
        <button class="server-btn" onclick={handleResetServer}>Reset</button>
        {#if serverStatusMessage}
          <span class="server-status {serverStatus}">
            {#if serverStatus === 'success'}
              <svg viewBox="0 0 24 24" fill="currentColor"><path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/></svg>
            {:else if serverStatus === 'error'}
              <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/></svg>
            {/if}
            {serverStatusMessage}
          </span>
        {/if}
      </div>

      <!-- Playback -->
      <h2 class="section-title">Playback</h2>
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

      <!-- Storage -->
      <h2 class="section-title">Storage</h2>
      <div class="info-row">
        <span class="info-label">Cache Size</span>
        <span class="info-value">{storageInfo ? formatBytes(storageInfo.used_bytes) : "0 B"} ({storageInfo?.file_count || 0} files)</span>
      </div>
      <div class="info-row">
        <span class="info-label">Free Space</span>
        <span class="info-value">{storageInfo ? formatBytes(storageInfo.free_bytes) : "?"}</span>
      </div>
      <div class="action-row">
        <button
          class="action-btn danger"
          id="clear-btn"
          onclick={handleClearCache}
          disabled={isClearing || (storageInfo?.used_bytes === 0)}
        >
          {#if isClearing}
            <div class="btn-spinner"></div>
            Clearing...
          {:else}
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
            </svg>
            Clear Downloads
          {/if}
        </button>
        <button class="action-btn" onclick={handleClearApiCache}>
          <svg viewBox="0 0 24 24" fill="currentColor">
            <path d="M17.65 6.35A7.958 7.958 0 0012 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08A5.99 5.99 0 0112 18c-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
          </svg>
          Refresh Data
        </button>
      </div>

      <!-- About -->
      <h2 class="section-title">About</h2>
      <div class="info-row last">
        <span class="info-label">Version</span>
        <span class="info-value">{appVersion}</span>
      </div>
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

  .back-btn:hover {
    background: rgba(255, 255, 255, 0.15);
  }

  .back-btn:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px #e50914;
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

  /* Section titles */
  .section-title {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: #555;
    margin: 24px 0 10px 2px;
  }

  .section-title:first-of-type {
    margin-top: 0;
  }

  /* Info rows */
  .info-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 13px 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }

  .info-row.last {
    border-bottom: none;
  }

  .info-label {
    font-size: 0.95rem;
    color: #999;
  }

  .info-value {
    font-size: 0.95rem;
    color: #fff;
    font-weight: 500;
  }

  /* Server */
  .server-input {
    width: 100%;
    background: rgba(255, 255, 255, 0.07);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 8px;
    padding: 11px 14px;
    color: #fff;
    font-size: 0.9rem;
    font-family: monospace;
    transition: all 0.15s;
    box-sizing: border-box;
    margin-bottom: 8px;
  }

  .server-input:focus {
    outline: none;
    box-shadow: 0 0 0 2px #e50914;
    border-color: #e50914;
  }

  .server-input::placeholder {
    color: #444;
  }

  .server-actions {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
    margin-bottom: 8px;
  }

  .server-btn {
    padding: 8px 16px;
    border-radius: 8px;
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
    white-space: nowrap;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: #888;
  }

  .server-btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.1);
    color: #ccc;
  }

  .server-btn:focus {
    outline: none;
    box-shadow: 0 0 0 2px #e50914;
  }

  .server-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .server-btn.primary {
    color: #ccc;
  }

  .server-btn.accent {
    background: #e50914;
    border-color: #e50914;
    color: #fff;
  }

  .server-btn.accent:hover {
    background: #f6121d;
  }

  .server-status {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 0.8rem;
    font-weight: 500;
    margin-left: auto;
  }

  .server-status svg {
    width: 14px;
    height: 14px;
  }

  .server-status.success {
    color: #4caf50;
  }

  .server-status.error {
    color: #f44336;
  }

  /* Language select */
  .lang-select {
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 6px;
    padding: 8px 12px;
    color: #fff;
    font-size: 0.9rem;
    min-width: 150px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .lang-select:hover {
    background: rgba(255, 255, 255, 0.12);
  }

  .lang-select:focus {
    outline: none;
    box-shadow: 0 0 0 2px #e50914;
    border-color: #e50914;
  }

  .lang-select option {
    background: #1a1a1a;
    color: #fff;
  }

  /* Action buttons row */
  .action-row {
    display: flex;
    gap: 8px;
    margin-top: 12px;
  }

  .action-btn {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 11px 14px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    color: #bbb;
    font-size: 0.85rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s;
  }

  .action-btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.1);
    color: #fff;
  }

  .action-btn:focus {
    outline: none;
    box-shadow: 0 0 0 2px #e50914;
  }

  .action-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .action-btn.danger {
    border-color: rgba(229, 9, 20, 0.2);
    color: #e57373;
  }

  .action-btn.danger:hover:not(:disabled) {
    background: rgba(229, 9, 20, 0.1);
    border-color: rgba(229, 9, 20, 0.35);
  }

  .action-btn svg {
    width: 16px;
    height: 16px;
  }

  .btn-spinner {
    width: 16px;
    height: 16px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: #fff;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  /* Wider screens */
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
      padding: 16px 0;
    }

    .info-label, .info-value {
      font-size: 1.05rem;
    }
  }
</style>
