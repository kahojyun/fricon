import { useState } from "react";
import {
  batchUpdateDatasetTags as batchUpdateDatasetTagsApi,
  deleteTag as deleteTagApi,
  renameTag as renameTagApi,
  mergeTag as mergeTagApi,
} from "./client";
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
  const [isUpdatingTags, setIsUpdatingTags] = useState(false);

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
      work: () => deleteTagApi(tag),
    });

  const renameTag = (oldName: string, newName: string): Promise<void> =>
    runTagMutation({
      setIsUpdatingTags,
      work: () => renameTagApi(oldName, newName),
    });

  const mergeTag = (source: string, target: string): Promise<void> =>
    runTagMutation({
      setIsUpdatingTags,
      work: () => mergeTagApi(source, target),
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
