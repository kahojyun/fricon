import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import type {
  ChartViewerAvailability,
  DatasetDetail,
  ScatterModeOptions,
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

function buildScatterModeOptions(
  derived: ReturnType<typeof deriveChartViewerState>,
): ScatterModeOptions | null {
  if (derived.effectiveScatterMode === "complex" && derived.scatterSeries) {
    return { mode: "complex", series: derived.scatterSeries.name };
  }
  if (
    derived.effectiveScatterMode === "trace_xy" &&
    derived.scatterTraceXColumn &&
    derived.scatterTraceYColumn
  ) {
    return {
      mode: "trace_xy",
      traceXColumn: derived.scatterTraceXColumn.name,
      traceYColumn: derived.scatterTraceYColumn.name,
    };
  }
  if (derived.scatterXColumn && derived.scatterYColumn) {
    return {
      mode: "xy",
      xColumn: derived.scatterXColumn.name,
      yColumn: derived.scatterYColumn.name,
      binColumn: derived.scatterBinColumn?.name ?? null,
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
}

export function useChartViewerData({
  datasetId,
  availability,
  datasetDetail,
  derived,
  selectedComplexView,
  selectedComplexViewSingle,
  isLiveMode,
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

  // Build live chart request when in live mode
  const liveChartRequest = (() => {
    if (!isLiveMode || !queriesEnabled) return null;

    const tailCount = 5;

    if (derived.effectiveChartType === "line" && derived.series) {
      return {
        chartType: "line" as const,
        series: derived.series.name,
        complexViews: derived.series.isComplex ? selectedComplexView : null,
        tailCount,
      };
    }

    if (derived.effectiveChartType === "heatmap" && derived.series) {
      return {
        chartType: "heatmap" as const,
        series: derived.series.name,
        complexViewSingle: derived.series.isComplex
          ? (selectedComplexViewSingle ?? "mag")
          : null,
      };
    }

    if (derived.effectiveChartType === "scatter") {
      const scatter = buildScatterModeOptions(derived);
      if (!scatter) return null;
      return {
        chartType: "scatter" as const,
        scatter,
        tailCount,
      };
    }

    return null;
  })();

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
