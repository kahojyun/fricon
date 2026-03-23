import { useState } from "react";
import { toast } from "sonner";
import { commands } from "@/shared/lib/bindings";
import type { UiPreviewImportResult } from "../api/types";

export function useDatasetImportFlow() {
  const [previewResults, setPreviewResults] = useState<UiPreviewImportResult[]>([]);
  const [isImporting, setIsImporting] = useState(false);
  const [isDialogOpen, setIsDialogOpen] = useState(false);

  const startImportDialog = () => {
    commands
      .previewImportDialog()
      .then((result) => {
        if (result.status === "ok" && result.data && result.data.length > 0) {
          setPreviewResults(result.data);
          setIsDialogOpen(true);
        } else if (result.status === "error") {
          toast.error(`Import preview failed: ${result.error.message}`);
        }
      })
      .catch((e) => {
        toast.error(`Import error: ${e instanceof Error ? e.message : String(e)}`);
      });
  };

  const startImportFromFiles = (paths: string[]) => {
    commands
      .previewImportFiles(paths)
      .then((result) => {
        if (result.status === "ok" && result.data.length > 0) {
          setPreviewResults(result.data);
          setIsDialogOpen(true);
        } else if (result.status === "error") {
          toast.error(`Import preview failed: ${result.error.message}`);
        }
      })
      .catch((e) => {
        toast.error(`Import error: ${e instanceof Error ? e.message : String(e)}`);
      });
  };

  const confirmImport = async () => {
    if (previewResults.length === 0) return;
    setIsImporting(true);

    let successCount = 0;
    let failCount = 0;

    for (const p of previewResults) {
      const force = p.preview.conflict !== null;
      try {
        const result = await commands.importDataset(p.archivePath, force);
        if (result.status === "ok") {
          successCount++;
        } else {
          failCount++;
          toast.error(`Failed to import ${p.preview.metadata.name}: ${result.error.message}`);
        }
      } catch (e) {
        failCount++;
        toast.error(`Error importing ${p.preview.metadata.name}: ${e instanceof Error ? e.message : String(e)}`);
      }
    }

    setIsImporting(false);
    setIsDialogOpen(false);
    setPreviewResults([]);

    if (successCount > 0 && failCount === 0) {
      toast.success(`Successfully imported ${successCount} dataset(s)`);
    } else if (successCount > 0 && failCount > 0) {
      toast.warning(`Imported ${successCount} dataset(s), but ${failCount} failed.`);
    }
  };

  const closeDialog = () => {
    setIsDialogOpen(false);
    setPreviewResults([]);
  };

  return {
    previewResults,
    isImporting,
    isDialogOpen,
    setIsDialogOpen: closeDialog,
    startImportDialog,
    startImportFromFiles,
    confirmImport,
  };
}
