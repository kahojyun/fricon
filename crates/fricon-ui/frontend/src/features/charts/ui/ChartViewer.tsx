import type { ChartViewerAvailability, DatasetDetail } from "../api/types";
import { useChartViewerData } from "../hooks/useChartViewerData";
import { useChartViewerSelection } from "../hooks/useChartViewerSelection";
import { ChartViewerControls } from "./ChartViewerControls";
import { ChartWrapper } from "./ChartWrapper";
import { FilterTable } from "./FilterTable";
import { Alert, AlertDescription, AlertTitle } from "@/shared/ui/alert";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/shared/ui/resizable";

interface ChartViewerProps {
  datasetId: number;
  datasetDetail: DatasetDetail | null;
}

export function ChartViewer({ datasetId, datasetDetail }: ChartViewerProps) {
  const availability: ChartViewerAvailability =
    datasetDetail === null
      ? "loading"
      : datasetDetail.payloadAvailable
        ? "available"
        : "tombstone";

  const columns = datasetDetail?.columns ?? [];
  const selection = useChartViewerSelection(columns, datasetDetail?.status);
  const isLiveMode = selection.controlState.isLiveMode;
  const { chartData, chartError, filterTableProps } = useChartViewerData({
    datasetId,
    availability,
    datasetDetail,
    derived: selection.derived,
    selectedComplexView: selection.controlState.selectedComplexView,
    selectedComplexViewSingle: selection.controlState.selectedComplexViewSingle,
    isLiveMode,
  });

  if (availability === "tombstone") {
    return (
      <div className="flex size-full min-h-0 flex-col overflow-hidden p-1.5">
        <Alert>
          <AlertTitle>Dataset Payload Deleted</AlertTitle>
          <AlertDescription>
            This dataset is retained as a tombstone. Charts and data access are
            no longer available.
          </AlertDescription>
        </Alert>
      </div>
    );
  }

  if (availability === "loading") {
    return (
      <div className="flex size-full items-center justify-center">
        <div className="text-xs text-muted-foreground">Loading dataset...</div>
      </div>
    );
  }

  return (
    <div className="flex size-full min-h-0 flex-col overflow-hidden">
      <ChartViewerControls
        derived={selection.derived}
        controlState={selection.controlState}
        actions={selection.actions}
        datasetStatus={datasetDetail?.status}
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
        {isLiveMode ? (
          <div className="h-full min-h-0">
            <ChartWrapper data={chartData} liveMode />
          </div>
        ) : (
          <ResizablePanelGroup
            orientation="vertical"
            className="h-full min-h-0"
          >
            <ResizablePanel defaultSize={70} minSize={35} className="min-h-0">
              <div className="h-full min-h-0">
                <ChartWrapper data={chartData} />
              </div>
            </ResizablePanel>
            <ResizableHandle withHandle />
            <ResizablePanel defaultSize={30} minSize={20} className="min-h-0">
              <div className="h-full min-h-0">
                <FilterTable {...filterTableProps} />
              </div>
            </ResizablePanel>
          </ResizablePanelGroup>
        )}
      </div>
    </div>
  );
}
