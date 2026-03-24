import type {
  DatasetDeleteResult,
  DatasetInfo,
  DatasetTagBatchResult,
} from "./types";
import type { DatasetQueryKey } from "./datasetTableShared";
import {
  useDatasetDeleteMutation,
  useDatasetRestoreMutation,
  useDatasetTrashMutation,
  useEmptyTrashMutation,
} from "./useDatasetDeleteMutation";
import { useDatasetFavoriteMutation } from "./useDatasetFavoriteMutation";
import { useDatasetTagMutation } from "./useDatasetTagMutation";

interface UseDatasetTableActionsArgs {
  datasetQueryKey: DatasetQueryKey;
  refreshDatasets: () => Promise<void>;
  removeActiveTag: (tag: string) => void;
  replaceActiveTag: (oldName: string, newName: string) => void;
}

export interface UseDatasetTableActionsResult {
  toggleFavorite: (dataset: DatasetInfo) => Promise<void>;
  trashDatasets: (ids: number[]) => Promise<DatasetDeleteResult[]>;
  restoreDatasets: (ids: number[]) => Promise<DatasetDeleteResult[]>;
  deleteDatasets: (ids: number[]) => Promise<DatasetDeleteResult[]>;
  emptyTrash: () => Promise<DatasetDeleteResult[]>;
  isMutatingDatasets: boolean;
  batchAddTags: (
    ids: number[],
    tags: string[],
  ) => Promise<DatasetTagBatchResult[]>;
  batchRemoveTags: (
    ids: number[],
    tags: string[],
  ) => Promise<DatasetTagBatchResult[]>;
  deleteTag: (tag: string) => Promise<void>;
  renameTag: (oldName: string, newName: string) => Promise<void>;
  mergeTag: (source: string, target: string) => Promise<void>;
  isUpdatingTags: boolean;
}

export function useDatasetTableActions({
  datasetQueryKey,
  refreshDatasets,
  removeActiveTag,
  replaceActiveTag,
}: UseDatasetTableActionsArgs): UseDatasetTableActionsResult {
  const { toggleFavorite } = useDatasetFavoriteMutation(
    datasetQueryKey,
    refreshDatasets,
  );
  const { trashDatasets, isTrashing } =
    useDatasetTrashMutation(refreshDatasets);
  const { restoreDatasets, isRestoring } =
    useDatasetRestoreMutation(refreshDatasets);
  const { deleteDatasets, isDeleting } =
    useDatasetDeleteMutation(refreshDatasets);
  const { emptyTrash, isEmptyingTrash } =
    useEmptyTrashMutation(refreshDatasets);
  const {
    batchAddTags,
    batchRemoveTags,
    deleteTag: deleteTagMutation,
    renameTag: renameTagMutation,
    mergeTag: mergeTagMutation,
    isUpdatingTags,
  } = useDatasetTagMutation(refreshDatasets);

  const deleteTag = async (tag: string) => {
    await deleteTagMutation(tag);
    removeActiveTag(tag);
  };

  const renameTag = async (oldName: string, newName: string) => {
    await renameTagMutation(oldName, newName);
    replaceActiveTag(oldName, newName);
  };

  const mergeTag = async (source: string, target: string) => {
    await mergeTagMutation(source, target);
    replaceActiveTag(source, target);
  };

  return {
    toggleFavorite,
    trashDatasets,
    restoreDatasets,
    deleteDatasets,
    emptyTrash,
    isMutatingDatasets:
      isTrashing || isRestoring || isDeleting || isEmptyingTrash,
    batchAddTags,
    batchRemoveTags,
    deleteTag,
    renameTag,
    mergeTag,
    isUpdatingTags,
  };
}
