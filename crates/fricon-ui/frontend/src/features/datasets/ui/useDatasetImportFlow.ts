import { useRef, useState } from "react";
import { toast } from "sonner";
import { isApiError } from "@/shared/lib/tauri";
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
  const activeFlowIdRef = useRef(0);
  const isImportingRef = useRef(false);
  const duplicateBatchConflicts = getDuplicateBatchConflicts(previewResults);
  const hasDuplicateBatchConflicts = duplicateBatchConflicts.length > 0;

  const clearDialogState = () => {
    setIsDialogOpen(false);
    setPreviewResults([]);
  };

  const setImportingState = (value: boolean) => {
    isImportingRef.current = value;
    setIsImporting(value);
  };

  const startPreviewFlow = (
    loadPreview: () => Promise<UiPreviewImportResult[] | null | undefined>,
  ) => {
    if (isImportingRef.current) {
      return;
    }

    const flowId = activeFlowIdRef.current + 1;
    activeFlowIdRef.current = flowId;

    loadPreview()
      .then((results) => {
        if (flowId !== activeFlowIdRef.current || isImportingRef.current) {
          return;
        }

        if (results && results.length > 0) {
          setPreviewResults(results);
          setIsDialogOpen(true);
        }
      })
      .catch((e) => {
        if (flowId !== activeFlowIdRef.current || isImportingRef.current) {
          return;
        }

        if (isApiError(e) && e.code === "archive_version_unsupported") {
          toast.error(
            "One or more selected archives were created by a newer version of fricon. Update fricon and try again.",
          );
          return;
        }

        toast.error(
          `Import error: ${e instanceof Error ? e.message : String(e)}`,
        );
      });
  };

  const startImportDialog = () => {
    startPreviewFlow(() => previewImportDialog());
  };

  const startImportFromFiles = (paths: string[]) => {
    startPreviewFlow(() => previewImportFiles(paths));
  };

  const confirmImport = async () => {
    if (isImportingRef.current) return;
    if (previewResults.length === 0) return;
    if (hasDuplicateBatchConflicts) {
      toast.error("Remove duplicate dataset UUIDs before importing.");
      return;
    }

    const flowId = activeFlowIdRef.current;
    const previewBatch = previewResults;

    setImportingState(true);

    let successCount = 0;
    let failCount = 0;

    await (async () => {
      for (const p of previewBatch) {
        const force = p.preview.conflict !== null;
        try {
          await importDataset(p.archivePath, force);
          successCount = successCount + 1;
        } catch (e) {
          failCount = failCount + 1;
          if (isApiError(e) && e.code === "archive_version_unsupported") {
            toast.error(
              `Can't import ${p.preview.metadata.name}: this archive was created by a newer version of fricon. Update fricon and try again.`,
            );
            continue;
          }

          toast.error(
            `Error importing ${p.preview.metadata.name}: ${e instanceof Error ? e.message : String(e)}`,
          );
        }
      }
    })().finally(() => {
      setImportingState(false);
      if (flowId === activeFlowIdRef.current) {
        clearDialogState();
      }
    });

    if (successCount > 0 && failCount === 0) {
      toast.success(`Successfully imported ${successCount} dataset(s)`);
    } else if (successCount > 0 && failCount > 0) {
      toast.warning(
        `Imported ${successCount} dataset(s), but ${failCount} failed.`,
      );
    }
  };

  const closeDialog = () => {
    if (isImportingRef.current) {
      return;
    }

    activeFlowIdRef.current += 1;
    clearDialogState();
  };

  return {
    previewResults,
    isImporting,
    isDialogOpen,
    duplicateBatchConflicts,
    hasDuplicateBatchConflicts,
    closeDialog,
    startImportDialog,
    startImportFromFiles,
    confirmImport,
  };
}
