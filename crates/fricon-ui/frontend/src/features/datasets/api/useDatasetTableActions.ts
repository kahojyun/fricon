import type { DatasetDeleteResult, DatasetInfo } from "./types";
import type { DatasetQueryKey } from "./datasetTableShared";
import { useDatasetDeleteMutation } from "./useDatasetDeleteMutation";
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
  deleteDatasets: (ids: number[]) => Promise<DatasetDeleteResult[]>;
  isDeleting: boolean;
  batchAddTags: (
    ids: number[],
    tags: string[],
  ) => Promise<DatasetDeleteResult[]>;
  batchRemoveTags: (
    ids: number[],
    tags: string[],
  ) => Promise<DatasetDeleteResult[]>;
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
    deleteDatasets,
    isDeleting,
    batchAddTags,
    batchRemoveTags,
    deleteTag,
    renameTag,
    mergeTag,
    isUpdatingTags,
  };
}
