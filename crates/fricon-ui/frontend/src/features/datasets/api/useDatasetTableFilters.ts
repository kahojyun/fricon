import { useEffect, useRef, useState } from "react";
import type { SortingState } from "@tanstack/react-table";
import { DATASET_PAGE_SIZE, type DatasetStatus } from "./types";
import {
  areDatasetQueryParamsEqual,
  type DatasetQueryParams,
} from "./datasetTableShared";

interface DatasetTableFiltersResult {
  searchQuery: string;
  setSearchQuery: (next: string) => void;
  selectedTags: string[];
  selectedStatuses: DatasetStatus[];
  favoriteOnly: boolean;
  setFavoriteOnly: (next: boolean) => void;
  sorting: SortingState;
  setSorting: (
    updater: SortingState | ((prev: SortingState) => SortingState),
  ) => void;
  debouncedQueryParams: DatasetQueryParams;
  visibleCount: number;
  loadNextPage: () => Promise<void>;
  handleTagToggle: (tag: string) => void;
  handleStatusToggle: (status: DatasetStatus) => void;
  removeSelectedTag: (tag: string) => void;
  replaceSelectedTag: (oldName: string, newName: string) => void;
  clearFilters: () => void;
  hasActiveFilters: boolean;
}

const DEFAULT_SORTING: SortingState = [{ id: "id", desc: true }];

export function useDatasetTableFilters(): DatasetTableFiltersResult {
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [selectedStatuses, setSelectedStatuses] = useState<DatasetStatus[]>([]);
  const [favoriteOnly, setFavoriteOnly] = useState(false);
  const [sortingState, setSortingState] =
    useState<SortingState>(DEFAULT_SORTING);
  const [visibleCount, setVisibleCount] = useState(DATASET_PAGE_SIZE);
  const [debouncedQueryParams, setDebouncedQueryParams] =
    useState<DatasetQueryParams>(() => ({
      search: searchQuery,
      tags: selectedTags,
      favoriteOnly,
      statuses: selectedStatuses,
      sorting: sortingState,
    }));
  const latestDebouncedQueryParamsRef = useRef(debouncedQueryParams);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      const nextDebouncedQueryParams = {
        search: searchQuery,
        tags: selectedTags,
        favoriteOnly,
        statuses: selectedStatuses,
        sorting: sortingState,
      };
      const didQueryParamsChange = !areDatasetQueryParamsEqual(
        latestDebouncedQueryParamsRef.current,
        nextDebouncedQueryParams,
      );

      setDebouncedQueryParams(nextDebouncedQueryParams);
      latestDebouncedQueryParamsRef.current = nextDebouncedQueryParams;

      if (didQueryParamsChange) {
        setVisibleCount(DATASET_PAGE_SIZE);
      }
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
  };

  const removeSelectedTag = (tag: string) => {
    setSelectedTags((prev) => prev.filter((item) => item !== tag));
  };

  const replaceSelectedTag = (oldName: string, newName: string) => {
    setSelectedTags((prev) => {
      if (!prev.includes(oldName)) {
        return prev;
      }

      const next = prev.map((item) => (item === oldName ? newName : item));
      return next.filter((item, index) => next.indexOf(item) === index);
    });
  };

  const hasActiveFilters =
    searchQuery.trim().length > 0 ||
    favoriteOnly ||
    selectedTags.length > 0 ||
    selectedStatuses.length > 0;

  const loadNextPage = () => {
    setVisibleCount((current) => current + DATASET_PAGE_SIZE);
    return Promise.resolve();
  };

  return {
    searchQuery,
    setSearchQuery,
    selectedTags,
    selectedStatuses,
    favoriteOnly,
    setFavoriteOnly,
    sorting: sortingState,
    setSorting,
    debouncedQueryParams,
    visibleCount,
    loadNextPage,
    handleTagToggle,
    handleStatusToggle,
    removeSelectedTag,
    replaceSelectedTag,
    clearFilters,
    hasActiveFilters,
  };
}
