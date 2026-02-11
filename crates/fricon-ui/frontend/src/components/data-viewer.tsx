import { useEffect, useState } from "react";
import { useWorkspaceStore } from "@/lib/useWorkspaceStore";
import { DatasetTable } from "@/components/dataset-table";
import { DatasetDetailPage } from "@/components/dataset-detail-page";
import { useWorkspaceInfoQuery } from "@/hooks/useWorkspaceInfoQuery";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/components/ui/resizable";

interface DataViewerProps {
  datasetId?: string;
}

export function DataViewer({ datasetId }: DataViewerProps) {
  const setPath = useWorkspaceStore((state) => state.setPath);
  const workspaceInfoQuery = useWorkspaceInfoQuery();
  const parsedDatasetId = (() => {
    if (!datasetId?.trim()) return undefined;
    const parsed = Number.parseInt(datasetId, 10);
    return Number.isFinite(parsed) ? parsed : undefined;
  })();
  const [selectedDatasetId, setSelectedDatasetId] = useState<
    number | undefined
  >(parsedDatasetId);

  useEffect(() => {
    if (workspaceInfoQuery.data) {
      setPath(workspaceInfoQuery.data.path);
      return;
    }
    if (workspaceInfoQuery.isError) {
      setPath("(no workspace)");
    }
  }, [workspaceInfoQuery.data, workspaceInfoQuery.isError, setPath]);

  useEffect(() => {
    setSelectedDatasetId(parsedDatasetId);
  }, [parsedDatasetId]);

  const handleDatasetSelected = (id: number) => {
    setSelectedDatasetId(id);
  };

  return (
    <div className="flex h-full min-h-0 flex-col">
      <ResizablePanelGroup orientation="horizontal" className="min-h-0 flex-1">
        <ResizablePanel defaultSize={35} minSize={25}>
          <DatasetTable
            selectedDatasetId={selectedDatasetId}
            onDatasetSelected={handleDatasetSelected}
          />
        </ResizablePanel>
        <ResizableHandle withHandle />
        <ResizablePanel defaultSize={65} minSize={35}>
          {selectedDatasetId ? (
            <div className="h-full min-h-0 p-4">
              <DatasetDetailPage datasetId={selectedDatasetId} />
            </div>
          ) : (
            <div className="p-4">
              <h2 className="text-lg font-semibold">No dataset selected</h2>
              <p className="text-muted-foreground mt-1 text-sm">
                Choose a dataset to view charts and metadata.
              </p>
            </div>
          )}
        </ResizablePanel>
      </ResizablePanelGroup>
    </div>
  );
}
