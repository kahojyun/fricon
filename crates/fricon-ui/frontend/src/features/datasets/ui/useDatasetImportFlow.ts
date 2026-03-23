import { useState } from "react";
import { toast } from "sonner";
import {
  importDataset,
  previewImportDialog,
  previewImportFiles,
} from "../api/client";
import type { UiPreviewImportResult } from "../api/types";

export interface DuplicateBatchConflict {
  uid: string;
  entries: UiPreviewImportResult[];
}

function getDuplicateBatchConflicts(
  previewResults: UiPreviewImportResult[],
): DuplicateBatchConflict[] {
  const entriesByUid = new Map<string, UiPreviewImportResult[]>();

  for (const result of previewResults) {
    const { uid } = result.preview.metadata;
    const entries = entriesByUid.get(uid);
    if (entries) {
      entries.push(result);
      continue;
    }
    entriesByUid.set(uid, [result]);
  }

  return Array.from(entriesByUid.entries())
    .filter(([, entries]) => entries.length > 1)
    .map(([uid, entries]) => ({ uid, entries }));
}

export function useDatasetImportFlow() {
  const [previewResults, setPreviewResults] = useState<UiPreviewImportResult[]>(
    [],
  );
  const [isImporting, setIsImporting] = useState(false);
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const duplicateBatchConflicts = getDuplicateBatchConflicts(previewResults);
  const hasDuplicateBatchConflicts = duplicateBatchConflicts.length > 0;

  const startImportDialog = () => {
    previewImportDialog()
      .then((results) => {
        if (results && results.length > 0) {
          setPreviewResults(results);
          setIsDialogOpen(true);
        }
      })
      .catch((e) => {
        toast.error(
          `Import error: ${e instanceof Error ? e.message : String(e)}`,
        );
      });
  };

  const startImportFromFiles = (paths: string[]) => {
    previewImportFiles(paths)
      .then((results) => {
        if (results.length > 0) {
          setPreviewResults(results);
          setIsDialogOpen(true);
        }
      })
      .catch((e) => {
        toast.error(
          `Import error: ${e instanceof Error ? e.message : String(e)}`,
        );
      });
  };

  const confirmImport = async () => {
    if (previewResults.length === 0) return;
    if (hasDuplicateBatchConflicts) {
      toast.error("Remove duplicate dataset UUIDs before importing.");
      return;
    }

    setIsImporting(true);

    let successCount = 0;
    let failCount = 0;

    for (const p of previewResults) {
      const force = p.preview.conflict !== null;
      try {
        await importDataset(p.archivePath, force);
        successCount++;
      } catch (e) {
        failCount++;
        toast.error(
          `Error importing ${p.preview.metadata.name}: ${e instanceof Error ? e.message : String(e)}`,
        );
      }
    }

    setIsImporting(false);
    setIsDialogOpen(false);
    setPreviewResults([]);

    if (successCount > 0 && failCount === 0) {
      toast.success(`Successfully imported ${successCount} dataset(s)`);
    } else if (successCount > 0 && failCount > 0) {
      toast.warning(
        `Imported ${successCount} dataset(s), but ${failCount} failed.`,
      );
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
    duplicateBatchConflicts,
    hasDuplicateBatchConflicts,
    setIsDialogOpen: closeDialog,
    startImportDialog,
    startImportFromFiles,
    confirmImport,
  };
}
