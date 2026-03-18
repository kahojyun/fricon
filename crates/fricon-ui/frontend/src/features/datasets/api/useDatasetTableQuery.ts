import { useEffect, useEffectEvent } from "react";
import {
  keepPreviousData,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import type { SortingState } from "@tanstack/react-table";
import { listDatasets } from "./client";
import { datasetKeys } from "./queryKeys";
import {
  buildDatasetListOptions,
  deriveHasMore,
  type DatasetQueryKey,
  type DatasetQueryParams,
} from "./datasetTableShared";
import type { DatasetStatus } from "./types";
import { useDebouncedValue } from "../hooks/useDebouncedValue";

interface UseDatasetTableQueryArgs {
  searchQuery: string;
  selectedTags: string[];
  selectedStatuses: DatasetStatus[];
  favoriteOnly: boolean;
  sorting: SortingState;
  visibleCount: number;
  searchTransitionVisibleCount: number | null;
  clearSearchTransition: () => void;
  incrementVisibleCount: () => Promise<void>;
}

export function useDatasetTableQuery(args: UseDatasetTableQueryArgs) {
  const {
    searchQuery,
    selectedTags,
    selectedStatuses,
    favoriteOnly,
    sorting,
    visibleCount,
    searchTransitionVisibleCount,
    clearSearchTransition,
    incrementVisibleCount,
  } = args;
  const queryClient = useQueryClient();
  const debouncedSearchQuery = useDebouncedValue(searchQuery, 300);
  const isSearchPending = searchQuery !== debouncedSearchQuery;
  const clearSearchTransitionEvent = useEffectEvent(() => {
    clearSearchTransition();
  });
  const queryVisibleCount = isSearchPending
    ? (searchTransitionVisibleCount ?? visibleCount)
    : visibleCount;
  const queryParams: DatasetQueryParams = {
    search: debouncedSearchQuery,
    tags: selectedTags,
    favoriteOnly,
    statuses: selectedStatuses,
    sorting,
  };
  const datasetQueryKey: DatasetQueryKey = datasetKeys.list(
    queryParams,
    queryVisibleCount,
  );

  const datasetsQuery = useQuery({
    queryKey: datasetQueryKey,
    queryFn: () =>
      listDatasets(
        buildDatasetListOptions(queryParams, {
          limit: queryVisibleCount,
          offset: 0,
        }),
      ),
    placeholderData: keepPreviousData,
  });

  const datasets = datasetsQuery.data ?? [];

  const refreshDatasets = async () => {
    await queryClient.invalidateQueries({ queryKey: datasetQueryKey });
  };

  const hasMore = datasetsQuery.isPlaceholderData
    ? true
    : deriveHasMore(datasets.length, queryVisibleCount);

  const loadNextPage = () => {
    if (isSearchPending || datasetsQuery.isFetching || !hasMore) {
      return Promise.resolve();
    }
    return incrementVisibleCount();
  };

  useEffect(() => {
    if (!isSearchPending && searchTransitionVisibleCount !== null) {
      clearSearchTransitionEvent();
    }
  }, [isSearchPending, searchTransitionVisibleCount]);

  return {
    datasetQueryKey,
    datasets,
    hasMore,
    refreshDatasets,
    loadNextPage,
  };
}
