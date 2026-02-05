<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { browser } from "$app/environment";
  import { page } from "$app/stores";
  import MovieRow from "$lib/components/MovieRow.svelte";
  import Top10Row from "$lib/components/Top10Row.svelte";
  import MovieGrid from "$lib/components/MovieGrid.svelte";
  import SeriesRow from "$lib/components/SeriesRow.svelte";
  import SeriesGrid from "$lib/components/SeriesGrid.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import SearchSection from "$lib/components/SearchSection.svelte";
  import { listMovies, getMovieRating, getLocalRatings, listSeries, getTopRatedSeries, getContinuingSeries, getCuratedLists, getCuratedList, getHomeData, getImdbImages, searchMovies, searchSeries, searchChannels, listChannels, getChannelCountries, getChannelCategories, type CuratedList, type HomeSection } from "$lib/api/commands";
  import { favoritesStore } from "$lib/stores/favorites.svelte";
  import { makeFocusable, addSection } from "$lib/utils/tvNavigation";
  import type { Movie, MovieRating, Series, ListSeriesParams, Channel } from "$lib/api/types";

  // Category rows data
  let featured = $state<Movie | null>(null);
  let featuredRating = $state<MovieRating | null>(null);
  let featuredBackground = $state<string | null>(null);

  // Hero background slideshow (multiple images for same movie)
  let heroBackgrounds = $state<string[]>([]);
  let currentBgIndex = $state(0);
  let prevBgIndex = $state(-1);
  let bgTransitioning = $state(false);
  let heroInterval: ReturnType<typeof setInterval> | null = null;

  let trending = $state<Movie[]>([]);
  let topRated = $state<Movie[]>([]);
  let newReleases = $state<Movie[]>([]);
  let action = $state<Movie[]>([]);
  let comedy = $state<Movie[]>([]);
  let thriller = $state<Movie[]>([]);
  let scifi = $state<Movie[]>([]);
  let drama = $state<Movie[]>([]);
  let homeSeries = $state<Series[]>([]);

  let loading = $state(true);
  let loadError = $state<string | null>(null);
  let retryCount = $state(0);
  let retryTimer: ReturnType<typeof setTimeout> | null = null;
  let activeNav = $state("home");

  // Search state
  let searchQuery = $state("");
  let searchResults = $state<Movie[]>([]);
  let searchSuggestions = $state<Movie[]>([]);
  let searchLoading = $state(false);
  let showSearch = $state(false);
  let showSuggestions = $state(false);
  let searchDebounceTimer: ReturnType<typeof setTimeout>;

  // Genre browsing
  let selectedGenre = $state<string | null>(null);
  let genreMovies = $state<Movie[]>([]);
  let genreLoading = $state(false);
  let genrePage = $state(1);
  let genreTotal = $state(0);
  let genreLoadingMore = $state(false);

  // Live Channels (IPTV) - using iptv-org API
  interface IPTVChannel {
    id: string;
    name: string;
    country: string;
    languages: string[];
    categories: string[];
    logo?: string;
    url?: string;
  }
  interface IPTVStream {
    channel: string;
    url: string;
  }
  interface IPTVCountry {
    code: string;
    name: string;
    flag: string;
  }
  interface IPTVCategory {
    id: string;
    name: string;
  }

  let iptvChannels = $state<IPTVChannel[]>([]);
  let iptvCountries = $state<IPTVCountry[]>([]);
  let iptvCategories = $state<IPTVCategory[]>([]);
  let channelGroups = $state<string[]>([]);
  let selectedGroup = $state<string | null>(null);
  let channelsLoading = $state(false);
  let channelsError = $state<string | null>(null);
  let channelGroupBy = $state<'country' | 'category' | 'search'>('country');
  let channelSearchQuery = $state('');
  let channelSearchResults = $state<Channel[]>([]);
  let channelSearchLoading = $state(false);

  // Movie browse tabs
  let movieBrowseMode = $state<'genre' | 'year' | 'curated' | 'search'>('curated');
  let movieSearchQuery = $state('');
  let movieSearchResults = $state<Movie[]>([]);
  let movieSearchLoading = $state(false);
  let selectedYear = $state<number | null>(null);
  let yearMovies = $state<Movie[]>([]);
  let yearLoading = $state(false);
  let curatedList = $state<string>('');
  let curatedMovies = $state<Movie[]>([]);
  let curatedLoading = $state(false);
  let curatedLists = $state<CuratedList[]>([]);
  let curatedListsLoading = $state(false);

  const years = Array.from({ length: 30 }, (_, i) => new Date().getFullYear() - i);

  // TV Series state
  let tvSeries = $state<Series[]>([]);
  let tvSeriesTopRated = $state<Series[]>([]);
  let tvSeriesOngoing = $state<Series[]>([]);
  let tvSeriesLoading = $state(false);
  let tvSeriesPage = $state(1);
  let tvSeriesTotal = $state(0);
  let tvSeriesLoadingMore = $state(false);
  let tvBrowseMode = $state<'curated' | 'genre' | 'network' | 'search'>('curated');
  let tvSearchQuery = $state('');
  let tvSearchResults = $state<Series[]>([]);
  let tvSearchLoading = $state(false);
  let tvCuratedList = $state<string>('top');
  let tvCuratedSeries = $state<Series[]>([]);
  let tvCuratedLoading = $state(false);
  let tvSelectedGenre = $state<string | null>(null);
  let tvGenreSeries = $state<Series[]>([]);
  let tvGenreLoading = $state(false);
  let tvSelectedNetwork = $state<string | null>(null);
  let tvNetworkSeries = $state<Series[]>([]);
  let tvNetworkLoading = $state(false);

  const tvCuratedLists = [
    { id: 'top', name: 'Top Rated', filter: { sort_by: 'rating', order_by: 'desc', minimum_rating: 8.5 } },
    { id: 'popular', name: 'Most Popular', filter: { sort_by: 'date_added', order_by: 'desc' } },
    { id: 'ongoing', name: 'Ongoing', filter: { status: 'Continuing' } },
    { id: 'classics', name: 'Classics', filter: { sort_by: 'rating', order_by: 'desc', maximum_year: 2010 } },
  ];

  const tvGenres = ['Drama', 'Comedy', 'Action', 'Adventure', 'Crime', 'Thriller', 'Sci-Fi', 'Fantasy', 'Horror', 'Mystery', 'Animation', 'Documentary', 'Romance', 'Biography', 'History', 'War'];
  const tvNetworks = ['HBO', 'Netflix', 'AMC', 'BBC', 'NBC', 'CBS', 'ABC', 'FOX', 'Disney+', 'Showtime', 'Comedy Central', 'Adult Swim', 'The CW', 'PBS', 'Nickelodeon'];

  // Pagination for category views
  let trendingPage = $state(1);
  let trendingTotal = $state(0);
  let trendingLoadingMore = $state(false);

  let newReleasesPage = $state(1);
  let newReleasesTotal = $state(0);
  let newReleasesLoadingMore = $state(false);

  let searchPage = $state(1);
  let searchTotal = $state(0);
  let searchLoadingMore = $state(false);

  const genres = [
    "Action", "Adventure", "Animation", "Biography", "Comedy", "Crime",
    "Documentary", "Drama", "Family", "Fantasy", "History", "Horror",
    "Music", "Mystery", "Romance", "Sci-Fi", "Sport", "Thriller", "War", "Western"
  ];

  onMount(async () => {
    // Check for tab parameter in URL
    const urlParams = new URLSearchParams(window.location.search);
    const tab = urlParams.get('tab');
    if (tab === 'tvshows' || tab === 'tv') {
      await handleNavClick('tv');
    } else if (tab === 'movies') {
      await handleNavClick('movies');
    } else if (tab === 'live' || tab === 'channels') {
      await handleNavClick('live');
    } else {
      await loadHomeContent();
    }
    // Refresh focusable elements and set initial focus
    setTimeout(() => {
      makeFocusable();

      // Add sidebar section with navigation to content on RIGHT
      addSection('sidebar', '.sidebar .nav-item', {
        restrict: 'self-first',
        leaveFor: {
          right: '.movie-card, .featured-actions button',
          down: '.movie-card, .featured-actions button'
        }
      });

      // Add content section
      addSection('content', '.main-content button, .main-content a, .movie-card', {
        restrict: 'self-first',
        leaveFor: {
          left: '.sidebar .nav-item.active'
        }
      });

      // Focus on first movie card by default for quick access
      const firstMovieCard = document.querySelector<HTMLElement>('.movie-card');
      if (firstMovieCard) {
        firstMovieCard.focus();
      }
    }, 300);

    // Cleanup on unmount
    return () => {
      if (retryTimer) clearTimeout(retryTimer);
      stopHeroAutoplay();
    };
  });

  // Dynamic home sections from API
  let homeSections = $state<HomeSection[]>([]);

  async function loadHomeContent() {
    loading = true;
    loadError = null;

    try {
      // First, try to load dynamic home data from server
      const homeData = await getHomeData();
      homeSections = homeData.sections || [];
      console.log('[Home] Loaded sections from API:', homeSections.length);

      // Get featured movie from hero_slider or hero section
      if (homeData.hero_slider?.[0]) {
        featured = homeData.hero_slider[0];
      } else {
        // Try sections with hero/banner display type
        const heroSection = homeSections.find(s => s.display_type === 'hero' || s.display_type === 'banner');
        if (heroSection?.movies?.[0]) {
          featured = heroSection.movies[0];
        } else {
          // Fallback to top rated
          const topData = await listMovies({ sort_by: "rating", limit: 1, minimum_rating: 8 });
          if (topData.movies?.[0]) {
            featured = topData.movies[0];
          }
        }
      }

      if (featured) {
        console.log('[Home] Featured movie:', featured.title);

        // Fetch multiple IMDB images for Ken Burns slideshow
        if (featured.imdb_code) {
          const images = await getImdbImages(featured.imdb_code);
          // Get horizontal images (good for backgrounds)
          const bgImages = images
            .filter(img => img.width > img.height) // horizontal only
            .filter(img => img.type === 'still_frame' || img.type === 'publicity' || img.type === 'production')
            .map(img => img.url)
            .slice(0, 5);

          if (bgImages.length > 0) {
            heroBackgrounds = bgImages;
            featuredBackground = bgImages[0];
            currentBgIndex = 0;
            console.log('[Home] Loaded', bgImages.length, 'background images');
            // Start background rotation
            startHeroAutoplay();
          } else {
            // Fallback to movie's default background
            featuredBackground = featured.background_image_original || featured.large_cover_image || null;
          }

          // Get rating
          const rating = await getMovieRating(featured.imdb_code);
          if (rating) featuredRating = rating;
        }
      }

      // Map API sections to local arrays for display
      for (const section of homeSections) {
        if (!section.movies || section.display_type === 'hero' || section.display_type === 'banner') continue;

        // Map by section type or id
        switch (section.type || section.id) {
          case 'recent':
          case 'recently_added':
            newReleases = section.movies;
            newReleasesTotal = section.movies.length;
            break;
          case 'top_rated':
            topRated = section.movies;
            break;
        }

        // Map by section id for custom sections
        if (section.id === 'trending' || section.id.includes('trending')) {
          trending = section.movies;
          trendingTotal = section.movies.length;
        }
      }

      // Fallback: If no API sections or missing data, load traditional way
      if (homeSections.length === 0 || !featured) {
        console.log('[Home] Using fallback data loading');
        await loadFallbackContent();
      }

      // Always load these additional sections (API might not have them)
      if (action.length === 0 || comedy.length === 0) {
        const [actionData, comedyData, thrillerData, scifiData, dramaData] = await Promise.all([
          listMovies({ genre: "Action", sort_by: "rating", limit: 20 }),
          listMovies({ genre: "Comedy", sort_by: "rating", limit: 20 }),
          listMovies({ genre: "Thriller", sort_by: "rating", limit: 20 }),
          listMovies({ genre: "Sci-Fi", sort_by: "rating", limit: 20 }),
          listMovies({ genre: "Drama", sort_by: "rating", limit: 20 }),
        ]);
        action = actionData.movies || [];
        comedy = comedyData.movies || [];
        thriller = thrillerData.movies || [];
        scifi = scifiData.movies || [];
        drama = dramaData.movies || [];
      }

      // Load TV series for home row (non-blocking)
      listSeries({ limit: 20, sort_by: 'rating', order_by: 'desc' })
        .then(data => { homeSeries = data.series || []; })
        .catch(err => console.warn('[Home] Failed to load series:', err));

      // Fetch OMDB rating for featured movie
      if (featured?.imdb_code) {
        try {
          featuredRating = await getMovieRating(featured.imdb_code);
        } catch (err) {
          console.warn('[Home] Failed to fetch featured movie rating:', err);
        }
      }

      // Reset retry count on success
      retryCount = 0;
      loadError = null;
    } catch (err) {
      console.error('[Home] Failed to load content:', err);
      loadError = "Unable to connect to server";

      // Auto-retry with exponential backoff (max 30 seconds)
      const delay = Math.min(3000 * Math.pow(2, retryCount), 30000);
      retryCount += 1;
      console.log(`[Home] Retrying in ${delay/1000}s (attempt ${retryCount})`);

      if (retryTimer) clearTimeout(retryTimer);
      retryTimer = setTimeout(() => {
        loadHomeContent();
      }, delay);
    } finally {
      loading = false;
    }
  }

  async function loadFallbackContent() {
    const [trendingData, topRatedData, newReleasesData] = await Promise.all([
      listMovies({ sort_by: "download_count", limit: 20 }),
      listMovies({ sort_by: "rating", limit: 20, minimum_rating: 8 }),
      listMovies({ sort_by: "date_added", limit: 20 }),
    ]);

    trending = trendingData.movies || [];
    trendingTotal = trendingData.movie_count || 0;
    topRated = topRatedData.movies || [];
    newReleases = newReleasesData.movies || [];
    newReleasesTotal = newReleasesData.movie_count || 0;

    // Enrich with local ratings
    const allMovies = [...trending, ...topRated, ...newReleases];
    const imdbCodes = [...new Set(allMovies.map(m => m.imdb_code).filter(Boolean))];
    if (imdbCodes.length > 0) {
      const localRatings = await getLocalRatings(imdbCodes);
      const enrichMovie = (m: Movie) => {
        const rating = localRatings[m.imdb_code];
        if (rating?.imdb_rating) m.rating = rating.imdb_rating;
        return m;
      };
      trending = trending.map(enrichMovie);
      topRated = topRated.map(enrichMovie);
      newReleases = newReleases.map(enrichMovie);
    }

    // Feature a top rated movie if no hero from API
    if (!featured && topRated.length > 0) {
      featured = topRated[0];
    }
  }

  // Hero background rotation (Ken Burns effect)
  function startHeroAutoplay() {
    stopHeroAutoplay();
    if (heroBackgrounds.length <= 1) {
      console.log('[Hero] Only 1 background, no rotation needed');
      return;
    }

    console.log('[Hero] Starting Ken Burns rotation with', heroBackgrounds.length, 'images');
    heroInterval = setInterval(() => {
      nextBackground();
    }, 5000); // 5 seconds per image
  }

  function stopHeroAutoplay() {
    if (heroInterval) {
      clearInterval(heroInterval);
      heroInterval = null;
    }
  }

  function nextBackground() {
    if (heroBackgrounds.length <= 1) return;

    // Start transition
    bgTransitioning = true;
    prevBgIndex = currentBgIndex;

    // After fade out, switch to next image
    setTimeout(() => {
      currentBgIndex = (currentBgIndex + 1) % heroBackgrounds.length;
      featuredBackground = heroBackgrounds[currentBgIndex];
      bgTransitioning = false;
      console.log('[Hero] Background', currentBgIndex + 1, '/', heroBackgrounds.length);
    }, 1000); // 1 second crossfade
  }

  function manualRetry() {
    retryCount = 0;
    if (retryTimer) clearTimeout(retryTimer);
    loadHomeContent();
  }

  async function loadChannels() {
    if (iptvChannels.length > 0) return; // Already loaded

    channelsLoading = true;
    channelsError = null;

    try {
      // Check cache first (1 hour TTL for channels)
      if (browser) {
        const cached = localStorage.getItem('iptv_data_v4');
        const cacheTime = localStorage.getItem('iptv_data_v4_time');
        const oneHour = 60 * 60 * 1000;

        if (cached && cacheTime && Date.now() - parseInt(cacheTime) < oneHour) {
          console.log('[IPTV] Loading from cache');
          const data = JSON.parse(cached);
          iptvChannels = data.channels;
          iptvCountries = data.countries;
          iptvCategories = data.categories;
          updateChannelGroups();
          channelsLoading = false;
          return;
        }
      }

      console.log('[IPTV] Fetching from API');

      // Fetch channels, countries, and categories in parallel
      const [channelsData, countries, categories] = await Promise.all([
        listChannels({ limit: 10000 }),
        getChannelCountries(),
        getChannelCategories()
      ]);

      iptvChannels = (channelsData.channels || []).map(ch => ({
        id: ch.id,
        name: ch.name,
        country: ch.country || '',
        languages: ch.languages || [],
        categories: ch.categories || [],
        logo: ch.logo,
        url: ch.stream_url
      }));
      iptvCountries = countries.map(c => ({ code: c.code, name: c.name, flag: c.flag || '' }));
      iptvCategories = categories.map(c => ({ id: c.id, name: c.name }));
      updateChannelGroups();

      // Cache for 1 hour
      if (browser) {
        localStorage.setItem('iptv_data_v4', JSON.stringify({
          channels: iptvChannels,
          countries: iptvCountries,
          categories: iptvCategories
        }));
        localStorage.setItem('iptv_data_v4_time', Date.now().toString());
      }

      console.log(`[IPTV] Loaded ${iptvChannels.length} channels from API`);
    } catch (err) {
      console.error('[IPTV] Failed to load channels:', err);
      channelsError = 'Failed to load channels. Is the server running?';
    } finally {
      channelsLoading = false;
    }
  }

  function updateChannelGroups() {
    if (channelGroupBy === 'country') {
      // Get unique countries that have channels
      const countrySet = new Set(iptvChannels.map(ch => ch.country));
      const countriesWithChannels = iptvCountries.filter(c => countrySet.has(c.code));
      channelGroups = countriesWithChannels.map(c => c.code).sort((a, b) => {
        const nameA = iptvCountries.find(c => c.code === a)?.name || a;
        const nameB = iptvCountries.find(c => c.code === b)?.name || b;
        return nameA.localeCompare(nameB);
      });
    } else {
      // Get unique categories that have channels
      const catSet = new Set(iptvChannels.flatMap(ch => ch.categories));
      channelGroups = iptvCategories
        .filter(c => catSet.has(c.id))
        .map(c => c.id)
        .sort((a, b) => {
          const nameA = iptvCategories.find(c => c.id === a)?.name || a;
          const nameB = iptvCategories.find(c => c.id === b)?.name || b;
          return nameA.localeCompare(nameB);
        });
    }
  }

  function getGroupName(code: string): string {
    if (channelGroupBy === 'country') {
      const country = iptvCountries.find(c => c.code === code);
      return country ? `${country.flag} ${country.name}` : code;
    } else {
      const cat = iptvCategories.find(c => c.id === code);
      return cat?.name || code;
    }
  }

  function getChannelsInGroup(groupCode: string): IPTVChannel[] {
    if (channelGroupBy === 'country') {
      return iptvChannels.filter(ch => ch.country === groupCode);
    } else {
      return iptvChannels.filter(ch => ch.categories.includes(groupCode));
    }
  }

  function switchGroupBy(mode: 'country' | 'category' | 'search') {
    channelGroupBy = mode;
    selectedGroup = null;
    if (mode !== 'search') {
      updateChannelGroups();
    } else {
      channelSearchQuery = '';
      channelSearchResults = [];
    }
  }

  async function handleChannelSearch() {
    if (!channelSearchQuery.trim()) return;
    channelSearchLoading = true;
    try {
      const data = await searchChannels(channelSearchQuery, 1, 50);
      channelSearchResults = data.channels || [];
    } catch (err) {
      console.error('[Search] Channel search failed:', err);
      channelSearchResults = [];
    } finally {
      channelSearchLoading = false;
    }
  }

  async function handleYearSelect(year: number) {
    selectedYear = year;
    yearLoading = true;
    try {
      const data = await listMovies({ year, sort_by: 'rating', limit: 50 });
      yearMovies = data.movies || [];
    } catch (err) {
      console.error('Failed to load year movies:', err);
    } finally {
      yearLoading = false;
    }
  }

  async function handleCuratedSelect(slug: string) {
    curatedList = slug;
    curatedLoading = true;
    try {
      const list = await getCuratedList(slug);
      curatedMovies = list?.movies || [];
    } catch (err) {
      console.error('Failed to load curated list:', err);
    } finally {
      curatedLoading = false;
    }
  }

  async function loadCuratedLists() {
    if (curatedLists.length > 0) return;
    curatedListsLoading = true;
    try {
      curatedLists = await getCuratedLists();
      // Auto-select first list if available
      if (curatedLists.length > 0 && !curatedList) {
        handleCuratedSelect(curatedLists[0].slug);
      }
    } catch (err) {
      console.error('Failed to load curated lists:', err);
    } finally {
      curatedListsLoading = false;
    }
  }

  function switchMovieBrowseMode(mode: 'genre' | 'year' | 'curated' | 'search') {
    movieBrowseMode = mode;
    selectedYear = null;
    selectedGenre = null;
    if (mode === 'curated') {
      loadCuratedLists();
    }
    if (mode === 'search') {
      movieSearchQuery = '';
      movieSearchResults = [];
    }
  }

  async function handleMovieSearch() {
    const query = movieSearchQuery.trim();
    if (!query) return;
    movieSearchLoading = true;
    console.log('[MovieSearch] Searching for:', query);
    try {
      // Use listMovies with query_term - same as main search
      const data = await listMovies({ query_term: query, limit: 50, page: 1 });
      console.log('[MovieSearch] Response:', data);
      movieSearchResults = data?.movies || [];
      console.log('[MovieSearch] Found:', movieSearchResults.length, 'movies');
    } catch (err) {
      console.error('[MovieSearch] Failed:', err);
      movieSearchResults = [];
    } finally {
      movieSearchLoading = false;
    }
  }

  function playChannel(channel: IPTVChannel) {
    if (channel.url) {
      goto(`/player/live?url=${encodeURIComponent(channel.url)}&title=${encodeURIComponent(channel.name)}`);
    }
  }

  async function loadTVSeries() {
    if (tvSeries.length > 0) return; // Already loaded

    tvSeriesLoading = true;
    try {
      const [allData, topData, ongoingData] = await Promise.all([
        listSeries({ limit: 50, sort_by: 'date_added', order_by: 'desc' }),
        listSeries({ limit: 20, sort_by: 'rating', order_by: 'desc', minimum_rating: 8 }),
        listSeries({ limit: 20, status: 'Continuing', sort_by: 'rating', order_by: 'desc' }),
      ]);

      tvSeries = allData.series || [];
      tvSeriesTotal = allData.series_count || 0;
      tvSeriesTopRated = topData.series || [];
      tvSeriesOngoing = ongoingData.series || [];
    } catch (err) {
      console.error('[TV] Failed to load series:', err);
      tvSeries = [];
    } finally {
      tvSeriesLoading = false;
    }
  }

  async function loadMoreTVSeries() {
    if (tvSeriesLoadingMore) return;
    tvSeriesLoadingMore = true;
    tvSeriesPage += 1;

    try {
      const data = await listSeries({ limit: 50, page: tvSeriesPage, sort_by: 'date_added', order_by: 'desc' });
      tvSeries = [...tvSeries, ...(data.series || [])];
    } catch {
      tvSeriesPage -= 1;
    } finally {
      tvSeriesLoadingMore = false;
    }
  }

  function switchTVBrowseMode(mode: 'curated' | 'genre' | 'network' | 'search') {
    tvBrowseMode = mode;
    if (mode === 'search') {
      tvSearchQuery = '';
      tvSearchResults = [];
    }
  }

  async function handleTVSearch() {
    if (!tvSearchQuery.trim()) return;
    tvSearchLoading = true;
    try {
      const data = await searchSeries(tvSearchQuery, 1, 50);
      tvSearchResults = data.series || [];
    } catch (err) {
      console.error('[Search] TV search failed:', err);
      tvSearchResults = [];
    } finally {
      tvSearchLoading = false;
    }
  }

  async function handleTVCuratedSelect(listId: string) {
    tvCuratedList = listId;
    tvCuratedLoading = true;
    tvCuratedSeries = [];

    const list = tvCuratedLists.find(l => l.id === listId);
    if (!list) {
      tvCuratedLoading = false;
      return;
    }

    try {
      const data = await listSeries({ limit: 50, ...list.filter });
      tvCuratedSeries = data.series || [];
    } catch (err) {
      console.error('Failed to load curated series:', err);
    } finally {
      tvCuratedLoading = false;
    }
  }

  async function handleTVGenreSelect(genre: string) {
    tvSelectedGenre = genre;
    tvGenreLoading = true;
    tvGenreSeries = [];

    try {
      const data = await listSeries({ limit: 50, genre: genre });
      tvGenreSeries = data.series || [];
    } catch (err) {
      console.error('Failed to load genre series:', err);
    } finally {
      tvGenreLoading = false;
    }
  }

  async function handleTVNetworkSelect(network: string) {
    tvSelectedNetwork = network;
    tvNetworkLoading = true;
    tvNetworkSeries = [];

    try {
      const data = await listSeries({ limit: 50, network: network });
      tvNetworkSeries = data.series || [];
    } catch (err) {
      console.error('Failed to load network series:', err);
    } finally {
      tvNetworkLoading = false;
    }
  }

  function handleSeriesClick(series: Series) {
    goto(`/series/${series.id}`);
  }

  async function handleNavClick(nav: string) {
    activeNav = nav;
    showSearch = nav === "search";
    selectedGenre = null;
    selectedGroup = null;

    if (nav === "home") {
      searchQuery = "";
      searchResults = [];
      // Scroll to top when going home
      window.scrollTo({ top: 0, behavior: 'smooth' });
      // Always refresh content when going to home
      await loadHomeContent();
    }

    if (nav === "live") {
      await loadChannels();
    }

    if (nav === "movies") {
      await loadCuratedLists();
    }

    if (nav === "tv") {
      await loadTVSeries();
    }

    // Auto-focus search input when entering search
    if (nav === "search") {
      setTimeout(() => {
        const searchInput = document.querySelector<HTMLInputElement>(".search-input");
        searchInput?.focus();
      }, 100);
    }
  }

  function handleSearchInput() {
    // Clear previous timer
    clearTimeout(searchDebounceTimer);

    if (!searchQuery.trim()) {
      searchSuggestions = [];
      searchResults = [];
      showSuggestions = false;
      return;
    }

    // Show suggestions after short delay (debounce)
    searchDebounceTimer = setTimeout(async () => {
      searchLoading = true;
      showSuggestions = true;
      try {
        const data = await listMovies({ query_term: searchQuery, limit: 8 });
        searchSuggestions = data.movies || [];
      } catch {
        searchSuggestions = [];
      } finally {
        searchLoading = false;
      }
    }, 300);
  }

  async function handleSearchSubmit() {
    clearTimeout(searchDebounceTimer);
    showSuggestions = false;

    if (!searchQuery.trim()) {
      searchResults = [];
      return;
    }

    searchLoading = true;
    searchPage = 1;
    try {
      const data = await listMovies({ query_term: searchQuery, limit: 20, page: 1 });
      searchResults = data.movies || [];
      searchTotal = data.movie_count || 0;
    } catch {
      searchResults = [];
      searchTotal = 0;
    } finally {
      searchLoading = false;
    }
  }

  function selectSuggestion(movie: Movie) {
    showSuggestions = false;
    goto(`/movie/${movie.id}`);
  }

  async function handleGenreSelect(genre: string) {
    selectedGenre = genre;
    activeNav = "movies";
    genreLoading = true;
    genrePage = 1;

    try {
      const data = await listMovies({ genre, sort_by: "rating", limit: 20, page: 1 });
      genreMovies = data.movies || [];
      genreTotal = data.movie_count || 0;
    } catch {
      genreMovies = [];
      genreTotal = 0;
    } finally {
      genreLoading = false;
    }
  }

  async function loadMoreGenre() {
    if (genreLoadingMore || !selectedGenre) return;
    genreLoadingMore = true;
    genrePage += 1;

    try {
      const data = await listMovies({ genre: selectedGenre, sort_by: "rating", limit: 20, page: genrePage });
      genreMovies = [...genreMovies, ...(data.movies || [])];
    } catch {
      genrePage -= 1;
    } finally {
      genreLoadingMore = false;
    }
  }

  async function loadMoreTrending() {
    if (trendingLoadingMore) return;
    trendingLoadingMore = true;
    trendingPage += 1;

    try {
      const data = await listMovies({ sort_by: "download_count", limit: 20, page: trendingPage });
      trending = [...trending, ...(data.movies || [])];
    } catch {
      trendingPage -= 1;
    } finally {
      trendingLoadingMore = false;
    }
  }

  async function loadMoreNewReleases() {
    if (newReleasesLoadingMore) return;
    newReleasesLoadingMore = true;
    newReleasesPage += 1;

    try {
      const data = await listMovies({ sort_by: "date_added", limit: 20, page: newReleasesPage });
      newReleases = [...newReleases, ...(data.movies || [])];
    } catch {
      newReleasesPage -= 1;
    } finally {
      newReleasesLoadingMore = false;
    }
  }

  async function loadMoreSearch() {
    if (searchLoadingMore || !searchQuery.trim()) return;
    searchLoadingMore = true;
    searchPage += 1;

    try {
      const data = await listMovies({ query_term: searchQuery, limit: 20, page: searchPage });
      searchResults = [...searchResults, ...(data.movies || [])];
    } catch {
      searchPage -= 1;
    } finally {
      searchLoadingMore = false;
    }
  }

  function handlePlay() {
    if (featured) {
      console.log('[Hero Play] Featured ID:', featured.id, 'Title:', featured.title);
      goto(`/movie/${featured.id}`);
    }
  }

  function handleMoreInfo() {
    if (featured) {
      goto(`/movie/${featured.id}`);
    }
  }

  function getRatingDisplay(rating: number): string {
    if (!rating || rating === 0) return "N/A";
    return rating.toFixed(1);
  }

  // Special handling for search input - go to sidebar on left arrow at start
  function handleSearchInputKeydown(e: KeyboardEvent) {
    const target = e.target as HTMLInputElement;
    if (e.key === "ArrowLeft" && target.selectionStart === 0) {
      e.preventDefault();
      const activeNavItem = document.querySelector<HTMLElement>(".nav-item.active");
      activeNavItem?.focus();
    }
  }
</script>

<svelte:head>
  <title>Omnius - Watch Movies & Live TV</title>
</svelte:head>

<div class="app">
  <!-- Left Sidebar -->
  <Sidebar {activeNav} onNavClick={handleNavClick} />

  <!-- Main Content -->
  <main class="main-content">
    {#if loading && !loadError}
      <div class="loading-page">
        <div class="spinner"></div>
      </div>
    {:else if loadError}
      <div class="error-page">
        <svg viewBox="0 0 24 24" fill="currentColor" class="error-icon">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
        </svg>
        <h2>Connection Error</h2>
        <p>{loadError}</p>
        <p class="retry-info">Retrying automatically... (attempt {retryCount})</p>
        <button class="btn-retry" onclick={manualRetry}>
          Retry Now
        </button>
      </div>
    {:else if showSearch}
      <!-- Search View -->
      <div class="search-view">
        <div class="search-header">
          <h1>Search</h1>
          <div class="search-box">
            <div class="search-input-container">
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
              </svg>
              <input
                type="text"
                placeholder="Search movies..."
                bind:value={searchQuery}
                oninput={handleSearchInput}
                onkeydown={(e) => { handleSearchInputKeydown(e); if (e.key === 'Enter') handleSearchSubmit(); }}
                onfocus={() => searchQuery && (showSuggestions = true)}
                class="search-input"
              />
              {#if searchQuery}
                <button class="clear-btn" onclick={() => { searchQuery = ""; searchResults = []; searchSuggestions = []; showSuggestions = false; }} aria-label="Clear search">
                  <svg viewBox="0 0 24 24" fill="currentColor">
                    <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
                  </svg>
                </button>
              {/if}
            </div>

            <!-- Live Suggestions Dropdown -->
            {#if showSuggestions && searchQuery}
              <div class="search-suggestions">
                {#if searchLoading}
                  <div class="suggestion-loading">Searching...</div>
                {:else if searchSuggestions.length === 0}
                  <div class="suggestion-empty">No results for "{searchQuery}"</div>
                {:else}
                  {#each searchSuggestions as movie (movie.id)}
                    <button
                      class="suggestion-item"
                      onclick={() => selectSuggestion(movie)}
                      tabindex="0"
                    >
                      <img
                        src={movie.small_cover_image}
                        alt=""
                        class="suggestion-poster"
                      />
                      <div class="suggestion-info">
                        <span class="suggestion-title">{movie.title}</span>
                        <span class="suggestion-meta">{movie.year} • {movie.rating ? movie.rating.toFixed(1) : 'N/A'}</span>
                      </div>
                    </button>
                  {/each}
                  <button class="suggestion-more" onclick={handleSearchSubmit} tabindex="0">
                    See all results for "{searchQuery}"
                  </button>
                {/if}
              </div>
            {/if}
          </div>
        </div>

        {#if searchResults.length > 0}
          <div class="search-results">
            <p class="results-count">{searchResults.length} of {searchTotal} results for "{searchQuery}"</p>
            <MovieGrid
              movies={searchResults}
              loading={false}
              error={null}
              hasMore={searchResults.length < searchTotal}
              loadingMore={searchLoadingMore}
              onLoadMore={loadMoreSearch}
            />
          </div>
        {:else if !searchQuery}
          <div class="search-genres">
            <h2>Browse by Genre</h2>
            <div class="genre-grid">
              {#each genres as genre (genre)}
                <button class="genre-card" onclick={() => handleGenreSelect(genre)}>
                  {genre}
                </button>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {:else if selectedGenre}
      <!-- Genre View -->
      <div class="genre-view">
        <div class="genre-header">
          <button class="back-btn" onclick={() => { selectedGenre = null; activeNav = "home"; }} aria-label="Go back">
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
            </svg>
          </button>
          <h1>{selectedGenre} Movies</h1>
        </div>
        <MovieGrid
          movies={genreMovies}
          loading={genreLoading}
          error={null}
          hasMore={genreMovies.length < genreTotal}
          loadingMore={genreLoadingMore}
          onLoadMore={loadMoreGenre}
        />
      </div>
    {:else if activeNav === "tv"}
      <!-- TV Series View -->
      <div class="category-view">
        <div class="tv-header">
          <h1 class="category-title">Browse TV Series</h1>
          <div class="group-toggle">
            <button
              class="toggle-btn tv-toggle"
              class:active={tvBrowseMode === 'curated'}
              onclick={() => switchTVBrowseMode('curated')}
            >
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z"/>
              </svg>
              Curated
            </button>
            <button
              class="toggle-btn tv-toggle"
              class:active={tvBrowseMode === 'genre'}
              onclick={() => switchTVBrowseMode('genre')}
            >
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M4 8h4V4H4v4zm6 12h4v-4h-4v4zm-6 0h4v-4H4v4zm0-6h4v-4H4v4zm6 0h4v-4h-4v4zm6-10v4h4V4h-4zm-6 4h4V4h-4v4zm6 6h4v-4h-4v4zm0 6h4v-4h-4v4z"/>
              </svg>
              Genre
            </button>
            <button
              class="toggle-btn tv-toggle"
              class:active={tvBrowseMode === 'network'}
              onclick={() => switchTVBrowseMode('network')}
            >
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M17 10.5V7c0-.55-.45-1-1-1H4c-.55 0-1 .45-1 1v10c0 .55.45 1 1 1h12c.55 0 1-.45 1-1v-3.5l4 4v-11l-4 4z"/>
              </svg>
              Network
            </button>
            <button
              class="toggle-btn tv-toggle"
              class:active={tvBrowseMode === 'search'}
              onclick={() => switchTVBrowseMode('search')}
            >
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
              </svg>
              Search
            </button>
          </div>
        </div>

        {#if tvBrowseMode === 'curated'}
          <!-- Curated Lists -->
          <div class="curated-tabs tv-curated">
            {#each tvCuratedLists as list (list.id)}
              <button
                class="curated-tab tv-tab"
                class:active={tvCuratedList === list.id}
                onclick={() => handleTVCuratedSelect(list.id)}
              >
                {list.name}
              </button>
            {/each}
          </div>
          {#if tvCuratedLoading}
            <div class="loading-spinner"><div class="spinner tv-spinner"></div></div>
          {:else if tvCuratedSeries.length > 0}
            <SeriesGrid series={tvCuratedSeries} loading={false} error={null} />
          {:else if tvSeriesLoading}
            <div class="loading-inline">
              <div class="spinner tv-spinner"></div>
              <p>Loading series...</p>
            </div>
          {:else}
            <SeriesGrid
              series={tvCuratedList === 'top' ? tvSeriesTopRated : tvCuratedList === 'ongoing' ? tvSeriesOngoing : tvSeries}
              loading={false}
              error={null}
              hasMore={tvCuratedList !== 'top' && tvCuratedList !== 'ongoing' && tvSeries.length < tvSeriesTotal}
              loadingMore={tvSeriesLoadingMore}
              onLoadMore={tvCuratedList !== 'top' && tvCuratedList !== 'ongoing' ? loadMoreTVSeries : undefined}
            />
          {/if}
        {:else if tvBrowseMode === 'genre'}
          <!-- Genre Grid -->
          {#if tvSelectedGenre}
            <div class="genre-header">
              <button class="back-btn" onclick={() => { tvSelectedGenre = null; tvGenreSeries = []; }} aria-label="Go back">
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
                </svg>
              </button>
              <h2>{tvSelectedGenre} Series</h2>
            </div>
            {#if tvGenreLoading}
              <div class="loading-spinner"><div class="spinner tv-spinner"></div></div>
            {:else}
              <SeriesGrid series={tvGenreSeries} loading={false} error={null} />
            {/if}
          {:else}
            <div class="genre-grid tv-genre-grid">
              {#each tvGenres as genre (genre)}
                <button class="genre-card tv-genre-card" onclick={() => handleTVGenreSelect(genre)}>
                  {genre}
                </button>
              {/each}
            </div>
          {/if}
        {:else if tvBrowseMode === 'network'}
          <!-- Network Selection -->
          {#if tvSelectedNetwork}
            <div class="genre-header">
              <button class="back-btn" onclick={() => { tvSelectedNetwork = null; tvNetworkSeries = []; }} aria-label="Go back">
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
                </svg>
              </button>
              <h2>{tvSelectedNetwork} Series</h2>
            </div>
            {#if tvNetworkLoading}
              <div class="loading-spinner"><div class="spinner tv-spinner"></div></div>
            {:else}
              <SeriesGrid series={tvNetworkSeries} loading={false} error={null} />
            {/if}
          {:else}
            <div class="network-grid">
              {#each tvNetworks as network (network)}
                <button class="network-card" onclick={() => handleTVNetworkSelect(network)}>
                  {network}
                </button>
              {/each}
            </div>
          {/if}
        {:else if tvBrowseMode === 'search'}
          <!-- TV Series Search -->
          <SearchSection type="tvshows" />
        {/if}
      </div>
    {:else if activeNav === "live"}
      <!-- Live Channels View -->
      <div class="category-view">
        <div class="live-header">
          <h1 class="category-title">Live Channels</h1>
          {#if iptvChannels.length > 0 && !selectedGroup}
            <div class="group-toggle">
              <button
                class="toggle-btn"
                class:active={channelGroupBy === 'country'}
                onclick={() => switchGroupBy('country')}
              >
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z"/>
                </svg>
                Countries
              </button>
              <button
                class="toggle-btn"
                class:active={channelGroupBy === 'category'}
                onclick={() => switchGroupBy('category')}
              >
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M4 8h4V4H4v4zm6 12h4v-4h-4v4zm-6 0h4v-4H4v4zm0-6h4v-4H4v4zm6 0h4v-4h-4v4zm6-10v4h4V4h-4zm-6 4h4V4h-4v4zm6 6h4v-4h-4v4zm0 6h4v-4h-4v4z"/>
                </svg>
                Categories
              </button>
              <button
                class="toggle-btn"
                class:active={channelGroupBy === 'search'}
                onclick={() => switchGroupBy('search')}
              >
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
                </svg>
                Search
              </button>
            </div>
          {/if}
        </div>
        {#if channelsLoading}
          <div class="loading-inline">
            <div class="spinner"></div>
            <p>Loading channels...</p>
          </div>
        {:else if channelsError}
          <div class="error-inline">
            <p>{channelsError}</p>
            <button class="btn-retry" onclick={loadChannels}>Retry</button>
          </div>
        {:else if channelGroupBy === 'search'}
          <!-- Channel Search -->
          <SearchSection type="channels" channels={iptvChannels.map(ch => ({ id: ch.id, name: ch.name, logo: ch.logo, stream_url: ch.url, country: ch.country, categories: ch.categories, languages: ch.languages }))} />
        {:else if selectedGroup}
          <div class="genre-header">
            <button class="back-btn" onclick={() => selectedGroup = null} aria-label="Go back">
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
              </svg>
            </button>
            <h2>{getGroupName(selectedGroup)}</h2>
            <span class="group-channel-count">{getChannelsInGroup(selectedGroup).length} channels</span>
          </div>
          <div class="channels-list">
            {#each getChannelsInGroup(selectedGroup) as channel (channel.id)}
              <button class="channel-list-item" onclick={() => playChannel(channel)}>
                {#if channel.logo}
                  <img src={channel.logo} alt="" class="channel-list-logo" />
                {:else}
                  <div class="channel-list-logo-placeholder">
                    <svg viewBox="0 0 24 24" fill="currentColor">
                      <path d="M17 10.5V7c0-.55-.45-1-1-1H4c-.55 0-1 .45-1 1v10c0 .55.45 1 1 1h12c.55 0 1-.45 1-1v-3.5l4 4v-11l-4 4z"/>
                    </svg>
                  </div>
                {/if}
                <span class="channel-list-name">{channel.name}</span>
                <svg class="channel-play-icon" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M8 5v14l11-7z"/>
                </svg>
              </button>
            {/each}
          </div>
        {:else}
          <p class="channels-info">{iptvChannels.length} channels available</p>
          <div class="genre-grid">
            {#each channelGroups as group (group)}
              <button class="genre-card" onclick={() => selectedGroup = group}>
                {getGroupName(group)}
                <span class="group-count">({getChannelsInGroup(group).length})</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>
    {:else if activeNav === "new"}
      <!-- New Releases View -->
      <div class="category-view">
        <h1 class="category-title">New Releases</h1>
        <MovieGrid
          movies={newReleases}
          loading={false}
          error={null}
          hasMore={newReleases.length < newReleasesTotal}
          loadingMore={newReleasesLoadingMore}
          onLoadMore={loadMoreNewReleases}
        />
      </div>
    {:else if activeNav === "movies"}
      <!-- Movies Browse View -->
      <div class="category-view">
        <div class="movies-header">
          <h1 class="category-title">Browse Movies</h1>
          <div class="group-toggle">
            <button
              class="toggle-btn"
              class:active={movieBrowseMode === 'curated'}
              onclick={() => switchMovieBrowseMode('curated')}
            >
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z"/>
              </svg>
              Curated
            </button>
            <button
              class="toggle-btn"
              class:active={movieBrowseMode === 'genre'}
              onclick={() => switchMovieBrowseMode('genre')}
            >
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M4 8h4V4H4v4zm6 12h4v-4h-4v4zm-6 0h4v-4H4v4zm0-6h4v-4H4v4zm6 0h4v-4h-4v4zm6-10v4h4V4h-4zm-6 4h4V4h-4v4zm6 6h4v-4h-4v4zm0 6h4v-4h-4v4z"/>
              </svg>
              Genre
            </button>
            <button
              class="toggle-btn"
              class:active={movieBrowseMode === 'year'}
              onclick={() => switchMovieBrowseMode('year')}
            >
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 3h-1V1h-2v2H8V1H6v2H5c-1.11 0-1.99.9-1.99 2L3 19c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H5V8h14v11zM9 10H7v2h2v-2zm4 0h-2v2h2v-2zm4 0h-2v2h2v-2zm-8 4H7v2h2v-2zm4 0h-2v2h2v-2zm4 0h-2v2h2v-2z"/>
              </svg>
              Year
            </button>
            <button
              class="toggle-btn"
              class:active={movieBrowseMode === 'search'}
              onclick={() => switchMovieBrowseMode('search')}
            >
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
              </svg>
              Search
            </button>
          </div>
        </div>

        {#if movieBrowseMode === 'curated'}
          <!-- Curated Lists -->
          {#if curatedListsLoading}
            <div class="loading-spinner"><div class="spinner"></div></div>
          {:else if curatedLists.length === 0}
            <p style="color: #888; text-align: center;">No curated lists available</p>
          {:else}
            <div class="curated-tabs">
              {#each curatedLists as list (list.slug)}
                <button
                  class="curated-tab"
                  class:active={curatedList === list.slug}
                  onclick={() => handleCuratedSelect(list.slug)}
                >
                  {list.name}
                </button>
              {/each}
            </div>
            {#if curatedLoading}
              <div class="loading-spinner"><div class="spinner"></div></div>
            {:else}
              <MovieGrid movies={curatedMovies} loading={false} error={null} />
            {/if}
          {/if}
        {:else if movieBrowseMode === 'genre'}
          <!-- Genre Grid -->
          {#if selectedGenre}
            <div class="genre-header">
              <button class="back-btn" onclick={() => { selectedGenre = null; }} aria-label="Go back">
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
                </svg>
              </button>
              <h2>{selectedGenre} Movies</h2>
            </div>
            <MovieGrid
              movies={genreMovies}
              loading={genreLoading}
              error={null}
              hasMore={genreMovies.length < genreTotal}
              loadingMore={genreLoadingMore}
              onLoadMore={loadMoreGenre}
            />
          {:else}
            <div class="genre-grid">
              {#each genres as genre (genre)}
                <button class="genre-card" onclick={() => handleGenreSelect(genre)}>
                  {genre}
                </button>
              {/each}
            </div>
          {/if}
        {:else if movieBrowseMode === 'year'}
          <!-- Year Selection -->
          {#if selectedYear}
            <div class="genre-header">
              <button class="back-btn" onclick={() => { selectedYear = null; yearMovies = []; }} aria-label="Go back">
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
                </svg>
              </button>
              <h2>{selectedYear} Movies</h2>
            </div>
            {#if yearLoading}
              <div class="loading-spinner"><div class="spinner"></div></div>
            {:else}
              <MovieGrid movies={yearMovies} loading={false} error={null} />
            {/if}
          {:else}
            <div class="year-grid">
              {#each years as year (year)}
                <button class="year-card" onclick={() => handleYearSelect(year)}>
                  {year}
                </button>
              {/each}
            </div>
          {/if}
        {:else if movieBrowseMode === 'search'}
          <!-- Movie Search -->
          <SearchSection type="movies" />
        {/if}
      </div>
    {:else if activeNav === "favorites"}
      <!-- Favorites View -->
      <div class="category-view">
        <div class="favorites-header">
          <h1 class="category-title">My List</h1>
          {#if favoritesStore.count > 0}
            <span class="favorites-count">{favoritesStore.count} movies</span>
          {/if}
        </div>
        {#if favoritesStore.favorites.length === 0}
          <div class="empty-state">
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/>
            </svg>
            <h2>Your list is empty</h2>
            <p>Add movies to your list to watch them later</p>
          </div>
        {:else}
          <MovieGrid movies={favoritesStore.favorites} loading={false} error={null} />
        {/if}
      </div>
    {:else}
      <!-- Home View -->
      {#if featured}
        <div
          class="hero"
          onmouseenter={stopHeroAutoplay}
          onmouseleave={startHeroAutoplay}
        >
          <!-- Dual-layer background for crossfade -->
          {#if heroBackgrounds.length > 1}
            <div
              class="hero-bg hero-bg-back"
              style="background-image: url({heroBackgrounds[prevBgIndex >= 0 ? prevBgIndex : 0]})"
            ></div>
            <div
              class="hero-bg hero-bg-front"
              class:visible={!bgTransitioning}
              style="background-image: url({heroBackgrounds[currentBgIndex]})"
            ></div>
          {:else}
            <div
              class="hero-bg"
              style="background-image: url({featuredBackground || featured.background_image_original || featured.large_cover_image})"
            ></div>
          {/if}
          <div class="hero-gradient"></div>
          <div class="hero-content">
            <div class="hero-badge">
              <svg class="badge-icon" viewBox="0 0 24 24" fill="currentColor">
                <path d="M4 6.47L5.76 10H20v8H4V6.47M22 4h-4l2 4h-3l-2-4h-2l2 4h-3l-2-4H8l2 4H7L5 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V4z"/>
              </svg>
              <span>MOVIE</span>
            </div>
            <h1 class="hero-title">{featured.title}</h1>
            <div class="hero-meta">
              <span class="hero-year">{featured.year}</span>
              {#if featuredRating?.imdb_rating}
                <span class="hero-rating">
                  <svg viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
                  </svg>
                  {featuredRating.imdb_rating.toFixed(1)} IMDb
                </span>
              {:else if featured.rating && featured.rating > 0}
                <span class="hero-rating">
                  <svg viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
                  </svg>
                  {featured.rating.toFixed(1)} IMDb
                </span>
              {/if}
              {#if featured.genres && featured.genres.length > 0}
                <span class="hero-genres">{featured.genres.slice(0, 2).join(" / ")}</span>
              {/if}
            </div>
            <p class="hero-description">
              {featured.summary || featured.description_full || featured.synopsis || "A thrilling movie experience awaits you."}
            </p>
            <div class="hero-buttons">
              <button class="btn-play" onclick={handlePlay} onfocus={() => window.scrollTo({ top: 0, behavior: 'smooth' })}>
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M8 5v14l11-7z"/>
                </svg>
                Play
              </button>
              <button class="btn-info" onclick={handleMoreInfo} onfocus={() => window.scrollTo({ top: 0, behavior: 'smooth' })}>
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/>
                </svg>
                More Info
              </button>
            </div>
          </div>

        </div>
      {/if}

      <!-- Movie Rows -->
      <div class="content">
        <!-- Dynamic sections from API (excludes hero/banner) -->
        {#each homeSections.filter(s => s.display_type !== 'hero' && s.display_type !== 'banner' && s.movies && s.movies.length > 0) as section}
          {#if section.display_type === 'top10'}
            <Top10Row title={section.title} movies={section.movies || []} sectionId={section.id} />
          {:else}
            <MovieRow title={section.title} movies={section.movies || []} />
          {/if}
        {/each}

        <!-- Fallback sections if no API sections loaded -->
        {#if homeSections.filter(s => s.display_type !== 'hero').length === 0}
          <MovieRow title="Trending Now" movies={trending} />
          <MovieRow title="Top Rated" movies={topRated} />
          <MovieRow title="New Releases" movies={newReleases} />
        {/if}

        <!-- TV Series row -->
        {#if homeSeries.length > 0}
          <SeriesRow title="Popular TV Series" series={homeSeries} />
        {/if}

        <!-- Genre sections (always shown) -->
        {#if action.length > 0}
          <MovieRow title="Action" movies={action} />
        {/if}
        {#if thriller.length > 0}
          <MovieRow title="Thriller" movies={thriller} />
        {/if}
        {#if scifi.length > 0}
          <MovieRow title="Sci-Fi" movies={scifi} />
        {/if}
        {#if comedy.length > 0}
          <MovieRow title="Comedy" movies={comedy} />
        {/if}
        {#if drama.length > 0}
          <MovieRow title="Drama" movies={drama} />
        {/if}
      </div>
    {/if}
  </main>
</div>

<style>
  :global(html, body) {
    background: #141414;
    margin: 0;
    padding: 0;
    height: 100%;
    overflow: hidden;
  }

  /* TV Remote focus styles */
  :global(*:focus) {
    outline: none;
  }

  :global(*:focus-visible:not(.movie-card)) {
    outline: 3px solid #e50914 !important;
    outline-offset: 2px;
  }

  .app {
    display: flex;
    height: 100vh;
    height: 100dvh;
    background: #141414;
    color: #fff;
    overflow: hidden;
  }

  /* Main Content - scrollable area */
  .main-content {
    flex: 1;
    margin-left: 70px;
    height: 100vh;
    height: 100dvh;
    overflow-y: auto;
    overflow-x: hidden;
    scroll-behavior: smooth;
  }

  /* Hero */
  .hero {
    position: relative;
    height: 85vh;
    min-height: 550px;
    max-height: 900px;
    display: flex;
    align-items: flex-end;
    overflow: hidden;
  }

  .hero-gradient {
    position: absolute;
    inset: 0;
    z-index: 5;
    background: linear-gradient(
      to right,
      rgba(20, 20, 20, 0.95) 0%,
      rgba(20, 20, 20, 0.7) 30%,
      rgba(20, 20, 20, 0.4) 50%,
      rgba(20, 20, 20, 0.2) 70%,
      transparent 100%
    ),
    linear-gradient(
      to top,
      #141414 0%,
      rgba(20, 20, 20, 0.8) 15%,
      transparent 40%
    );
  }

  .hero-content {
    position: relative;
    z-index: 10;
    padding: 0 80px 100px;
    max-width: 650px;
  }

  .hero-badge {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    color: #e50914;
    font-size: 0.9rem;
    font-weight: 600;
    letter-spacing: 3px;
    margin-bottom: 12px;
  }

  .badge-icon {
    width: 24px;
    height: 24px;
  }

  .hero-title {
    font-size: 3.5rem;
    font-weight: 700;
    margin: 0 0 16px;
    text-shadow: 2px 2px 8px rgba(0, 0, 0, 0.8);
    line-height: 1.1;
  }

  .hero-meta {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 16px;
    font-size: 1rem;
  }

  .hero-year {
    color: #fff;
    font-weight: 500;
  }

  .hero-rating {
    display: flex;
    align-items: center;
    gap: 6px;
    color: #f5c518;
    font-weight: 600;
  }

  .hero-rating svg {
    width: 18px;
    height: 18px;
  }

  .hero-genres {
    color: #aaa;
  }

  .hero-description {
    font-size: 1.1rem;
    line-height: 1.5;
    color: #ddd;
    margin: 0 0 24px;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
    text-shadow: 1px 1px 4px rgba(0, 0, 0, 0.8);
  }

  .hero-buttons {
    display: flex;
    gap: 16px;
  }

  .btn-play, .btn-info {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 14px 32px;
    border: none;
    border-radius: 6px;
    font-size: 1.1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn-play {
    background: #fff;
    color: #141414;
  }

  .btn-play:hover {
    background: rgba(255, 255, 255, 0.85);
  }

  .btn-play:focus,
  .btn-play:focus-visible {
    outline: none;
    box-shadow: 0 0 0 4px #e50914;
    transform: scale(1.05);
  }

  .btn-info {
    background: rgba(109, 109, 110, 0.7);
    color: #fff;
  }

  .btn-info:hover {
    background: rgba(109, 109, 110, 0.5);
  }

  .btn-info:focus,
  .btn-info:focus-visible {
    outline: none;
    box-shadow: 0 0 0 4px #e50914;
    background: rgba(229, 9, 20, 0.4);
    transform: scale(1.05);
  }

  .btn-play svg, .btn-info svg {
    width: 28px;
    height: 28px;
  }

  /* Hero background layers for crossfade */
  .hero-bg {
    position: absolute;
    inset: 0;
    background-size: cover;
    background-position: center top;
  }

  .hero-bg-back {
    z-index: 0;
  }

  .hero-bg-front {
    z-index: 1;
    opacity: 0;
    transition: opacity 1s ease-in-out;
  }

  .hero-bg-front.visible {
    opacity: 1;
  }

  /* Content */
  .content {
    position: relative;
    margin-top: 20px;
    padding-bottom: 60px;
    z-index: 10;
  }

  /* Search View */
  .search-view, .genre-view, .category-view {
    padding: 40px 80px;
  }

  .search-header {
    margin-bottom: 40px;
  }

  .search-header h1 {
    font-size: 2rem;
    font-weight: 600;
    margin: 0 0 24px;
  }

  .search-input-container {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 16px 24px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    max-width: 600px;
  }

  .search-input-container svg {
    width: 24px;
    height: 24px;
    color: #888;
    flex-shrink: 0;
  }

  .search-input {
    flex: 1;
    background: none;
    border: none;
    color: #fff;
    font-size: 1.2rem;
    font-family: inherit;
    outline: none;
  }

  .search-input:focus {
    outline: none;
  }

  .search-input-container:focus-within {
    box-shadow: 0 0 0 3px #e50914;
    background: rgba(255, 255, 255, 0.15);
  }

  .search-input::placeholder {
    color: #666;
  }

  .clear-btn {
    background: none;
    border: none;
    color: #888;
    cursor: pointer;
    padding: 4px;
    display: flex;
  }

  .clear-btn:hover {
    color: #fff;
  }

  .clear-btn svg {
    width: 20px;
    height: 20px;
  }

  /* Search Suggestions */
  .search-box {
    position: relative;
    max-width: 600px;
  }

  .search-suggestions {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    background: rgba(30, 30, 30, 0.98);
    border-radius: 0 0 12px 12px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-top: none;
    max-height: 350px;
    overflow-y: auto;
    z-index: 100;
  }

  .suggestion-loading,
  .suggestion-empty {
    padding: 16px 20px;
    color: #888;
    font-size: 0.95rem;
  }

  .suggestion-item {
    display: flex;
    align-items: center;
    gap: 14px;
    width: 100%;
    padding: 10px 16px;
    background: none;
    border: none;
    color: #fff;
    cursor: pointer;
    text-align: left;
    transition: background 0.15s;
  }

  .suggestion-item:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .suggestion-item:focus,
  .suggestion-item:focus-visible {
    outline: none;
    background: rgba(229, 9, 20, 0.4);
    box-shadow: inset 0 0 0 2px #e50914;
  }

  .suggestion-poster {
    width: 40px;
    height: 60px;
    object-fit: cover;
    border-radius: 4px;
    background: #333;
  }

  .suggestion-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .suggestion-title {
    font-size: 1rem;
    font-weight: 500;
  }

  .suggestion-meta {
    font-size: 0.85rem;
    color: #888;
  }

  .suggestion-more {
    display: block;
    width: 100%;
    padding: 14px 16px;
    background: none;
    border: none;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
    color: #e50914;
    font-size: 0.95rem;
    font-weight: 500;
    cursor: pointer;
    text-align: center;
    transition: background 0.15s;
  }

  .suggestion-more:hover {
    background: rgba(229, 9, 20, 0.1);
  }

  .suggestion-more:focus,
  .suggestion-more:focus-visible {
    outline: none;
    background: rgba(229, 9, 20, 0.3);
    box-shadow: inset 0 0 0 2px #e50914;
  }

  .results-count {
    font-size: 1rem;
    color: #888;
    margin: 0 0 24px;
  }

  .search-genres {
    margin-top: 40px;
  }

  .search-genres h2 {
    font-size: 1.4rem;
    font-weight: 600;
    margin: 0 0 24px;
  }

  .movies-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 24px;
    flex-wrap: wrap;
    gap: 16px;
  }

  .curated-tabs {
    display: flex;
    gap: 12px;
    margin-bottom: 24px;
    flex-wrap: wrap;
  }

  .curated-tab {
    padding: 12px 24px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 24px;
    color: #aaa;
    font-size: 0.95rem;
    font-weight: 500;
    font-family: inherit;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .curated-tab:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #fff;
  }

  .curated-tab.active {
    background: #e50914;
    border-color: #e50914;
    color: #fff;
  }

  .curated-tab:focus,
  .curated-tab:focus-visible {
    outline: none;
    box-shadow: 0 0 0 3px rgba(229, 9, 20, 0.5);
  }

  .year-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
    gap: 12px;
  }

  .year-card {
    padding: 20px 16px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 8px;
    color: #fff;
    font-size: 1.1rem;
    font-weight: 600;
    font-family: inherit;
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: center;
  }

  .year-card:hover {
    background: rgba(229, 9, 20, 0.3);
    border-color: rgba(229, 9, 20, 0.5);
    transform: scale(1.05);
  }

  .year-card:focus,
  .year-card:focus-visible {
    outline: none;
    border-color: #e50914;
    box-shadow: 0 0 0 3px rgba(229, 9, 20, 0.5);
  }

  .loading-spinner {
    display: flex;
    justify-content: center;
    padding: 60px 0;
  }

  .genre-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 16px;
  }

  .genre-card {
    padding: 24px 16px;
    background: linear-gradient(135deg, rgba(229, 9, 20, 0.3), rgba(229, 9, 20, 0.1));
    border: 1px solid rgba(229, 9, 20, 0.3);
    border-radius: 8px;
    color: #fff;
    font-size: 1rem;
    font-weight: 500;
    font-family: inherit;
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: center;
  }

  .genre-card:hover {
    background: linear-gradient(135deg, rgba(229, 9, 20, 0.5), rgba(229, 9, 20, 0.2));
    border-color: #e50914;
    transform: scale(1.02);
  }

  .genre-card:focus,
  .genre-card:focus-visible {
    outline: none;
    border-color: #e50914;
    box-shadow: 0 0 0 4px #e50914;
    background: linear-gradient(135deg, rgba(229, 9, 20, 0.6), rgba(229, 9, 20, 0.3));
    transform: scale(1.05);
  }

  .genre-header {
    display: flex;
    align-items: center;
    gap: 20px;
    margin-bottom: 32px;
  }

  .genre-header h1 {
    font-size: 2rem;
    font-weight: 600;
    margin: 0;
  }

  .back-btn {
    width: 44px;
    height: 44px;
    background: rgba(255, 255, 255, 0.1);
    border: none;
    border-radius: 50%;
    color: #fff;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.2s ease;
  }

  .back-btn:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  .back-btn:focus {
    outline: none;
    box-shadow: 0 0 0 3px rgba(255, 255, 255, 0.2);
  }

  .back-btn svg {
    width: 24px;
    height: 24px;
  }

  .category-title {
    font-size: 2rem;
    font-weight: 600;
    margin: 0 0 32px;
  }

  .favorites-header {
    display: flex;
    align-items: baseline;
    gap: 16px;
    margin-bottom: 32px;
  }

  .favorites-header .category-title {
    margin: 0;
  }

  .favorites-count {
    font-size: 1rem;
    color: #888;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 80px 40px;
    text-align: center;
  }

  .empty-state svg {
    width: 80px;
    height: 80px;
    color: #333;
    margin-bottom: 24px;
  }

  .empty-state h2 {
    font-size: 1.5rem;
    font-weight: 600;
    margin: 0 0 12px;
    color: #fff;
  }

  .empty-state p {
    font-size: 1rem;
    color: #888;
    margin: 0;
  }

  /* Loading */
  .loading-page {
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .spinner {
    width: 60px;
    height: 60px;
    border: 4px solid rgba(229, 9, 20, 0.2);
    border-top-color: #e50914;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  /* Error page */
  .error-page {
    height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: 40px;
  }

  .error-icon {
    width: 80px;
    height: 80px;
    color: #e50914;
    margin-bottom: 24px;
  }

  .error-page h2 {
    font-size: 1.8rem;
    font-weight: 600;
    margin: 0 0 12px;
  }

  .error-page p {
    color: #888;
    margin: 0 0 8px;
  }

  .retry-info {
    font-size: 0.9rem;
    color: #666;
    margin-bottom: 24px;
  }

  .btn-retry {
    padding: 14px 32px;
    background: #e50914;
    color: #fff;
    border: none;
    border-radius: 6px;
    font-size: 1.1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn-retry:hover {
    background: #f40d17;
  }

  .btn-retry:focus,
  .btn-retry:focus-visible {
    outline: none;
    box-shadow: 0 0 0 4px rgba(229, 9, 20, 0.5);
    transform: scale(1.05);
  }

  /* Coming soon */
  .coming-soon {
    color: #888;
    font-size: 1.1rem;
    text-align: center;
    padding: 60px 20px;
  }

  /* Loading/error inline */
  .loading-inline, .error-inline {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    padding: 60px 20px;
    color: #888;
  }

  .channels-info {
    color: #888;
    margin-bottom: 24px;
  }

  /* Live header with toggle */
  .live-header,
  .tv-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 24px;
    flex-wrap: wrap;
    gap: 16px;
  }

  .live-header .category-title,
  .tv-header .category-title {
    margin: 0;
  }

  .group-toggle {
    display: flex;
    gap: 8px;
    background: rgba(255, 255, 255, 0.05);
    padding: 4px;
    border-radius: 8px;
  }

  .toggle-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 16px;
    background: transparent;
    border: none;
    border-radius: 6px;
    color: #888;
    font-size: 0.95rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .toggle-btn svg {
    width: 18px;
    height: 18px;
  }

  .toggle-btn:hover {
    color: #fff;
    background: rgba(255, 255, 255, 0.1);
  }

  .toggle-btn.active {
    color: #fff;
    background: #e50914;
  }

  .toggle-btn:focus,
  .toggle-btn:focus-visible {
    outline: none;
    box-shadow: 0 0 0 3px rgba(229, 9, 20, 0.5);
  }

  /* TV Series green theme */
  .toggle-btn.tv-toggle.active {
    background: #2ecc71;
  }

  .toggle-btn.tv-toggle:focus,
  .toggle-btn.tv-toggle:focus-visible {
    box-shadow: 0 0 0 3px rgba(46, 204, 113, 0.5);
  }

  .curated-tab.tv-tab.active {
    color: #fff;
    background: #2ecc71;
  }

  .curated-tab.tv-tab:focus,
  .curated-tab.tv-tab:focus-visible {
    box-shadow: 0 0 0 3px rgba(46, 204, 113, 0.5);
    border-color: #2ecc71;
  }

  .spinner.tv-spinner {
    border-color: rgba(46, 204, 113, 0.2);
    border-top-color: #2ecc71;
  }

  .genre-card.tv-genre-card:hover {
    background: rgba(46, 204, 113, 0.15);
    border-color: #2ecc71;
  }

  .genre-card.tv-genre-card:focus,
  .genre-card.tv-genre-card:focus-visible {
    border-color: #2ecc71;
    box-shadow: 0 0 0 3px rgba(46, 204, 113, 0.5);
  }

  /* Network grid */
  .network-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 16px;
    margin-top: 24px;
  }

  .network-card {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 20px 16px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    color: #fff;
    font-size: 1rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .network-card:hover {
    background: rgba(46, 204, 113, 0.15);
    border-color: #2ecc71;
  }

  .network-card:focus,
  .network-card:focus-visible {
    outline: none;
    border-color: #2ecc71;
    box-shadow: 0 0 0 3px rgba(46, 204, 113, 0.5);
  }

  .group-count {
    font-size: 0.8rem;
    color: #666;
    margin-left: 8px;
  }

  /* Channels grid */
  .channels-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 20px;
    margin-top: 24px;
  }

  .channel-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 20px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .channel-card:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.2);
    transform: scale(1.02);
  }

  .channel-card:focus,
  .channel-card:focus-visible {
    outline: none;
    border-color: #e50914;
    box-shadow: 0 0 0 3px rgba(229, 9, 20, 0.5);
    transform: scale(1.05);
  }

  .channel-logo {
    width: 80px;
    height: 80px;
    object-fit: contain;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 8px;
  }

  .channel-logo-placeholder {
    width: 80px;
    height: 80px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    color: #666;
  }

  .channel-logo-placeholder svg {
    width: 40px;
    height: 40px;
  }

  .channel-name {
    font-size: 0.95rem;
    font-weight: 500;
    text-align: center;
    color: #fff;
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
  }

  /* Channel list view */
  .group-channel-count {
    font-size: 0.9rem;
    color: #888;
    margin-left: auto;
  }

  .channels-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: 20px;
  }

  .channel-list-item {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 12px 16px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid transparent;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.15s ease;
    text-align: left;
  }

  .channel-list-item:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .channel-list-item:focus,
  .channel-list-item:focus-visible {
    outline: none;
    background: rgba(229, 9, 20, 0.2);
    border-color: #e50914;
  }

  .channel-list-logo {
    width: 48px;
    height: 48px;
    object-fit: contain;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    flex-shrink: 0;
  }

  .channel-list-logo-placeholder {
    width: 48px;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #666;
    flex-shrink: 0;
  }

  .channel-list-logo-placeholder svg {
    width: 24px;
    height: 24px;
  }

  .channel-list-name {
    flex: 1;
    font-size: 1rem;
    font-weight: 500;
    color: #fff;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .channel-play-icon {
    width: 24px;
    height: 24px;
    color: #888;
    flex-shrink: 0;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .channel-list-item:hover .channel-play-icon,
  .channel-list-item:focus .channel-play-icon {
    opacity: 1;
    color: #e50914;
  }

  /* Responsive */
  @media (max-width: 1200px) {
    .hero-content {
      padding: 0 60px 80px;
    }

    .hero-title {
      font-size: 3rem;
    }

    .search-view, .genre-view, .category-view {
      padding: 30px 60px;
    }
  }

  @media (max-width: 900px) {
    .main-content {
      margin-left: 60px;
      height: 100vh;
      height: 100dvh;
    }

    .hero-content {
      padding: 0 40px 60px;
      max-width: 500px;
    }

    .hero-title {
      font-size: 2.5rem;
    }

    .hero-description {
      font-size: 1rem;
      -webkit-line-clamp: 2;
    }

    .btn-play, .btn-info {
      padding: 12px 24px;
      font-size: 1rem;
    }

    .search-view, .genre-view, .category-view {
      padding: 24px 40px;
    }
  }

  @media (max-width: 600px) {
    .main-content {
      margin-left: 0;
    }

    .hero {
      height: 70vh;
      min-height: 400px;
    }

    .hero-content {
      padding: 0 24px 50px;
    }

    .hero-title {
      font-size: 2rem;
    }

    .btn-play, .btn-info {
      padding: 12px 20px;
      font-size: 1rem;
      gap: 8px;
    }

    .btn-play svg, .btn-info svg {
      width: 24px;
      height: 24px;
    }

    .content {
      margin-top: 10px;
    }

    .search-view, .genre-view, .category-view {
      padding: 20px 24px;
    }

    .genre-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }

  /* Inline Search Styles */
  .search-section {
    padding: 20px 0;
  }

  .inline-search-form {
    display: flex;
    gap: 12px;
    max-width: 500px;
    margin-bottom: 24px;
  }

  .inline-search-input {
    flex: 1;
    padding: 12px 16px;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 8px;
    color: #fff;
    font-size: 1rem;
    transition: all 0.2s ease;
  }

  .inline-search-input:focus {
    outline: none;
    border-color: #e50914;
    background: rgba(255, 255, 255, 0.15);
  }

  .inline-search-input::placeholder {
    color: rgba(255, 255, 255, 0.5);
  }

  .inline-search-btn {
    padding: 12px 20px;
    background: #e50914;
    border: none;
    border-radius: 8px;
    color: #fff;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.2s ease;
  }

  .inline-search-btn:hover {
    background: #f40612;
  }

  .inline-search-btn:disabled {
    background: #666;
    cursor: not-allowed;
  }

  .inline-search-btn svg {
    width: 20px;
    height: 20px;
  }

  .spinner.small {
    width: 20px;
    height: 20px;
  }

  .search-results-count {
    color: #888;
    margin-bottom: 16px;
    font-size: 0.9rem;
  }

  .no-results {
    color: #888;
    text-align: center;
    padding: 40px 0;
    font-size: 1.1rem;
  }
</style>
