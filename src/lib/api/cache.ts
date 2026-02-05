// API Response Caching with TTL and validation
import { browser, dev } from "$app/environment";

// Disable cache in dev mode
const CACHE_DISABLED = dev;

// Cache version - increment this when data source changes
const CACHE_VERSION = 4;  // v4: franchise data updated

interface CacheEntry<T> {
  data: T;
  timestamp: number;
  version?: number;  // Cache version for invalidation
  etag?: string;
  hash?: string;  // Simple hash of the data for comparison
}

interface CacheConfig {
  ttl: number;  // Time to live in milliseconds
  key: string;
}

// Cache TTLs
export const CACHE_TTL = {
  MOVIES_LIST: 15 * 60 * 1000,      // 15 minutes for movie lists
  MOVIE_DETAILS: 5 * 60 * 1000,     // 5 minutes for movie details
  SUGGESTIONS: 15 * 60 * 1000,      // 15 minutes for suggestions
  HOME_DATA: 10 * 60 * 1000,        // 10 minutes for home page data
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

    // Check cache version - invalidate old caches
    if (entry.version !== CACHE_VERSION) {
      console.log(`[Cache] VERSION MISMATCH: ${key} (v${entry.version} vs v${CACHE_VERSION})`);
      localStorage.removeItem(`cache_${key}`);
      return null;
    }

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
      version: CACHE_VERSION,
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
          // Remove if old version or expired
          if (entry.version !== CACHE_VERSION || now - entry.timestamp > maxAge) {
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

// Initialize cache - clear old version caches
export function initCache(): void {
  if (!browser) return;
  console.log(`[Cache] Initializing v${CACHE_VERSION}`);
  clearOldCaches();
}

// Wrapper for cached API calls
export async function cachedFetch<T>(
  key: string,
  ttl: number,
  fetchFn: () => Promise<T>,
  forceRefresh: boolean = false
): Promise<T> {
  // Skip cache entirely in dev mode
  if (CACHE_DISABLED) {
    console.log(`[Cache] DEV MODE - skipping cache for: ${key}`);
    return await fetchFn();
  }

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
