<script lang="ts">
  interface Props {
    value?: string;
    placeholder?: string;
    onsearch?: (query: string) => void;
    onclear?: () => void;
  }

  let {
    value = "",
    placeholder = "Search movies...",
    onsearch,
    onclear,
  }: Props = $props();

  let inputValue = $state(value);
  let inputElement: HTMLInputElement;
  let isFocused = $state(false);

  // Sync with external value
  $effect(() => {
    inputValue = value;
  });

  function handleSubmit(event: Event) {
    event.preventDefault();
    onsearch?.(inputValue.trim());
  }

  function handleClear() {
    inputValue = "";
    onclear?.();
    inputElement?.focus();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      handleClear();
    }
  }
</script>

<form class="search-bar" class:focused={isFocused} onsubmit={handleSubmit}>
  <div class="search-icon-container">
    <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor">
      <circle cx="11" cy="11" r="8" />
      <path d="m21 21-4.35-4.35" />
    </svg>
  </div>

  <input
    bind:this={inputElement}
    bind:value={inputValue}
    type="text"
    {placeholder}
    class="search-input"
    onkeydown={handleKeydown}
    onfocus={() => (isFocused = true)}
    onblur={() => (isFocused = false)}
  />

  {#if inputValue}
    <button type="button" class="clear-button" onclick={handleClear} aria-label="Clear search">
      <svg viewBox="0 0 24 24" fill="currentColor">
        <path
          d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"
        />
      </svg>
    </button>
  {/if}

  <button type="submit" class="search-button" aria-label="Search">
    <span>Search</span>
  </button>
</form>

<style>
  .search-bar {
    display: flex;
    align-items: center;
    gap: 12px;
    background: #1a1a2e;
    border: 2px solid #2a2a4e;
    border-radius: 12px;
    padding: 8px 16px;
    transition:
      border-color 0.2s ease,
      box-shadow 0.2s ease;
    max-width: 600px;
    width: 100%;
  }

  .search-bar:hover {
    border-color: #3a3a6e;
  }

  .search-bar.focused {
    border-color: #e94560;
    box-shadow: 0 0 0 3px rgba(233, 69, 96, 0.2);
  }

  .search-icon-container {
    flex-shrink: 0;
  }

  .search-icon {
    width: 24px;
    height: 24px;
    stroke-width: 2.5;
    color: #888;
  }

  .search-input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    font-size: 1.2rem;
    color: #ffffff;
    padding: 8px 0;
    min-width: 0;
  }

  .search-input::placeholder {
    color: #666;
  }

  .clear-button {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    background: rgba(255, 255, 255, 0.1);
    border: none;
    border-radius: 50%;
    cursor: pointer;
    transition:
      background 0.2s ease,
      transform 0.2s ease;
  }

  .clear-button:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  .clear-button:focus {
    outline: 2px solid #e94560;
    outline-offset: 2px;
  }

  .clear-button svg {
    width: 18px;
    height: 18px;
    color: #888;
  }

  .search-button {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 10px 24px;
    background: #e94560;
    border: none;
    border-radius: 8px;
    font-size: 1.1rem;
    font-weight: 600;
    color: #ffffff;
    cursor: pointer;
    transition:
      background 0.2s ease,
      transform 0.2s ease;
  }

  .search-button:hover {
    background: #d63350;
    transform: translateY(-1px);
  }

  .search-button:focus {
    outline: 2px solid #ffffff;
    outline-offset: 2px;
  }

  .search-button:active {
    transform: translateY(0);
  }

  /* TV-optimized styles */
  @media (min-width: 1920px) {
    .search-bar {
      padding: 12px 24px;
      gap: 16px;
      border-radius: 16px;
      max-width: 800px;
    }

    .search-icon {
      width: 32px;
      height: 32px;
    }

    .search-input {
      font-size: 1.5rem;
      padding: 12px 0;
    }

    .clear-button {
      width: 44px;
      height: 44px;
    }

    .clear-button svg {
      width: 24px;
      height: 24px;
    }

    .search-button {
      padding: 14px 32px;
      font-size: 1.3rem;
      border-radius: 10px;
    }
  }
</style>
