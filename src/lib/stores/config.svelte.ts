// Svelte 5 store for server configuration (enabled services)
import { browser } from "$app/environment";
import { getApiUrl } from "$lib/api/commands";

export interface ServiceConfig {
  id: string;
  label: string;
  enabled: boolean;
  icon: string;
  display_order: number;
}

class ConfigStore {
  services = $state<ServiceConfig[]>([]);
  loaded = $state(false);
  error = $state<string | null>(null);
  serverConfigured = $state(true);

  constructor() {
    if (browser) {
      // Check if server URL exists
      const url = localStorage.getItem('omnius_server_url');
      this.serverConfigured = !!url;
    }
  }

  async fetchConfig() {
    try {
      const response = await fetch(`${getApiUrl()}/api/v2/config.json`);
      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      const data = await response.json();
      this.services = data.data?.services || [];
      this.loaded = true;
      this.error = null;
      this.serverConfigured = true;
    } catch (err) {
      console.error("[Config] Failed to fetch config:", err);
      this.error = (err as Error).message;
      // Default to all enabled if server unreachable
      this.services = [
        { id: "movies", label: "Movies", enabled: true, icon: "movie", display_order: 1 },
        { id: "series", label: "TV Shows", enabled: true, icon: "tv", display_order: 2 },
        { id: "channels", label: "Live TV", enabled: false, icon: "live", display_order: 3 },
      ];
      this.loaded = true;
    }
  }

  isEnabled(serviceId: string): boolean {
    const service = this.services.find(s => s.id === serviceId);
    return service?.enabled ?? false;
  }

  getEnabledServices(): ServiceConfig[] {
    return this.services.filter(s => s.enabled).sort((a, b) => a.display_order - b.display_order);
  }

  getLabel(serviceId: string): string {
    const service = this.services.find(s => s.id === serviceId);
    return service?.label ?? serviceId;
  }
}

export const configStore = new ConfigStore();
