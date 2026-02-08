<script lang="ts">
  import { onMount } from "svelte";

  interface MenuItem {
    label: string;
    action: () => void;
  }

  interface Props {
    visible: boolean;
    x: number;
    y: number;
    items: MenuItem[];
    onclose: () => void;
  }

  let { visible, x, y, items, onclose }: Props = $props();

  let focusedIndex = $state(0);
  let menuEl: HTMLDivElement | undefined = $state();

  $effect(() => {
    if (visible) {
      focusedIndex = 0;
      // Focus the menu after it renders
      setTimeout(() => menuEl?.focus(), 10);
    }
  });

  function handleKeydown(e: KeyboardEvent) {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        e.stopPropagation();
        focusedIndex = (focusedIndex + 1) % items.length;
        break;
      case 'ArrowUp':
        e.preventDefault();
        e.stopPropagation();
        focusedIndex = (focusedIndex - 1 + items.length) % items.length;
        break;
      case 'Enter':
        e.preventDefault();
        e.stopPropagation();
        items[focusedIndex]?.action();
        onclose();
        break;
      case 'Escape':
      case 'Backspace':
        e.preventDefault();
        e.stopPropagation();
        onclose();
        break;
    }
  }

  function handleBackdropClick() {
    onclose();
  }

  // Clamp position so menu doesn't overflow viewport
  let clampedX = $derived(Math.min(x, (typeof window !== 'undefined' ? window.innerWidth : 1920) - 220));
  let clampedY = $derived(Math.min(y, (typeof window !== 'undefined' ? window.innerHeight : 1080) - items.length * 48 - 20));
</script>

{#if visible}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="context-backdrop" onclick={handleBackdropClick} onkeydown={handleKeydown}>
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <div
      class="context-menu"
      style="left: {clampedX}px; top: {clampedY}px"
      bind:this={menuEl}
      tabindex="0"
      role="menu"
    >
      {#each items as item, i (i)}
        <button
          class="context-item"
          class:focused={i === focusedIndex}
          role="menuitem"
          onclick={(e) => { e.stopPropagation(); item.action(); onclose(); }}
        >
          {item.label}
        </button>
      {/each}
    </div>
  </div>
{/if}

<style>
  .context-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    z-index: 9000;
  }

  .context-menu {
    position: absolute;
    min-width: 200px;
    background: #1e1e2e;
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 10px;
    padding: 6px;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.6);
    outline: none;
  }

  .context-item {
    display: block;
    width: 100%;
    padding: 12px 16px;
    background: none;
    border: none;
    border-radius: 6px;
    color: #ddd;
    font-size: 0.9rem;
    font-family: inherit;
    text-align: left;
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .context-item:hover,
  .context-item.focused {
    background: rgba(229, 9, 20, 0.4);
    color: #fff;
  }

  .context-item:focus,
  .context-item:focus-visible {
    outline: none;
    background: rgba(229, 9, 20, 0.4);
    color: #fff;
  }
</style>
