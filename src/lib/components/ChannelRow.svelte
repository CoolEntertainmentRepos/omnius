<script lang="ts">
  import type { Channel } from "$lib/api/types";
  import { goto } from "$app/navigation";

  interface Props {
    title: string;
    channels: Channel[];
    loading?: boolean;
  }

  let { title, channels, loading = false }: Props = $props();

  let scrollContainer: HTMLDivElement;
  let showLeftArrow = $state(false);
  let showRightArrow = $state(true);

  function handleScroll() {
    if (!scrollContainer) return;
    showLeftArrow = scrollContainer.scrollLeft > 0;
    showRightArrow =
      scrollContainer.scrollLeft <
      scrollContainer.scrollWidth - scrollContainer.clientWidth - 10;
  }

  function scrollLeft() {
    scrollContainer?.scrollBy({ left: -650, behavior: "smooth" });
  }

  function scrollRight() {
    scrollContainer?.scrollBy({ left: 650, behavior: "smooth" });
  }

  function handleChannelClick(channel: Channel) {
    if (channel.stream_url) {
      goto(`/live?url=${encodeURIComponent(channel.stream_url)}&title=${encodeURIComponent(channel.name)}`);
    }
  }

  function handleKeydown(event: KeyboardEvent, channel: Channel) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      handleChannelClick(channel);
    }
  }
</script>

<section class="channel-row">
  <h2 class="row-title">{title}</h2>

  <div class="row-container">
    {#if showLeftArrow}
      <button class="scroll-btn scroll-left" onclick={scrollLeft} tabindex="-1" aria-label="Scroll left">
        <svg viewBox="0 0 24 24" fill="currentColor">
          <path d="M15.41 7.41L14 6l-6 6 6 6 1.41-1.41L10.83 12z"/>
        </svg>
      </button>
    {/if}

    <div
      class="channel-scroll"
      bind:this={scrollContainer}
      onscroll={handleScroll}
    >
      {#if loading}
        {#each Array(6) as _, i (i)}
          <div class="channel-card-skeleton">
            <div class="skeleton-logo"></div>
            <div class="skeleton-text"></div>
          </div>
        {/each}
      {:else}
        {#each channels as channel (channel.id)}
          <article
            class="channel-card"
            tabindex="0"
            role="button"
            aria-label="Watch {channel.name}"
            onclick={() => handleChannelClick(channel)}
            onkeydown={(e) => handleKeydown(e, channel)}
          >
            <div class="logo-container">
              {#if channel.logo}
                <img
                  src={channel.logo}
                  alt="{channel.name} logo"
                  class="channel-logo"
                  loading="lazy"
                  onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }}
                />
              {:else}
                <div class="channel-placeholder">
                  <svg viewBox="0 0 24 24" fill="currentColor">
                    <path d="M21 6h-7.59l3.29-3.29L16 2l-4 4-4-4-.71.71L10.59 6H3c-1.1 0-2 .89-2 2v12c0 1.1.9 2 2 2h18c1.1 0 2-.9 2-2V8c0-1.11-.9-2-2-2zm0 14H3V8h18v12z"/>
                  </svg>
                </div>
              {/if}
            </div>
            <div class="channel-info">
              <h3 class="channel-name">{channel.name}</h3>
              {#if channel.country}
                <span class="country-badge">{channel.country}</span>
              {/if}
            </div>
            {#if channel.stream_url}
              <div class="live-indicator">
                <span class="live-dot"></span>
                LIVE
              </div>
            {/if}
          </article>
        {/each}
      {/if}
    </div>

    {#if showRightArrow && channels.length > 4}
      <button class="scroll-btn scroll-right" onclick={scrollRight} tabindex="-1" aria-label="Scroll right">
        <svg viewBox="0 0 24 24" fill="currentColor">
          <path d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z"/>
        </svg>
      </button>
    {/if}
  </div>
</section>

<style>
  .channel-row {
    margin-bottom: 20px;
  }

  .row-title {
    font-size: 1.1rem;
    font-weight: 600;
    margin: 0 0 8px;
    padding: 0 40px;
    color: #fff;
  }

  .row-container {
    position: relative;
  }

  .channel-scroll {
    display: flex;
    gap: 10px;
    overflow-x: auto;
    scroll-behavior: smooth;
    padding: 8px 40px 12px;
    scrollbar-width: none;
  }

  .channel-scroll::-webkit-scrollbar {
    display: none;
  }

  .channel-card {
    flex-shrink: 0;
    width: 180px;
    background: #1e1e1e;
    border-radius: 8px;
    overflow: hidden;
    cursor: pointer;
    transition: transform 0.3s ease, box-shadow 0.3s ease;
    position: relative;
  }

  .channel-card:hover,
  .channel-card:focus-visible {
    transform: scale(1.05);
    box-shadow: 0 8px 24px rgba(229, 9, 20, 0.4);
    outline: none;
  }

  .channel-card:focus-visible {
    box-shadow: 0 0 0 3px #e50914, 0 8px 24px rgba(229, 9, 20, 0.5);
  }

  .logo-container {
    width: 100%;
    aspect-ratio: 16 / 9;
    background: #2a2a2a;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
  }

  .channel-logo {
    max-width: 80%;
    max-height: 80%;
    object-fit: contain;
  }

  .channel-placeholder svg {
    width: 40px;
    height: 40px;
    color: #555;
  }

  .channel-info {
    padding: 8px 10px;
  }

  .channel-name {
    font-size: 0.8rem;
    font-weight: 600;
    color: #fff;
    margin: 0;
    line-height: 1.2;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .country-badge {
    display: inline-block;
    margin-top: 4px;
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 0.65rem;
    font-weight: 600;
    background: rgba(255, 255, 255, 0.1);
    color: #aaa;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .live-indicator {
    position: absolute;
    top: 6px;
    right: 6px;
    display: flex;
    align-items: center;
    gap: 4px;
    background: rgba(229, 9, 20, 0.9);
    padding: 3px 8px;
    border-radius: 4px;
    font-size: 0.6rem;
    font-weight: 700;
    color: #fff;
    letter-spacing: 0.5px;
  }

  .live-dot {
    width: 6px;
    height: 6px;
    background: #fff;
    border-radius: 50%;
    animation: pulse 1.5s infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }

  .scroll-btn {
    position: absolute;
    top: 10px;
    bottom: 30px;
    width: 70px;
    background: rgba(20, 20, 20, 0.8);
    border: none;
    color: #fff;
    cursor: pointer;
    z-index: 10;
    opacity: 0;
    transition: opacity 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .row-container:hover .scroll-btn {
    opacity: 1;
  }

  .scroll-btn:hover {
    background: rgba(20, 20, 20, 0.95);
  }

  .scroll-btn:focus {
    outline: none;
    opacity: 1;
    background: rgba(20, 20, 20, 0.95);
  }

  .scroll-btn svg {
    width: 48px;
    height: 48px;
  }

  .scroll-left {
    left: 0;
  }

  .scroll-right {
    right: 0;
  }

  /* Skeleton loading */
  .channel-card-skeleton {
    flex-shrink: 0;
    width: 180px;
    background: #1e1e1e;
    border-radius: 8px;
    overflow: hidden;
  }

  .skeleton-logo {
    width: 100%;
    aspect-ratio: 16 / 9;
    background: linear-gradient(90deg, #2a2a2a 25%, #3a3a3a 50%, #2a2a2a 75%);
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
  }

  .skeleton-text {
    height: 14px;
    margin: 10px;
    border-radius: 4px;
    background: linear-gradient(90deg, #2a2a2a 25%, #3a3a3a 50%, #2a2a2a 75%);
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
  }

  @keyframes shimmer {
    0% { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }

  @media (max-width: 1200px) {
    .row-title {
      padding: 0 40px;
      font-size: 1rem;
    }

    .channel-scroll {
      padding: 8px 40px 12px;
    }

    .scroll-btn {
      width: 50px;
    }

    .scroll-btn svg {
      width: 36px;
      height: 36px;
    }
  }

  @media (max-width: 900px) {
    .row-title {
      padding: 0 32px;
      font-size: 0.95rem;
    }

    .channel-scroll {
      padding: 6px 32px 10px;
      gap: 8px;
    }

    .channel-card {
      width: 160px;
    }

    .scroll-btn {
      width: 40px;
    }

    .scroll-btn svg {
      width: 28px;
      height: 28px;
    }
  }

  @media (max-width: 600px) {
    .row-title {
      padding: 0 20px;
      font-size: 0.9rem;
    }

    .channel-scroll {
      padding: 6px 20px 10px;
      gap: 8px;
    }

    .channel-card {
      width: 140px;
    }

    .scroll-btn {
      display: none;
    }
  }
</style>
