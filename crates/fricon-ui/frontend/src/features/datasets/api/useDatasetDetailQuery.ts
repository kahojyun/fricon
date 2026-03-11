import { useQuery } from "@tanstack/react-query";
import { getDatasetDetail } from "./client";
import { datasetKeys } from "./queryKeys";

export function useDatasetDetailQuery(datasetId: number) {
  return useQuery({
    queryKey: datasetKeys.detail(datasetId),
    queryFn: () => getDatasetDetail(datasetId),
    staleTime: 10_000,
    refetchOnWindowFocus: false,
  });
}
