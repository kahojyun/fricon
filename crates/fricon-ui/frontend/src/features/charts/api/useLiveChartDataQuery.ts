import { useRef } from "react";
import { useQuery } from "@tanstack/react-query";
import { fetchLiveChartData } from "./client";
import { chartKeys } from "./queryKeys";
import type { LiveChartDataOptions } from "./types";
import {
  applyLiveChartUpdate,
  type LiveChartState,
} from "../model/liveChartModel";

export function useLiveChartDataQuery(
  datasetId: number,
  options: LiveChartDataOptions | null,
) {
  const liveStateRef = useRef<{
    requestKey: string;
    state: LiveChartState | null;
  } | null>(null);
  const optionsKey = JSON.stringify(options);
  const requestKey = `${datasetId}:${optionsKey}`;

  const query = useQuery({
    queryKey: [...chartKeys.liveChartData(datasetId), options],
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
    refetchInterval: 1000,
  });

  return query;
}
