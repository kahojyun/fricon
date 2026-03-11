import {
  keepPreviousData,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { listDatasets } from "./client";
import { datasetKeys } from "./queryKeys";
import {
  buildDatasetListOptions,
  deriveHasMore,
  type DatasetQueryKey,
  type DatasetQueryParams,
} from "./datasetTableShared";

export function useDatasetTableQuery(
  queryParams: DatasetQueryParams,
  visibleCount: number,
  incrementVisibleCount: () => Promise<void>,
) {
  const queryClient = useQueryClient();
  const datasetQueryKey: DatasetQueryKey = datasetKeys.list(
    queryParams,
    visibleCount,
  );

  const datasetsQuery = useQuery({
    queryKey: datasetQueryKey,
    queryFn: () =>
      listDatasets(
        buildDatasetListOptions(queryParams, {
          limit: visibleCount,
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
    : deriveHasMore(datasets.length, visibleCount);

  const loadNextPage = () => {
    if (datasetsQuery.isFetching || !hasMore) return Promise.resolve();
    return incrementVisibleCount();
  };

  return {
    datasetQueryKey,
    datasets,
    hasMore,
    refreshDatasets,
    loadNextPage,
  };
}
