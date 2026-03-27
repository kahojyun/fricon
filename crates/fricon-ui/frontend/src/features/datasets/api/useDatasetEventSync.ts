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

  const handleEvent = useEffectEvent(({ info, kind }: DatasetChangedEvent) => {
    const id = info.id;
    switch (kind) {
      case "created":
        void queryClient.invalidateQueries({
          queryKey: ["datasets", "list"],
        });
        void queryClient.invalidateQueries({ queryKey: datasetKeys.tags() });
        break;

      case "statusChanged":
        void queryClient.invalidateQueries({
          queryKey: ["datasets", "list"],
        });
        void queryClient.invalidateQueries({
          queryKey: datasetKeys.detail(id),
        });
        break;

      case "metadataUpdated":
        void queryClient.invalidateQueries({
          queryKey: ["datasets", "list"],
        });
        void queryClient.invalidateQueries({
          queryKey: datasetKeys.detail(id),
        });
        break;

      case "tagsChanged":
        void queryClient.invalidateQueries({
          queryKey: ["datasets", "list"],
        });
        void queryClient.invalidateQueries({ queryKey: datasetKeys.tags() });
        void queryClient.invalidateQueries({
          queryKey: datasetKeys.detail(id),
        });
        break;

      case "trashed":
      case "deleted":
        void queryClient.invalidateQueries({
          queryKey: ["datasets", "list"],
        });
        void queryClient.invalidateQueries({ queryKey: datasetKeys.tags() });
        queryClient.removeQueries({ queryKey: datasetKeys.detail(id) });
        break;

      case "restored":
        void queryClient.invalidateQueries({
          queryKey: ["datasets", "list"],
        });
        break;

      case "imported":
        void queryClient.invalidateQueries({
          queryKey: ["datasets", "list"],
        });
        void queryClient.invalidateQueries({
          queryKey: datasetKeys.detail(id),
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
