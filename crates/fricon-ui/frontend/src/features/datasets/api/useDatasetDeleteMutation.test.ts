import { createElement, type ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  useDatasetDeleteMutation,
  useDatasetRestoreMutation,
  useDatasetTrashMutation,
} from "./useDatasetDeleteMutation";
import type { DatasetDeleteResult } from "./types";

type DatasetMutationFn = (ids: number[]) => Promise<DatasetDeleteResult[]>;

const deleteDatasetsMock = vi.fn<DatasetMutationFn>();
const trashDatasetsMock = vi.fn<DatasetMutationFn>();
const restoreDatasetsMock = vi.fn<DatasetMutationFn>();

vi.mock("./client", () => ({
  deleteDatasets: (ids: number[]) => deleteDatasetsMock(ids),
  trashDatasets: (ids: number[]) => trashDatasetsMock(ids),
  restoreDatasets: (ids: number[]) => restoreDatasetsMock(ids),
  emptyTrash: vi.fn(),
}));

function createWrapper(queryClient: QueryClient) {
  return function Wrapper({ children }: { children: ReactNode }) {
    return createElement(
      QueryClientProvider,
      { client: queryClient },
      children,
    );
  };
}

describe("useDatasetDeleteMutation", () => {
  let consoleErrorSpy: { mockRestore: () => void };

  beforeEach(() => {
    deleteDatasetsMock.mockReset();
    trashDatasetsMock.mockReset();
    restoreDatasetsMock.mockReset();
    consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(() => {
      return undefined;
    });
  });

  afterEach(() => {
    consoleErrorSpy.mockRestore();
  });

  it("returns per-id results from the delete API", async () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    const results: DatasetDeleteResult[] = [
      { id: 1, success: true, error: null },
      {
        id: 2,
        success: false,
        error: { code: "dataset_not_found", message: "missing" },
      },
    ];

    deleteDatasetsMock.mockResolvedValue(results);

    const { result } = renderHook(() => useDatasetDeleteMutation(), {
      wrapper: createWrapper(queryClient),
    });

    let actualResults: DatasetDeleteResult[] | undefined;
    await act(async () => {
      actualResults = await result.current.deleteDatasets([1, 2]);
    });

    expect(actualResults).toEqual(results);
    expect(deleteDatasetsMock).toHaveBeenCalledWith([1, 2]);
  });

  it("rethrows API failures and clears deleting state", async () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    let rejectDelete: ((reason?: unknown) => void) | undefined;
    deleteDatasetsMock.mockImplementation(
      () =>
        new Promise<DatasetDeleteResult[]>((_, reject) => {
          rejectDelete = reject;
        }),
    );

    const { result } = renderHook(() => useDatasetDeleteMutation(), {
      wrapper: createWrapper(queryClient),
    });

    let deletePromise!: Promise<DatasetDeleteResult[]>;
    act(() => {
      deletePromise = result.current.deleteDatasets([1]);
    });
    void deletePromise.catch(() => undefined);

    await waitFor(() => {
      expect(result.current.isDeleting).toBe(true);
    });

    await act(async () => {
      rejectDelete?.(new Error("delete failed"));
      await expect(deletePromise).rejects.toThrow("delete failed");
    });
    await waitFor(() => {
      expect(result.current.isDeleting).toBe(false);
    });
  });

  it("returns per-id results from the trash API", async () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    const results: DatasetDeleteResult[] = [
      { id: 3, success: true, error: null },
      { id: 4, success: true, error: null },
    ];

    trashDatasetsMock.mockResolvedValue(results);

    const { result } = renderHook(() => useDatasetTrashMutation(), {
      wrapper: createWrapper(queryClient),
    });

    let actualResults: DatasetDeleteResult[] | undefined;
    await act(async () => {
      actualResults = await result.current.trashDatasets([3, 4]);
    });

    expect(actualResults).toEqual(results);
    expect(trashDatasetsMock).toHaveBeenCalledWith([3, 4]);
  });

  it("returns per-id results from the restore API", async () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    const results: DatasetDeleteResult[] = [
      { id: 5, success: true, error: null },
    ];

    restoreDatasetsMock.mockResolvedValue(results);

    const { result } = renderHook(() => useDatasetRestoreMutation(), {
      wrapper: createWrapper(queryClient),
    });

    let actualResults: DatasetDeleteResult[] | undefined;
    await act(async () => {
      actualResults = await result.current.restoreDatasets([5]);
    });

    expect(actualResults).toEqual(results);
    expect(restoreDatasetsMock).toHaveBeenCalledWith([5]);
  });
});
