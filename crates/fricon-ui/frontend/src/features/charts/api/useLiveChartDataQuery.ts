import { useQuery } from "@tanstack/react-query";
import type { LiveChartDataOptions } from "@/shared/lib/bindings";
import { fetchLiveChartData } from "./client";
import { chartKeys } from "./queryKeys";

export function useLiveChartDataQuery(
  datasetId: number,
  options: LiveChartDataOptions | null,
) {
  return useQuery({
    queryKey: [...chartKeys.liveChartData(datasetId), options],
    queryFn: () => fetchLiveChartData(datasetId, options!),
    enabled: Boolean(options),
    refetchInterval: 1000,
  });
}
