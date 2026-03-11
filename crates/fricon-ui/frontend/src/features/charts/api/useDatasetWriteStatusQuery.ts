import { useEffect, useRef } from "react";
import {
  useQuery,
  useQueryClient,
  type QueryClient,
} from "@tanstack/react-query";
import { datasetDetailQueryKey } from "@/shared/lib/queryKeys";
import { getDatasetWriteStatus } from "./client";
import { chartKeys } from "./queryKeys";

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
      queryKey: datasetDetailQueryKey(datasetId),
    });
  }
  void queryClient.invalidateQueries({
    queryKey: chartKeys.filterTableData(datasetId),
  });
  void queryClient.invalidateQueries({
    queryKey: chartKeys.chartData(datasetId),
  });
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
    queryKey: chartKeys.writeStatus(datasetId),
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
