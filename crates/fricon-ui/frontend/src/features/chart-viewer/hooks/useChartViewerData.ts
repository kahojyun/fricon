import type { DatasetDetail } from "@/shared/lib/backend";
import type { ComplexViewOption } from "@/shared/lib/chartTypes";
import { useChartDataQuery } from "../api/useChartDataQuery";
import { useDatasetWriteStatusQuery } from "../api/useDatasetWriteStatusQuery";
import { useFilterTableDataQuery } from "../api/useFilterTableDataQuery";
import { useCascadeSelection } from "./useCascadeSelection";
import {
  buildChartRequest,
  deriveChartViewerState,
} from "../model/chartViewerLogic";

interface UseChartViewerDataArgs {
  datasetId: number;
  datasetDetail: DatasetDetail | null;
  derived: ReturnType<typeof deriveChartViewerState>;
  selectedComplexView: ComplexViewOption[];
  selectedComplexViewSingle: ComplexViewOption;
}

export function useChartViewerData({
  datasetId,
  datasetDetail,
  derived,
  selectedComplexView,
  selectedComplexViewSingle,
}: UseChartViewerDataArgs) {
  const filterTableQuery = useFilterTableDataQuery(
    datasetId,
    derived.excludeColumns,
    Boolean(datasetDetail),
  );
  const filterTableData = filterTableQuery.data ?? null;

  const cascade = useCascadeSelection(filterTableData);
  const filterRow = cascade.resolvedRow ?? null;
  const hasFilters = (filterTableData?.fields.length ?? 0) > 0;
  const indexFilters = hasFilters ? filterRow?.valueIndices : undefined;

  useDatasetWriteStatusQuery(datasetId, datasetDetail?.status === "Writing");

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

  const chartQuery = useChartDataQuery(datasetId, chartRequest);
  const chartData = chartQuery.data;
  const chartError = chartQuery.error
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
