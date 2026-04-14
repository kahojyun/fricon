import { useEffect, useRef } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { fetchLiveChartData } from "./client";
import { chartKeys } from "./queryKeys";
import type { LiveChartDataOptions } from "./types";
import {
  applyLiveChartUpdate,
  type LiveChartState,
} from "../model/liveChartModel";

function isLiveChartQueryKey(value: unknown, datasetId: number) {
  if (!Array.isArray(value)) {
    return false;
  }

  const queryKey = value as readonly unknown[];
  const [scope, resource, id] = queryKey;
  return scope === "charts" && resource === "liveChartData" && id === datasetId;
}

export function useLiveChartDataQuery(
  datasetId: number,
  options: LiveChartDataOptions | null,
) {
  const queryClient = useQueryClient();
  const liveStateRef = useRef<{
    requestKey: string;
    state: LiveChartState | null;
  } | null>(null);
  const optionsKey = JSON.stringify(options);
  const requestKey = `${datasetId}:${optionsKey}`;
  const queryKey = [...chartKeys.liveChartData(datasetId), options] as const;

  useEffect(() => {
    return queryClient.getQueryCache().subscribe((event) => {
      if (
        event.type !== "updated" ||
        event.action.type !== "invalidate" ||
        !isLiveChartQueryKey(event.query.queryKey, datasetId)
      ) {
        return;
      }

      liveStateRef.current = null;
    });
  }, [datasetId, queryClient]);

  const query = useQuery({
    queryKey,
    queryFn: async () => {
      const previousState =
        liveStateRef.current?.requestKey === requestKey
          ? liveStateRef.current.state
          : null;
      const update = await fetchLiveChartData(
        datasetId,
        options!,
        previousState?.rowCount ?? null,
      );
      const nextState = applyLiveChartUpdate(previousState, update);
      liveStateRef.current = { requestKey, state: nextState };
      return nextState?.chart;
    },
    enabled: Boolean(options),
  });

  return query;
}
