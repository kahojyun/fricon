import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import {
  deleteDatasets as deleteDatasetsApi,
  emptyTrash as emptyTrashApi,
  restoreDatasets as restoreDatasetsApi,
  trashDatasets as trashDatasetsApi,
} from "./client";
import { datasetKeys } from "./queryKeys";

interface EmptyTrashResult {
  deletedCount: number;
}

async function executeDeleteMutation<T>({
  ids,
  mutateAsync,
  refreshDatasets,
  setIsRefreshingAfterDelete,
}: {
  ids: number[];
  mutateAsync: (ids: number[]) => Promise<T>;
  refreshDatasets: () => Promise<void>;
  setIsRefreshingAfterDelete: (next: boolean) => void;
}): Promise<T> {
  setIsRefreshingAfterDelete(true);
  try {
    const results = await mutateAsync(ids);
    await refreshDatasets();
    return results;
  } catch (error) {
    console.error("Failed to delete datasets:", error);
    throw error;
  } finally {
    setIsRefreshingAfterDelete(false);
  }
}

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

  const deleteDatasets = (ids: number[]) =>
    executeDeleteMutation({
      ids,
      mutateAsync: deleteMutation.mutateAsync,
      refreshDatasets,
      setIsRefreshingAfterDelete,
    });

  return {
    deleteDatasets,
    isDeleting: deleteMutation.isPending || isRefreshingAfterDelete,
  };
}

export function useDatasetTrashMutation(refreshDatasets: () => Promise<void>) {
  const queryClient = useQueryClient();
  const [isRefreshingAfterTrash, setIsRefreshingAfterTrash] = useState(false);
  const trashMutation = useMutation({
    mutationFn: (ids: number[]) => trashDatasetsApi(ids),
    onSuccess: (_, ids) => {
      void queryClient.invalidateQueries({ queryKey: datasetKeys.tags() });

      ids.forEach((id) => {
        void queryClient.invalidateQueries({
          queryKey: datasetKeys.detail(id),
        });
      });
    },
  });

  const trashDatasets = (ids: number[]) =>
    executeDeleteMutation({
      ids,
      mutateAsync: trashMutation.mutateAsync,
      refreshDatasets,
      setIsRefreshingAfterDelete: setIsRefreshingAfterTrash,
    });

  return {
    trashDatasets,
    isTrashing: trashMutation.isPending || isRefreshingAfterTrash,
  };
}

export function useDatasetRestoreMutation(
  refreshDatasets: () => Promise<void>,
) {
  const queryClient = useQueryClient();
  const [isRefreshingAfterRestore, setIsRefreshingAfterRestore] =
    useState(false);
  const restoreMutation = useMutation({
    mutationFn: (ids: number[]) => restoreDatasetsApi(ids),
    onSuccess: (_, ids) => {
      void queryClient.invalidateQueries({ queryKey: datasetKeys.tags() });

      ids.forEach((id) => {
        void queryClient.invalidateQueries({
          queryKey: datasetKeys.detail(id),
        });
      });
    },
  });

  const restoreDatasets = (ids: number[]) =>
    executeDeleteMutation({
      ids,
      mutateAsync: restoreMutation.mutateAsync,
      refreshDatasets,
      setIsRefreshingAfterDelete: setIsRefreshingAfterRestore,
    });

  return {
    restoreDatasets,
    isRestoring: restoreMutation.isPending || isRefreshingAfterRestore,
  };
}

export function useEmptyTrashMutation(refreshDatasets: () => Promise<void>) {
  const queryClient = useQueryClient();
  const [isRefreshingAfterEmptyTrash, setIsRefreshingAfterEmptyTrash] =
    useState(false);
  const emptyTrashMutation = useMutation({
    mutationFn: () => emptyTrashApi(),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: datasetKeys.tags() });
    },
  });

  const emptyTrash = async (): Promise<EmptyTrashResult> => {
    setIsRefreshingAfterEmptyTrash(true);

    return emptyTrashMutation
      .mutateAsync()
      .then(async (result) => {
        await refreshDatasets();
        return result;
      })
      .catch((error: unknown) => {
        console.error("Failed to empty trash:", error);
        throw error;
      })
      .finally(() => {
        setIsRefreshingAfterEmptyTrash(false);
      });
  };

  return {
    emptyTrash,
    isEmptyingTrash:
      emptyTrashMutation.isPending || isRefreshingAfterEmptyTrash,
  };
}
