import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { SortingState } from "@tanstack/react-table";
import {
  DATASET_PAGE_SIZE,
  type DatasetInfo,
  type ListDatasetsOptions,
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

function buildDatasetListOptions(params: {
  search: string;
  tags: string[];
  favoriteOnly: boolean;
  statuses: DatasetStatus[];
  sorting: SortingState;
  limit: number;
  offset: number;
}): ListDatasetsOptions {
  const { sortBy, sortDir } = sortingToBackend(params.sorting);
  return {
    search: params.search,
    tags: params.tags,
    favoriteOnly: params.favoriteOnly,
    statuses: params.statuses,
    sortBy,
    sortDir,
    limit: params.limit,
    offset: params.offset,
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

export function useDatasetTableData(): UseDatasetTableDataResult {
  const [datasets, setDatasets] = useState<DatasetInfo[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [selectedStatuses, setSelectedStatuses] = useState<DatasetStatus[]>([]);
  const [favoriteOnly, setFavoriteOnly] = useState(false);
  const [tagFilterQuery, setTagFilterQuery] = useState("");
  const [hasMore, setHasMore] = useState(true);
  const [allTags, setAllTags] = useState<string[]>([]);
  const [sortingState, setSortingState] = useState<SortingState>([
    { id: "id", desc: true },
  ]);

  const datasetsRef = useRef<DatasetInfo[]>([]);
  const searchRef = useRef("");
  const selectedTagsRef = useRef<string[]>([]);
  const selectedStatusesRef = useRef<DatasetStatus[]>([]);
  const favoriteOnlyRef = useRef(false);
  const sortingRef = useRef<SortingState>([{ id: "id", desc: true }]);
  const isLoadingRef = useRef(false);
  const hasMoreRef = useRef(true);
  const searchDebounce = useRef<number | null>(null);
  const statusRefreshTimer = useRef<number | null>(null);

  const setDatasetsState = useCallback((next: DatasetInfo[]) => {
    datasetsRef.current = next;
    setDatasets(next);
  }, []);

  const setHasMoreState = useCallback((next: boolean) => {
    hasMoreRef.current = next;
    setHasMore(next);
  }, []);

  const setIsLoadingState = useCallback((next: boolean) => {
    isLoadingRef.current = next;
  }, []);

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

  useEffect(() => {
    searchRef.current = searchQuery;
  }, [searchQuery]);

  useEffect(() => {
    selectedTagsRef.current = selectedTags;
  }, [selectedTags]);

  useEffect(() => {
    selectedStatusesRef.current = selectedStatuses;
  }, [selectedStatuses]);

  useEffect(() => {
    favoriteOnlyRef.current = favoriteOnly;
  }, [favoriteOnly]);

  useEffect(() => {
    sortingRef.current = sortingState;
  }, [sortingState]);

  const loadTags = useCallback(async () => {
    const tags = await listDatasetTags();
    setAllTags(tags);
  }, []);

  const loadDatasets = useCallback(
    async ({ append = false } = {}) => {
      if (isLoadingRef.current || (append && !hasMoreRef.current)) return;
      setIsLoadingState(true);
      try {
        const offset = append ? datasetsRef.current.length : 0;
        const next = await listDatasets(
          buildDatasetListOptions({
            search: searchRef.current,
            tags: selectedTagsRef.current,
            favoriteOnly: favoriteOnlyRef.current,
            statuses: selectedStatusesRef.current,
            sorting: sortingRef.current,
            limit: DATASET_PAGE_SIZE,
            offset,
          }),
        );
        setHasMoreState(deriveHasMore(next.length, DATASET_PAGE_SIZE));
        if (append) {
          setDatasetsState([...datasetsRef.current, ...next]);
        } else {
          setDatasetsState(next);
        }
      } finally {
        setIsLoadingState(false);
      }
    },
    [setDatasetsState, setHasMoreState, setIsLoadingState],
  );

  const refreshDatasets = useCallback(async () => {
    if (isLoadingRef.current) return;
    setIsLoadingState(true);
    try {
      const limit = Math.max(datasetsRef.current.length, DATASET_PAGE_SIZE);
      const next = await listDatasets(
        buildDatasetListOptions({
          search: searchRef.current,
          tags: selectedTagsRef.current,
          favoriteOnly: favoriteOnlyRef.current,
          statuses: selectedStatusesRef.current,
          sorting: sortingRef.current,
          limit,
          offset: 0,
        }),
      );
      setDatasetsState(next);
      setHasMoreState(deriveHasMore(next.length, limit));
    } finally {
      setIsLoadingState(false);
    }
  }, [setDatasetsState, setHasMoreState, setIsLoadingState]);

  useEffect(() => {
    void loadDatasets();
    void loadTags();

    let unlistenCreated: (() => void) | undefined;
    let unlistenUpdated: (() => void) | undefined;
    let active = true;

    void onDatasetCreated(() => {
      if (!active) return;
      void refreshDatasets();
      void loadTags();
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
      void loadTags();
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
  }, [loadDatasets, loadTags, refreshDatasets]);

  useEffect(() => {
    if (searchDebounce.current) {
      window.clearTimeout(searchDebounce.current);
    }
    searchDebounce.current = window.setTimeout(() => {
      setHasMoreState(true);
      void loadDatasets();
    }, 300);
    return () => {
      if (searchDebounce.current) {
        window.clearTimeout(searchDebounce.current);
      }
    };
  }, [
    favoriteOnly,
    loadDatasets,
    searchQuery,
    selectedStatuses,
    selectedTags,
    setHasMoreState,
    sortingState,
  ]);

  useEffect(() => {
    const hasWriting = datasets.some((dataset) => dataset.status === "Writing");
    if (hasWriting && statusRefreshTimer.current == null) {
      statusRefreshTimer.current = window.setInterval(() => {
        void refreshDatasets();
      }, 2000);
    }
    if (!hasWriting && statusRefreshTimer.current != null) {
      window.clearInterval(statusRefreshTimer.current);
      statusRefreshTimer.current = null;
    }
    return () => {
      if (statusRefreshTimer.current != null) {
        window.clearInterval(statusRefreshTimer.current);
        statusRefreshTimer.current = null;
      }
    };
  }, [datasets, refreshDatasets]);

  const toggleFavorite = useCallback(
    async (dataset: DatasetInfo) => {
      const nextFavorite = !dataset.favorite;
      setDatasetsState(
        datasetsRef.current.map((item) =>
          item.id === dataset.id ? { ...item, favorite: nextFavorite } : item,
        ),
      );
      try {
        await updateDatasetFavorite(dataset.id, nextFavorite);
        await refreshDatasets();
      } catch {
        setDatasetsState(
          datasetsRef.current.map((item) =>
            item.id === dataset.id
              ? { ...item, favorite: dataset.favorite }
              : item,
          ),
        );
      }
    },
    [refreshDatasets, setDatasetsState],
  );

  const filteredTagOptions = useMemo(() => {
    const normalized = tagFilterQuery.trim().toLowerCase();
    if (!normalized) return allTags;
    return allTags.filter((tag) => tag.toLowerCase().includes(normalized));
  }, [allTags, tagFilterQuery]);

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
    await loadDatasets({ append: true });
  }, [loadDatasets]);

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
    hasMore,
    hasActiveFilters,
    toggleFavorite,
    handleTagToggle,
    handleStatusToggle,
    clearFilters,
    loadNextPage,
  };
}
