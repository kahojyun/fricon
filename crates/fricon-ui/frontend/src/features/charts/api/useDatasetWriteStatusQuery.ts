import { useQuery } from "@tanstack/react-query";
import { getDatasetWriteStatus } from "./client";
import { chartKeys } from "./queryKeys";

export function useDatasetWriteStatusQuery(
  datasetId: number,
  enabled: boolean,
) {
  return useQuery({
    queryKey: chartKeys.writeStatus(datasetId),
    queryFn: () => getDatasetWriteStatus(datasetId),
    enabled,
    refetchInterval: 2000,
  });
}
