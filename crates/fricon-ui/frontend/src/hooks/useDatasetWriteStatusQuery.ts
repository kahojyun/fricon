import { useEffect } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { getDatasetWriteStatus } from "@/lib/backend";

export function useDatasetWriteStatusQuery(
  datasetId: number,
  enabled: boolean,
) {
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: ["datasetWriteStatus", datasetId],
    queryFn: () => getDatasetWriteStatus(datasetId),
    enabled,
    refetchInterval: (queryInstance) =>
      queryInstance.state.data?.isComplete ? false : 1000,
  });

  useEffect(() => {
    const data = query.data;
    if (!data) return;
    if (data.isComplete) {
      void queryClient.invalidateQueries({
        queryKey: ["datasetDetail", datasetId],
      });
    }
    void queryClient.invalidateQueries({
      queryKey: ["filterTableData", datasetId],
    });
    void queryClient.invalidateQueries({ queryKey: ["chartData", datasetId] });
  }, [datasetId, query.data, queryClient]);

  return query;
}
