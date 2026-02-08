<script lang="ts">
  import { goto } from "$app/navigation";
  import { tick } from "svelte";
  import { favoritesStore, type FavoriteCountry } from "$lib/stores/favorites.svelte";
  import { getChannelsByCountry } from "$lib/api/commands";
  import { makeFocusable, setFocus } from "$lib/utils/tvNavigation";
  import type { Channel } from "$lib/api/types";

  let selectedCountry = $state<FavoriteCountry | null>(null);
  let countryChannels = $state<Channel[]>([]);
  let countryLoading = $state(false);
  let focusedIndex = $state(0);

  // Combined flat list: countries + individual channels
  let items = $derived([
    ...favoritesStore.favoriteCountries.map(c => ({ type: 'country' as const, data: c })),
    ...favoritesStore.favoriteChannels.map(ch => ({ type: 'channel' as const, data: ch })),
  ]);

  async function drillIntoCountry(country: FavoriteCountry) {
    selectedCountry = country;
    countryLoading = true;
    try {
      countryChannels = await getChannelsByCountry(country.code, 5000);
    } catch (err) {
      console.error('[LiveTV] Failed to load country channels:', err);
      countryChannels = [];
    } finally {
      countryLoading = false;
      await tick();
      setTimeout(() => { makeFocusable(); setFocus('.livetv-channel-item'); }, 100);
    }
  }

  function goBack() {
    selectedCountry = null;
    countryChannels = [];
    tick().then(() => setTimeout(() => { makeFocusable(); setFocus('.livetv-item'); }, 100));
  }

  function playChannel(channel: Channel, channelList?: Channel[]) {
    if (!channel.stream_url) return;
    const list = channelList || countryChannels;
    const playable = list.filter(ch => ch.stream_url);
    const channelListData = playable.map(ch => ({ name: ch.name, url: ch.stream_url! }));
    const idx = channelListData.findIndex(ch => ch.url === channel.stream_url);
    sessionStorage.setItem('liveChannelList', JSON.stringify(channelListData));
    goto(`/live/play?url=${encodeURIComponent(channel.stream_url)}&title=${encodeURIComponent(channel.name)}&chIdx=${idx}`);
  }

  function playFavoriteChannel(channel: Channel) {
    // When playing from favorites list, use all favorite channels as the list
    const favChannels = favoritesStore.favoriteChannels.filter(ch => ch.stream_url);
    playChannel(channel, favChannels);
  }

  function handleListKeydown(e: KeyboardEvent) {
    const total = items.length;
    if (total === 0) return;

    if (e.key === 'ArrowDown') {
      e.preventDefault();
      focusedIndex = (focusedIndex + 1) % total;
      focusItem(focusedIndex);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      focusedIndex = (focusedIndex - 1 + total) % total;
      focusItem(focusedIndex);
    }
  }

  function handleCountryListKeydown(e: KeyboardEvent) {
    const total = countryChannels.length;
    if (total === 0) return;

    if (e.key === 'ArrowDown') {
      e.preventDefault();
      focusedIndex = (focusedIndex + 1) % total;
      focusCountryItem(focusedIndex);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      focusedIndex = (focusedIndex - 1 + total) % total;
      focusCountryItem(focusedIndex);
    }
  }

  function focusItem(idx: number) {
    const items = document.querySelectorAll('.livetv-item');
    const el = items[idx] as HTMLElement;
    if (el) el.focus();
  }

  function focusCountryItem(idx: number) {
    const items = document.querySelectorAll('.livetv-channel-item');
    const el = items[idx] as HTMLElement;
    if (el) el.focus();
  }
</script>

<div class="livetv-favorites">
  {#if selectedCountry}
    <!-- Country drill-down -->
    <div class="livetv-header">
      <button class="back-btn" onclick={goBack} aria-label="Go back">
        <svg viewBox="0 0 24 24" fill="currentColor">
          <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
        </svg>
      </button>
      <h2>{selectedCountry.flag || ''} {selectedCountry.name}</h2>
      <span class="channel-count">{countryChannels.length} channels</span>
    </div>
    {#if countryLoading}
      <div class="loading-inline">
        <div class="spinner"></div>
        <p>Loading channels...</p>
      </div>
    {:else if countryChannels.length === 0}
      <div class="empty-state">
        <p>No channels found for this country</p>
      </div>
    {:else}
      <div class="livetv-list" onkeydown={handleCountryListKeydown}>
        {#each countryChannels as channel, i (channel.id)}
          <button
            class="livetv-channel-item"
            onclick={() => playChannel(channel)}
            onfocus={() => { focusedIndex = i; }}
          >
            {#if channel.logo}
              <img src={channel.logo} alt="" class="channel-logo" />
            {:else}
              <div class="channel-logo-placeholder">
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M17 10.5V7c0-.55-.45-1-1-1H4c-.55 0-1 .45-1 1v10c0 .55.45 1 1 1h12c.55 0 1-.45 1-1v-3.5l4 4v-11l-4 4z"/>
                </svg>
              </div>
            {/if}
            <span class="channel-name">{channel.name}</span>
            <span class="live-badge">LIVE</span>
            <svg class="play-icon" viewBox="0 0 24 24" fill="currentColor">
              <path d="M8 5v14l11-7z"/>
            </svg>
          </button>
        {/each}
      </div>
    {/if}
  {:else if items.length === 0}
    <!-- Empty state -->
    <div class="empty-state">
      <svg viewBox="0 0 24 24" fill="currentColor" class="empty-icon">
        <path d="M17 10.5V7c0-.55-.45-1-1-1H4c-.55 0-1 .45-1 1v10c0 .55.45 1 1 1h12c.55 0 1-.45 1-1v-3.5l4 4v-11l-4 4z"/>
      </svg>
      <h2>No Live TV Favorites</h2>
      <p>Browse Live Channels to add countries and channels to your favorites.</p>
    </div>
  {:else}
    <!-- Favorites list -->
    <div class="livetv-list" onkeydown={handleListKeydown}>
      {#each items as item, i (item.type === 'country' ? `c-${item.data.code}` : `ch-${item.data.id}`)}
        {#if item.type === 'country'}
          <button
            class="livetv-item livetv-country"
            onclick={() => drillIntoCountry(item.data as FavoriteCountry)}
            onfocus={() => { focusedIndex = i; }}
          >
            <span class="country-flag">{(item.data as FavoriteCountry).flag || ''}</span>
            <span class="item-name">{(item.data as FavoriteCountry).name}</span>
            <svg class="arrow-icon" viewBox="0 0 24 24" fill="currentColor">
              <path d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z"/>
            </svg>
          </button>
        {:else}
          {@const ch = item.data as Channel}
          <button
            class="livetv-item livetv-channel"
            onclick={() => playFavoriteChannel(ch)}
            onfocus={() => { focusedIndex = i; }}
          >
            {#if ch.logo}
              <img src={ch.logo} alt="" class="channel-logo" />
            {:else}
              <div class="channel-logo-placeholder">
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M17 10.5V7c0-.55-.45-1-1-1H4c-.55 0-1 .45-1 1v10c0 .55.45 1 1 1h12c.55 0 1-.45 1-1v-3.5l4 4v-11l-4 4z"/>
                </svg>
              </div>
            {/if}
            <span class="item-name">{ch.name}</span>
            <span class="live-badge">LIVE</span>
            <svg class="play-icon" viewBox="0 0 24 24" fill="currentColor">
              <path d="M8 5v14l11-7z"/>
            </svg>
          </button>
        {/if}
      {/each}
    </div>
  {/if}
</div>

<style>
  .livetv-favorites {
    padding: 20px 40px;
  }

  .livetv-header {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 20px;
  }

  .livetv-header h2 {
    font-size: 1.4rem;
    font-weight: 600;
    margin: 0;
  }

  .channel-count {
    color: #888;
    font-size: 0.85rem;
  }

  .back-btn {
    background: rgba(255, 255, 255, 0.1);
    border: none;
    color: #fff;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.2s;
  }

  .back-btn:hover, .back-btn:focus {
    background: rgba(255, 255, 255, 0.2);
  }

  .back-btn svg {
    width: 20px;
    height: 20px;
  }

  .livetv-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .livetv-item, .livetv-channel-item {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 12px 16px;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid transparent;
    border-radius: 8px;
    color: #fff;
    cursor: pointer;
    transition: all 0.15s ease;
    text-align: left;
    width: 100%;
  }

  .livetv-item:hover, .livetv-channel-item:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .livetv-item:focus, .livetv-channel-item:focus {
    outline: none;
    background: rgba(229, 9, 20, 0.15);
    border-color: #e50914;
    box-shadow: 0 0 0 2px rgba(229, 9, 20, 0.3);
  }

  .country-flag {
    font-size: 1.6rem;
    width: 40px;
    text-align: center;
    flex-shrink: 0;
  }

  .item-name, .channel-name {
    flex: 1;
    font-size: 0.95rem;
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .channel-logo {
    width: 40px;
    height: 40px;
    object-fit: contain;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.05);
    flex-shrink: 0;
  }

  .channel-logo-placeholder {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 6px;
    flex-shrink: 0;
    color: #555;
  }

  .channel-logo-placeholder svg {
    width: 22px;
    height: 22px;
  }

  .live-badge {
    background: #e50914;
    color: #fff;
    font-size: 0.65rem;
    font-weight: 700;
    padding: 2px 6px;
    border-radius: 3px;
    letter-spacing: 1px;
    flex-shrink: 0;
  }

  .arrow-icon {
    width: 20px;
    height: 20px;
    color: #666;
    flex-shrink: 0;
  }

  .play-icon {
    width: 18px;
    height: 18px;
    color: #888;
    flex-shrink: 0;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .livetv-item:hover .play-icon,
  .livetv-item:focus .play-icon,
  .livetv-channel-item:hover .play-icon,
  .livetv-channel-item:focus .play-icon {
    opacity: 1;
    color: #e50914;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 60px 20px;
    color: #888;
    text-align: center;
  }

  .empty-icon {
    width: 64px;
    height: 64px;
    color: #444;
    margin-bottom: 16px;
  }

  .empty-state h2 {
    color: #ccc;
    margin: 0 0 8px;
    font-size: 1.2rem;
  }

  .empty-state p {
    font-size: 0.9rem;
    max-width: 300px;
    margin: 0;
  }

  .loading-inline {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 40px;
    color: #888;
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid rgba(255, 255, 255, 0.1);
    border-top-color: #e50914;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
