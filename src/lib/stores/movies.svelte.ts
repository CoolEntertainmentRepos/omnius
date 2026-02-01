// Svelte 5 store for movie list state
import type { Movie, MovieListData, ListMoviesParams } from "$lib/api/types";
import { listMovies, searchMovies } from "$lib/api/commands";

class MoviesStore {
  movies = $state<Movie[]>([]);
  loading = $state(false);
  error = $state<string | null>(null);
  currentPage = $state(1);
  totalPages = $state(1);
  totalCount = $state(0);
  searchQuery = $state("");
  selectedGenre = $state<string | null>(null);

  private ITEMS_PER_PAGE = 20;

  get hasNextPage() {
    return this.currentPage < this.totalPages;
  }

  get hasPrevPage() {
    return this.currentPage > 1;
  }

  async fetchMovies(params: ListMoviesParams = {}) {
    this.loading = true;
    this.error = null;

    try {
      const data: MovieListData = await listMovies({
        limit: this.ITEMS_PER_PAGE,
        page: this.currentPage,
        ...params,
      });

      this.movies = data.movies || [];
      this.totalCount = data.movie_count;
      this.totalPages = Math.ceil(data.movie_count / this.ITEMS_PER_PAGE);
    } catch (err) {
      this.error = err instanceof Error ? err.message : "Failed to fetch movies";
      this.movies = [];
    } finally {
      this.loading = false;
    }
  }

  async search(query: string) {
    this.searchQuery = query;
    this.currentPage = 1;
    this.selectedGenre = null;

    if (!query.trim()) {
      await this.fetchMovies();
      return;
    }

    this.loading = true;
    this.error = null;

    try {
      const data = await searchMovies(query, 1, this.ITEMS_PER_PAGE);
      this.movies = data.movies || [];
      this.totalCount = data.movie_count;
      this.totalPages = Math.ceil(data.movie_count / this.ITEMS_PER_PAGE);
    } catch (err) {
      this.error = err instanceof Error ? err.message : "Search failed";
      this.movies = [];
    } finally {
      this.loading = false;
    }
  }

  async filterByGenre(genre: string | null) {
    this.selectedGenre = genre;
    this.currentPage = 1;
    this.searchQuery = "";

    const params: ListMoviesParams = {};
    if (genre) {
      params.genre = genre;
    }

    await this.fetchMovies(params);
  }

  async goToPage(page: number) {
    if (page < 1 || page > this.totalPages) return;

    this.currentPage = page;

    const params: ListMoviesParams = {};
    if (this.searchQuery) {
      params.query_term = this.searchQuery;
    }
    if (this.selectedGenre) {
      params.genre = this.selectedGenre;
    }

    await this.fetchMovies(params);
  }

  async nextPage() {
    await this.goToPage(this.currentPage + 1);
  }

  async prevPage() {
    await this.goToPage(this.currentPage - 1);
  }

  reset() {
    this.movies = [];
    this.loading = false;
    this.error = null;
    this.currentPage = 1;
    this.totalPages = 1;
    this.totalCount = 0;
    this.searchQuery = "";
    this.selectedGenre = null;
  }
}

export const moviesStore = new MoviesStore();
