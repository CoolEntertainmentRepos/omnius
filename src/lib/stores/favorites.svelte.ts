// Svelte 5 store for favorites/watchlist
import { browser } from "$app/environment";
import type { Movie, MovieDetails, Channel } from "$lib/api/types";

const STORAGE_KEY = "streamer_favorites";
const CHANNELS_KEY = "streamer_fav_channels";
const COUNTRIES_KEY = "streamer_fav_countries";

// Type that works for both Movie and MovieDetails
type FavoriteMovie = Movie | MovieDetails;

export interface FavoriteCountry {
  code: string;
  name: string;
  flag?: string;
}

class FavoritesStore {
  favorites = $state<Movie[]>([]);
  favoriteChannels = $state<Channel[]>([]);
  favoriteCountries = $state<FavoriteCountry[]>([]);

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

    try {
      const storedChannels = localStorage.getItem(CHANNELS_KEY);
      if (storedChannels) {
        this.favoriteChannels = JSON.parse(storedChannels);
      }
    } catch (err) {
      console.error("Failed to load favorite channels:", err);
      this.favoriteChannels = [];
    }

    try {
      const storedCountries = localStorage.getItem(COUNTRIES_KEY);
      if (storedCountries) {
        this.favoriteCountries = JSON.parse(storedCountries);
      }
    } catch (err) {
      console.error("Failed to load favorite countries:", err);
      this.favoriteCountries = [];
    }
  }

  private save() {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(this.favorites));
    } catch (err) {
      console.error("Failed to save favorites:", err);
    }
  }

  private saveChannels() {
    try {
      localStorage.setItem(CHANNELS_KEY, JSON.stringify(this.favoriteChannels));
    } catch (err) {
      console.error("Failed to save favorite channels:", err);
    }
  }

  private saveCountries() {
    try {
      localStorage.setItem(COUNTRIES_KEY, JSON.stringify(this.favoriteCountries));
    } catch (err) {
      console.error("Failed to save favorite countries:", err);
    }
  }

  // Movie favorites
  add(movie: FavoriteMovie) {
    if (!this.isFavorite(movie.id)) {
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

  // Channel favorites
  addChannel(ch: Channel) {
    if (!this.isChannelFavorite(ch.id)) {
      this.favoriteChannels = [...this.favoriteChannels, { ...ch }];
      this.saveChannels();
    }
  }

  removeChannel(id: string) {
    this.favoriteChannels = this.favoriteChannels.filter((c) => c.id !== id);
    this.saveChannels();
  }

  toggleChannel(ch: Channel) {
    if (this.isChannelFavorite(ch.id)) {
      this.removeChannel(ch.id);
    } else {
      this.addChannel(ch);
    }
  }

  isChannelFavorite(id: string): boolean {
    return this.favoriteChannels.some((c) => c.id === id);
  }

  get channelCount() {
    return this.favoriteChannels.length;
  }

  // Country favorites
  addCountry(c: FavoriteCountry) {
    if (!this.isCountryFavorite(c.code)) {
      this.favoriteCountries = [...this.favoriteCountries, { ...c }];
      this.saveCountries();
    }
  }

  removeCountry(code: string) {
    this.favoriteCountries = this.favoriteCountries.filter((c) => c.code !== code);
    this.saveCountries();
  }

  toggleCountry(c: FavoriteCountry) {
    if (this.isCountryFavorite(c.code)) {
      this.removeCountry(c.code);
    } else {
      this.addCountry(c);
    }
  }

  isCountryFavorite(code: string): boolean {
    return this.favoriteCountries.some((c) => c.code === code);
  }

  get countryCount() {
    return this.favoriteCountries.length;
  }

  get totalCount() {
    return this.favorites.length + this.favoriteChannels.length + this.favoriteCountries.length;
  }
}

export const favoritesStore = new FavoritesStore();
