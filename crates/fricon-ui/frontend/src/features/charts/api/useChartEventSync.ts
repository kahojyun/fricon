import { useEffect, useEffectEvent } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { events, type DatasetChanged } from "@/shared/lib/bindings";
import { chartKeys } from "./queryKeys";

/**
 * Subscribes to backend dataset change events and invalidates chart-specific
 * queries. Handles only the event kinds that affect chart data:
 * - `writeProgress`: writer appended rows while the dataset is still open
 * - `statusChanged`: dataset transitions from Writing to Completed/Aborted
 * - `imported`: a force-import replaced existing dataset data
 *
 * Mount this hook once at the root layout level (inside QueryClientProvider),
 * alongside `useDatasetEventSync`.
 */
export function useChartEventSync() {
  const queryClient = useQueryClient();

  const handleEvent = useEffectEvent((payload: DatasetChanged) => {
    if (payload.kind === "writeProgress") {
      const datasetId = payload.progress.id;
      void queryClient.invalidateQueries({
        queryKey: chartKeys.chartData(datasetId),
      });
      void queryClient.invalidateQueries({
        queryKey: chartKeys.filterTableData(datasetId),
      });
      void queryClient.refetchQueries({
        queryKey: chartKeys.liveChartData(datasetId),
        type: "active",
      });
      return;
    }

    if (payload.kind === "globalTagsChanged") {
      return;
    }

    const datasetId = payload.info.id;

    if (payload.kind !== "statusChanged" && payload.kind !== "imported") {
      return;
    }

    void queryClient.invalidateQueries({
      queryKey: chartKeys.chartData(datasetId),
    });
    void queryClient.invalidateQueries({
      queryKey: chartKeys.liveChartData(datasetId),
    });
    void queryClient.invalidateQueries({
      queryKey: chartKeys.filterTableData(datasetId),
    });
  });

  useEffect(() => {
    let active = true;
    let unlisten: (() => void) | undefined;

    void events.datasetChanged
      .listen((event) => {
        if (!active) return;
        handleEvent(event.payload);
      })
      .then((fn) => {
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
