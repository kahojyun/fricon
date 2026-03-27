import { useMutation } from "@tanstack/react-query";
import {
  deleteDatasets as deleteDatasetsApi,
  emptyTrash as emptyTrashApi,
  restoreDatasets as restoreDatasetsApi,
  trashDatasets as trashDatasetsApi,
} from "./client";
import type { DatasetDeleteResult } from "./types";

export function useDatasetDeleteMutation() {
  const mutation = useMutation({
    mutationFn: (ids: number[]) => deleteDatasetsApi(ids),
  });
  return {
    deleteDatasets: (ids: number[]) => mutation.mutateAsync(ids),
    isDeleting: mutation.isPending,
  };
}

export function useDatasetTrashMutation() {
  const mutation = useMutation({
    mutationFn: (ids: number[]) => trashDatasetsApi(ids),
  });
  return {
    trashDatasets: (ids: number[]) => mutation.mutateAsync(ids),
    isTrashing: mutation.isPending,
  };
}

export function useDatasetRestoreMutation() {
  const mutation = useMutation({
    mutationFn: (ids: number[]) => restoreDatasetsApi(ids),
  });
  return {
    restoreDatasets: (ids: number[]) => mutation.mutateAsync(ids),
    isRestoring: mutation.isPending,
  };
}

export function useEmptyTrashMutation() {
  const mutation = useMutation({
    mutationFn: (): Promise<DatasetDeleteResult[]> => emptyTrashApi(),
  });
  return {
    emptyTrash: () => mutation.mutateAsync(),
    isEmptyingTrash: mutation.isPending,
  };
}
