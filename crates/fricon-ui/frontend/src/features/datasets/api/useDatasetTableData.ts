import { useQuery } from "@tanstack/react-query";
import { listDatasetTags } from "./client";
import { datasetKeys } from "./queryKeys";
import { type UseDatasetTableDataResult } from "./datasetTableShared";
import { useDatasetTableFilters } from "./useDatasetTableFilters";
import { useDatasetTableQuery } from "./useDatasetTableQuery";
import { useDatasetTableRefreshSync } from "./useDatasetTableRefreshSync";
import { useDatasetTableActions } from "./useDatasetTableActions";

export function useDatasetTableData(): UseDatasetTableDataResult {
  const filters = useDatasetTableFilters();
  const { datasetQueryKey, datasets, hasMore, refreshDatasets, loadNextPage } =
    useDatasetTableQuery(
      filters.debouncedQueryParams,
      filters.visibleCount,
      filters.loadNextPage,
    );

  const tagsQuery = useQuery({
    queryKey: datasetKeys.tags(),
    queryFn: listDatasetTags,
  });

  useDatasetTableRefreshSync(datasets, refreshDatasets);
  const actions = useDatasetTableActions({
    datasetQueryKey,
    refreshDatasets,
    removeSelectedTag: filters.removeSelectedTag,
    replaceSelectedTag: filters.replaceSelectedTag,
  });

  const allTags = tagsQuery.data ?? [];

  return {
    datasets,
    searchQuery: filters.searchQuery,
    setSearchQuery: filters.setSearchQuery,
    selectedTags: filters.selectedTags,
    selectedStatuses: filters.selectedStatuses,
    sorting: filters.sorting,
    setSorting: filters.setSorting,
    allTags,
    favoriteOnly: filters.favoriteOnly,
    setFavoriteOnly: filters.setFavoriteOnly,
    hasMore,
    hasActiveFilters: filters.hasActiveFilters,
    toggleFavorite: actions.toggleFavorite,
    deleteDatasets: actions.deleteDatasets,
    isDeleting: actions.isDeleting,
    batchAddTags: actions.batchAddTags,
    batchRemoveTags: actions.batchRemoveTags,
    deleteTag: actions.deleteTag,
    renameTag: actions.renameTag,
    mergeTag: actions.mergeTag,
    isUpdatingTags: actions.isUpdatingTags,
    handleTagToggle: filters.handleTagToggle,
    handleStatusToggle: filters.handleStatusToggle,
    clearFilters: filters.clearFilters,
    loadNextPage,
  };
}
