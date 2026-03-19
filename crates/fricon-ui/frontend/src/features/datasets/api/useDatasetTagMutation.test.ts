import { createElement, type ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useDatasetTagMutation } from "./useDatasetTagMutation";
import { datasetKeys } from "./queryKeys";

type BatchUpdateDatasetTagsFn = (
  ids: number[],
  add?: string[],
  remove?: string[],
) => Promise<{ id: number; success: boolean; error: string | null }[]>;
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

  it("invalidates all dataset detail queries after deleting a tag", async () => {
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
        },
      },
    });
    const invalidateQueriesSpy = vi.spyOn(queryClient, "invalidateQueries");
    const refreshDatasets = vi.fn().mockResolvedValue(undefined);

    deleteTagMock.mockResolvedValue(undefined);

    const { result } = renderHook(
      () => useDatasetTagMutation(refreshDatasets),
      { wrapper: createWrapper(queryClient) },
    );

    await act(async () => {
      await result.current.deleteTag("vision");
    });

    expect(deleteTagMock).toHaveBeenCalledWith("vision");
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: ["datasets", "detail"],
    });
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: datasetKeys.tags(),
    });
    expect(refreshDatasets).toHaveBeenCalledTimes(1);
  });

  it("batch-adds tags, invalidates affected details, and stays pending until refresh completes", async () => {
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
        },
      },
    });
    const invalidateQueriesSpy = vi.spyOn(queryClient, "invalidateQueries");
    let resolveRefresh: (() => void) | undefined;
    const refreshDatasets = vi.fn().mockImplementation(
      () =>
        new Promise<void>((resolve) => {
          resolveRefresh = resolve;
        }),
    );
    const results = [{ id: 1, success: true, error: null }];
    batchUpdateDatasetTagsMock.mockResolvedValue(results);

    const { result } = renderHook(
      () => useDatasetTagMutation(refreshDatasets),
      {
        wrapper: createWrapper(queryClient),
      },
    );

    let mutationPromise!: Promise<
      { id: number; success: boolean; error: string | null }[]
    >;
    act(() => {
      mutationPromise = result.current.batchAddTags([1], ["vision"]);
    });

    await waitFor(() => {
      expect(batchUpdateDatasetTagsMock).toHaveBeenCalledWith(
        [1],
        ["vision"],
        [],
      );
      expect(result.current.isUpdatingTags).toBe(true);
    });
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: datasetKeys.detail(1),
    });

    await act(async () => {
      resolveRefresh?.();
      await expect(mutationPromise).resolves.toEqual(results);
    });
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: datasetKeys.tags(),
    });
    await waitFor(() => {
      expect(result.current.isUpdatingTags).toBe(false);
    });
  });

  it("batch-removes tags and refreshes affected dataset queries", async () => {
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
        },
      },
    });
    const invalidateQueriesSpy = vi.spyOn(queryClient, "invalidateQueries");
    const refreshDatasets = vi.fn().mockResolvedValue(undefined);
    const results = [{ id: 1, success: true, error: null }];
    batchUpdateDatasetTagsMock.mockResolvedValue(results);

    const { result } = renderHook(
      () => useDatasetTagMutation(refreshDatasets),
      {
        wrapper: createWrapper(queryClient),
      },
    );

    await act(async () => {
      await result.current.batchRemoveTags([1], ["vision"]);
    });

    expect(batchUpdateDatasetTagsMock).toHaveBeenCalledWith(
      [1],
      [],
      ["vision"],
    );
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: datasetKeys.detail(1),
    });
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: datasetKeys.tags(),
    });
    expect(refreshDatasets).toHaveBeenCalledTimes(1);
  });

  it("invalidates all detail queries after renaming a tag", async () => {
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
        },
      },
    });
    const invalidateQueriesSpy = vi.spyOn(queryClient, "invalidateQueries");
    const refreshDatasets = vi.fn().mockResolvedValue(undefined);
    renameTagMock.mockResolvedValue(undefined);

    const { result } = renderHook(
      () => useDatasetTagMutation(refreshDatasets),
      {
        wrapper: createWrapper(queryClient),
      },
    );

    await act(async () => {
      await result.current.renameTag("vision", "images");
    });

    expect(renameTagMock).toHaveBeenCalledWith("vision", "images");
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: ["datasets", "detail"],
    });
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: datasetKeys.tags(),
    });
    expect(refreshDatasets).toHaveBeenCalledTimes(1);
  });

  it("invalidates all detail queries after merging tags", async () => {
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
        },
      },
    });
    const invalidateQueriesSpy = vi.spyOn(queryClient, "invalidateQueries");
    const refreshDatasets = vi.fn().mockResolvedValue(undefined);
    mergeTagMock.mockResolvedValue(undefined);

    const { result } = renderHook(
      () => useDatasetTagMutation(refreshDatasets),
      {
        wrapper: createWrapper(queryClient),
      },
    );

    await act(async () => {
      await result.current.mergeTag("vision", "archive");
    });

    expect(mergeTagMock).toHaveBeenCalledWith("vision", "archive");
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: ["datasets", "detail"],
    });
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({
      queryKey: datasetKeys.tags(),
    });
    expect(refreshDatasets).toHaveBeenCalledTimes(1);
  });

  it("clears isUpdatingTags after a failed tag mutation", async () => {
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
        },
      },
    });
    const refreshDatasets = vi.fn().mockResolvedValue(undefined);
    let rejectMerge: ((reason?: unknown) => void) | undefined;
    mergeTagMock.mockImplementation(
      () =>
        new Promise<void>((_, reject) => {
          rejectMerge = reject;
        }),
    );

    const { result } = renderHook(
      () => useDatasetTagMutation(refreshDatasets),
      {
        wrapper: createWrapper(queryClient),
      },
    );

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
