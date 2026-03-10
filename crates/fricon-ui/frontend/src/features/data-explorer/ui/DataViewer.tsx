import { useEffect, useState } from "react";
import { MousePointerClick } from "lucide-react";
import { DatasetTable } from "@/features/dataset-table";
import { DatasetDetailPage } from "@/features/dataset-detail";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/shared/ui/resizable";

interface DataViewerProps {
  datasetId?: string;
}

export function DataViewer({ datasetId }: DataViewerProps) {
  const parsedDatasetId = (() => {
    if (!datasetId?.trim()) return undefined;
    const parsed = Number.parseInt(datasetId, 10);
    return Number.isFinite(parsed) ? parsed : undefined;
  })();
  const [selectedDatasetId, setSelectedDatasetId] = useState<
    number | undefined
  >(parsedDatasetId);

  useEffect(() => {
    setSelectedDatasetId(parsedDatasetId);
  }, [parsedDatasetId]);

  const handleDatasetSelected = (id: number) => {
    setSelectedDatasetId(id);
  };

  return (
    <div className="flex h-full min-h-0 flex-col">
      <ResizablePanelGroup orientation="horizontal" className="min-h-0 flex-1">
        <ResizablePanel defaultSize={30} minSize={20}>
          <DatasetTable
            selectedDatasetId={selectedDatasetId}
            onDatasetSelected={handleDatasetSelected}
          />
        </ResizablePanel>
        <ResizableHandle withHandle />
        <ResizablePanel defaultSize={70} minSize={35}>
          {selectedDatasetId ? (
            <div className="h-full min-h-0 p-3">
              <DatasetDetailPage datasetId={selectedDatasetId} />
            </div>
          ) : (
            <div className="flex h-full flex-col items-center justify-center gap-3 text-muted-foreground">
              <MousePointerClick className="size-10 opacity-30" />
              <div className="text-center">
                <p className="text-sm font-medium">No dataset selected</p>
                <p className="mt-0.5 text-xs">
                  Choose a dataset from the list to view charts and metadata.
                </p>
              </div>
            </div>
          )}
        </ResizablePanel>
      </ResizablePanelGroup>
    </div>
  );
}
