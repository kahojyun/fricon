import { useState } from "react";
import type {
  ChartOptions,
  ChartType,
  ComplexViewOption,
  ScatterMode,
} from "@/shared/lib/chartTypes";
import { ChartViewerControls } from "@/features/chart-viewer/ui/ChartViewerControls";
import { ChartWrapper } from "@/features/chart-viewer/ui/ChartWrapper";
import { FilterTable } from "@/features/chart-viewer/ui/FilterTable";
import {
  buildChartRequest,
  deriveChartViewerState,
} from "@/features/chart-viewer/model/chartViewerLogic";
import { useCascadeSelection } from "@/features/chart-viewer/hooks/useCascadeSelection";
import { useChartDataQuery } from "@/features/chart-viewer/api/useChartDataQuery";
import { useDatasetWriteStatusQuery } from "@/features/chart-viewer/api/useDatasetWriteStatusQuery";
import { useFilterTableDataQuery } from "@/features/chart-viewer/api/useFilterTableDataQuery";
import { useDatasetDetailQuery } from "@/features/dataset-detail/api/useDatasetDetailQuery";
import { Alert, AlertDescription, AlertTitle } from "@/shared/ui/alert";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/shared/ui/resizable";

interface ChartViewerProps {
  datasetId: number;
}

export function ChartViewer({ datasetId }: ChartViewerProps) {
  const [chartType, setChartType] = useState<ChartType>("line");
  const [selectedComplexView, setSelectedComplexView] = useState<
    ComplexViewOption[]
  >(["real", "imag"]);
  const [selectedComplexViewSingle, setSelectedComplexViewSingle] =
    useState<ComplexViewOption>("mag");

  const [seriesName, setSeriesName] = useState<string | null>(null);
  const [xColumnName, setXColumnName] = useState<string | null>(null);
  const [yColumnName, setYColumnName] = useState<string | null>(null);

  const [scatterMode, setScatterMode] = useState<ScatterMode>("complex");
  const [scatterSeriesName, setScatterSeriesName] = useState<string | null>(
    null,
  );
  const [scatterTraceXName, setScatterTraceXName] = useState<string | null>(
    null,
  );
  const [scatterTraceYName, setScatterTraceYName] = useState<string | null>(
    null,
  );
  const [scatterXName, setScatterXName] = useState<string | null>(null);
  const [scatterYName, setScatterYName] = useState<string | null>(null);
  const [scatterBinName, setScatterBinName] = useState<string | null>(null);

  const datasetDetailQuery = useDatasetDetailQuery(datasetId);
  const datasetDetail = datasetDetailQuery.data ?? null;
  const columns = datasetDetail?.columns ?? [];
  const derived = deriveChartViewerState(columns, {
    chartType,
    seriesName,
    xColumnName,
    yColumnName,
    scatterMode,
    scatterSeriesName,
    scatterTraceXName,
    scatterTraceYName,
    scatterXName,
    scatterYName,
    scatterBinName,
  });
  const { excludeColumns } = derived;

  const filterTableQuery = useFilterTableDataQuery(
    datasetId,
    excludeColumns,
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
  const data: ChartOptions | undefined = chartQuery.data;
  const chartError = chartQuery.error
    ? chartQuery.error instanceof Error
      ? chartQuery.error.message
      : "Failed to load chart data."
    : null;

  return (
    <div className="flex size-full min-h-0 flex-col overflow-hidden">
      <ChartViewerControls
        derived={derived}
        selectedComplexView={selectedComplexView}
        selectedComplexViewSingle={selectedComplexViewSingle}
        setChartType={setChartType}
        setSeriesName={setSeriesName}
        setXColumnName={setXColumnName}
        setYColumnName={setYColumnName}
        setScatterMode={setScatterMode}
        setScatterSeriesName={setScatterSeriesName}
        setScatterTraceXName={setScatterTraceXName}
        setScatterTraceYName={setScatterTraceYName}
        setScatterXName={setScatterXName}
        setScatterYName={setScatterYName}
        setScatterBinName={setScatterBinName}
        setSelectedComplexView={setSelectedComplexView}
        setSelectedComplexViewSingle={setSelectedComplexViewSingle}
      />

      {chartError ? (
        <div className="px-1.5">
          <Alert variant="destructive">
            <AlertTitle>Chart load failed</AlertTitle>
            <AlertDescription>{chartError}</AlertDescription>
          </Alert>
        </div>
      ) : null}

      <div className="min-h-0 flex-1 overflow-hidden p-1.5">
        <ResizablePanelGroup orientation="vertical" className="h-full min-h-0">
          <ResizablePanel defaultSize={70} minSize={35} className="min-h-0">
            <div className="h-full min-h-0">
              <ChartWrapper data={data} />
            </div>
          </ResizablePanel>
          <ResizableHandle withHandle />
          <ResizablePanel defaultSize={30} minSize={20} className="min-h-0">
            <div className="h-full min-h-0">
              <FilterTable
                data={filterTableData ?? undefined}
                mode={cascade.state.mode}
                onModeChange={cascade.setMode}
                selectedRowIndex={filterRow?.index ?? null}
                onSelectRow={cascade.selectRow}
                selectedValueIndices={cascade.selectedValueIndices}
                onSelectFieldValue={(fieldIndex, valueIndex) => {
                  if (!filterTableData) return;
                  cascade.selectFieldValue(
                    fieldIndex,
                    valueIndex,
                    filterRow?.index ?? null,
                  );
                }}
              />
            </div>
          </ResizablePanel>
        </ResizablePanelGroup>
      </div>
    </div>
  );
}
