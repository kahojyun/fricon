import { useEffect, useEffectEvent } from "react";
import { useQueryClient } from "@tanstack/react-query";
import type { DatasetInfo } from "@/shared/lib/backend";
import { onDatasetCreated, onDatasetUpdated } from "@/shared/lib/backend";

export function useDatasetTableRefreshSync(
  datasets: DatasetInfo[],
  refreshDatasets: () => Promise<void>,
) {
  const queryClient = useQueryClient();
  const refreshAndInvalidateTags = useEffectEvent(() => {
    void refreshDatasets();
    void queryClient.invalidateQueries({ queryKey: ["datasetTags"] });
  });

  const refreshDatasetsEvent = useEffectEvent(() => {
    void refreshDatasets();
  });

  useEffect(() => {
    let unlistenCreated: (() => void) | undefined;
    let unlistenUpdated: (() => void) | undefined;
    let active = true;

    void onDatasetCreated(() => {
      if (!active) return;
      refreshAndInvalidateTags();
    }).then((unlisten) => {
      if (!active) {
        unlisten();
        return;
      }
      unlistenCreated = unlisten;
    });

    void onDatasetUpdated(() => {
      if (!active) return;
      refreshAndInvalidateTags();
    }).then((unlisten) => {
      if (!active) {
        unlisten();
        return;
      }
      unlistenUpdated = unlisten;
    });

    return () => {
      active = false;
      unlistenCreated?.();
      unlistenUpdated?.();
    };
  }, []);

  useEffect(() => {
    const hasWriting = datasets.some((dataset) => dataset.status === "Writing");
    if (!hasWriting) return;

    const timer = window.setInterval(() => {
      refreshDatasetsEvent();
    }, 2000);

    return () => {
      window.clearInterval(timer);
    };
  }, [datasets]);
}
