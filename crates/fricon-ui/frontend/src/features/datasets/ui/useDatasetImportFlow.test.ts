import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useDatasetImportFlow } from "./useDatasetImportFlow";
import type { UiPreviewImportResult } from "../api/types";

type PreviewImportFilesFn = (paths: string[]) => Promise<unknown>;
type ImportDatasetFn = (
  archivePath: string,
  force: boolean,
) => Promise<unknown>;

const { previewImportFilesMock, importDatasetMock, toastError, toastSuccess } =
  vi.hoisted(() => ({
    previewImportFilesMock: vi.fn<PreviewImportFilesFn>(),
    importDatasetMock: vi.fn<ImportDatasetFn>(),
    toastError: vi.fn(),
    toastSuccess: vi.fn(),
  }));

vi.mock("@/shared/lib/bindings", () => ({
  commands: {
    previewImportDialog: vi.fn(),
    previewImportFiles: (paths: string[]) => previewImportFilesMock(paths),
    importDataset: (archivePath: string, force: boolean) =>
      importDatasetMock(archivePath, force),
  },
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

describe("useDatasetImportFlow", () => {
  beforeEach(() => {
    previewImportFilesMock.mockReset();
    importDatasetMock.mockReset();
    toastError.mockReset();
    toastSuccess.mockReset();
  });

  it("blocks confirm when the preview batch contains duplicate UUIDs", async () => {
    previewImportFilesMock.mockResolvedValue({
      status: "ok",
      data: [
        makePreviewResult("/tmp/alpha.tar.zst", "dup-uid", { name: "Alpha" }),
        makePreviewResult("/tmp/beta.tar.zst", "dup-uid", { name: "Beta" }),
      ],
    });

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
    previewImportFilesMock.mockResolvedValue({
      status: "ok",
      data: [
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
      ],
    });
    importDatasetMock.mockResolvedValue({
      status: "ok",
      data: null,
    });

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
});
