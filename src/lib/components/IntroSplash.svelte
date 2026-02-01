<script lang="ts">
  import { onMount } from "svelte";

  interface Props {
    onComplete: () => void;
  }

  let { onComplete }: Props = $props();

  let videoElement: HTMLVideoElement;
  let showSkip = $state(false);
  let isFadingOut = $state(false);

  onMount(() => {
    // Show skip button after 2 seconds
    setTimeout(() => {
      showSkip = true;
    }, 2000);

    // Fallback if video fails to load - wait 10 seconds max
    const fallbackTimer = setTimeout(() => {
      handleComplete();
    }, 10000);

    return () => clearTimeout(fallbackTimer);
  });

  function handleVideoEnd() {
    handleComplete();
  }

  function handleComplete() {
    if (isFadingOut) return;
    isFadingOut = true;
    // Fade out then complete
    setTimeout(() => {
      onComplete();
    }, 500);
  }

  function handleSkip() {
    handleComplete();
  }
</script>

<div class="intro-splash" class:fading={isFadingOut}>
  <video
    bind:this={videoElement}
    autoplay
    muted
    playsinline
    onended={handleVideoEnd}
    class="intro-video"
  >
    <source src="/intro.webm" type="video/webm" />
    <source src="/intro.mp4" type="video/mp4" />
  </video>

  {#if showSkip && !isFadingOut}
    <button class="skip-btn" onclick={handleSkip}>
      Skip
      <svg viewBox="0 0 24 24" fill="currentColor">
        <path d="M5 4.27L18.18 12 5 19.73V4.27M21 12L3 21V3l18 9z"/>
      </svg>
    </button>
  {/if}
</div>

<style>
  .intro-splash {
    position: fixed;
    inset: 0;
    background: #000;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10000;
    opacity: 1;
    transition: opacity 0.5s ease-out;
  }

  .intro-splash.fading {
    opacity: 0;
  }

  .intro-video {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  .skip-btn {
    position: absolute;
    bottom: 40px;
    right: 40px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 24px;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.3);
    border-radius: 6px;
    color: #fff;
    font-size: 1rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    animation: fadeIn 0.3s ease;
  }

  .skip-btn:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  .skip-btn:focus {
    outline: none;
    box-shadow: 0 0 0 3px #e50914;
  }

  .skip-btn svg {
    width: 18px;
    height: 18px;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
