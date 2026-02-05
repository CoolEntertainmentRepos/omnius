// Svelte 5 store for "Remind Me" feature - stores IMDB codes of coming soon movies
import { browser } from "$app/environment";

const STORAGE_KEY = "streamer_reminders";

export interface ReminderMovie {
  imdb_code: string;
  title: string;
  poster?: string;
  added_at: number;
}

class RemindersStore {
  reminders = $state<ReminderMovie[]>([]);

  constructor() {
    if (browser) {
      this.load();
    }
  }

  private load() {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) {
        this.reminders = JSON.parse(stored);
      }
    } catch (err) {
      console.error("Failed to load reminders:", err);
      this.reminders = [];
    }
  }

  private save() {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(this.reminders));
    } catch (err) {
      console.error("Failed to save reminders:", err);
    }
  }

  add(imdbCode: string, title: string, poster?: string) {
    if (!this.isReminded(imdbCode)) {
      this.reminders = [...this.reminders, {
        imdb_code: imdbCode,
        title,
        poster,
        added_at: Date.now(),
      }];
      this.save();
    }
  }

  remove(imdbCode: string) {
    this.reminders = this.reminders.filter((r) => r.imdb_code !== imdbCode);
    this.save();
  }

  toggle(imdbCode: string, title: string, poster?: string) {
    if (this.isReminded(imdbCode)) {
      this.remove(imdbCode);
    } else {
      this.add(imdbCode, title, poster);
    }
  }

  isReminded(imdbCode: string): boolean {
    return this.reminders.some((r) => r.imdb_code === imdbCode);
  }

  getAll(): ReminderMovie[] {
    return this.reminders;
  }

  getAllImdbCodes(): string[] {
    return this.reminders.map((r) => r.imdb_code);
  }

  clear() {
    this.reminders = [];
    this.save();
  }

  get count() {
    return this.reminders.length;
  }
}

export const remindersStore = new RemindersStore();
