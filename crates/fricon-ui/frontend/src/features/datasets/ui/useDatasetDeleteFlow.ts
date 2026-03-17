import { useState } from "react";
import type { DatasetDeleteResult } from "../api/types";
import {
  buildSelectionFromIds,
  summarizeDatasetDeleteResults,
} from "../model/datasetTableDeleteFlowLogic";

interface DatasetDeleteFlowNotifier {
  success: (message: string) => void;
  error: (message: string, options?: { description?: string }) => void;
  warning: (message: string, options?: { description?: string }) => void;
}

interface UseDatasetDeleteFlowArgs {
  deleteDatasets: (ids: number[]) => Promise<DatasetDeleteResult[]>;
  isDeleting: boolean;
  selectedDatasetId?: number;
  onDatasetSelected: (id?: number) => void;
  setRowSelection: (selection: Record<string, boolean>) => void;
  notify: DatasetDeleteFlowNotifier;
}

export function useDatasetDeleteFlow({
  deleteDatasets,
  isDeleting,
  selectedDatasetId,
  onDatasetSelected,
  setRowSelection,
  notify,
}: UseDatasetDeleteFlowArgs) {
  const [idsToDelete, setIdsToDelete] = useState<number[]>([]);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);

  const openDeleteDialog = (ids: number[]) => {
    setIdsToDelete(ids);
    setIsDeleteDialogOpen(true);
  };

  const closeDeleteDialog = () => {
    if (isDeleting) {
      return;
    }

    setIsDeleteDialogOpen(false);
  };

  const confirmDelete = async () => {
    try {
      const results = await deleteDatasets(idsToDelete);
      const summary = summarizeDatasetDeleteResults(results);

      if (
        selectedDatasetId !== undefined &&
        summary.successIds.includes(selectedDatasetId)
      ) {
        onDatasetSelected(undefined);
      }

      if (summary.outcome === "success") {
        setRowSelection({});
        setIdsToDelete([]);
        setIsDeleteDialogOpen(false);
        notify.success(
          `Successfully deleted ${summary.successIds.length} dataset(s)`,
        );
        return;
      }

      if (summary.outcome === "failure") {
        setRowSelection(buildSelectionFromIds(summary.failedIds));
        notify.error(
          `Failed to delete ${summary.failedResults.length} dataset(s)`,
        );
        return;
      }

      setRowSelection(buildSelectionFromIds(summary.failedIds));
      setIdsToDelete(summary.failedIds);
      notify.warning(
        `Successfully deleted ${summary.successIds.length} dataset(s), but ${summary.failedResults.length} failed.`,
        {
          description: summary.failedResults
            .map((result) => `ID ${result.id}: ${result.error}`)
            .join("\n"),
        },
      );
    } catch (error) {
      notify.error(
        error instanceof Error ? error.message : "Failed to delete dataset(s)",
      );
    }
  };

  return {
    idsToDelete,
    isDeleteDialogOpen,
    openDeleteDialog,
    closeDeleteDialog,
    setIsDeleteDialogOpen,
    confirmDelete,
  };
}
