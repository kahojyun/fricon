import type { SortingState } from "@tanstack/react-table";
import {
  type DatasetDeleteResult,
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

export type DatasetQueryKey = ReturnType<typeof datasetKeys.list>;

export interface UseDatasetTableDataResult {
  datasets: DatasetInfo[];
  searchInput: string;
  setSearchInput: (next: string) => void;
  activeTags: string[];
  activeStatuses: DatasetStatus[];
  sorting: SortingState;
  setSorting: (
    updater: SortingState | ((prev: SortingState) => SortingState),
  ) => void;
  allTags: string[];
  showFavoritesOnly: boolean;
  setShowFavoritesOnly: (next: boolean) => void;
  hasMore: boolean;
  hasActiveFilters: boolean;
  toggleFavorite: (dataset: DatasetInfo) => Promise<void>;
  deleteDatasets: (ids: number[]) => Promise<DatasetDeleteResult[]>;
  isDeleting: boolean;
  batchAddTags: (
    ids: number[],
    tags: string[],
  ) => Promise<DatasetDeleteResult[]>;
  batchRemoveTags: (
    ids: number[],
    tags: string[],
  ) => Promise<DatasetDeleteResult[]>;
  deleteTag: (tag: string) => Promise<void>;
  renameTag: (oldName: string, newName: string) => Promise<void>;
  mergeTag: (source: string, target: string) => Promise<void>;
  isUpdatingTags: boolean;
  handleTagToggle: (tag: string) => void;
  handleStatusToggle: (status: DatasetStatus) => void;
  clearFilters: () => void;
  loadNextPage: () => Promise<void>;
}
