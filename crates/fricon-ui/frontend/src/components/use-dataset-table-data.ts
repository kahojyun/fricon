import { useCallback, useEffect, useMemo, useState } from "react";
import {
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import type { SortingState } from "@tanstack/react-table";
import {
  DATASET_PAGE_SIZE,
  type DatasetInfo,
  type DatasetListSortBy,
  type DatasetStatus,
  listDatasetTags,
  listDatasets,
  onDatasetCreated,
  onDatasetUpdated,
  updateDatasetFavorite,
} from "@/lib/backend";

function deriveHasMore(receivedCount: number, requestedLimit: number): boolean {
  return receivedCount >= requestedLimit;
}

function sortingToBackend(sorting: SortingState): {
  sortBy?: DatasetListSortBy;
  sortDir?: "asc" | "desc";
} {
  const current = sorting[0];
  if (!current) return {};
  if (
    current.id !== "id" &&
    current.id !== "name" &&
    current.id !== "createdAt"
  ) {
    return {};
  }
  return {
    sortBy: current.id,
    sortDir: current.desc ? "desc" : "asc",
  };
}

interface DatasetQueryParams {
  search: string;
  tags: string[];
  favoriteOnly: boolean;
  statuses: DatasetStatus[];
  sorting: SortingState;
}

function buildDatasetListOptions(
  params: DatasetQueryParams,
  pagination: { limit: number; offset: number },
) {
  const { sortBy, sortDir } = sortingToBackend(params.sorting);
  return {
    search: params.search,
    tags: params.tags,
    favoriteOnly: params.favoriteOnly,
    statuses: params.statuses,
    sortBy,
    sortDir,
    limit: pagination.limit,
    offset: pagination.offset,
  };
}

interface UseDatasetTableDataResult {
  datasets: DatasetInfo[];
  searchQuery: string;
  setSearchQuery: (next: string) => void;
  selectedTags: string[];
  selectedStatuses: DatasetStatus[];
  tagFilterQuery: string;
  setTagFilterQuery: (next: string) => void;
  sorting: SortingState;
  setSorting: (
    updater: SortingState | ((prev: SortingState) => SortingState),
  ) => void;
  filteredTagOptions: string[];
  favoriteOnly: boolean;
  setFavoriteOnly: (next: boolean) => void;
  hasMore: boolean;
  hasActiveFilters: boolean;
  toggleFavorite: (dataset: DatasetInfo) => Promise<void>;
  handleTagToggle: (tag: string) => void;
  handleStatusToggle: (status: DatasetStatus) => void;
  clearFilters: () => void;
  loadNextPage: () => Promise<void>;
}

const DEFAULT_SORTING: SortingState = [{ id: "id", desc: true }];

interface DatasetTableFiltersResult {
  searchQuery: string;
  setSearchQuery: (next: string) => void;
  selectedTags: string[];
  selectedStatuses: DatasetStatus[];
  favoriteOnly: boolean;
  setFavoriteOnly: (next: boolean) => void;
  tagFilterQuery: string;
  setTagFilterQuery: (next: string) => void;
  sorting: SortingState;
  setSorting: (
    updater: SortingState | ((prev: SortingState) => SortingState),
  ) => void;
  debouncedQueryParams: DatasetQueryParams;
  handleTagToggle: (tag: string) => void;
  handleStatusToggle: (status: DatasetStatus) => void;
  clearFilters: () => void;
  hasActiveFilters: boolean;
}

function useDatasetTableFilters(): DatasetTableFiltersResult {
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [selectedStatuses, setSelectedStatuses] = useState<DatasetStatus[]>([]);
  const [favoriteOnly, setFavoriteOnly] = useState(false);
  const [tagFilterQuery, setTagFilterQuery] = useState("");
  const [sortingState, setSortingState] =
    useState<SortingState>(DEFAULT_SORTING);
  const [debouncedQueryParams, setDebouncedQueryParams] =
    useState<DatasetQueryParams>(() => ({
      search: searchQuery,
      tags: selectedTags,
      favoriteOnly,
      statuses: selectedStatuses,
      sorting: sortingState,
    }));

  useEffect(() => {
    const timer = window.setTimeout(() => {
      setDebouncedQueryParams({
        search: searchQuery,
        tags: selectedTags,
        favoriteOnly,
        statuses: selectedStatuses,
        sorting: sortingState,
      });
    }, 300);
    return () => {
      window.clearTimeout(timer);
    };
  }, [searchQuery, selectedTags, favoriteOnly, selectedStatuses, sortingState]);

  const setSorting = (
    updater: SortingState | ((prev: SortingState) => SortingState),
  ) => {
    setSortingState((prev) => {
      const next = typeof updater === "function" ? updater(prev) : updater;
      const first = next[0];
      return first ? [first] : [];
    });
  };

  const handleTagToggle = (tag: string) => {
    setSelectedTags((prev) =>
      prev.includes(tag) ? prev.filter((item) => item !== tag) : [...prev, tag],
    );
  };

  const handleStatusToggle = (status: DatasetStatus) => {
    setSelectedStatuses((prev) =>
      prev.includes(status)
        ? prev.filter((item) => item !== status)
        : [...prev, status],
    );
  };

  const clearFilters = () => {
    setSearchQuery("");
    setSelectedTags([]);
    setSelectedStatuses([]);
    setFavoriteOnly(false);
    setTagFilterQuery("");
  };

  const hasActiveFilters =
    searchQuery.trim().length > 0 ||
    favoriteOnly ||
    selectedTags.length > 0 ||
    selectedStatuses.length > 0;

  return {
    searchQuery,
    setSearchQuery,
    selectedTags,
    selectedStatuses,
    favoriteOnly,
    setFavoriteOnly,
    tagFilterQuery,
    setTagFilterQuery,
    sorting: sortingState,
    setSorting,
    debouncedQueryParams,
    handleTagToggle,
    handleStatusToggle,
    clearFilters,
    hasActiveFilters,
  };
}

function useDatasetTableQuery(
  queryParams: DatasetQueryParams,
  queryClient: ReturnType<typeof useQueryClient>,
) {
  const [visibleCount, setVisibleCount] = useState(DATASET_PAGE_SIZE);

  const datasetQueryKey = useMemo(
    () => ["datasets", "list", queryParams, visibleCount] as const,
    [queryParams, visibleCount],
  );

  const datasetsQuery = useQuery({
    queryKey: datasetQueryKey,
    queryFn: () =>
      listDatasets(
        buildDatasetListOptions(queryParams, {
          limit: visibleCount,
          offset: 0,
        }),
      ),
  });

  const datasets = datasetsQuery.data ?? [];

  const refreshDatasets = useCallback(async () => {
    await queryClient.invalidateQueries({ queryKey: datasetQueryKey });
  }, [datasetQueryKey, queryClient]);

  const hasMore = deriveHasMore(datasets.length, visibleCount);

  const loadNextPage = () => {
    if (datasetsQuery.isFetching || !hasMore) return Promise.resolve();
    setVisibleCount((current) => current + DATASET_PAGE_SIZE);
    return Promise.resolve();
  };

  return {
    datasetQueryKey,
    datasetsQuery,
    datasets,
    hasMore,
    refreshDatasets,
    loadNextPage,
  };
}

function useDatasetTableRefreshSync(
  datasets: DatasetInfo[],
  refreshDatasets: () => Promise<void>,
  queryClient: ReturnType<typeof useQueryClient>,
) {
  useEffect(() => {
    let unlistenCreated: (() => void) | undefined;
    let unlistenUpdated: (() => void) | undefined;
    let active = true;

    void onDatasetCreated(() => {
      if (!active) return;
      void refreshDatasets();
      void queryClient.invalidateQueries({ queryKey: ["datasetTags"] });
    }).then((unlisten) => {
      if (!active) {
        unlisten();
        return;
      }
      unlistenCreated = unlisten;
    });

    void onDatasetUpdated(() => {
      if (!active) return;
      void refreshDatasets();
      void queryClient.invalidateQueries({ queryKey: ["datasetTags"] });
    }).then((unlisten) => {
      if (!active) {
        unlisten();
        return;
      }
      unlistenUpdated = unlisten;
    });

    return () => {
      active = false;
      unlistenCreated?.();
      unlistenUpdated?.();
    };
  }, [queryClient, refreshDatasets]);

  useEffect(() => {
    const hasWriting = datasets.some((dataset) => dataset.status === "Writing");
    if (!hasWriting) return;

    const timer = window.setInterval(() => {
      void refreshDatasets();
    }, 2000);

    return () => {
      window.clearInterval(timer);
    };
  }, [datasets, refreshDatasets]);
}

function useDatasetFavoriteMutation(
  datasetQueryKey: readonly ["datasets", "list", DatasetQueryParams, number],
  refreshDatasets: () => Promise<void>,
  queryClient: ReturnType<typeof useQueryClient>,
) {
  const favoriteMutation = useMutation({
    mutationFn: ({ id, favorite }: { id: number; favorite: boolean }) =>
      updateDatasetFavorite(id, favorite),
  });

  const toggleFavorite = async (dataset: DatasetInfo) => {
    const nextFavorite = !dataset.favorite;
    const previousData = queryClient.getQueryData<DatasetInfo[]>(datasetQueryKey);

    queryClient.setQueryData<DatasetInfo[]>(datasetQueryKey, (current) => {
      if (!current) return current;
      return current.map((item) =>
        item.id === dataset.id ? { ...item, favorite: nextFavorite } : item,
      );
    });

    try {
      await favoriteMutation.mutateAsync({
        id: dataset.id,
        favorite: nextFavorite,
      });
    } catch {
      queryClient.setQueryData(datasetQueryKey, previousData);
      return;
    }

    try {
      await refreshDatasets();
    } catch {
      // Keep optimistic state if backend write succeeded but refresh failed.
    }
  };

  return { toggleFavorite };
}

export function useDatasetTableData(): UseDatasetTableDataResult {
  const queryClient = useQueryClient();
  const filters = useDatasetTableFilters();
  const {
    datasetQueryKey,
    datasets,
    hasMore,
    refreshDatasets,
    loadNextPage,
  } =
    useDatasetTableQuery(filters.debouncedQueryParams, queryClient);

  const tagsQuery = useQuery({
    queryKey: ["datasetTags"],
    queryFn: listDatasetTags,
  });

  useDatasetTableRefreshSync(datasets, refreshDatasets, queryClient);
  const { toggleFavorite } = useDatasetFavoriteMutation(
    datasetQueryKey,
    refreshDatasets,
    queryClient,
  );

  const allTags = tagsQuery.data ?? [];
  const normalizedTagFilterQuery = filters.tagFilterQuery.trim().toLowerCase();
  const filteredTagOptions = normalizedTagFilterQuery
    ? allTags.filter((tag) =>
        tag.toLowerCase().includes(normalizedTagFilterQuery),
      )
    : allTags;

  return {
    datasets,
    searchQuery: filters.searchQuery,
    setSearchQuery: filters.setSearchQuery,
    selectedTags: filters.selectedTags,
    selectedStatuses: filters.selectedStatuses,
    tagFilterQuery: filters.tagFilterQuery,
    setTagFilterQuery: filters.setTagFilterQuery,
    sorting: filters.sorting,
    setSorting: filters.setSorting,
    filteredTagOptions,
    favoriteOnly: filters.favoriteOnly,
    setFavoriteOnly: filters.setFavoriteOnly,
    hasMore,
    hasActiveFilters: filters.hasActiveFilters,
    toggleFavorite,
    handleTagToggle: filters.handleTagToggle,
    handleStatusToggle: filters.handleStatusToggle,
    clearFilters: filters.clearFilters,
    loadNextPage,
  };
}
