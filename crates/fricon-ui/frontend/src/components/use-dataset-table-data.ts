import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { ColumnFiltersState, SortingState } from "@tanstack/react-table";
import {
  DATASET_PAGE_SIZE,
  type DatasetInfo,
  listDatasets,
  onDatasetCreated,
  onDatasetUpdated,
  updateDatasetFavorite,
} from "@/lib/backend";

function deriveHasMore(receivedCount: number, requestedLimit: number): boolean {
  return receivedCount >= requestedLimit;
}

interface UseDatasetTableDataResult {
  datasets: DatasetInfo[];
  searchQuery: string;
  setSearchQuery: (next: string) => void;
  selectedTags: string[];
  tagFilterQuery: string;
  setTagFilterQuery: (next: string) => void;
  sorting: SortingState;
  setSorting: (
    updater: SortingState | ((prev: SortingState) => SortingState),
  ) => void;
  columnFilters: ColumnFiltersState;
  setColumnFilters: (
    updater:
      | ColumnFiltersState
      | ((prev: ColumnFiltersState) => ColumnFiltersState),
  ) => void;
  filteredTagOptions: string[];
  favoriteOnly: boolean;
  hasMore: boolean;
  hasActiveFilters: boolean;
  toggleFavorite: (dataset: DatasetInfo) => Promise<void>;
  handleTagToggle: (tag: string) => void;
  clearFilters: () => void;
  loadNextPage: () => Promise<void>;
}

export function useDatasetTableData(): UseDatasetTableDataResult {
  const [datasets, setDatasets] = useState<DatasetInfo[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [tagFilterQuery, setTagFilterQuery] = useState("");
  const [hasMore, setHasMore] = useState(true);
  const [sorting, setSorting] = useState<SortingState>([
    { id: "createdAt", desc: true },
  ]);
  const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([]);

  const datasetsRef = useRef<DatasetInfo[]>([]);
  const searchRef = useRef("");
  const selectedTagsRef = useRef<string[]>([]);
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

  useEffect(() => {
    searchRef.current = searchQuery;
  }, [searchQuery]);

  useEffect(() => {
    selectedTagsRef.current = selectedTags;
  }, [selectedTags]);

  const loadDatasets = useCallback(
    async ({ append = false } = {}) => {
      if (isLoadingRef.current || (append && !hasMoreRef.current)) return;
      setIsLoadingState(true);
      try {
        const offset = append ? datasetsRef.current.length : 0;
        const next = await listDatasets(
          searchRef.current,
          selectedTagsRef.current,
          DATASET_PAGE_SIZE,
          offset,
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
        searchRef.current,
        selectedTagsRef.current,
        limit,
        0,
      );
      setDatasetsState(next);
      setHasMoreState(deriveHasMore(next.length, limit));
    } finally {
      setIsLoadingState(false);
    }
  }, [setDatasetsState, setHasMoreState, setIsLoadingState]);

  useEffect(() => {
    void loadDatasets();

    let unlistenCreated: (() => void) | undefined;
    let unlistenUpdated: (() => void) | undefined;
    let active = true;

    void onDatasetCreated((event) => {
      if (!active) return;
      setDatasetsState([event, ...datasetsRef.current]);
      if (searchRef.current.trim() || selectedTagsRef.current.length > 0) {
        void loadDatasets();
      }
    }).then((unlisten) => {
      if (!active) {
        unlisten();
        return;
      }
      unlistenCreated = unlisten;
    });

    void onDatasetUpdated((event) => {
      if (!active) return;
      const next = [...datasetsRef.current];
      const index = next.findIndex((dataset) => dataset.id === event.id);
      if (index >= 0) {
        next[index] = event;
        setDatasetsState(next);
        return;
      }
      if (!searchRef.current.trim() && selectedTagsRef.current.length === 0) {
        setDatasetsState([event, ...next]);
      }
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
  }, [loadDatasets, setDatasetsState]);

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
  }, [loadDatasets, searchQuery, selectedTags, setHasMoreState]);

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
    [setDatasetsState],
  );

  const tagOptions = useMemo(() => {
    const tagSet = new Set<string>();
    datasets.forEach((dataset) => {
      dataset.tags.forEach((tag) => tagSet.add(tag));
    });
    return Array.from(tagSet).sort((a, b) => a.localeCompare(b));
  }, [datasets]);

  const filteredTagOptions = useMemo(() => {
    const normalized = tagFilterQuery.trim().toLowerCase();
    if (!normalized) return tagOptions;
    return tagOptions.filter((tag) => tag.toLowerCase().includes(normalized));
  }, [tagFilterQuery, tagOptions]);

  const favoriteOnly =
    columnFilters.find((filter) => filter.id === "favorite")?.value === true;
  const hasActiveFilters =
    searchQuery.trim().length > 0 || favoriteOnly || selectedTags.length > 0;

  const handleTagToggle = useCallback((tag: string) => {
    setSelectedTags((prev) =>
      prev.includes(tag) ? prev.filter((item) => item !== tag) : [...prev, tag],
    );
  }, []);

  const clearFilters = useCallback(() => {
    setSearchQuery("");
    setSelectedTags([]);
    setTagFilterQuery("");
    setColumnFilters((prev) =>
      prev.filter((filter) => filter.id !== "favorite"),
    );
  }, []);

  const loadNextPage = useCallback(async () => {
    await loadDatasets({ append: true });
  }, [loadDatasets]);

  return {
    datasets,
    searchQuery,
    setSearchQuery,
    selectedTags,
    tagFilterQuery,
    setTagFilterQuery,
    sorting,
    setSorting,
    columnFilters,
    setColumnFilters,
    filteredTagOptions,
    favoriteOnly,
    hasMore,
    hasActiveFilters,
    toggleFavorite,
    handleTagToggle,
    clearFilters,
    loadNextPage,
  };
}
