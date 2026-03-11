import type { SortingState } from "@tanstack/react-table";
import {
  type DatasetInfo,
  type DatasetListSortBy,
  type DatasetStatus,
} from "./types";
import { datasetKeys } from "./queryKeys";

export function deriveHasMore(
  receivedCount: number,
  requestedLimit: number,
): boolean {
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

export interface DatasetQueryParams {
  search: string;
  tags: string[];
  favoriteOnly: boolean;
  statuses: DatasetStatus[];
  sorting: SortingState;
}

export function buildDatasetListOptions(
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

function areStringArraysEqual(a: string[], b: string[]): boolean {
  return a.length === b.length && a.every((value, index) => value === b[index]);
}

function areSortingStatesEqual(a: SortingState, b: SortingState): boolean {
  if (a.length !== b.length) return false;
  return a.every((entry, index) => {
    const other = b[index];
    return entry.id === other?.id && entry.desc === other?.desc;
  });
}

export function areDatasetQueryParamsEqual(
  a: DatasetQueryParams,
  b: DatasetQueryParams,
): boolean {
  return (
    a.search === b.search &&
    a.favoriteOnly === b.favoriteOnly &&
    areStringArraysEqual(a.tags, b.tags) &&
    areStringArraysEqual(a.statuses, b.statuses) &&
    areSortingStatesEqual(a.sorting, b.sorting)
  );
}

export type DatasetQueryKey = ReturnType<typeof datasetKeys.list>;

export interface UseDatasetTableDataResult {
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
