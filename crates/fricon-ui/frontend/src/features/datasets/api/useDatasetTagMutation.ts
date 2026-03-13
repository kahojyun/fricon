import { useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import {
  batchUpdateDatasetTags as batchUpdateDatasetTagsApi,
  deleteTag as deleteTagApi,
  renameTag as renameTagApi,
  mergeTag as mergeTagApi,
} from "./client";
import { datasetKeys } from "./queryKeys";
import type { DatasetDeleteResult } from "./types";

export function useDatasetTagMutation(refreshDatasets: () => Promise<void>) {
  const queryClient = useQueryClient();
  const [isUpdatingTags, setIsUpdatingTags] = useState(false);

  const invalidateAfterTagChange = async () => {
    await queryClient.invalidateQueries({ queryKey: datasetKeys.tags() });
    await refreshDatasets();
  };

  const batchAddTags = async (
    ids: number[],
    tags: string[],
  ): Promise<DatasetDeleteResult[]> => {
    setIsUpdatingTags(true);
    try {
      const results = await batchUpdateDatasetTagsApi(ids, tags, []);
      // Invalidate detail queries for affected datasets
      ids.forEach((id) => {
        void queryClient.invalidateQueries({
          queryKey: datasetKeys.detail(id),
        });
      });
      await invalidateAfterTagChange();
      return results;
    } finally {
      setIsUpdatingTags(false);
    }
  };

  const batchRemoveTags = async (
    ids: number[],
    tags: string[],
  ): Promise<DatasetDeleteResult[]> => {
    setIsUpdatingTags(true);
    try {
      const results = await batchUpdateDatasetTagsApi(ids, [], tags);
      ids.forEach((id) => {
        void queryClient.invalidateQueries({
          queryKey: datasetKeys.detail(id),
        });
      });
      await invalidateAfterTagChange();
      return results;
    } finally {
      setIsUpdatingTags(false);
    }
  };

  const deleteTag = async (tag: string): Promise<void> => {
    setIsUpdatingTags(true);
    try {
      await deleteTagApi(tag);
      await invalidateAfterTagChange();
    } finally {
      setIsUpdatingTags(false);
    }
  };

  const renameTag = async (oldName: string, newName: string): Promise<void> => {
    setIsUpdatingTags(true);
    try {
      await renameTagApi(oldName, newName);
      await invalidateAfterTagChange();
    } finally {
      setIsUpdatingTags(false);
    }
  };

  const mergeTag = async (source: string, target: string): Promise<void> => {
    setIsUpdatingTags(true);
    try {
      await mergeTagApi(source, target);
      await invalidateAfterTagChange();
    } finally {
      setIsUpdatingTags(false);
    }
  };

  return {
    batchAddTags,
    batchRemoveTags,
    deleteTag,
    renameTag,
    mergeTag,
    isUpdatingTags,
  };
}
