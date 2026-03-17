import type { KeyboardEvent, PointerEvent } from "react";
import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useDatasetTableSelection } from "./useDatasetTableSelection";

function createRows() {
  return [
    { id: "1", original: { id: 1 } },
    { id: "2", original: { id: 2 } },
    { id: "3", original: { id: 3 } },
    { id: "4", original: { id: 4 } },
  ];
}

function createPointerEvent(
  overrides: Partial<PointerEvent<HTMLTableRowElement>> = {},
) {
  return {
    button: 0,
    ctrlKey: false,
    metaKey: false,
    shiftKey: false,
    target: document.createElement("div"),
    currentTarget: { focus: vi.fn() },
    preventDefault: vi.fn(),
    ...overrides,
  } as unknown as PointerEvent<HTMLTableRowElement>;
}

function createKeyboardEvent(
  key: string,
  currentTarget: HTMLTableRowElement,
  overrides: Partial<KeyboardEvent<HTMLTableRowElement>> = {},
) {
  return {
    key,
    target: currentTarget,
    currentTarget,
    preventDefault: vi.fn(),
    ...overrides,
  } as KeyboardEvent<HTMLTableRowElement>;
}

describe("useDatasetTableSelection", () => {
  it("moves row focus and selection with keyboard navigation", () => {
    const rows = createRows();
    const onDatasetSelected = vi.fn();
    const scrollToIndex = vi.fn();
    const measureElement = vi.fn();
    const secondRow = document.createElement("tr");
    const secondRowFocus = vi.spyOn(secondRow, "focus");

    const { result } = renderHook(() =>
      useDatasetTableSelection({
        rows,
        rowVirtualizer: { measureElement, scrollToIndex },
        onDatasetSelected,
      }),
    );

    act(() => {
      result.current.registerRowElement("2", secondRow);
    });

    act(() => {
      result.current.handleRowKeyDown(
        createKeyboardEvent("ArrowDown", document.createElement("tr")),
        0,
      );
    });

    expect(onDatasetSelected).toHaveBeenCalledWith(2);
    expect(scrollToIndex).toHaveBeenCalledWith(1, { align: "auto" });
    expect(result.current.rowSelection).toEqual({ "2": true });
    expect(secondRowFocus).toHaveBeenCalledTimes(1);
  });

  it("supports single, range, drag, and meta-toggle selection flows", () => {
    const rows = createRows();
    const onDatasetSelected = vi.fn();

    const { result } = renderHook(() =>
      useDatasetTableSelection({
        rows,
        rowVirtualizer: {
          measureElement: vi.fn(),
          scrollToIndex: vi.fn(),
        },
        onDatasetSelected,
      }),
    );

    act(() => {
      result.current.handleRowPointerDown(createPointerEvent(), 0, "1", 1);
    });

    expect(result.current.rowSelection).toEqual({ "1": true });

    act(() => {
      result.current.handleRowPointerEnter(2);
    });

    expect(result.current.rowSelection).toEqual({
      "1": true,
      "2": true,
      "3": true,
    });

    act(() => {
      result.current.handleRowPointerDown(
        createPointerEvent({ shiftKey: true }),
        3,
        "4",
        4,
      );
    });

    expect(result.current.rowSelection).toEqual({
      "1": true,
      "2": true,
      "3": true,
      "4": true,
    });

    act(() => {
      result.current.handleRowPointerDown(
        createPointerEvent({ metaKey: true }),
        1,
        "2",
        2,
      );
    });

    expect(result.current.rowSelection).toEqual({
      "1": true,
      "3": true,
      "4": true,
    });

    act(() => {
      result.current.handleRowPointerEnter(2);
    });

    expect(result.current.rowSelection).toEqual({
      "1": true,
      "4": true,
    });
    expect(onDatasetSelected).toHaveBeenLastCalledWith(2);
  });

  it("uses the latest visible row order after rerender", () => {
    const onDatasetSelected = vi.fn();
    const { result, rerender } = renderHook(
      ({ rows }) =>
        useDatasetTableSelection({
          rows,
          rowVirtualizer: {
            measureElement: vi.fn(),
            scrollToIndex: vi.fn(),
          },
          onDatasetSelected,
        }),
      {
        initialProps: {
          rows: createRows(),
        },
      },
    );

    rerender({
      rows: [
        { id: "4", original: { id: 4 } },
        { id: "3", original: { id: 3 } },
        { id: "2", original: { id: 2 } },
        { id: "1", original: { id: 1 } },
      ],
    });

    act(() => {
      result.current.handleRowKeyDown(
        createKeyboardEvent("ArrowDown", document.createElement("tr")),
        0,
      );
    });

    expect(onDatasetSelected).toHaveBeenLastCalledWith(3);
    expect(result.current.rowSelection).toEqual({ "3": true });
  });
});
