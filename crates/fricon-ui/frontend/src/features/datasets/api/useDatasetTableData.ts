import { useQuery } from "@tanstack/react-query";
import { listDatasetTags } from "./client";
import { datasetKeys } from "./queryKeys";
import { type UseDatasetTableDataResult } from "./datasetTableShared";
import { useDatasetFavoriteMutation } from "./useDatasetFavoriteMutation";
import { useDatasetDeleteMutation } from "./useDatasetDeleteMutation";
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

  const allTags = tagsQuery.data ?? [];
  const normalizedTagFilterQuery = filters.tagFilterQuery.trim().toLowerCase();
  const filteredTagOptions = normalizedTagFilterQuery
    ? allTags.filter((tag) =>
        tag.toLowerCase().includes(normalizedTagFilterQuery),
      )
    : allTags;

  return {
    datasets,
    searchQuery: filters.searchQuery,
    setSearchQuery: filters.setSearchQuery,
    selectedTags: filters.selectedTags,
    selectedStatuses: filters.selectedStatuses,
    tagFilterQuery: filters.tagFilterQuery,
    setTagFilterQuery: filters.setTagFilterQuery,
    sorting: filters.sorting,
    setSorting: filters.setSorting,
    filteredTagOptions,
    favoriteOnly: filters.favoriteOnly,
    setFavoriteOnly: filters.setFavoriteOnly,
    hasMore,
    hasActiveFilters: filters.hasActiveFilters,
    toggleFavorite,
    deleteDatasets,
    isDeleting,
    handleTagToggle: filters.handleTagToggle,
    handleStatusToggle: filters.handleStatusToggle,
    clearFilters: filters.clearFilters,
    loadNextPage,
  };
}
