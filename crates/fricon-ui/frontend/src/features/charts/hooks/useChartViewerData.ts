import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import type { ChartViewerAvailability, DatasetDetail } from "../api/types";
import type { ComplexViewOption } from "@/shared/lib/chartTypes";
import { useChartDataQuery } from "../api/useChartDataQuery";
import { useDatasetWriteStatusQuery } from "../api/useDatasetWriteStatusQuery";
import { useFilterTableDataQuery } from "../api/useFilterTableDataQuery";
import { chartKeys } from "../api/queryKeys";
import { useCascadeSelection } from "./useCascadeSelection";
import {
  buildChartRequest,
  deriveChartViewerState,
} from "../model/chartViewerLogic";

interface UseChartViewerDataArgs {
  datasetId: number;
  availability: ChartViewerAvailability;
  datasetDetail: DatasetDetail | null;
  derived: ReturnType<typeof deriveChartViewerState>;
  selectedComplexView: ComplexViewOption[];
  selectedComplexViewSingle: ComplexViewOption;
}

export function useChartViewerData({
  datasetId,
  availability,
  datasetDetail,
  derived,
  selectedComplexView,
  selectedComplexViewSingle,
}: UseChartViewerDataArgs) {
  const queryClient = useQueryClient();
  const queriesEnabled = availability === "available";
  const filterTableQuery = useFilterTableDataQuery(
    datasetId,
    derived.excludeColumns,
    queriesEnabled,
  );
  const filterTableData = filterTableQuery.data ?? null;

  const cascade = useCascadeSelection(filterTableData);
  const filterRow = cascade.resolvedRow ?? null;
  const hasFilters = (filterTableData?.fields.length ?? 0) > 0;
  const indexFilters = hasFilters ? filterRow?.valueIndices : undefined;

  const writeStatus = useDatasetWriteStatusQuery(
    datasetId,
    queriesEnabled && datasetDetail?.status === "Writing",
  );

  useEffect(() => {
    if (writeStatus.data?.rowCount !== undefined) {
      void queryClient.invalidateQueries({
        queryKey: chartKeys.chartData(datasetId),
      });
      void queryClient.invalidateQueries({
        queryKey: chartKeys.filterTableData(datasetId),
      });
    }
  }, [queryClient, datasetId, writeStatus.data?.rowCount]);

  const chartRequest = buildChartRequest({
    datasetDetail,
    filterTableData,
    hasFilters,
    filterRow,
    selectedComplexView,
    selectedComplexViewSingle,
    indexFilters,
    derived,
  });

  const chartQuery = useChartDataQuery(
    datasetId,
    queriesEnabled ? chartRequest : null,
  );
  const chartData = chartQuery.data;
  const chartError = !queriesEnabled
    ? null
    : chartQuery.error
      ? chartQuery.error instanceof Error
        ? chartQuery.error.message
        : "Failed to load chart data."
      : null;

  return {
    chartData,
    chartError,
    filterTableProps: {
      data: filterTableData ?? undefined,
      mode: cascade.state.mode,
      onModeChange: cascade.setMode,
      selectedRowIndex: filterRow?.index ?? null,
      onSelectRow: cascade.selectRow,
      selectedValueIndices: cascade.selectedValueIndices,
      onSelectFieldValue: (fieldIndex: number, valueIndex: number) => {
        cascade.selectFieldValue(
          fieldIndex,
          valueIndex,
          filterRow?.index ?? null,
        );
      },
    },
  };
}
