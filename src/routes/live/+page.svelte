<script lang="ts">
  import { onMount, tick } from "svelte";
  import { goto } from "$app/navigation";
  import { browser } from "$app/environment";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import ContextMenu from "$lib/components/ContextMenu.svelte";
  import SearchSection from "$lib/components/SearchSection.svelte";
  import { listChannels, getChannelCountries, getChannelCategories, searchChannels } from "$lib/api/commands";
  import { favoritesStore } from "$lib/stores/favorites.svelte";
  import { makeFocusable, setFocus } from "$lib/utils/tvNavigation";
  import type { Channel } from "$lib/api/types";

  interface IPTVChannel {
    id: string;
    name: string;
    country: string;
    languages: string[];
    categories: string[];
    logo?: string;
    url?: string;
  }
  interface IPTVCountry {
    code: string;
    name: string;
    flag: string;
  }
  interface IPTVCategory {
    id: string;
    name: string;
  }

  let iptvChannels = $state<IPTVChannel[]>([]);
  let iptvCountries = $state<IPTVCountry[]>([]);
  let iptvCategories = $state<IPTVCategory[]>([]);
  let channelGroups = $state<string[]>([]);
  let selectedGroup = $state<string | null>(null);
  let channelsLoading = $state(false);
  let channelsError = $state<string | null>(null);
  let channelGroupBy = $state<'country' | 'category' | 'search'>('country');

  // Channel context menu
  let channelContextMenu = $state({ visible: false, x: 0, y: 0, channel: null as (IPTVChannel | Channel | null) });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' || e.key === 'Backspace' || e.key === 'GoBack') {
      if ((e.target as HTMLElement).tagName === 'INPUT') return;

      // Context menu open → close it
      if (channelContextMenu.visible) {
        e.preventDefault();
        e.stopImmediatePropagation();
        console.log('[Live] Back → close context menu');
        closeChannelContextMenu();
        return;
      }

      // Inside a group → go back to group list
      if (selectedGroup) {
        e.preventDefault();
        e.stopImmediatePropagation();
        console.log('[Live] Back → deselect group');
        deselectGroup();
        return;
      }

      // In search/category mode → go back to country mode
      if (channelGroupBy !== 'country') {
        e.preventDefault();
        e.stopImmediatePropagation();
        console.log('[Live] Back → switch to country mode');
        switchGroupBy('country');
        return;
      }

      // At top level → let layout handle (navigate to /)
      console.log('[Live] Back → letting layout navigate to /');
    }
  }

  onMount(() => {
    loadChannels();

    if (browser) {
      document.addEventListener('keydown', handleKeydown, true);
    }

    let focusTimer = setTimeout(() => {
      makeFocusable();
      setFocus('.genre-card, .country-list-btn');
    }, 300);

    return () => {
      clearTimeout(focusTimer);
      if (browser) {
        document.removeEventListener('keydown', handleKeydown, true);
      }
    };
  });

  async function loadChannels() {
    if (iptvChannels.length > 0) return;
    channelsLoading = true;
    channelsError = null;

    try {
      if (browser) {
        const cached = localStorage.getItem('iptv_data_v5');
        const cacheTime = localStorage.getItem('iptv_data_v5_time');
        const oneHour = 60 * 60 * 1000;
        if (cached && cacheTime && Date.now() - parseInt(cacheTime) < oneHour) {
          const data = JSON.parse(cached);
          iptvChannels = data.channels;
          iptvCountries = data.countries;
          iptvCategories = data.categories;
          updateChannelGroups();
          channelsLoading = false;
          return;
        }
      }

      const [channelsData, countries, categories] = await Promise.all([
        listChannels({ limit: 10000 }),
        getChannelCountries(),
        getChannelCategories()
      ]);

      iptvChannels = (channelsData.channels || []).map(ch => ({
        id: ch.id,
        name: ch.name,
        country: ch.country || '',
        languages: ch.languages || [],
        categories: ch.categories || [],
        logo: ch.logo,
        url: ch.stream_url
      }));
      iptvCountries = countries.map(c => ({ code: c.code, name: c.name, flag: c.flag || '' }));
      iptvCategories = categories.map(c => ({ id: c.id, name: c.name }));
      updateChannelGroups();

      if (browser) {
        localStorage.setItem('iptv_data_v5', JSON.stringify({
          channels: iptvChannels,
          countries: iptvCountries,
          categories: iptvCategories
        }));
        localStorage.setItem('iptv_data_v5_time', Date.now().toString());
      }
    } catch (err) {
      console.error('[IPTV] Failed to load channels:', err);
      channelsError = 'Failed to load channels. Is the server running?';
    } finally {
      channelsLoading = false;
    }
  }

  function updateChannelGroups() {
    if (channelGroupBy === 'country') {
      const countrySet = new Set(iptvChannels.map(ch => ch.country));
      const countriesWithChannels = iptvCountries.filter(c => countrySet.has(c.code));
      channelGroups = countriesWithChannels.map(c => c.code).sort((a, b) => {
        const nameA = iptvCountries.find(c => c.code === a)?.name || a;
        const nameB = iptvCountries.find(c => c.code === b)?.name || b;
        return nameA.localeCompare(nameB);
      });
    } else {
      const catSet = new Set(iptvChannels.flatMap(ch => ch.categories));
      channelGroups = iptvCategories
        .filter(c => catSet.has(c.id))
        .map(c => c.id)
        .sort((a, b) => {
          const nameA = iptvCategories.find(c => c.id === a)?.name || a;
          const nameB = iptvCategories.find(c => c.id === b)?.name || b;
          return nameA.localeCompare(nameB);
        });
    }
  }

  function getGroupName(code: string): string {
    if (channelGroupBy === 'country') {
      const country = iptvCountries.find(c => c.code === code);
      return country ? `${country.flag} ${country.name}` : code;
    } else {
      const cat = iptvCategories.find(c => c.id === code);
      return cat?.name || code;
    }
  }

  function getChannelsInGroup(groupCode: string): IPTVChannel[] {
    if (channelGroupBy === 'country') {
      return iptvChannels.filter(ch => ch.country === groupCode);
    } else {
      return iptvChannels.filter(ch => ch.categories.includes(groupCode));
    }
  }

  function switchGroupBy(mode: 'country' | 'category' | 'search') {
    channelGroupBy = mode;
    selectedGroup = null;
    if (mode !== 'search') {
      updateChannelGroups();
      tick().then(() => setTimeout(() => { makeFocusable(); setFocus('.country-list-btn'); }, 100));
    }
  }

  async function selectGroup(group: string) {
    selectedGroup = group;
    await tick();
    setTimeout(() => { makeFocusable(); setFocus('.channel-list-item'); }, 100);
  }

  async function deselectGroup() {
    selectedGroup = null;
    await tick();
    setTimeout(() => { makeFocusable(); setFocus('.country-list-btn'); }, 100);
  }

  function playChannel(channel: IPTVChannel) {
    if (channel.url) {
      const group = selectedGroup;
      const channels = group ? getChannelsInGroup(group) : iptvChannels;
      const channelList = channels.filter(ch => ch.url).map(ch => ({ name: ch.name, url: ch.url! }));
      const idx = channelList.findIndex(ch => ch.url === channel.url);
      sessionStorage.setItem('liveChannelList', JSON.stringify(channelList));
      goto(`/live/play?url=${encodeURIComponent(channel.url)}&title=${encodeURIComponent(channel.name)}&chIdx=${idx}`);
    }
  }

  function showChannelContextMenu(target: HTMLElement, channel: IPTVChannel | Channel) {
    const rect = target.getBoundingClientRect();
    channelContextMenu = {
      visible: true,
      x: rect.left + rect.width / 2 - 100,
      y: rect.top - 10,
      channel,
    };
  }

  function closeChannelContextMenu() {
    channelContextMenu = { visible: false, x: 0, y: 0, channel: null };
  }

  function getChannelContextMenuItems(channel: IPTVChannel | Channel) {
    const ch: Channel = {
      id: 'id' in channel ? channel.id : '',
      name: channel.name,
      logo: channel.logo,
      stream_url: 'stream_url' in channel ? channel.stream_url : ('url' in channel ? (channel as IPTVChannel).url : undefined),
      country: channel.country,
      categories: channel.categories,
      languages: channel.languages,
    };
    const isFav = favoritesStore.isChannelFavorite(ch.id);
    return [{
      label: isFav ? 'Remove from My List' : 'Add to My List',
      action: () => { favoritesStore.toggleChannel(ch); },
    }];
  }
</script>

<svelte:head>
  <title>Live Channels - Omnius</title>
</svelte:head>

<div class="app">
  <Sidebar />

  <main class="main-content">
    <div class="category-view">
      <div class="live-header">
        <h1 class="category-title">Live Channels</h1>
        {#if iptvChannels.length > 0 && !selectedGroup}
          <div class="group-toggle">
            <button
              class="toggle-btn"
              class:active={channelGroupBy === 'country'}
              onclick={() => switchGroupBy('country')}
            >
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z"/>
              </svg>
              Countries
            </button>
            <button
              class="toggle-btn"
              class:active={channelGroupBy === 'category'}
              onclick={() => switchGroupBy('category')}
            >
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M4 8h4V4H4v4zm6 12h4v-4h-4v4zm-6 0h4v-4H4v4zm0-6h4v-4H4v4zm6 0h4v-4h-4v4zm6-10v4h4V4h-4zm-6 4h4V4h-4v4zm6 6h4v-4h-4v4zm0 6h4v-4h-4v4z"/>
              </svg>
              Categories
            </button>
            <button
              class="toggle-btn"
              class:active={channelGroupBy === 'search'}
              onclick={() => switchGroupBy('search')}
            >
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
              </svg>
              Search
            </button>
          </div>
        {/if}
      </div>

      {#if channelsLoading}
        <div class="loading-inline">
          <div class="spinner"></div>
          <p>Loading channels...</p>
        </div>
      {:else if channelsError}
        <div class="error-inline">
          <p>{channelsError}</p>
          <button class="btn-retry" onclick={loadChannels}>Retry</button>
        </div>
      {:else if channelGroupBy === 'search'}
        <SearchSection type="channels" channels={iptvChannels.map(ch => ({ id: ch.id, name: ch.name, logo: ch.logo, stream_url: ch.url, country: ch.country, categories: ch.categories, languages: ch.languages }))} />
      {:else if selectedGroup}
        <div class="genre-header">
          <button class="back-btn" onclick={() => deselectGroup()} aria-label="Go back">
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
            </svg>
          </button>
          <h2>{getGroupName(selectedGroup)}</h2>
          <span class="group-channel-count">{getChannelsInGroup(selectedGroup).length} channels</span>
        </div>
        <div class="channels-list">
          {#each getChannelsInGroup(selectedGroup) as channel (channel.id)}
            <div class="channel-list-row">
              <button class="channel-list-item" onclick={() => playChannel(channel)}>
                {#if channel.logo}
                  <img src={channel.logo} alt="" class="channel-list-logo" />
                {:else}
                  <div class="channel-list-logo-placeholder">
                    <svg viewBox="0 0 24 24" fill="currentColor">
                      <path d="M17 10.5V7c0-.55-.45-1-1-1H4c-.55 0-1 .45-1 1v10c0 .55.45 1 1 1h12c.55 0 1-.45 1-1v-3.5l4 4v-11l-4 4z"/>
                    </svg>
                  </div>
                {/if}
                <span class="channel-list-name">{channel.name}</span>
                <svg class="channel-play-icon" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M8 5v14l11-7z"/>
                </svg>
              </button>
              <button
                class="channel-menu-btn"
                onclick={(e) => { e.stopPropagation(); showChannelContextMenu(e.currentTarget as HTMLElement, channel); }}
                aria-label="More options"
              >
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <circle cx="12" cy="5" r="2"/><circle cx="12" cy="12" r="2"/><circle cx="12" cy="19" r="2"/>
                </svg>
              </button>
            </div>
          {/each}
        </div>
      {:else}
        <p class="channels-info">{iptvChannels.length} channels available</p>
        <div class="country-list">
          {#each channelGroups as group (group)}
            <button class="country-list-btn" onclick={() => selectGroup(group)}>
              <span class="country-name">{getGroupName(group)}</span>
              <span class="group-count">({getChannelsInGroup(group).length})</span>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </main>
</div>

{#if channelContextMenu.visible && channelContextMenu.channel}
  <ContextMenu
    visible={channelContextMenu.visible}
    x={channelContextMenu.x}
    y={channelContextMenu.y}
    items={getChannelContextMenuItems(channelContextMenu.channel)}
    onclose={closeChannelContextMenu}
  />
{/if}

<style>
  .app {
    display: flex;
    height: 100vh;
    height: 100dvh;
    background: #141414;
    color: #fff;
    overflow: hidden;
  }

  .main-content {
    flex: 1;
    margin-left: 70px;
    height: 100vh;
    height: 100dvh;
    overflow-y: auto;
    overflow-x: hidden;
    scroll-behavior: smooth;
  }

  .category-view {
    padding: 20px 40px;
  }

  .live-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
    flex-wrap: wrap;
    gap: 10px;
  }

  .live-header .category-title {
    margin: 0;
  }

  .category-title {
    font-size: 1.3rem;
    font-weight: 600;
    margin: 0 0 16px;
  }

  .group-toggle {
    display: flex;
    gap: 8px;
    background: rgba(255, 255, 255, 0.05);
    padding: 4px;
    border-radius: 8px;
  }

  .toggle-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background: transparent;
    border: none;
    border-radius: 6px;
    color: #888;
    font-size: 0.8rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .toggle-btn svg {
    width: 18px;
    height: 18px;
  }

  .toggle-btn:hover {
    color: #fff;
    background: rgba(255, 255, 255, 0.1);
  }

  .toggle-btn.active {
    color: #fff;
    background: #e50914;
  }

  .toggle-btn:focus,
  .toggle-btn:focus-visible {
    outline: none;
    box-shadow: 0 0 0 3px rgba(229, 9, 20, 0.5);
  }

  .channels-info {
    color: #888;
    margin-bottom: 12px;
    font-size: 0.85rem;
  }

  .country-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-width: 600px;
  }

  .country-list-btn {
    flex: 1;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 16px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    color: #e0e0e0;
    font-size: 0.9rem;
    font-weight: 500;
    font-family: inherit;
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: left;
  }

  .country-list-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.15);
  }

  .country-list-btn:focus,
  .country-list-btn:focus-visible {
    outline: none;
    border-color: #e50914;
    box-shadow: 0 0 0 3px rgba(229, 9, 20, 0.5);
    background: rgba(255, 255, 255, 0.1);
  }

  .country-name {
    flex: 1;
  }

  .group-count {
    font-size: 0.8rem;
    color: #666;
    margin-left: 8px;
  }

  .genre-header {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 16px;
  }

  .genre-header h2 {
    font-size: 1.3rem;
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
    transition: background 0.2s ease;
  }

  .back-btn:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  .back-btn:focus {
    outline: none;
    box-shadow: 0 0 0 3px rgba(255, 255, 255, 0.2);
  }

  .back-btn svg {
    width: 24px;
    height: 24px;
  }

  .group-channel-count {
    font-size: 0.9rem;
    color: #888;
    margin-left: auto;
  }

  .channels-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: 20px;
  }

  .channel-list-row {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .channel-list-row .channel-list-item {
    flex: 1;
  }

  .channel-list-item {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 12px 16px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid transparent;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.15s ease;
    text-align: left;
  }

  .channel-list-item:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .channel-list-item:focus,
  .channel-list-item:focus-visible {
    outline: none;
    background: rgba(229, 9, 20, 0.2);
    border-color: #e50914;
  }

  .channel-list-logo {
    width: 48px;
    height: 48px;
    object-fit: contain;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    flex-shrink: 0;
  }

  .channel-list-logo-placeholder {
    width: 48px;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #666;
    flex-shrink: 0;
  }

  .channel-list-logo-placeholder svg {
    width: 24px;
    height: 24px;
  }

  .channel-list-name {
    flex: 1;
    font-size: 1rem;
    font-weight: 500;
    color: #fff;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .channel-play-icon {
    width: 24px;
    height: 24px;
    color: #888;
    flex-shrink: 0;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .channel-list-item:hover .channel-play-icon,
  .channel-list-item:focus .channel-play-icon {
    opacity: 1;
    color: #e50914;
  }

  .channel-menu-btn {
    flex-shrink: 0;
    width: 36px;
    height: 36px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 50%;
    color: rgba(255, 255, 255, 0.5);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    transition: all 0.2s ease;
  }

  .channel-menu-btn:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #fff;
  }

  .channel-menu-btn:focus,
  .channel-menu-btn:focus-visible {
    outline: none;
    border-color: #e50914;
    box-shadow: 0 0 0 3px rgba(229, 9, 20, 0.5);
    color: #fff;
  }

  .channel-menu-btn svg {
    width: 18px;
    height: 18px;
  }

  .loading-inline, .error-inline {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 30px 20px;
    color: #888;
  }

  .btn-retry {
    padding: 10px 24px;
    background: #e50914;
    color: #fff;
    border: none;
    border-radius: 6px;
    font-size: 0.9rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn-retry:hover {
    background: #f40d17;
  }

  .btn-retry:focus,
  .btn-retry:focus-visible {
    outline: none;
    box-shadow: 0 0 0 4px rgba(229, 9, 20, 0.5);
  }

  .spinner {
    width: 60px;
    height: 60px;
    border: 4px solid rgba(229, 9, 20, 0.2);
    border-top-color: #e50914;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  @media (max-width: 900px) {
    .main-content {
      margin-left: 60px;
    }
    .category-view {
      padding: 16px 32px;
    }
  }

  @media (max-width: 600px) {
    .main-content {
      margin-left: 0;
    }
    .category-view {
      padding: 16px 20px;
    }
  }
</style>
