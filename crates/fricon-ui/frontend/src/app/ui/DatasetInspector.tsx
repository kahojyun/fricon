import { MousePointerClick } from "lucide-react";
import { ChartViewer } from "@/features/charts";
import { DatasetPropertiesPanel, useDatasetDetailQuery } from "@/features/datasets";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/shared/ui/tabs";

interface DatasetInspectorProps {
  datasetId?: number;
  onDatasetUpdated?: () => void;
}

export function DatasetInspector({
  datasetId,
  onDatasetUpdated,
}: DatasetInspectorProps) {
  if (!datasetId) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-3 text-muted-foreground">
        <MousePointerClick className="size-10 opacity-30" />
        <div className="text-center">
          <p className="text-sm font-medium">No dataset selected</p>
          <p className="mt-0.5 text-xs">
            Choose a dataset from the list to view charts and metadata.
          </p>
        </div>
      </div>
    );
  }

  return (
    <SelectedDatasetInspector
      datasetId={datasetId}
      onDatasetUpdated={onDatasetUpdated}
    />
  );
}

interface SelectedDatasetInspectorProps {
  datasetId: number;
  onDatasetUpdated?: () => void;
}

function SelectedDatasetInspector({
  datasetId,
  onDatasetUpdated,
}: SelectedDatasetInspectorProps) {
  const detailQuery = useDatasetDetailQuery(datasetId);
  const detail = detailQuery.data ?? null;
  const loadErrorMessage =
    detailQuery.error instanceof Error ? detailQuery.error.message : null;

  return (
    <div className="flex h-full min-h-0 flex-col overflow-hidden">
      <Tabs defaultValue="charts" className="flex h-full min-h-0 flex-col">
        <TabsList>
          <TabsTrigger value="charts">Charts</TabsTrigger>
          <TabsTrigger value="properties">Properties</TabsTrigger>
        </TabsList>

        <TabsContent
          value="charts"
          className="flex min-h-0 flex-1 flex-col overflow-hidden"
        >
          <div className="min-h-0 flex-1 overflow-hidden">
            <ChartViewer datasetId={datasetId} datasetDetail={detail} />
          </div>
        </TabsContent>

        <TabsContent
          value="properties"
          className="flex min-h-0 flex-1 flex-col overflow-hidden"
        >
          <DatasetPropertiesPanel
            datasetId={datasetId}
            detail={detail}
            isLoading={detailQuery.isLoading}
            loadErrorMessage={loadErrorMessage}
            onDatasetUpdated={onDatasetUpdated}
          />
        </TabsContent>
      </Tabs>
    </div>
  );
}
