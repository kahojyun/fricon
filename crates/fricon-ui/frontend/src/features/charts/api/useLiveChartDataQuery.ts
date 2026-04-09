import { useEffect, useRef, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import type { LiveChartDataOptions } from "@/shared/lib/bindings";
import type { ChartModel } from "@/shared/lib/chartTypes";
import { fetchLiveChartData } from "./client";
import { chartKeys } from "./queryKeys";
import {
  applyLiveChartUpdate,
  type LiveChartState,
} from "../model/liveChartModel";

export function useLiveChartDataQuery(
  datasetId: number,
  options: LiveChartDataOptions | null,
) {
  const liveStateRef = useRef<LiveChartState | null>(null);
  const [chartData, setChartData] = useState<ChartModel | undefined>(undefined);
  const optionsKey = JSON.stringify(options);

  useEffect(() => {
    liveStateRef.current = null;
    setChartData(undefined);
  }, [datasetId, optionsKey]);

  const query = useQuery({
    queryKey: [...chartKeys.liveChartData(datasetId), options],
    queryFn: () =>
      fetchLiveChartData(
        datasetId,
        options!,
        liveStateRef.current?.rowCount ?? null,
      ),
    enabled: Boolean(options),
    refetchInterval: 1000,
  });

  useEffect(() => {
    if (!query.data) return;
    const nextState = applyLiveChartUpdate(liveStateRef.current, query.data);
    if (!nextState) return;
    liveStateRef.current = nextState;
    setChartData(nextState.chart);
  }, [query.data]);

  return {
    ...query,
    data: chartData,
  };
}
