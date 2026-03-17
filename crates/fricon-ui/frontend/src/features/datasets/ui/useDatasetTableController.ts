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
import { createDatasetSelectionColumn } from "./DatasetTableColumns";
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
  datasets: visibleDatasets,
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
    count: visibleDatasets.length,
    getScrollElement: () => tableContainerRef.current,
    estimateSize: () => 36,
    overscan: 8,
  });

  const selectionRows = useMemo(
    () =>
      // Selection assumes the backend already provides the visible row order.
      // This controller does not apply additional client-side row transforms.
      visibleDatasets.map((dataset) => ({
        id: dataset.id.toString(),
        original: { id: dataset.id },
      })),
    [visibleDatasets],
  );

  const selection = useDatasetTableSelection({
    rows: selectionRows,
    rowVirtualizer,
    onDatasetSelected,
  });
  const {
    rowSelection,
    setRowSelection,
    registerRowElement,
    handleRowPointerDown,
    handleRowPointerEnter,
    handleRowKeyDown,
  } = selection;

  const selectionActions = useMemo(
    () => ({
      toggleDatasetRowSelected: (rowId: string, isSelected: boolean) => {
        setRowSelection((previous) => {
          const nextSelection = { ...previous };
          if (isSelected) {
            nextSelection[rowId] = true;
          } else {
            delete nextSelection[rowId];
          }
          return nextSelection;
        });
      },
      toggleAllDatasetRowsSelected: (isSelected: boolean) => {
        setRowSelection(() => {
          if (!isSelected) {
            return {};
          }

          return Object.fromEntries(
            visibleDatasets.map((dataset) => [dataset.id.toString(), true]),
          );
        });
      },
    }),
    [visibleDatasets, setRowSelection],
  );

  const tableColumns = useMemo(
    () => [
      createDatasetSelectionColumn({
        toggleRowSelected: selectionActions.toggleDatasetRowSelected,
        toggleAllRowsSelected: selectionActions.toggleAllDatasetRowsSelected,
      }),
      ...columns,
    ],
    [columns, selectionActions],
  );

  const tableOptions = useMemo(
    () => ({
      data: visibleDatasets,
      columns: tableColumns,
      state: {
        sorting,
        columnVisibility,
        rowSelection,
      },
      onSortingChange: setSorting,
      getCoreRowModel: getCoreRowModel(),
      getRowId: (row: DatasetInfo) => row.id.toString(),
      enableRowSelection: true,
      autoResetRowSelection: false,
      manualSorting: true,
    }),
    [
      visibleDatasets,
      tableColumns,
      sorting,
      columnVisibility,
      rowSelection,
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
    rowSelection,
    setRowSelection,
    registerRowElement,
    handleRowPointerDown,
    handleRowPointerEnter,
    handleRowKeyDown,
  };
}
