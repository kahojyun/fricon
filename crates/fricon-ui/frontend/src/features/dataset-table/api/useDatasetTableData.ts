import { useQuery } from "@tanstack/react-query";
import { listDatasetTags } from "@/shared/lib/backend";
import { type UseDatasetTableDataResult } from "./datasetTableShared";
import { useDatasetFavoriteMutation } from "./useDatasetFavoriteMutation";
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
    queryKey: ["datasetTags"],
    queryFn: listDatasetTags,
  });

  useDatasetTableRefreshSync(datasets, refreshDatasets);
  const { toggleFavorite } = useDatasetFavoriteMutation(
    datasetQueryKey,
    refreshDatasets,
  );

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
    handleTagToggle: filters.handleTagToggle,
    handleStatusToggle: filters.handleStatusToggle,
    clearFilters: filters.clearFilters,
    loadNextPage,
  };
}
