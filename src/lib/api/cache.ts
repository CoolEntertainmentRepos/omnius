// API Response Caching with TTL and validation
import { browser } from "$app/environment";

interface CacheEntry<T> {
  data: T;
  timestamp: number;
  etag?: string;
  hash?: string;  // Simple hash of the data for comparison
}

interface CacheConfig {
  ttl: number;  // Time to live in milliseconds
  key: string;
}

// Cache TTLs
export const CACHE_TTL = {
  MOVIES_LIST: 30 * 60 * 1000,      // 30 minutes for movie lists
  MOVIE_DETAILS: 60 * 60 * 1000,    // 1 hour for movie details
  SUGGESTIONS: 60 * 60 * 1000,      // 1 hour for suggestions
  HOME_DATA: 15 * 60 * 1000,        // 15 minutes for home page data
  IPTV: 24 * 60 * 60 * 1000,        // 24 hours for IPTV data
};

// Simple hash function for data comparison
function simpleHash(data: unknown): string {
  const str = JSON.stringify(data);
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash;
  }
  return hash.toString(36);
}

// Get cached data if valid
export function getCache<T>(key: string, ttl: number): T | null {
  if (!browser) return null;

  try {
    const raw = localStorage.getItem(`cache_${key}`);
    if (!raw) return null;

    const entry: CacheEntry<T> = JSON.parse(raw);
    const age = Date.now() - entry.timestamp;

    if (age < ttl) {
      console.log(`[Cache] HIT: ${key} (age: ${Math.round(age / 1000)}s)`);
      return entry.data;
    }

    console.log(`[Cache] EXPIRED: ${key} (age: ${Math.round(age / 1000)}s, ttl: ${ttl / 1000}s)`);
    return null;
  } catch (e) {
    console.error(`[Cache] Error reading ${key}:`, e);
    return null;
  }
}

// Set cache data
export function setCache<T>(key: string, data: T): void {
  if (!browser) return;

  try {
    const entry: CacheEntry<T> = {
      data,
      timestamp: Date.now(),
      hash: simpleHash(data),
    };
    localStorage.setItem(`cache_${key}`, JSON.stringify(entry));
    console.log(`[Cache] SET: ${key}`);
  } catch (e) {
    console.error(`[Cache] Error writing ${key}:`, e);
    // If storage is full, clear old caches
    clearOldCaches();
  }
}

// Get cache entry with metadata (for validation)
export function getCacheEntry<T>(key: string): CacheEntry<T> | null {
  if (!browser) return null;

  try {
    const raw = localStorage.getItem(`cache_${key}`);
    if (!raw) return null;
    return JSON.parse(raw);
  } catch {
    return null;
  }
}

// Check if data has changed (compare hashes)
export function hasDataChanged<T>(key: string, newData: T): boolean {
  const entry = getCacheEntry<T>(key);
  if (!entry) return true;

  const newHash = simpleHash(newData);
  return entry.hash !== newHash;
}

// Clear specific cache
export function clearCache(key: string): void {
  if (!browser) return;
  localStorage.removeItem(`cache_${key}`);
}

// Clear all API caches
export function clearAllCaches(): void {
  if (!browser) return;

  const keys = Object.keys(localStorage);
  let count = 0;
  for (const key of keys) {
    if (key.startsWith('cache_')) {
      localStorage.removeItem(key);
      count++;
    }
  }
  console.log(`[Cache] Cleared ${count} cache entries`);
}

// Clear old/expired caches to free up space
export function clearOldCaches(): void {
  if (!browser) return;

  const keys = Object.keys(localStorage);
  const now = Date.now();
  const maxAge = 24 * 60 * 60 * 1000; // 24 hours max

  for (const key of keys) {
    if (key.startsWith('cache_')) {
      try {
        const raw = localStorage.getItem(key);
        if (raw) {
          const entry = JSON.parse(raw);
          if (now - entry.timestamp > maxAge) {
            localStorage.removeItem(key);
            console.log(`[Cache] Removed old cache: ${key}`);
          }
        }
      } catch {
        localStorage.removeItem(key);
      }
    }
  }
}

// Wrapper for cached API calls
export async function cachedFetch<T>(
  key: string,
  ttl: number,
  fetchFn: () => Promise<T>,
  forceRefresh: boolean = false
): Promise<T> {
  // Check cache first (unless force refresh)
  if (!forceRefresh) {
    const cached = getCache<T>(key, ttl);
    if (cached !== null) {
      return cached;
    }
  }

  // Fetch fresh data
  console.log(`[Cache] MISS: ${key} - fetching...`);
  const data = await fetchFn();

  // Store in cache
  setCache(key, data);

  return data;
}

// Generate cache key from params
export function makeCacheKey(prefix: string, params: Record<string, unknown>): string {
  const sortedParams = Object.keys(params)
    .sort()
    .filter(k => params[k] !== undefined && params[k] !== null)
    .map(k => `${k}=${params[k]}`)
    .join('&');
  return `${prefix}_${sortedParams || 'default'}`;
}
