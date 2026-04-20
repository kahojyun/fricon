import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useDatasetDeleteFlow } from "./useDatasetDeleteFlow";

function createNotifier() {
  return {
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
  };
}

describe("useDatasetDeleteFlow", () => {
  it("clears selection and closes the dialog after a successful delete", async () => {
    const notifier = createNotifier();
    const setRowSelection = vi.fn();
    const onDatasetSelected = vi.fn();
    const deleteDatasets = vi.fn().mockResolvedValue([
      { id: 11, success: true, error: null },
      { id: 12, success: true, error: null },
    ]);

    const { result } = renderHook(() =>
      useDatasetDeleteFlow({
        deleteDatasets,
        isDeleting: false,
        selectedDatasetId: 12,
        onDatasetSelected,
        setRowSelection,
        notify: notifier,
        messages: {
          actionLabel: "Delete",
          success: (count) => `Deleted ${count} dataset(s)`,
          failure: (count) => `Failed to delete ${count} dataset(s)`,
          partial: (successCount, failureCount) =>
            `Deleted ${successCount} dataset(s), but ${failureCount} failed.`,
        },
      }),
    );

    act(() => {
      result.current.openDeleteDialog([11, 12]);
    });

    await act(async () => {
      await result.current.confirmDelete();
    });

    expect(deleteDatasets).toHaveBeenCalledWith([11, 12]);
    expect(setRowSelection).toHaveBeenCalledWith({});
    expect(onDatasetSelected).toHaveBeenCalledWith(undefined);
    expect(notifier.success).toHaveBeenCalledWith("Deleted 2 dataset(s)");
    expect(result.current.idsToDelete).toEqual([]);
    expect(result.current.isDeleteDialogOpen).toBe(false);
  });

  it("keeps failed ids selected after a full delete failure", async () => {
    const notifier = createNotifier();
    const setRowSelection = vi.fn();
    const deleteDatasets = vi.fn().mockResolvedValue([
      {
        id: 12,
        success: false,
        error: { code: "internal", message: "locked" },
      },
    ]);

    const { result } = renderHook(() =>
      useDatasetDeleteFlow({
        deleteDatasets,
        isDeleting: false,
        onDatasetSelected: vi.fn(),
        setRowSelection,
        notify: notifier,
        messages: {
          actionLabel: "Delete",
          success: (count) => `Deleted ${count} dataset(s)`,
          failure: (count) => `Failed to delete ${count} dataset(s)`,
          partial: (successCount, failureCount) =>
            `Deleted ${successCount} dataset(s), but ${failureCount} failed.`,
        },
      }),
    );

    act(() => {
      result.current.openDeleteDialog([12]);
    });

    await act(async () => {
      await result.current.confirmDelete();
    });

    expect(setRowSelection).toHaveBeenCalledWith({ "12": true });
    expect(notifier.error).toHaveBeenCalledWith(
      "Failed to delete 1 dataset(s)",
    );
    expect(result.current.idsToDelete).toEqual([12]);
    expect(result.current.isDeleteDialogOpen).toBe(true);
  });

  it("keeps only failed ids pending after a partial delete failure", async () => {
    const notifier = createNotifier();
    const setRowSelection = vi.fn();
    const onDatasetSelected = vi.fn();
    const deleteDatasets = vi.fn().mockResolvedValue([
      { id: 11, success: true, error: null },
      {
        id: 12,
        success: false,
        error: { code: "internal", message: "locked" },
      },
    ]);

    const { result } = renderHook(() =>
      useDatasetDeleteFlow({
        deleteDatasets,
        isDeleting: false,
        selectedDatasetId: 11,
        onDatasetSelected,
        setRowSelection,
        notify: notifier,
        messages: {
          actionLabel: "Delete",
          success: (count) => `Deleted ${count} dataset(s)`,
          failure: (count) => `Failed to delete ${count} dataset(s)`,
          partial: (successCount, failureCount) =>
            `Deleted ${successCount} dataset(s), but ${failureCount} failed.`,
        },
      }),
    );

    act(() => {
      result.current.openDeleteDialog([11, 12]);
    });

    await act(async () => {
      await result.current.confirmDelete();
    });

    expect(setRowSelection).toHaveBeenCalledWith({ "12": true });
    expect(onDatasetSelected).toHaveBeenCalledWith(undefined);
    expect(notifier.warning).toHaveBeenCalledWith(
      "Deleted 1 dataset(s), but 1 failed.",
      {
        description: "ID 12: locked",
      },
    );
    expect(result.current.idsToDelete).toEqual([12]);
    expect(result.current.isDeleteDialogOpen).toBe(true);
  });
});
