import { useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import {
  batchUpdateDatasetTags as batchUpdateDatasetTagsApi,
  deleteTag as deleteTagApi,
  renameTag as renameTagApi,
  mergeTag as mergeTagApi,
} from "./client";
import { datasetKeys } from "./queryKeys";
import type { DatasetTagBatchResult } from "./types";

async function runTagMutation<T>({
  setIsUpdatingTags,
  work,
}: {
  setIsUpdatingTags: (next: boolean) => void;
  work: () => Promise<T>;
}): Promise<T> {
  setIsUpdatingTags(true);
  try {
    return await work();
  } finally {
    setIsUpdatingTags(false);
  }
}

export function useDatasetTagMutation() {
  const queryClient = useQueryClient();
  const [isUpdatingTags, setIsUpdatingTags] = useState(false);

  // Global tag operations (delete/rename/merge) are not covered by per-dataset
  // events; invalidate all affected queries manually.
  const invalidateGlobalTagChange = async () => {
    await queryClient.invalidateQueries({ queryKey: ["datasets", "list"] });
    await queryClient.invalidateQueries({ queryKey: datasetKeys.tags() });
    await queryClient.invalidateQueries({ queryKey: ["datasets", "detail"] });
  };

  const batchAddTags = (
    ids: number[],
    tags: string[],
  ): Promise<DatasetTagBatchResult[]> =>
    runTagMutation({
      setIsUpdatingTags,
      work: () => batchUpdateDatasetTagsApi(ids, tags, []),
    });

  const batchRemoveTags = (
    ids: number[],
    tags: string[],
  ): Promise<DatasetTagBatchResult[]> =>
    runTagMutation({
      setIsUpdatingTags,
      work: () => batchUpdateDatasetTagsApi(ids, [], tags),
    });

  const deleteTag = (tag: string): Promise<void> =>
    runTagMutation({
      setIsUpdatingTags,
      work: async () => {
        await deleteTagApi(tag);
        await invalidateGlobalTagChange();
      },
    });

  const renameTag = (oldName: string, newName: string): Promise<void> =>
    runTagMutation({
      setIsUpdatingTags,
      work: async () => {
        await renameTagApi(oldName, newName);
        await invalidateGlobalTagChange();
      },
    });

  const mergeTag = (source: string, target: string): Promise<void> =>
    runTagMutation({
      setIsUpdatingTags,
      work: async () => {
        await mergeTagApi(source, target);
        await invalidateGlobalTagChange();
      },
    });

  return {
    batchAddTags,
    batchRemoveTags,
    deleteTag,
    renameTag,
    mergeTag,
    isUpdatingTags,
  };
}
