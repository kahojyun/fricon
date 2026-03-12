import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { deleteDatasets as deleteDatasetsApi } from "./client";
import { datasetKeys } from "./queryKeys";

export function useDatasetDeleteMutation(refreshDatasets: () => Promise<void>) {
  const queryClient = useQueryClient();
  const [isRefreshingAfterDelete, setIsRefreshingAfterDelete] = useState(false);
  const deleteMutation = useMutation({
    mutationFn: (ids: number[]) => deleteDatasetsApi(ids),
    onSuccess: (_, ids) => {
      // Invalidate tags query as well since deleting datasets might remove tags
      void queryClient.invalidateQueries({ queryKey: datasetKeys.tags() });

      // Invalidate detail queries for deleted datasets to prevent stale data in the inspector
      ids.forEach((id) => {
        void queryClient.invalidateQueries({
          queryKey: datasetKeys.detail(id),
        });
      });
    },
  });

  const deleteDatasets = async (ids: number[]) => {
    setIsRefreshingAfterDelete(true);
    try {
      const results = await deleteMutation.mutateAsync(ids);
      await refreshDatasets();
      return results;
    } catch (error) {
      console.error("Failed to delete datasets:", error);
      throw error;
    } finally {
      setIsRefreshingAfterDelete(false);
    }
  };

  return {
    deleteDatasets,
    isDeleting: deleteMutation.isPending || isRefreshingAfterDelete,
  };
}
