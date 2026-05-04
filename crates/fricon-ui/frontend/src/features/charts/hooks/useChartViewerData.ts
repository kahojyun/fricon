import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import type {
  ChartViewerAvailability,
  DatasetDetail,
  LiveChartDataOptions,
} from "../api/types";
import type { ComplexViewOption } from "@/shared/lib/chartTypes";
import { useChartDataQuery } from "../api/useChartDataQuery";
import { useLiveChartDataQuery } from "../api/useLiveChartDataQuery";
import { useDatasetWriteStatusQuery } from "../api/useDatasetWriteStatusQuery";
import { useFilterTableDataQuery } from "../api/useFilterTableDataQuery";
import { chartKeys } from "../api/queryKeys";
import { useCascadeSelection } from "./useCascadeSelection";
import {
  buildChartRequest,
  deriveChartViewerState,
} from "../model/chartViewerLogic";

function buildLiveChartRequest(
  derived: ReturnType<typeof deriveChartViewerState>,
  selectedComplexView: ComplexViewOption[],
  selectedComplexViewSingle: ComplexViewOption,
  liveWindowCount: number,
): LiveChartDataOptions | null {
  const tailCount = liveWindowCount;

  if (derived.effectiveView === "heatmap" && derived.heatmapQuantity) {
    return {
      view: "heatmap",
      quantity: derived.heatmapQuantity.name,
      complexViewSingle: derived.heatmapQuantity.isComplex
        ? selectedComplexViewSingle
        : undefined,
    };
  }

  if (derived.effectiveView !== "xy" || !derived.effectiveDrawStyle) {
    return null;
  }

  const roleOptions = derived.liveMonitorUsesForcedRoles
    ? {
        traceGroupIndexColumns:
          derived.liveMonitorTraceGroupIndexColumnNames.length > 0
            ? derived.liveMonitorTraceGroupIndexColumnNames
            : undefined,
        sweepIndexColumn: derived.liveMonitorSweepIndexColumnName ?? undefined,
      }
    : {};

  if (
    derived.effectivePlotMode === "quantity_vs_sweep" &&
    derived.sweepQuantity
  ) {
    return {
      view: "xy",
      plotMode: "quantity_vs_sweep",
      drawStyle: derived.effectiveDrawStyle,
      quantity: derived.sweepQuantity.name,
      complexViews: derived.sweepQuantity.isComplex
        ? selectedComplexView
        : undefined,
      tailCount,
      ...roleOptions,
    };
  }

  if (
    derived.effectivePlotMode === "xy" &&
    derived.xyXColumn &&
    derived.xyYColumn
  ) {
    return {
      view: "xy",
      plotMode: "xy",
      drawStyle: derived.effectiveDrawStyle,
      xColumn: derived.xyXColumn.name,
      yColumn: derived.xyYColumn.name,
      tailCount,
      ...roleOptions,
    };
  }

  if (
    derived.effectivePlotMode === "complex_plane" &&
    derived.complexPlaneQuantity
  ) {
    return {
      view: "xy",
      plotMode: "complex_plane",
      drawStyle: derived.effectiveDrawStyle,
      quantity: derived.complexPlaneQuantity.name,
      tailCount,
      ...roleOptions,
    };
  }

  return null;
}

interface UseChartViewerDataArgs {
  datasetId: number;
  availability: ChartViewerAvailability;
  datasetDetail: DatasetDetail | null;
  derived: ReturnType<typeof deriveChartViewerState>;
  selectedComplexView: ComplexViewOption[];
  selectedComplexViewSingle: ComplexViewOption;
  isLiveMode: boolean;
  liveWindowCount: number;
}

export function useChartViewerData({
  datasetId,
  availability,
  datasetDetail,
  derived,
  selectedComplexView,
  selectedComplexViewSingle,
  isLiveMode,
  liveWindowCount,
}: UseChartViewerDataArgs) {
  const queryClient = useQueryClient();
  const queriesEnabled = availability === "available";
  const filterTableQuery = useFilterTableDataQuery(
    datasetId,
    derived.excludeColumns,
    queriesEnabled && !isLiveMode,
  );
  const filterTableData = filterTableQuery.data ?? null;

  const cascade = useCascadeSelection(filterTableData);
  const filterRow = cascade.resolvedRow ?? null;
  const hasFilters = (filterTableData?.fields.length ?? 0) > 0;
  const indexFilters = hasFilters ? filterRow?.valueIndices : undefined;
  const writeStatus = useDatasetWriteStatusQuery(
    datasetId,
    queriesEnabled && datasetDetail?.status === "Writing" && !isLiveMode,
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

  const liveChartRequest =
    isLiveMode && queriesEnabled
      ? buildLiveChartRequest(
          derived,
          selectedComplexView,
          selectedComplexViewSingle,
          liveWindowCount,
        )
      : null;

  const liveChartQuery = useLiveChartDataQuery(
    datasetId,
    isLiveMode ? liveChartRequest : null,
  );

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
    queriesEnabled && !isLiveMode ? chartRequest : null,
  );

  const activeChartData = isLiveMode ? liveChartQuery.data : chartQuery.data;
  const activeChartError = isLiveMode ? liveChartQuery.error : chartQuery.error;
  const chartInteractionKey = JSON.stringify([
    datasetId,
    isLiveMode ? liveChartRequest : chartRequest,
  ]);

  const chartData = activeChartData;
  const chartError = !queriesEnabled
    ? null
    : activeChartError
      ? activeChartError instanceof Error
        ? activeChartError.message
        : "Failed to load chart data."
      : null;

  return {
    chartData,
    chartInteractionKey,
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
