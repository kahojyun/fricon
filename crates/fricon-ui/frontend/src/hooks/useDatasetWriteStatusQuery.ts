import { useEffect, useRef } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { getDatasetWriteStatus } from "@/lib/backend";

export function useDatasetWriteStatusQuery(
  datasetId: number,
  enabled: boolean,
) {
  const queryClient = useQueryClient();
  const lastInvalidatedStatusRef = useRef<{
    rowCount: number;
    isComplete: boolean;
  } | null>(null);

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
    const lastInvalidatedStatus = lastInvalidatedStatusRef.current;
    const hasMeaningfulChange =
      lastInvalidatedStatus?.rowCount !== data.rowCount ||
      lastInvalidatedStatus?.isComplete !== data.isComplete;
    if (!hasMeaningfulChange) return;

    lastInvalidatedStatusRef.current = {
      rowCount: data.rowCount,
      isComplete: data.isComplete,
    };

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
