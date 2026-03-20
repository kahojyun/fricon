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

interface DatasetDeleteFlowMessages {
  actionLabel: string;
  success: (count: number) => string;
  failure: (count: number) => string;
  partial: (successCount: number, failureCount: number) => string;
}

interface UseDatasetDeleteFlowArgs {
  deleteDatasets: (ids: number[]) => Promise<DatasetDeleteResult[]>;
  isDeleting: boolean;
  selectedDatasetId?: number;
  onDatasetSelected: (id?: number) => void;
  setRowSelection: (selection: Record<string, boolean>) => void;
  notify: DatasetDeleteFlowNotifier;
  messages: DatasetDeleteFlowMessages;
}

async function requestDatasetDelete({
  deleteDatasets,
  ids,
  notify,
}: {
  deleteDatasets: (ids: number[]) => Promise<DatasetDeleteResult[]>;
  ids: number[];
  notify: DatasetDeleteFlowNotifier;
}): Promise<DatasetDeleteResult[] | null> {
  try {
    return await deleteDatasets(ids);
  } catch (error) {
    notify.error(
      error instanceof Error ? error.message : "Failed to delete dataset(s)",
    );
    return null;
  }
}

export function useDatasetDeleteFlow({
  deleteDatasets,
  isDeleting,
  selectedDatasetId,
  onDatasetSelected,
  setRowSelection,
  notify,
  messages,
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

  const executeDelete = async (
    ids: number[],
    closeDialogOnSuccess: boolean,
  ) => {
    const results = await requestDatasetDelete({
      deleteDatasets,
      ids,
      notify,
    });
    if (!results) {
      return;
    }

    const summary = summarizeDatasetDeleteResults(results);

    if (
      selectedDatasetId !== undefined &&
      summary.successIds.includes(selectedDatasetId)
    ) {
      onDatasetSelected(undefined);
    }

    if (summary.outcome === "success") {
      setRowSelection({});
      if (closeDialogOnSuccess) {
        setIdsToDelete([]);
        setIsDeleteDialogOpen(false);
      }
      notify.success(messages.success(summary.successIds.length));
      return;
    }

    if (summary.outcome === "failure") {
      setRowSelection(buildSelectionFromIds(summary.failedIds));
      notify.error(messages.failure(summary.failedResults.length));
      return;
    }

    setRowSelection(buildSelectionFromIds(summary.failedIds));
    if (closeDialogOnSuccess) {
      setIdsToDelete(summary.failedIds);
    }
    notify.warning(
      messages.partial(summary.successIds.length, summary.failedResults.length),
      {
        description: summary.failedResults
          .map((result) => `ID ${result.id}: ${result.error}`)
          .join("\n"),
      },
    );
  };

  const confirmDelete = async () => {
    await executeDelete(idsToDelete, true);
  };

  const performDelete = async (ids: number[]) => {
    await executeDelete(ids, false);
  };

  return {
    actionLabel: messages.actionLabel,
    idsToDelete,
    isDeleteDialogOpen,
    openDeleteDialog,
    closeDeleteDialog,
    setIsDeleteDialogOpen,
    confirmDelete,
    performDelete,
  };
}
