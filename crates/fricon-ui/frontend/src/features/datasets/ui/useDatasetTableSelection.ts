import { useEffect, useRef, useState } from "react";
import type { DatasetInfo } from "../api/types";
import {
  applyToggleSelectionRange,
  buildRangeSelection,
  findRowIndexById,
  getSelectionRange,
  type DatasetRowSelection,
} from "../model/datasetTableSelectionLogic";

interface DatasetTableRow {
  id: string;
  original: Pick<DatasetInfo, "id">;
}

interface DatasetTableVirtualizer {
  measureElement: (element: Element) => void;
  scrollToIndex?: (index: number, options: { align: "auto" }) => void;
}

interface UseDatasetTableSelectionArgs {
  getRows: () => DatasetTableRow[];
  rowVirtualizer: DatasetTableVirtualizer;
  onDatasetSelected: (id?: number) => void;
}

type DragState = {
  initialSelection: DatasetRowSelection;
  mode: "replace" | "toggle";
  targetValue?: boolean;
} | null;

function isMacPlatform() {
  const platform = navigator.userAgent;
  return platform.toUpperCase().includes("MAC");
}

export function useDatasetTableSelection({
  getRows,
  rowVirtualizer,
  onDatasetSelected,
}: UseDatasetTableSelectionArgs) {
  const [rowSelection, setRowSelection] = useState<DatasetRowSelection>({});
  const [anchorId, setAnchorId] = useState<string | null>(null);
  const [dragState, setDragState] = useState<DragState>(null);
  const rowElementMapRef = useRef(new Map<string, HTMLTableRowElement>());
  const pendingFocusRowIdRef = useRef<string | null>(null);

  useEffect(() => {
    const handleMouseUp = () => setDragState(null);
    window.addEventListener("pointerup", handleMouseUp);
    return () => window.removeEventListener("pointerup", handleMouseUp);
  }, []);

  const focusRow = (rowId: string) => {
    pendingFocusRowIdRef.current = rowId;
    const rowElement = rowElementMapRef.current.get(rowId);
    if (!rowElement) {
      return;
    }

    rowElement.focus();
    pendingFocusRowIdRef.current = null;
  };

  const registerRowElement = (
    rowId: string,
    rowElement: HTMLTableRowElement | null,
  ) => {
    if (!rowElement) {
      rowElementMapRef.current.delete(rowId);
      return;
    }

    rowElementMapRef.current.set(rowId, rowElement);
    rowVirtualizer.measureElement(rowElement);

    if (pendingFocusRowIdRef.current === rowId) {
      rowElement.focus();
      pendingFocusRowIdRef.current = null;
    }
  };

  const selectSingleDatasetRow = (
    rowIndex: number,
    options: {
      focus?: boolean;
      scroll?: boolean;
    } = {},
  ) => {
    const rows = getRows();
    const row = rows[rowIndex];
    if (!row) {
      return;
    }

    if (options.scroll) {
      rowVirtualizer.scrollToIndex?.(rowIndex, { align: "auto" });
    }

    setRowSelection({ [row.id]: true });
    setAnchorId(row.id);
    setDragState(null);
    onDatasetSelected(row.original.id);

    if (options.focus) {
      focusRow(row.id);
    }
  };

  const handleRowKeyDown = (
    event: React.KeyboardEvent<HTMLTableRowElement>,
    rowIndex: number,
  ) => {
    if (event.target !== event.currentTarget) {
      return;
    }

    if (event.key === "ArrowUp" || event.key === "ArrowDown") {
      event.preventDefault();
      const rows = getRows();
      const nextRowIndex =
        event.key === "ArrowUp"
          ? Math.max(rowIndex - 1, 0)
          : Math.min(rowIndex + 1, rows.length - 1);

      if (nextRowIndex !== rowIndex) {
        selectSingleDatasetRow(nextRowIndex, {
          focus: true,
          scroll: true,
        });
      }

      return;
    }

    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      const rows = getRows();
      const row = rows[rowIndex];
      if (!row) {
        return;
      }

      onDatasetSelected(row.original.id);
      focusRow(row.id);
    }
  };

  const handleRowPointerDown = (
    event: React.PointerEvent<HTMLTableRowElement>,
    rowIndex: number,
    rowId: string,
    datasetId: number,
  ) => {
    if (event.button !== 0) {
      return;
    }

    const target = event.target as HTMLElement;
    if (target.closest('button:not([role="checkbox"]), a')) {
      return;
    }

    const isCheckbox = target.closest(
      'button[role="checkbox"], [data-slot="checkbox"], [data-slot="checkbox-indicator"]',
    );

    if (isCheckbox) {
      return;
    }

    event.currentTarget.focus();
    const rows = getRows();

    if (event.shiftKey) {
      event.preventDefault();
      const anchorIndex = findRowIndexById(rows, anchorId);
      const effectiveAnchorIndex = anchorIndex !== -1 ? anchorIndex : rowIndex;
      const { start, end } = getSelectionRange(effectiveAnchorIndex, rowIndex);

      setRowSelection(buildRangeSelection(rows, start, end));
      setDragState({
        initialSelection: {},
        mode: "replace",
      });

      if (anchorIndex === -1) {
        setAnchorId(rowId);
      }

      onDatasetSelected(datasetId);
      return;
    }

    if (isMacPlatform() && event.ctrlKey) {
      return;
    }

    if (event.ctrlKey || event.metaKey) {
      event.preventDefault();
      const isSelected = !!rowSelection[rowId];
      const nextValue = !isSelected;

      setRowSelection((previous) => {
        const nextSelection = { ...previous };
        if (nextValue) {
          nextSelection[rowId] = true;
        } else {
          delete nextSelection[rowId];
        }
        return nextSelection;
      });

      setAnchorId(rowId);
      setDragState({
        initialSelection: rowSelection,
        mode: "toggle",
        targetValue: nextValue,
      });
      onDatasetSelected(datasetId);
      return;
    }

    setRowSelection({ [rowId]: true });
    setAnchorId(rowId);
    setDragState({
      initialSelection: {},
      mode: "replace",
    });
    onDatasetSelected(datasetId);
  };

  const handleRowPointerEnter = (rowIndex: number) => {
    if (!dragState) {
      return;
    }

    const rows = getRows();
    const anchorIndex = findRowIndexById(rows, anchorId);
    const effectiveAnchorIndex = anchorIndex !== -1 ? anchorIndex : rowIndex;
    const { start, end } = getSelectionRange(effectiveAnchorIndex, rowIndex);

    if (dragState.mode === "replace") {
      setRowSelection(buildRangeSelection(rows, start, end));
      return;
    }

    setRowSelection(
      applyToggleSelectionRange(
        rows,
        start,
        end,
        dragState.initialSelection,
        !!dragState.targetValue,
      ),
    );
  };

  return {
    rowSelection,
    setRowSelection,
    registerRowElement,
    selectSingleDatasetRow,
    handleRowKeyDown,
    handleRowPointerDown,
    handleRowPointerEnter,
    clearDragState: () => setDragState(null),
  };
}
