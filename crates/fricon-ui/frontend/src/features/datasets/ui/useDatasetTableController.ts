import { useEffect, useMemo, type RefObject } from "react";
import {
  getCoreRowModel,
  type ColumnDef,
  type OnChangeFn,
  type SortingState,
  type VisibilityState,
  useReactTable,
} from "@tanstack/react-table";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { DatasetInfo } from "../api/types";
import { useDatasetTableSelection } from "./useDatasetTableSelection";

interface UseDatasetTableControllerArgs {
  datasets: DatasetInfo[];
  columns: ColumnDef<DatasetInfo>[];
  sorting: SortingState;
  setSorting: OnChangeFn<SortingState>;
  columnVisibility: VisibilityState;
  hasMore: boolean;
  loadNextPage: () => Promise<void>;
  onDatasetSelected: (id?: number) => void;
  tableContainerRef: RefObject<HTMLDivElement | null>;
}

export function useDatasetTableController({
  datasets,
  columns,
  sorting,
  setSorting,
  columnVisibility,
  hasMore,
  loadNextPage,
  onDatasetSelected,
  tableContainerRef,
}: UseDatasetTableControllerArgs) {
  // TanStack Virtual is an intentional compiler boundary for this hook.
  // eslint-disable-next-line react-hooks/incompatible-library
  const rowVirtualizer = useVirtualizer({
    count: datasets.length,
    getScrollElement: () => tableContainerRef.current,
    estimateSize: () => 36,
    overscan: 8,
  });

  const selectionRows = useMemo(
    () =>
      // Selection follows the backend-provided visible row order.
      // This controller does not apply additional client-side row transforms.
      datasets.map((dataset) => ({
        id: dataset.id.toString(),
        original: { id: dataset.id },
      })),
    [datasets],
  );

  const selection = useDatasetTableSelection({
    rows: selectionRows,
    rowVirtualizer,
    onDatasetSelected,
  });

  const tableOptions = useMemo(
    () => ({
      data: datasets,
      columns,
      state: {
        sorting,
        columnVisibility,
        rowSelection: selection.rowSelection,
      },
      onSortingChange: setSorting,
      onRowSelectionChange: selection.setRowSelection,
      getCoreRowModel: getCoreRowModel(),
      getRowId: (row: DatasetInfo) => row.id.toString(),
      manualSorting: true,
      autoResetRowSelection: false,
    }),
    [
      datasets,
      columns,
      sorting,
      columnVisibility,
      selection.rowSelection,
      selection.setRowSelection,
      setSorting,
    ],
  );

  const table = useReactTable(tableOptions);
  const { rows } = table.getRowModel();
  const visibleColumnCount = table.getVisibleLeafColumns().length;
  const virtualItems = rowVirtualizer.getVirtualItems();

  useEffect(() => {
    const last = virtualItems.at(-1);
    if (!last) {
      return;
    }

    if (hasMore && last.index >= rows.length - 10) {
      void loadNextPage();
    }
  }, [hasMore, loadNextPage, rows.length, virtualItems]);

  const virtualPaddingTop =
    virtualItems.length > 0 ? (virtualItems[0]?.start ?? 0) : 0;
  const virtualPaddingBottom =
    virtualItems.length > 0
      ? rowVirtualizer.getTotalSize() -
        (virtualItems[virtualItems.length - 1]?.end ?? 0)
      : 0;

  return {
    table,
    rows,
    visibleColumnCount,
    virtualItems,
    virtualPaddingTop,
    virtualPaddingBottom,
    rowSelection: selection.rowSelection,
    setRowSelection: selection.setRowSelection,
    registerRowElement: selection.registerRowElement,
    handleRowPointerDown: selection.handleRowPointerDown,
    handleRowPointerEnter: selection.handleRowPointerEnter,
    handleRowKeyDown: selection.handleRowKeyDown,
  };
}
