import { useState } from "react";
import { toast } from "sonner";
import { commands } from "@/shared/lib/bindings";
import type { UiPreviewImportResult } from "../api/types";

export function useDatasetImportFlow() {
  const [previewResult, setPreviewResult] = useState<UiPreviewImportResult | null>(null);
  const [isImporting, setIsImporting] = useState(false);
  const [isDialogOpen, setIsDialogOpen] = useState(false);

  const startImportDialog = () => {
    commands
      .previewImportDialog()
      .then((result) => {
        if (result.status === "ok" && result.data) {
          setPreviewResult(result.data);
          setIsDialogOpen(true);
        } else if (result.status === "error") {
          toast.error(`Import preview failed: ${result.error.message}`);
        }
      })
      .catch((e) => {
        toast.error(`Import error: ${e instanceof Error ? e.message : String(e)}`);
      });
  };

  const startImportFromFile = (path: string) => {
    commands
      .previewImportFile(path)
      .then((result) => {
        if (result.status === "ok") {
          setPreviewResult(result.data);
          setIsDialogOpen(true);
        } else if (result.status === "error") {
          toast.error(`Import preview failed: ${result.error.message}`);
        }
      })
      .catch((e) => {
        toast.error(`Import error: ${e instanceof Error ? e.message : String(e)}`);
      });
  };

  const confirmImport = () => {
    if (!previewResult) return;
    const force = previewResult.preview.conflict !== null;
    setIsImporting(true);

    commands.importDataset(previewResult.archivePath, force)
      .then((result) => {
        if (result.status === "ok") {
          toast.success(`Dataset imported successfully`);
          setIsDialogOpen(false);
          setPreviewResult(null);
        } else if (result.status === "error") {
          toast.error(`Import failed: ${result.error.message}`);
        }
      })
      .catch((e) => {
        toast.error(`Import error: ${e instanceof Error ? e.message : String(e)}`);
      })
      .finally(() => {
        setIsImporting(false);
      });
  };

  const closeDialog = () => {
    setIsDialogOpen(false);
    setPreviewResult(null);
  };

  return {
    previewResult,
    isImporting,
    isDialogOpen,
    setIsDialogOpen: closeDialog,
    startImportDialog,
    startImportFromFile,
    confirmImport,
  };
}
