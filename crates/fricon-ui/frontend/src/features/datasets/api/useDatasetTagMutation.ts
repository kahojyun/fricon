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

export function useDatasetTagMutation(refreshDatasets: () => Promise<void>) {
  const queryClient = useQueryClient();
  const [isUpdatingTags, setIsUpdatingTags] = useState(false);

  const invalidateAfterTagChange = async (invalidateAllDetails = false) => {
    if (invalidateAllDetails) {
      await queryClient.invalidateQueries({
        queryKey: ["datasets", "detail"],
      });
    }
    await queryClient.invalidateQueries({ queryKey: datasetKeys.tags() });
    await refreshDatasets();
  };

  const batchAddTags = async (
    ids: number[],
    tags: string[],
  ): Promise<DatasetDeleteResult[]> =>
    runTagMutation({
      setIsUpdatingTags,
      work: async () => {
        const results = await batchUpdateDatasetTagsApi(ids, tags, []);
        // Invalidate detail queries for affected datasets
        ids.forEach((id) => {
          void queryClient.invalidateQueries({
            queryKey: datasetKeys.detail(id),
          });
        });
        await invalidateAfterTagChange();
        return results;
      },
    });

  const batchRemoveTags = async (
    ids: number[],
    tags: string[],
  ): Promise<DatasetDeleteResult[]> =>
    runTagMutation({
      setIsUpdatingTags,
      work: async () => {
        const results = await batchUpdateDatasetTagsApi(ids, [], tags);
        ids.forEach((id) => {
          void queryClient.invalidateQueries({
            queryKey: datasetKeys.detail(id),
          });
        });
        await invalidateAfterTagChange();
        return results;
      },
    });

  const deleteTag = (tag: string): Promise<void> =>
    runTagMutation({
      setIsUpdatingTags,
      work: async () => {
        await deleteTagApi(tag);
        await invalidateAfterTagChange(true);
      },
    });

  const renameTag = (oldName: string, newName: string): Promise<void> =>
    runTagMutation({
      setIsUpdatingTags,
      work: async () => {
        await renameTagApi(oldName, newName);
        await invalidateAfterTagChange(true);
      },
    });

  const mergeTag = (source: string, target: string): Promise<void> =>
    runTagMutation({
      setIsUpdatingTags,
      work: async () => {
        await mergeTagApi(source, target);
        await invalidateAfterTagChange(true);
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
