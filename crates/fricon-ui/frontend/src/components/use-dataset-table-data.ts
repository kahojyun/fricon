import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  useInfiniteQuery,
  useMutation,
  useQuery,
  useQueryClient,
  type InfiniteData,
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

export function useDatasetTableData(): UseDatasetTableDataResult {
  const queryClient = useQueryClient();
  const pendingRefreshRef = useRef(false);
  const isRefreshingRef = useRef(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [selectedStatuses, setSelectedStatuses] = useState<DatasetStatus[]>([]);
  const [favoriteOnly, setFavoriteOnly] = useState(false);
  const [tagFilterQuery, setTagFilterQuery] = useState("");
  const [sortingState, setSortingState] =
    useState<SortingState>(DEFAULT_SORTING);

  const setSorting = useCallback(
    (updater: SortingState | ((prev: SortingState) => SortingState)) => {
      setSortingState((prev) => {
        const next = typeof updater === "function" ? updater(prev) : updater;
        const first = next[0];
        return first ? [first] : [];
      });
    },
    [],
  );

  const currentQueryParams = useMemo<DatasetQueryParams>(
    () => ({
      search: searchQuery,
      tags: selectedTags,
      favoriteOnly,
      statuses: selectedStatuses,
      sorting: sortingState,
    }),
    [favoriteOnly, searchQuery, selectedStatuses, selectedTags, sortingState],
  );

  const [debouncedQueryParams, setDebouncedQueryParams] =
    useState<DatasetQueryParams>(currentQueryParams);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      setDebouncedQueryParams(currentQueryParams);
    }, 300);
    return () => {
      window.clearTimeout(timer);
    };
  }, [currentQueryParams]);

  const datasetQueryKey = useMemo(
    () => ["datasets", debouncedQueryParams] as const,
    [debouncedQueryParams],
  );

  const datasetsQuery = useInfiniteQuery({
    queryKey: datasetQueryKey,
    queryFn: ({ pageParam }) =>
      listDatasets(
        buildDatasetListOptions(debouncedQueryParams, {
          limit: DATASET_PAGE_SIZE,
          offset: pageParam,
        }),
      ),
    initialPageParam: 0,
    getNextPageParam: (lastPage, allPages) => {
      if (!deriveHasMore(lastPage.length, DATASET_PAGE_SIZE)) {
        return undefined;
      }
      return allPages.reduce((total, page) => total + page.length, 0);
    },
  });

  const datasets = useMemo(
    () => datasetsQuery.data?.pages.flat() ?? [],
    [datasetsQuery.data],
  );

  const refreshDatasets = useCallback(async () => {
    if (
      isRefreshingRef.current ||
      queryClient.isFetching({ queryKey: datasetQueryKey }) > 0
    ) {
      pendingRefreshRef.current = true;
      return;
    }

    do {
      pendingRefreshRef.current = false;
      isRefreshingRef.current = true;
      try {
        const limit = Math.max(datasets.length, DATASET_PAGE_SIZE);
        const next = await listDatasets(
          buildDatasetListOptions(debouncedQueryParams, { limit, offset: 0 }),
        );
        queryClient.setQueryData<InfiniteData<DatasetInfo[], number>>(
          datasetQueryKey,
          {
            pages: [next],
            pageParams: [0],
          },
        );
      } finally {
        isRefreshingRef.current = false;
      }
    } while (pendingRefreshRef.current);
  }, [datasetQueryKey, datasets.length, debouncedQueryParams, queryClient]);

  useEffect(() => {
    if (!pendingRefreshRef.current) return;
    if (datasetsQuery.isFetching || isRefreshingRef.current) return;
    void refreshDatasets();
  }, [datasetsQuery.isFetching, refreshDatasets]);

  const tagsQuery = useQuery({
    queryKey: ["datasetTags"],
    queryFn: listDatasetTags,
  });

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

  const favoriteMutation = useMutation({
    mutationFn: ({ id, favorite }: { id: number; favorite: boolean }) =>
      updateDatasetFavorite(id, favorite),
  });

  const toggleFavorite = useCallback(
    async (dataset: DatasetInfo) => {
      const nextFavorite = !dataset.favorite;
      const previousData =
        queryClient.getQueryData<InfiniteData<DatasetInfo[], number>>(
          datasetQueryKey,
        );

      queryClient.setQueryData<InfiniteData<DatasetInfo[], number>>(
        datasetQueryKey,
        (current) => {
          if (!current) return current;
          return {
            ...current,
            pages: current.pages.map((page) =>
              page.map((item) =>
                item.id === dataset.id
                  ? { ...item, favorite: nextFavorite }
                  : item,
              ),
            ),
          };
        },
      );

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
    },
    [datasetQueryKey, favoriteMutation, queryClient, refreshDatasets],
  );

  const filteredTagOptions = useMemo(() => {
    const allTags = tagsQuery.data ?? [];
    const normalized = tagFilterQuery.trim().toLowerCase();
    if (!normalized) return allTags;
    return allTags.filter((tag) => tag.toLowerCase().includes(normalized));
  }, [tagFilterQuery, tagsQuery.data]);

  const hasActiveFilters =
    searchQuery.trim().length > 0 ||
    favoriteOnly ||
    selectedTags.length > 0 ||
    selectedStatuses.length > 0;

  const handleTagToggle = useCallback((tag: string) => {
    setSelectedTags((prev) =>
      prev.includes(tag) ? prev.filter((item) => item !== tag) : [...prev, tag],
    );
  }, []);

  const handleStatusToggle = useCallback((status: DatasetStatus) => {
    setSelectedStatuses((prev) =>
      prev.includes(status)
        ? prev.filter((item) => item !== status)
        : [...prev, status],
    );
  }, []);

  const clearFilters = useCallback(() => {
    setSearchQuery("");
    setSelectedTags([]);
    setSelectedStatuses([]);
    setFavoriteOnly(false);
    setTagFilterQuery("");
  }, []);

  const loadNextPage = useCallback(async () => {
    if (!datasetsQuery.hasNextPage || datasetsQuery.isFetchingNextPage) return;
    await datasetsQuery.fetchNextPage();
  }, [datasetsQuery]);

  return {
    datasets,
    searchQuery,
    setSearchQuery,
    selectedTags,
    selectedStatuses,
    tagFilterQuery,
    setTagFilterQuery,
    sorting: sortingState,
    setSorting,
    filteredTagOptions,
    favoriteOnly,
    setFavoriteOnly,
    hasMore: Boolean(datasetsQuery.hasNextPage),
    hasActiveFilters,
    toggleFavorite,
    handleTagToggle,
    handleStatusToggle,
    clearFilters,
    loadNextPage,
  };
}
