import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useDatasetImportFlow } from "./useDatasetImportFlow";
import type { UiPreviewImportResult } from "../api/types";
import { ApiError } from "@/shared/lib/tauri";

type PreviewImportFilesFn = (paths: string[]) => Promise<unknown>;
type PreviewImportDialogFn = () => Promise<unknown>;
type ImportDatasetFn = (
  archivePath: string,
  force: boolean,
) => Promise<unknown>;

const {
  previewImportDialogMock,
  previewImportFilesMock,
  importDatasetMock,
  toastError,
  toastSuccess,
} = vi.hoisted(() => ({
  previewImportDialogMock: vi.fn<PreviewImportDialogFn>(),
  previewImportFilesMock: vi.fn<PreviewImportFilesFn>(),
  importDatasetMock: vi.fn<ImportDatasetFn>(),
  toastError: vi.fn(),
  toastSuccess: vi.fn(),
}));

vi.mock("../api/client", () => ({
  previewImportDialog: () => previewImportDialogMock(),
  previewImportFiles: (paths: string[]) => previewImportFilesMock(paths),
  importDataset: (archivePath: string, force: boolean) =>
    importDatasetMock(archivePath, force),
}));

vi.mock("sonner", () => ({
  toast: {
    error: toastError,
    success: toastSuccess,
    warning: vi.fn(),
  },
}));

function makePreviewResult(
  archivePath: string,
  uid: string,
  options: {
    name?: string;
    conflict?: UiPreviewImportResult["preview"]["conflict"];
  } = {},
): UiPreviewImportResult {
  return {
    archivePath,
    preview: {
      metadata: {
        uid,
        name: options.name ?? uid,
        description: "import preview",
        favorite: false,
        status: "Completed",
        createdAt: "2026-01-01T00:00:00Z",
        tags: [],
      },
      conflict: options.conflict ?? null,
    },
  };
}

function createDeferredPromise<T>() {
  let resolve!: (value: T | PromiseLike<T>) => void;
  const promise = new Promise<T>((res) => {
    resolve = res;
  });

  return { promise, resolve };
}

describe("useDatasetImportFlow", () => {
  beforeEach(() => {
    previewImportDialogMock.mockReset();
    previewImportFilesMock.mockReset();
    importDatasetMock.mockReset();
    toastError.mockReset();
    toastSuccess.mockReset();
  });

  it("blocks confirm when the preview batch contains duplicate UUIDs", async () => {
    previewImportFilesMock.mockResolvedValue([
      makePreviewResult("/tmp/alpha.tar.zst", "dup-uid", { name: "Alpha" }),
      makePreviewResult("/tmp/beta.tar.zst", "dup-uid", { name: "Beta" }),
    ]);

    const { result } = renderHook(() => useDatasetImportFlow());

    act(() => {
      result.current.startImportFromFiles([
        "/tmp/alpha.tar.zst",
        "/tmp/beta.tar.zst",
      ]);
    });

    await waitFor(() => {
      expect(result.current.hasDuplicateBatchConflicts).toBe(true);
    });

    await act(async () => {
      await result.current.confirmImport();
    });

    expect(importDatasetMock).not.toHaveBeenCalled();
    expect(toastError).toHaveBeenCalledWith(
      "Remove duplicate dataset UUIDs before importing.",
    );
  });

  it("allows a non-duplicate batch and forces only workspace conflicts", async () => {
    previewImportFilesMock.mockResolvedValue([
      makePreviewResult("/tmp/alpha.tar.zst", "uid-a", { name: "Alpha" }),
      makePreviewResult("/tmp/beta.tar.zst", "uid-b", {
        name: "Beta",
        conflict: {
          existing: {
            uid: "uid-b",
            name: "Existing Beta",
            description: "existing",
            favorite: false,
            status: "Completed",
            createdAt: "2026-01-01T00:00:00Z",
            tags: [],
          },
          diffs: [],
        },
      }),
    ]);
    importDatasetMock.mockResolvedValue(undefined);

    const { result } = renderHook(() => useDatasetImportFlow());

    act(() => {
      result.current.startImportFromFiles([
        "/tmp/alpha.tar.zst",
        "/tmp/beta.tar.zst",
      ]);
    });

    await waitFor(() => {
      expect(result.current.hasDuplicateBatchConflicts).toBe(false);
      expect(result.current.previewResults).toHaveLength(2);
    });

    await act(async () => {
      await result.current.confirmImport();
    });

    expect(importDatasetMock).toHaveBeenNthCalledWith(
      1,
      "/tmp/alpha.tar.zst",
      false,
    );
    expect(importDatasetMock).toHaveBeenNthCalledWith(
      2,
      "/tmp/beta.tar.zst",
      true,
    );
    expect(toastSuccess).toHaveBeenCalledWith(
      "Successfully imported 2 dataset(s)",
    );
  });

  it("ignores new preview requests while an import is in progress", async () => {
    const importDeferred = createDeferredPromise<void>();
    previewImportFilesMock.mockResolvedValueOnce([
      makePreviewResult("/tmp/alpha.tar.zst", "uid-a", { name: "Alpha" }),
    ]);
    importDatasetMock.mockImplementation(() => importDeferred.promise);

    const { result } = renderHook(() => useDatasetImportFlow());

    act(() => {
      result.current.startImportFromFiles(["/tmp/alpha.tar.zst"]);
    });

    await waitFor(() => {
      expect(result.current.previewResults).toHaveLength(1);
      expect(result.current.isDialogOpen).toBe(true);
    });

    let confirmImportPromise!: Promise<void>;
    act(() => {
      confirmImportPromise = result.current.confirmImport();
    });

    expect(result.current.isImporting).toBe(true);

    act(() => {
      result.current.startImportFromFiles(["/tmp/beta.tar.zst"]);
    });

    expect(previewImportFilesMock).toHaveBeenCalledTimes(1);
    expect(result.current.previewResults).toEqual([
      makePreviewResult("/tmp/alpha.tar.zst", "uid-a", { name: "Alpha" }),
    ]);

    await act(async () => {
      importDeferred.resolve();
      await confirmImportPromise;
    });

    await waitFor(() => {
      expect(result.current.isImporting).toBe(false);
      expect(result.current.isDialogOpen).toBe(false);
      expect(result.current.previewResults).toEqual([]);
    });
  });

  it("shows a friendly message for unsupported archive versions during preview", async () => {
    previewImportFilesMock.mockRejectedValue(
      new ApiError({
        code: "archive_version_unsupported",
        message: "archive format version 2 is not supported",
      }),
    );

    const { result } = renderHook(() => useDatasetImportFlow());

    act(() => {
      result.current.startImportFromFiles(["/tmp/future.tar.zst"]);
    });

    await waitFor(() => {
      expect(toastError).toHaveBeenCalledWith(
        "One or more selected archives were created by a newer version of fricon. Update fricon and try again.",
      );
    });
  });

  it("shows a friendly message for unsupported archive versions during import", async () => {
    previewImportFilesMock.mockResolvedValue([
      makePreviewResult("/tmp/future.tar.zst", "uid-a", { name: "Future" }),
    ]);
    importDatasetMock.mockRejectedValue(
      new ApiError({
        code: "archive_version_unsupported",
        message: "archive format version 2 is not supported",
      }),
    );

    const { result } = renderHook(() => useDatasetImportFlow());

    act(() => {
      result.current.startImportFromFiles(["/tmp/future.tar.zst"]);
    });

    await waitFor(() => {
      expect(result.current.previewResults).toHaveLength(1);
    });

    await act(async () => {
      await result.current.confirmImport();
    });

    expect(toastError).toHaveBeenCalledWith(
      "Can't import Future: this archive was created by a newer version of fricon. Update fricon and try again.",
    );
  });
});
