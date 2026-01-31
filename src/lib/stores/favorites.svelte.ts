// Svelte 5 store for favorites/watchlist
import { browser } from "$app/environment";
import type { Movie, MovieDetails } from "$lib/api/types";

const STORAGE_KEY = "streamer_favorites";

// Type that works for both Movie and MovieDetails
type FavoriteMovie = Movie | MovieDetails;

class FavoritesStore {
  favorites = $state<Movie[]>([]);

  constructor() {
    if (browser) {
      this.load();
    }
  }

  private load() {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) {
        this.favorites = JSON.parse(stored);
      }
    } catch (err) {
      console.error("Failed to load favorites:", err);
      this.favorites = [];
    }
  }

  private save() {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(this.favorites));
    } catch (err) {
      console.error("Failed to save favorites:", err);
    }
  }

  add(movie: FavoriteMovie) {
    if (!this.isFavorite(movie.id)) {
      // Store only the essential fields
      const movieData: Movie = {
        id: movie.id,
        url: movie.url,
        imdb_code: movie.imdb_code,
        title: movie.title,
        title_long: movie.title_long,
        slug: movie.slug,
        year: movie.year,
        rating: movie.rating,
        runtime: movie.runtime,
        genres: movie.genres,
        summary: movie.summary,
        description_full: movie.description_full,
        synopsis: movie.synopsis,
        yt_trailer_code: movie.yt_trailer_code,
        language: movie.language,
        background_image: movie.background_image,
        background_image_original: movie.background_image_original,
        small_cover_image: movie.small_cover_image,
        medium_cover_image: movie.medium_cover_image,
        large_cover_image: movie.large_cover_image,
        torrents: movie.torrents,
      };
      this.favorites = [...this.favorites, movieData];
      this.save();
    }
  }

  remove(movieId: number) {
    this.favorites = this.favorites.filter((m) => m.id !== movieId);
    this.save();
  }

  toggle(movie: FavoriteMovie) {
    if (this.isFavorite(movie.id)) {
      this.remove(movie.id);
    } else {
      this.add(movie);
    }
  }

  isFavorite(movieId: number): boolean {
    return this.favorites.some((m) => m.id === movieId);
  }

  clear() {
    this.favorites = [];
    this.save();
  }

  get count() {
    return this.favorites.length;
  }
}

export const favoritesStore = new FavoritesStore();
