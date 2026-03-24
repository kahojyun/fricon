import { renderHook, act } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useDatasetTableActions } from "./useDatasetTableActions";
import type {
  DatasetDeleteResult,
  DatasetInfo,
  DatasetTagBatchResult,
} from "./types";
import { datasetKeys } from "./queryKeys";

type ToggleFavoriteFn = (dataset: DatasetInfo) => Promise<void>;
type DeleteDatasetsFn = (ids: number[]) => Promise<DatasetDeleteResult[]>;
type EmptyTrashFn = () => Promise<DatasetDeleteResult[]>;
type BatchTagMutationFn = (
  ids: number[],
  tags: string[],
) => Promise<DatasetTagBatchResult[]>;
type TagMutationFn = (...args: string[]) => Promise<void>;

const toggleFavoriteMock = vi.fn<ToggleFavoriteFn>();
const trashDatasetsMock = vi.fn<DeleteDatasetsFn>();
const restoreDatasetsMock = vi.fn<DeleteDatasetsFn>();
const deleteDatasetsMock = vi.fn<DeleteDatasetsFn>();
const emptyTrashMock = vi.fn<EmptyTrashFn>();
const batchAddTagsMock = vi.fn<BatchTagMutationFn>();
const batchRemoveTagsMock = vi.fn<BatchTagMutationFn>();
const deleteTagMock = vi.fn<TagMutationFn>();
const renameTagMock = vi.fn<TagMutationFn>();
const mergeTagMock = vi.fn<TagMutationFn>();
const refreshDatasetsMock = vi.fn<() => Promise<void>>();

const favoriteMutationState = vi.hoisted(() => ({
  isPending: false,
}));
const deleteMutationState = vi.hoisted(() => ({
  isDeleting: false,
  isTrashing: false,
  isRestoring: false,
  isEmptyingTrash: false,
}));
const tagMutationState = vi.hoisted(() => ({
  isUpdatingTags: false,
}));

vi.mock("./useDatasetFavoriteMutation", () => ({
  useDatasetFavoriteMutation: () => ({
    toggleFavorite: toggleFavoriteMock,
    isPending: favoriteMutationState.isPending,
  }),
}));

vi.mock("./useDatasetDeleteMutation", () => ({
  useDatasetTrashMutation: () => ({
    trashDatasets: trashDatasetsMock,
    isTrashing: deleteMutationState.isTrashing,
  }),
  useDatasetRestoreMutation: () => ({
    restoreDatasets: restoreDatasetsMock,
    isRestoring: deleteMutationState.isRestoring,
  }),
  useDatasetDeleteMutation: () => ({
    deleteDatasets: deleteDatasetsMock,
    isDeleting: deleteMutationState.isDeleting,
  }),
  useEmptyTrashMutation: () => ({
    emptyTrash: emptyTrashMock,
    isEmptyingTrash: deleteMutationState.isEmptyingTrash,
  }),
}));

vi.mock("./useDatasetTagMutation", () => ({
  useDatasetTagMutation: () => ({
    batchAddTags: batchAddTagsMock,
    batchRemoveTags: batchRemoveTagsMock,
    deleteTag: deleteTagMock,
    renameTag: renameTagMock,
    mergeTag: mergeTagMock,
    isUpdatingTags: tagMutationState.isUpdatingTags,
  }),
}));

describe("useDatasetTableActions", () => {
  beforeEach(() => {
    toggleFavoriteMock.mockReset();
    trashDatasetsMock.mockReset();
    restoreDatasetsMock.mockReset();
    deleteDatasetsMock.mockReset();
    emptyTrashMock.mockReset();
    batchAddTagsMock.mockReset();
    batchRemoveTagsMock.mockReset();
    deleteTagMock.mockReset();
    renameTagMock.mockReset();
    mergeTagMock.mockReset();
    refreshDatasetsMock.mockReset();

    toggleFavoriteMock.mockResolvedValue(undefined);
    trashDatasetsMock.mockResolvedValue([]);
    restoreDatasetsMock.mockResolvedValue([]);
    deleteDatasetsMock.mockResolvedValue([]);
    emptyTrashMock.mockResolvedValue([]);
    batchAddTagsMock.mockResolvedValue([]);
    batchRemoveTagsMock.mockResolvedValue([]);
    deleteTagMock.mockResolvedValue(undefined);
    renameTagMock.mockResolvedValue(undefined);
    mergeTagMock.mockResolvedValue(undefined);
    refreshDatasetsMock.mockResolvedValue(undefined);

    favoriteMutationState.isPending = false;
    deleteMutationState.isDeleting = false;
    deleteMutationState.isTrashing = false;
    deleteMutationState.isRestoring = false;
    deleteMutationState.isEmptyingTrash = false;
    tagMutationState.isUpdatingTags = false;
  });

  it("forwards favorite, trash, restore, delete, and batch tag actions", async () => {
    const removeActiveTag = vi.fn();
    const replaceActiveTag = vi.fn();
    const dataset = {
      id: 11,
      favorite: false,
    } as DatasetInfo;

    const { result } = renderHook(() =>
      useDatasetTableActions({
        datasetQueryKey: datasetKeys.list(
          {
            search: "",
            tags: [],
            favoriteOnly: false,
            statuses: [],
            viewMode: "active",
            sorting: [{ id: "id", desc: true }],
          },
          3,
        ),
        refreshDatasets: refreshDatasetsMock,
        removeActiveTag,
        replaceActiveTag,
      }),
    );

    await act(async () => {
      await result.current.toggleFavorite(dataset);
      await result.current.trashDatasets([11, 12]);
      await result.current.restoreDatasets([11]);
      await result.current.deleteDatasets([11, 12]);
      await result.current.emptyTrash();
      await result.current.batchAddTags([11], ["vision"]);
      await result.current.batchRemoveTags([11], ["vision"]);
    });

    expect(toggleFavoriteMock).toHaveBeenCalledWith(dataset);
    expect(trashDatasetsMock).toHaveBeenCalledWith([11, 12]);
    expect(restoreDatasetsMock).toHaveBeenCalledWith([11]);
    expect(deleteDatasetsMock).toHaveBeenCalledWith([11, 12]);
    expect(emptyTrashMock).toHaveBeenCalled();
    expect(batchAddTagsMock).toHaveBeenCalledWith([11], ["vision"]);
    expect(batchRemoveTagsMock).toHaveBeenCalledWith([11], ["vision"]);
    expect(removeActiveTag).not.toHaveBeenCalled();
    expect(replaceActiveTag).not.toHaveBeenCalled();
  });

  it("removes a selected tag after delete", async () => {
    const removeActiveTag = vi.fn();
    const replaceActiveTag = vi.fn();

    const { result } = renderHook(() =>
      useDatasetTableActions({
        datasetQueryKey: datasetKeys.list(
          {
            search: "",
            tags: [],
            favoriteOnly: false,
            statuses: [],
            viewMode: "active",
            sorting: [{ id: "id", desc: true }],
          },
          3,
        ),
        refreshDatasets: refreshDatasetsMock,
        removeActiveTag,
        replaceActiveTag,
      }),
    );

    await act(async () => {
      await result.current.deleteTag("vision");
    });

    expect(deleteTagMock).toHaveBeenCalledWith("vision");
    expect(removeActiveTag).toHaveBeenCalledWith("vision");
    expect(replaceActiveTag).not.toHaveBeenCalled();
  });

  it("replaces the selected tag after rename and merge", async () => {
    const removeActiveTag = vi.fn();
    const replaceActiveTag = vi.fn();

    const { result } = renderHook(() =>
      useDatasetTableActions({
        datasetQueryKey: datasetKeys.list(
          {
            search: "",
            tags: [],
            favoriteOnly: false,
            statuses: [],
            viewMode: "active",
            sorting: [{ id: "id", desc: true }],
          },
          3,
        ),
        refreshDatasets: refreshDatasetsMock,
        removeActiveTag,
        replaceActiveTag,
      }),
    );

    await act(async () => {
      await result.current.renameTag("vision", "images");
      await result.current.mergeTag("images", "archive");
    });

    expect(renameTagMock).toHaveBeenCalledWith("vision", "images");
    expect(mergeTagMock).toHaveBeenCalledWith("images", "archive");
    expect(replaceActiveTag).toHaveBeenNthCalledWith(1, "vision", "images");
    expect(replaceActiveTag).toHaveBeenNthCalledWith(2, "images", "archive");
    expect(removeActiveTag).not.toHaveBeenCalled();
  });

  it("reflects underlying dataset mutation and tag loading flags", () => {
    deleteMutationState.isDeleting = true;
    deleteMutationState.isTrashing = true;
    tagMutationState.isUpdatingTags = true;

    const { result } = renderHook(() =>
      useDatasetTableActions({
        datasetQueryKey: datasetKeys.list(
          {
            search: "",
            tags: [],
            favoriteOnly: false,
            statuses: [],
            viewMode: "active",
            sorting: [{ id: "id", desc: true }],
          },
          3,
        ),
        refreshDatasets: refreshDatasetsMock,
        removeActiveTag: vi.fn(),
        replaceActiveTag: vi.fn(),
      }),
    );

    expect(result.current.isMutatingDatasets).toBe(true);
    expect(result.current.isUpdatingTags).toBe(true);
  });
});
