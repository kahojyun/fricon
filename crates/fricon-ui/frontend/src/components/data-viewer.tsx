import { useEffect, useMemo, useState } from "react";
import { useNavigate } from "@tanstack/react-router";
import { getWorkspaceInfo } from "@/lib/backend";
import { useWorkspaceStore } from "@/lib/useWorkspaceStore";
import { DatasetTable } from "@/components/dataset-table";
import { DatasetDetailPage } from "@/components/dataset-detail-page";

interface DataViewerProps {
  datasetId?: string;
}

export function DataViewer({ datasetId }: DataViewerProps) {
  const setPath = useWorkspaceStore((state) => state.setPath);
  const navigate = useNavigate();
  const parsedDatasetId = useMemo(() => {
    if (!datasetId?.trim()) return undefined;
    const parsed = Number.parseInt(datasetId, 10);
    return Number.isFinite(parsed) ? parsed : undefined;
  }, [datasetId]);
  const [selectedDatasetId, setSelectedDatasetId] = useState<
    number | undefined
  >(parsedDatasetId);

  useEffect(() => {
    let isActive = true;
    getWorkspaceInfo()
      .then((info) => {
        if (isActive) {
          setPath(info.path);
        }
      })
      .catch(() => {
        if (isActive) {
          setPath("(no workspace)");
        }
      });
    return () => {
      isActive = false;
    };
  }, [setPath]);

  useEffect(() => {
    setSelectedDatasetId(parsedDatasetId);
  }, [parsedDatasetId]);

  const handleDatasetSelected = (id: number) => {
    setSelectedDatasetId(id);
    if (datasetId !== String(id)) {
      void navigate({ to: "/datasets/$id", params: { id: String(id) } });
    }
  };

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="grid min-h-0 flex-1 grid-cols-[minmax(0,1fr)_minmax(0,2fr)]">
        <section className="min-h-0 overflow-hidden border-r">
          <DatasetTable
            selectedDatasetId={selectedDatasetId}
            onDatasetSelected={handleDatasetSelected}
          />
        </section>
        <section className="min-h-0 overflow-hidden p-4">
          {selectedDatasetId ? (
            <DatasetDetailPage datasetId={selectedDatasetId} />
          ) : (
            <>
              <h2 className="text-lg font-semibold">No dataset selected</h2>
              <p className="text-muted-foreground mt-1 text-sm">
                Choose a dataset to view charts and metadata.
              </p>
            </>
          )}
        </section>
      </div>
    </div>
  );
}
