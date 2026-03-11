import { useMutation, useQueryClient } from "@tanstack/react-query";
import { type DatasetInfo, updateDatasetFavorite } from "@/shared/lib/backend";
import type { DatasetQueryKey } from "./datasetTableShared";

export function useDatasetFavoriteMutation(
  datasetQueryKey: DatasetQueryKey,
  refreshDatasets: () => Promise<void>,
) {
  const queryClient = useQueryClient();
  const favoriteMutation = useMutation({
    mutationFn: ({ id, favorite }: { id: number; favorite: boolean }) =>
      updateDatasetFavorite(id, favorite),
  });

  const toggleFavorite = async (dataset: DatasetInfo) => {
    const nextFavorite = !dataset.favorite;
    const previousData =
      queryClient.getQueryData<DatasetInfo[]>(datasetQueryKey);

    queryClient.setQueryData<DatasetInfo[]>(datasetQueryKey, (current) => {
      if (!current) return current;
      return current.map((item) =>
        item.id === dataset.id ? { ...item, favorite: nextFavorite } : item,
      );
    });

    try {
      await favoriteMutation.mutateAsync({
        id: dataset.id,
        favorite: nextFavorite,
      });
    } catch {
      queryClient.setQueryData(datasetQueryKey, previousData);
      return;
    }

    try {
      await refreshDatasets();
    } catch {
      // Keep optimistic state if backend write succeeded but refresh failed.
    }
  };

  return { toggleFavorite };
}
