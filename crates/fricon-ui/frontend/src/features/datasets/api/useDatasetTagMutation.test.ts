import { createElement, type ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useDatasetTagMutation } from "./useDatasetTagMutation";
import { datasetKeys } from "./queryKeys";
import type { DatasetTagBatchResult } from "./types";

type BatchUpdateDatasetTagsFn = (
  ids: number[],
  add?: string[],
  remove?: string[],
) => Promise<DatasetTagBatchResult[]>;
type DeleteTagFn = (tag: string) => Promise<void>;
type RenameTagFn = (oldName: string, newName: string) => Promise<void>;
type MergeTagFn = (source: string, target: string) => Promise<void>;

const batchUpdateDatasetTagsMock = vi.fn<BatchUpdateDatasetTagsFn>();
const deleteTagMock = vi.fn<DeleteTagFn>();
const renameTagMock = vi.fn<RenameTagFn>();
const mergeTagMock = vi.fn<MergeTagFn>();

vi.mock("./client", () => ({
  batchUpdateDatasetTags: (ids: number[], add?: string[], remove?: string[]) =>
    batchUpdateDatasetTagsMock(ids, add, remove),
  deleteTag: (tag: string) => deleteTagMock(tag),
  renameTag: (oldName: string, newName: string) =>
    renameTagMock(oldName, newName),
  mergeTag: (source: string, target: string) => mergeTagMock(source, target),
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

describe("useDatasetTagMutation", () => {
  beforeEach(() => {
    batchUpdateDatasetTagsMock.mockReset();
    deleteTagMock.mockReset();
    renameTagMock.mockReset();
    mergeTagMock.mockReset();
  });

  it("invalidates list, tags, and detail queries after deleting a tag", async () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    const invalidateQueriesSpy = vi.spyOn(queryClient, "invalidateQueries");

    deleteTagMock.mockResolvedValue(undefined);

    const { result } = renderHook(() => useDatasetTagMutation(), {
      wrapper: createWrapper(queryClient),
    });

    await act(async () => {
      await result.current.deleteTag("vision");
    });

    expect(deleteTagMock).toHaveBeenCalledWith("vision");
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: ["datasets", "list"],
    });
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: datasetKeys.tags(),
    });
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: ["datasets", "detail"],
    });
  });

  it("batch-adds tags and returns results", async () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    const results = [
      { id: 1, success: true, addError: null, removeError: null },
    ];
    batchUpdateDatasetTagsMock.mockResolvedValue(results);

    const { result } = renderHook(() => useDatasetTagMutation(), {
      wrapper: createWrapper(queryClient),
    });

    let actualResults: DatasetTagBatchResult[] | undefined;
    await act(async () => {
      actualResults = await result.current.batchAddTags([1], ["vision"]);
    });

    expect(batchUpdateDatasetTagsMock).toHaveBeenCalledWith(
      [1],
      ["vision"],
      [],
    );
    expect(actualResults).toEqual(results);
  });

  it("batch-removes tags and returns results", async () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    const results = [
      { id: 1, success: true, addError: null, removeError: null },
    ];
    batchUpdateDatasetTagsMock.mockResolvedValue(results);

    const { result } = renderHook(() => useDatasetTagMutation(), {
      wrapper: createWrapper(queryClient),
    });

    let actualResults: DatasetTagBatchResult[] | undefined;
    await act(async () => {
      actualResults = await result.current.batchRemoveTags([1], ["vision"]);
    });

    expect(batchUpdateDatasetTagsMock).toHaveBeenCalledWith(
      [1],
      [],
      ["vision"],
    );
    expect(actualResults).toEqual(results);
  });

  it("invalidates list, tags, and detail queries after renaming a tag", async () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    const invalidateQueriesSpy = vi.spyOn(queryClient, "invalidateQueries");
    renameTagMock.mockResolvedValue(undefined);

    const { result } = renderHook(() => useDatasetTagMutation(), {
      wrapper: createWrapper(queryClient),
    });

    await act(async () => {
      await result.current.renameTag("vision", "images");
    });

    expect(renameTagMock).toHaveBeenCalledWith("vision", "images");
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: ["datasets", "list"],
    });
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: datasetKeys.tags(),
    });
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: ["datasets", "detail"],
    });
  });

  it("invalidates list, tags, and detail queries after merging tags", async () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    const invalidateQueriesSpy = vi.spyOn(queryClient, "invalidateQueries");
    mergeTagMock.mockResolvedValue(undefined);

    const { result } = renderHook(() => useDatasetTagMutation(), {
      wrapper: createWrapper(queryClient),
    });

    await act(async () => {
      await result.current.mergeTag("vision", "archive");
    });

    expect(mergeTagMock).toHaveBeenCalledWith("vision", "archive");
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: ["datasets", "list"],
    });
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: datasetKeys.tags(),
    });
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: ["datasets", "detail"],
    });
  });

  it("clears isUpdatingTags after a failed tag mutation", async () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    let rejectMerge: ((reason?: unknown) => void) | undefined;
    mergeTagMock.mockImplementation(
      () =>
        new Promise<void>((_, reject) => {
          rejectMerge = reject;
        }),
    );

    const { result } = renderHook(() => useDatasetTagMutation(), {
      wrapper: createWrapper(queryClient),
    });

    let mutationPromise!: Promise<void>;
    act(() => {
      mutationPromise = result.current.mergeTag("vision", "archive");
    });

    await waitFor(() => {
      expect(result.current.isUpdatingTags).toBe(true);
    });

    await act(async () => {
      rejectMerge?.(new Error("merge failed"));
      await expect(mutationPromise).rejects.toThrow("merge failed");
    });
    await waitFor(() => {
      expect(result.current.isUpdatingTags).toBe(false);
    });
  });
});
