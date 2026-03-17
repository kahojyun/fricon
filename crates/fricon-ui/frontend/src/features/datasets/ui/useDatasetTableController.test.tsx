import type { KeyboardEvent } from "react";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { SortingState, VisibilityState } from "@tanstack/react-table";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { DatasetInfo } from "../api/types";
import { useDatasetTableController } from "./useDatasetTableController";

const {
  getVirtualItemsMock,
  getTotalSizeMock,
  measureElementMock,
  scrollToIndexMock,
} = vi.hoisted(() => ({
  getVirtualItemsMock: vi.fn(),
  getTotalSizeMock: vi.fn(),
  measureElementMock: vi.fn(),
  scrollToIndexMock: vi.fn(),
}));

vi.mock("@tanstack/react-virtual", () => ({
  useVirtualizer: vi.fn(() => ({
    getVirtualItems: getVirtualItemsMock,
    getTotalSize: getTotalSizeMock,
    measureElement: measureElementMock,
    scrollToIndex: scrollToIndexMock,
  })),
}));

function makeDataset(overrides: Partial<DatasetInfo> = {}): DatasetInfo {
  return {
    id: 1,
    name: "Dataset 1",
    description: "desc",
    favorite: false,
    tags: [],
    status: "Completed",
    createdAt: new Date("2026-01-01T00:00:00Z"),
    ...overrides,
  };
}

function createKeyboardEvent(
  key: string,
  currentTarget: HTMLTableRowElement,
): KeyboardEvent<HTMLTableRowElement> {
  return {
    key,
    target: currentTarget,
    currentTarget,
    preventDefault: vi.fn(),
  } as unknown as KeyboardEvent<HTMLTableRowElement>;
}

function renderController({
  datasets = [makeDataset(), makeDataset({ id: 2, name: "Dataset 2" })],
  hasMore = false,
  loadNextPage = vi.fn().mockResolvedValue(undefined),
  onDatasetSelected = vi.fn(),
}: {
  datasets?: DatasetInfo[];
  hasMore?: boolean;
  loadNextPage?: () => Promise<void>;
  onDatasetSelected?: (id?: number) => void;
} = {}) {
  const sorting: SortingState = [];
  const columnVisibility: VisibilityState = {};
  const tableContainerRef = { current: document.createElement("div") };

  const hook = renderHook(
    ({ visibleDatasets, canLoadMore }) =>
      useDatasetTableController({
        datasets: visibleDatasets,
        columns: [],
        sorting,
        setSorting: vi.fn(),
        columnVisibility,
        hasMore: canLoadMore,
        loadNextPage,
        onDatasetSelected,
        tableContainerRef,
      }),
    {
      initialProps: {
        visibleDatasets: datasets,
        canLoadMore: hasMore,
      },
    },
  );

  return {
    ...hook,
    loadNextPage,
    onDatasetSelected,
  };
}

describe("useDatasetTableController", () => {
  beforeEach(() => {
    getVirtualItemsMock.mockReset();
    getTotalSizeMock.mockReset();
    measureElementMock.mockReset();
    scrollToIndexMock.mockReset();
    getVirtualItemsMock.mockReturnValue([]);
    getTotalSizeMock.mockReturnValue(0);
  });

  it("loads the next page when the last virtual row reaches the fetch threshold", async () => {
    getVirtualItemsMock.mockReturnValue([
      { index: 10, start: 360, end: 396 },
      { index: 11, start: 396, end: 432 },
    ]);

    const datasets = Array.from({ length: 20 }, (_, index) =>
      makeDataset({ id: index + 1, name: `Dataset ${index + 1}` }),
    );
    const { result, loadNextPage } = renderController({
      datasets,
      hasMore: true,
    });

    await waitFor(() => {
      expect(loadNextPage).toHaveBeenCalledTimes(1);
    });
    expect(result.current.rows).toHaveLength(20);
  });

  it("does not fetch when there are no more backend pages", async () => {
    getVirtualItemsMock.mockReturnValue([
      { index: 10, start: 360, end: 396 },
      { index: 11, start: 396, end: 432 },
    ]);

    const datasets = Array.from({ length: 20 }, (_, index) =>
      makeDataset({ id: index + 1, name: `Dataset ${index + 1}` }),
    );
    const { result, loadNextPage } = renderController({
      datasets,
      hasMore: false,
    });

    await waitFor(() => {
      expect(result.current.rows).toHaveLength(20);
    });
    expect(loadNextPage).not.toHaveBeenCalled();
  });

  it("uses the latest backend-visible row order after rerender", () => {
    const datasets = [
      makeDataset({ id: 1, name: "Dataset 1" }),
      makeDataset({ id: 2, name: "Dataset 2" }),
      makeDataset({ id: 3, name: "Dataset 3" }),
    ];
    const { result, rerender, onDatasetSelected } = renderController({
      datasets,
    });

    expect(result.current.rows.map((row) => row.id)).toEqual(["1", "2", "3"]);

    rerender({
      visibleDatasets: [datasets[2], datasets[0], datasets[1]],
      canLoadMore: false,
    });

    expect(result.current.rows.map((row) => row.id)).toEqual(["3", "1", "2"]);

    act(() => {
      result.current.handleRowKeyDown(
        createKeyboardEvent("ArrowDown", document.createElement("tr")),
        0,
      );
    });

    expect(onDatasetSelected).toHaveBeenLastCalledWith(1);
    expect(result.current.rowSelection).toEqual({ "1": true });
  });
});
