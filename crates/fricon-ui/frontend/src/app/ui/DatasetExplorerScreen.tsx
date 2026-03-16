import { useState } from "react";
import { DatasetTable } from "@/features/datasets";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/shared/ui/resizable";
import { DatasetInspector } from "./DatasetInspector";

export function DatasetExplorerScreen() {
  const [selectedDatasetId, setSelectedDatasetId] = useState<
    number | undefined
  >(undefined);

  return (
    <div className="flex h-full min-h-0 flex-col">
      <ResizablePanelGroup orientation="horizontal" className="min-h-0 flex-1">
        <ResizablePanel defaultSize={30} minSize={20}>
          <DatasetTable
            selectedDatasetId={selectedDatasetId}
            onDatasetSelected={setSelectedDatasetId}
          />
        </ResizablePanel>
        <ResizableHandle withHandle />
        <ResizablePanel defaultSize={70} minSize={35}>
          <div className="h-full min-h-0 p-3">
            <DatasetInspector datasetId={selectedDatasetId} />
          </div>
        </ResizablePanel>
      </ResizablePanelGroup>
    </div>
  );
}
