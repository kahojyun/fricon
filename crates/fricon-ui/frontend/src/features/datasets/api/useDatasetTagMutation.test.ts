import { createElement, type ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook } from "@testing-library/react";
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
});
