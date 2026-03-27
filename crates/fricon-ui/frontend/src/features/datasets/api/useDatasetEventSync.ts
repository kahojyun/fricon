import { useEffect, useEffectEvent } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { onDatasetChanged, type DatasetChangedEvent } from "./events";
import { datasetKeys } from "./queryKeys";

/**
 * Subscribes to backend dataset change events and performs targeted React
 * Query cache invalidation.
 *
 * Mount this hook once at the root layout level (inside QueryClientProvider).
 * Each event kind maps to the minimum set of queries that need to be
 * refreshed, eliminating the full-table refetch and write-status polling that
 * the previous implementation required.
 */
export function useDatasetEventSync() {
  const queryClient = useQueryClient();

  const handleEvent = useEffectEvent((event: DatasetChangedEvent) => {
    const invalidateList = () =>
      void queryClient.invalidateQueries({ queryKey: ["datasets", "list"] });
    const invalidateTags = () =>
      void queryClient.invalidateQueries({ queryKey: datasetKeys.tags() });

    switch (event.kind) {
      case "created":
        invalidateList();
        invalidateTags();
        break;

      case "statusChanged":
      case "metadataUpdated":
        invalidateList();
        void queryClient.invalidateQueries({
          queryKey: datasetKeys.detail(event.info.id),
        });
        break;

      case "tagsChanged":
      case "imported":
        invalidateList();
        invalidateTags();
        void queryClient.invalidateQueries({
          queryKey: datasetKeys.detail(event.info.id),
        });
        break;

      case "trashed":
      case "deleted":
        invalidateList();
        invalidateTags();
        queryClient.removeQueries({
          queryKey: datasetKeys.detail(event.info.id),
        });
        break;

      case "restored":
        invalidateList();
        break;

      case "globalTagsChanged":
        invalidateList();
        invalidateTags();
        void queryClient.invalidateQueries({
          queryKey: ["datasets", "detail"],
        });
        break;
    }
  });

  useEffect(() => {
    let active = true;
    let unlisten: (() => void) | undefined;

    void onDatasetChanged((event) => {
      if (!active) return;
      handleEvent(event);
    }).then((fn) => {
      if (!active) {
        fn();
        return;
      }
      unlisten = fn;
    });

    return () => {
      active = false;
      unlisten?.();
    };
  }, []);
}
