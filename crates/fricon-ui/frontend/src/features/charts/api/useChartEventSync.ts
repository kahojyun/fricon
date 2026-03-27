import { useEffect, useEffectEvent } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { events } from "@/shared/lib/bindings";
import { chartKeys } from "./queryKeys";

/**
 * Subscribes to backend dataset change events and invalidates chart-specific
 * queries. Handles only the event kinds that affect chart data:
 * - `statusChanged`: dataset transitions from Writing to Completed/Aborted
 * - `imported`: a force-import replaced existing dataset data
 *
 * Mount this hook once at the root layout level (inside QueryClientProvider),
 * alongside `useDatasetEventSync`.
 */
export function useChartEventSync() {
  const queryClient = useQueryClient();

  const handleEvent = useEffectEvent((datasetId: number) => {
    void queryClient.invalidateQueries({
      queryKey: chartKeys.chartData(datasetId),
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
        const p = event.payload;
        if (p.kind === "statusChanged" || p.kind === "imported") {
          handleEvent(p.info.id);
        }
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
