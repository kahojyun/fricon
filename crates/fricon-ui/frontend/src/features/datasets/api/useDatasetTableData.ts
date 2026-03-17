import { useQuery } from "@tanstack/react-query";
import { listDatasetTags } from "./client";
import { datasetKeys } from "./queryKeys";
import { type UseDatasetTableDataResult } from "./datasetTableShared";
import { useDatasetFavoriteMutation } from "./useDatasetFavoriteMutation";
import { useDatasetDeleteMutation } from "./useDatasetDeleteMutation";
import { useDatasetTagMutation } from "./useDatasetTagMutation";
import { useDatasetTableFilters } from "./useDatasetTableFilters";
import { useDatasetTableQuery } from "./useDatasetTableQuery";
import { useDatasetTableRefreshSync } from "./useDatasetTableRefreshSync";

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
  const { toggleFavorite } = useDatasetFavoriteMutation(
    datasetQueryKey,
    refreshDatasets,
  );
  const { deleteDatasets, isDeleting } =
    useDatasetDeleteMutation(refreshDatasets);
  const {
    batchAddTags,
    batchRemoveTags,
    deleteTag: deleteTagMutation,
    renameTag: renameTagMutation,
    mergeTag: mergeTagMutation,
    isUpdatingTags,
  } = useDatasetTagMutation(refreshDatasets);

  const allTags = tagsQuery.data ?? [];
  const deleteTag = async (tag: string) => {
    await deleteTagMutation(tag);
    filters.removeSelectedTag(tag);
  };

  const renameTag = async (oldName: string, newName: string) => {
    await renameTagMutation(oldName, newName);
    filters.replaceSelectedTag(oldName, newName);
  };

  const mergeTag = async (source: string, target: string) => {
    await mergeTagMutation(source, target);
    filters.replaceSelectedTag(source, target);
  };

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
    toggleFavorite,
    deleteDatasets,
    isDeleting,
    batchAddTags,
    batchRemoveTags,
    deleteTag,
    renameTag,
    mergeTag,
    isUpdatingTags,
    handleTagToggle: filters.handleTagToggle,
    handleStatusToggle: filters.handleStatusToggle,
    clearFilters: filters.clearFilters,
    loadNextPage,
  };
}
