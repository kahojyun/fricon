import { useQuery } from "@tanstack/react-query";
import { getDatasetDetail } from "@/lib/backend";

export function useDatasetDetailQuery(datasetId: number) {
  return useQuery({
    queryKey: ["datasetDetail", datasetId],
    queryFn: () => getDatasetDetail(datasetId),
    staleTime: 10_000,
  });
}
