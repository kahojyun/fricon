import { useEffect, useReducer } from "react";
import {
  keepPreviousData,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import type { SortingState } from "@tanstack/react-table";
import { listDatasets, listDatasetTags } from "./client";
import { datasetKeys } from "./queryKeys";
import {
  buildDatasetListOptions,
  deriveHasMore,
  type DatasetQueryKey,
  type DatasetQueryParams,
  type UseDatasetTableDataResult,
} from "./datasetTableShared";
import { useDatasetTableRefreshSync } from "./useDatasetTableRefreshSync";
import { useDatasetTableActions } from "./useDatasetTableActions";
import type { DatasetStatus } from "./types";
import {
  createInitialDatasetTableState,
  datasetTableStateReducer,
} from "../model/datasetTableStateReducer";
import { useDebouncedValue } from "../hooks/useDebouncedValue";

export function useDatasetTableData(): UseDatasetTableDataResult {
  const [state, dispatch] = useReducer(
    datasetTableStateReducer,
    undefined,
    createInitialDatasetTableState,
  );
  const debouncedSearchInput = useDebouncedValue(state.searchInput, 300);

  useEffect(() => {
    if (debouncedSearchInput !== state.appliedSearchQuery) {
      dispatch({ type: "commit_search_input" });
    }
  }, [debouncedSearchInput, state.appliedSearchQuery]);

  const queryClient = useQueryClient();
  const datasetQueryParams: DatasetQueryParams = {
    search: state.appliedSearchQuery,
    tags: state.activeTags,
    favoriteOnly: state.showFavoritesOnly,
    statuses: state.activeStatuses,
    viewMode: state.viewMode,
    sorting: state.sorting,
  };
  const datasetQueryKey: DatasetQueryKey = datasetKeys.list(
    datasetQueryParams,
    state.queryLimit,
  );
  const datasetsQuery = useQuery({
    queryKey: datasetQueryKey,
    queryFn: () =>
      listDatasets(
        buildDatasetListOptions(datasetQueryParams, {
          limit: state.queryLimit,
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
    : deriveHasMore(datasets.length, state.queryLimit);

  const setSearchInput = (next: string) => {
    dispatch({ type: "set_search_input", next });
  };

  const setViewMode = (next: "active" | "trash") => {
    dispatch({ type: "set_view_mode", next });
  };

  const setSorting = (
    updater: SortingState | ((prev: SortingState) => SortingState),
  ) => {
    dispatch({ type: "set_sorting", updater });
  };

  const handleTagToggle = (tag: string) => {
    dispatch({ type: "toggle_tag", tag });
  };

  const handleStatusToggle = (status: DatasetStatus) => {
    dispatch({ type: "toggle_status", status });
  };

  const clearFilters = () => {
    dispatch({ type: "clear_filters" });
  };

  const removeActiveTag = (tag: string) => {
    dispatch({ type: "remove_active_tag", tag });
  };

  const replaceActiveTag = (oldName: string, newName: string) => {
    dispatch({ type: "replace_active_tag", oldName, newName });
  };

  const setShowFavoritesOnly = (next: boolean) => {
    dispatch({ type: "set_show_favorites_only", next });
  };

  const loadNextPage = () => {
    if (
      state.searchInput !== state.appliedSearchQuery ||
      datasetsQuery.isFetching ||
      !hasMore
    ) {
      return Promise.resolve();
    }

    dispatch({ type: "load_next_page" });
    return Promise.resolve();
  };

  const tagsQuery = useQuery({
    queryKey: datasetKeys.tags(),
    queryFn: listDatasetTags,
  });

  useDatasetTableRefreshSync(datasets, refreshDatasets);
  const actions = useDatasetTableActions({
    datasetQueryKey,
    refreshDatasets,
    removeActiveTag,
    replaceActiveTag,
  });

  const allTags = tagsQuery.data ?? [];

  return {
    datasets,
    viewMode: state.viewMode,
    setViewMode,
    searchInput: state.searchInput,
    setSearchInput,
    activeTags: state.activeTags,
    activeStatuses: state.activeStatuses,
    sorting: state.sorting,
    setSorting,
    allTags,
    showFavoritesOnly: state.showFavoritesOnly,
    setShowFavoritesOnly,
    hasMore,
    hasActiveFilters:
      state.searchInput.trim().length > 0 ||
      state.showFavoritesOnly ||
      state.activeTags.length > 0 ||
      state.activeStatuses.length > 0,
    toggleFavorite: actions.toggleFavorite,
    trashDatasets: actions.trashDatasets,
    restoreDatasets: actions.restoreDatasets,
    deleteDatasets: actions.deleteDatasets,
    emptyTrash: actions.emptyTrash,
    isMutatingDatasets: actions.isMutatingDatasets,
    batchAddTags: actions.batchAddTags,
    batchRemoveTags: actions.batchRemoveTags,
    deleteTag: actions.deleteTag,
    renameTag: actions.renameTag,
    mergeTag: actions.mergeTag,
    isUpdatingTags: actions.isUpdatingTags,
    handleTagToggle,
    handleStatusToggle,
    clearFilters,
    loadNextPage,
  };
}
