import { useEffect, useMemo, useRef } from "react";
import {
  flexRender,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";
import { useVirtualizer } from "@tanstack/react-virtual";
import { toast } from "sonner";
import { useDatasetTableData } from "../api/useDatasetTableData";
import type { DatasetInfo } from "../api/types";
import { useDatasetColumnVisibility } from "../model/useDatasetColumnVisibility";
import {
  createDatasetColumns,
  createDatasetSelectionColumn,
} from "./DatasetTableColumns";
import { DatasetTableBody } from "./DatasetTableBody";
import { DatasetTableToolbar } from "./DatasetTableToolbar";
import { useDatasetDeleteFlow } from "./useDatasetDeleteFlow";
import { useDatasetTableSelection } from "./useDatasetTableSelection";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/shared/ui/alert-dialog";
import { Table, TableHead, TableHeader, TableRow } from "@/shared/ui/table";
import { TooltipProvider } from "@/shared/ui/tooltip";

interface DatasetTableProps {
  selectedDatasetId?: number;
  onDatasetSelected: (id?: number) => void;
}

const datasetCoreRowModel = getCoreRowModel();

function sortStateToAriaSort(sorted: false | "asc" | "desc") {
  if (sorted === "asc") {
    return "ascending";
  }
  if (sorted === "desc") {
    return "descending";
  }
  return "none";
}

export function DatasetTable({
  selectedDatasetId,
  onDatasetSelected,
}: DatasetTableProps) {
  "use no memo";
  const {
    datasets,
    searchInput,
    setSearchInput,
    activeTags,
    activeStatuses,
    sorting,
    setSorting,
    allTags,
    showFavoritesOnly,
    setShowFavoritesOnly,
    hasMore,
    hasActiveFilters,
    toggleFavorite,
    handleTagToggle,
    handleStatusToggle,
    clearFilters,
    loadNextPage,
    deleteDatasets,
    isDeleting,
    batchAddTags,
    batchRemoveTags,
    deleteTag,
    renameTag,
    mergeTag,
    isUpdatingTags,
  } = useDatasetTableData();

  const dataColumns = useMemo(
    () => createDatasetColumns({ toggleFavorite }),
    [toggleFavorite],
  );

  const {
    columnVisibility,
    resetColumnVisibilityToDefault,
    showAllColumns,
    handleColumnVisibilityChange,
  } = useDatasetColumnVisibility(dataColumns);

  const tableContainerRef = useRef<HTMLDivElement>(null);
  // TanStack Virtual stays in this leaf component as an explicit compiler boundary.
  // eslint-disable-next-line react-hooks/incompatible-library
  const rowVirtualizer = useVirtualizer({
    count: datasets.length,
    getScrollElement: () => tableContainerRef.current,
    estimateSize: () => 36,
    overscan: 8,
  });

  const selectionRows = useMemo(
    () =>
      // Selection assumes the backend already provides the visible row order.
      // This component does not apply additional client-side row transforms.
      datasets.map((dataset) => ({
        id: dataset.id.toString(),
        original: { id: dataset.id },
      })),
    [datasets],
  );

  const {
    rowSelection,
    setRowSelection,
    registerRowElement,
    handleRowPointerDown,
    handleRowPointerEnter,
    handleRowKeyDown,
  } = useDatasetTableSelection({
    rows: selectionRows,
    rowVirtualizer,
    onDatasetSelected,
  });

  const tableColumns = useMemo(
    () => [
      createDatasetSelectionColumn({
        toggleRowSelected: (rowId: string, isSelected: boolean) => {
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
        toggleAllRowsSelected: (isSelected: boolean) => {
          setRowSelection(() => {
            if (!isSelected) {
              return {};
            }

            return Object.fromEntries(
              datasets.map((dataset) => [dataset.id.toString(), true]),
            );
          });
        },
      }),
      ...dataColumns,
    ],
    [dataColumns, datasets, setRowSelection],
  );

  const tableOptions = {
    data: datasets,
    columns: tableColumns,
    state: {
      sorting,
      columnVisibility,
      rowSelection,
    },
    onSortingChange: setSorting,
    getCoreRowModel: datasetCoreRowModel,
    getRowId: (row: DatasetInfo) => row.id.toString(),
    enableRowSelection: true,
    autoResetRowSelection: false,
    manualSorting: true,
  };
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

  const deleteFlow = useDatasetDeleteFlow({
    deleteDatasets,
    isDeleting,
    selectedDatasetId,
    onDatasetSelected,
    setRowSelection,
    notify: toast,
  });

  return (
    <TooltipProvider>
      <div className="flex h-full min-h-0 flex-col bg-background">
        <DatasetTableToolbar
          table={table}
          hasActiveFilters={hasActiveFilters}
          activeTags={activeTags}
          activeStatuses={activeStatuses}
          showFavoritesOnly={showFavoritesOnly}
          searchInput={searchInput}
          allTags={allTags}
          isUpdatingTags={isUpdatingTags}
          setShowFavoritesOnly={setShowFavoritesOnly}
          setSearchInput={setSearchInput}
          handleTagToggle={handleTagToggle}
          handleStatusToggle={handleStatusToggle}
          clearFilters={clearFilters}
          resetColumnVisibilityToDefault={resetColumnVisibilityToDefault}
          showAllColumns={showAllColumns}
          onColumnVisibilityChange={handleColumnVisibilityChange}
          onDeleteTag={deleteTag}
          onRenameTag={renameTag}
          onMergeTag={mergeTag}
        />
        <div
          className="min-h-0 flex-1 overflow-auto bg-background"
          ref={tableContainerRef}
        >
          <Table withContainer={false}>
            <TableHeader className="sticky top-0 z-10 border-b bg-background shadow-sm">
              {table.getHeaderGroups().map((headerGroup) => (
                <TableRow key={headerGroup.id}>
                  {headerGroup.headers.map((header) => (
                    <TableHead
                      key={header.id}
                      style={{ width: header.getSize() }}
                      className="text-muted-foreground"
                      aria-sort={sortStateToAriaSort(
                        header.column.getIsSorted(),
                      )}
                    >
                      {header.isPlaceholder
                        ? null
                        : flexRender(
                            header.column.columnDef.header,
                            header.getContext(),
                          )}
                    </TableHead>
                  ))}
                </TableRow>
              ))}
            </TableHeader>
            <DatasetTableBody
              rows={rows}
              rowSelection={rowSelection}
              visibleColumnCount={visibleColumnCount}
              virtualItems={virtualItems}
              virtualPaddingTop={virtualPaddingTop}
              virtualPaddingBottom={virtualPaddingBottom}
              selectedDatasetId={selectedDatasetId}
              allTags={allTags}
              isUpdatingTags={isUpdatingTags}
              registerRowElement={registerRowElement}
              handleRowPointerDown={handleRowPointerDown}
              handleRowPointerEnter={handleRowPointerEnter}
              handleRowKeyDown={handleRowKeyDown}
              onDatasetSelected={onDatasetSelected}
              openDeleteDialog={deleteFlow.openDeleteDialog}
              batchAddTags={batchAddTags}
              batchRemoveTags={batchRemoveTags}
            />
          </Table>
        </div>
      </div>

      <AlertDialog
        open={deleteFlow.isDeleteDialogOpen}
        onOpenChange={(open) => {
          if (!open) {
            deleteFlow.closeDeleteDialog();
            return;
          }

          deleteFlow.setIsDeleteDialogOpen(true);
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Dataset</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete {deleteFlow.idsToDelete.length}{" "}
              dataset(s)? This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={isDeleting}>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={(event) => {
                event.preventDefault();
                void deleteFlow.confirmDelete();
              }}
              disabled={isDeleting}
              className="text-destructive-foreground bg-destructive hover:bg-destructive/90"
            >
              {isDeleting ? "Deleting..." : "Delete"}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </TooltipProvider>
  );
}
