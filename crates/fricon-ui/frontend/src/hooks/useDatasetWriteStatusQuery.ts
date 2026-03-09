import { useEffect, useRef } from "react";
import {
  useQuery,
  useQueryClient,
  type QueryClient,
} from "@tanstack/react-query";
import { getDatasetWriteStatus } from "@/lib/backend";

interface DatasetWriteStatusSnapshot {
  rowCount: number;
  isComplete: boolean;
}

function hasWriteStatusChanged(
  previous: DatasetWriteStatusSnapshot | null,
  next: DatasetWriteStatusSnapshot,
): boolean {
  return (
    previous?.rowCount !== next.rowCount ||
    previous?.isComplete !== next.isComplete
  );
}

function invalidateWriteDependentQueries(
  queryClient: QueryClient,
  datasetId: number,
  isComplete: boolean,
) {
  if (isComplete) {
    void queryClient.invalidateQueries({
      queryKey: ["datasetDetail", datasetId],
    });
  }
  void queryClient.invalidateQueries({
    queryKey: ["filterTableData", datasetId],
  });
  void queryClient.invalidateQueries({ queryKey: ["chartData", datasetId] });
}

export function useDatasetWriteStatusQuery(
  datasetId: number,
  enabled: boolean,
) {
  const queryClient = useQueryClient();
  const lastInvalidatedStatusRef = useRef<DatasetWriteStatusSnapshot | null>(
    null,
  );

  const query = useQuery({
    queryKey: ["datasetWriteStatus", datasetId],
    queryFn: () => getDatasetWriteStatus(datasetId),
    enabled,
    refetchInterval: (queryInstance) =>
      queryInstance.state.data?.isComplete ? false : 1000,
  });

  useEffect(() => {
    lastInvalidatedStatusRef.current = null;
  }, [datasetId]);

  useEffect(() => {
    const data = query.data;
    if (!data) return;
    const snapshot: DatasetWriteStatusSnapshot = {
      rowCount: data.rowCount,
      isComplete: data.isComplete,
    };
    if (!hasWriteStatusChanged(lastInvalidatedStatusRef.current, snapshot)) {
      return;
    }

    lastInvalidatedStatusRef.current = snapshot;
    invalidateWriteDependentQueries(queryClient, datasetId, data.isComplete);
  }, [datasetId, query.data, queryClient]);

  return query;
}
