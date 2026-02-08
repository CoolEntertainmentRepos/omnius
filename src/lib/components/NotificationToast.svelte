<script lang="ts">
  import { goto } from "$app/navigation";

  interface Notification {
    id: string;
    title: string;
    message: string;
    movieId?: number;
    poster?: string;
  }

  let notifications = $state<Notification[]>([]);

  export function show(title: string, message: string, movieId?: number, poster?: string) {
    const id = Math.random().toString(36).substring(7);
    notifications = [...notifications, { id, title, message, movieId, poster }];

    // Auto-dismiss after 8 seconds
    setTimeout(() => {
      dismiss(id);
    }, 8000);
  }

  function dismiss(id: string) {
    notifications = notifications.filter(n => n.id !== id);
  }

  function handleClick(notification: Notification) {
    if (notification.movieId) {
      goto(`/movie/${notification.movieId}`);
    }
    dismiss(notification.id);
  }
</script>

<div class="toast-container">
  {#each notifications as notification (notification.id)}
    <div class="toast" onclick={() => handleClick(notification)}>
      {#if notification.poster}
        <img src={notification.poster} alt="" class="toast-poster" />
      {/if}
      <div class="toast-content">
        <div class="toast-title">{notification.title}</div>
        <div class="toast-message">{notification.message}</div>
      </div>
      <button class="toast-close" onclick={(e) => { e.stopPropagation(); dismiss(notification.id); }}>
        <svg viewBox="0 0 24 24" fill="currentColor">
          <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
        </svg>
      </button>
    </div>
  {/each}
</div>

<style>
  .toast-container {
    position: fixed;
    bottom: 24px;
    right: 24px;
    z-index: 9999;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .toast {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px;
    background: rgba(30, 30, 30, 0.95);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
    cursor: pointer;
    max-width: 400px;
    animation: slideIn 0.3s ease;
  }

  @keyframes slideIn {
    from {
      transform: translateX(100%);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }

  .toast:hover {
    background: rgba(40, 40, 40, 0.95);
    border-color: #e50914;
  }

  .toast-poster {
    width: 48px;
    height: 72px;
    object-fit: cover;
    border-radius: 6px;
  }

  .toast-content {
    flex: 1;
  }

  .toast-title {
    font-weight: 600;
    color: #fff;
    margin-bottom: 4px;
  }

  .toast-message {
    font-size: 14px;
    color: #aaa;
  }

  .toast-close {
    padding: 4px;
    background: none;
    border: none;
    color: #666;
    cursor: pointer;
  }

  .toast-close:hover {
    color: #fff;
  }

  .toast-close svg {
    width: 20px;
    height: 20px;
  }
</style>
