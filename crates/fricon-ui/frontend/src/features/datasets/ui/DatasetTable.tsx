import { useEffect, useMemo, useRef } from "react";
import {
  flexRender,
  getCoreRowModel,
  type Table as TanStackTable,
  useReactTable,
} from "@tanstack/react-table";
import { useVirtualizer } from "@tanstack/react-virtual";
import { toast } from "sonner";
import { useDatasetTableData } from "../api/useDatasetTableData";
import { useDatasetColumnVisibility } from "../model/useDatasetColumnVisibility";
import { createDatasetColumns } from "./DatasetTableColumns";
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

export function DatasetTable({
  selectedDatasetId,
  onDatasetSelected,
}: DatasetTableProps) {
  const {
    datasets,
    searchQuery,
    setSearchQuery,
    selectedTags,
    selectedStatuses,
    sorting,
    setSorting,
    allTags,
    favoriteOnly,
    setFavoriteOnly,
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

  const columns = useMemo(
    () => createDatasetColumns({ toggleFavorite }),
    [toggleFavorite],
  );

  const {
    columnVisibility,
    resetColumnVisibilityToDefault,
    showAllColumns,
    handleColumnVisibilityChange,
  } = useDatasetColumnVisibility(columns);

  const tableContainerRef = useRef<HTMLDivElement>(null);
  const tableRef = useRef<TanStackTable<(typeof datasets)[0]> | null>(null);
  // TanStack Virtual is an intentional compiler boundary for this component.
  // eslint-disable-next-line react-hooks/incompatible-library
  const rowVirtualizer = useVirtualizer({
    count: datasets.length,
    getScrollElement: () => tableContainerRef.current,
    estimateSize: () => 36,
    overscan: 8,
  });

  const selection = useDatasetTableSelection({
    getRows: () => tableRef.current?.getRowModel().rows ?? [],
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
      getRowId: (row: (typeof datasets)[0]) => row.id.toString(),
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
  tableRef.current = table;
  const { rows } = table.getRowModel();
  const visibleColumnCount = table.getVisibleLeafColumns().length;

  const deleteFlow = useDatasetDeleteFlow({
    deleteDatasets,
    isDeleting,
    selectedDatasetId,
    onDatasetSelected,
    setRowSelection: selection.setRowSelection,
    notify: toast,
  });

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

  return (
    <TooltipProvider>
      <div className="flex h-full min-h-0 flex-col bg-background">
        <DatasetTableToolbar
          table={table}
          hasActiveFilters={hasActiveFilters}
          selectedTags={selectedTags}
          selectedStatuses={selectedStatuses}
          favoriteOnly={favoriteOnly}
          searchQuery={searchQuery}
          allTags={allTags}
          isUpdatingTags={isUpdatingTags}
          setFavoriteOnly={setFavoriteOnly}
          setSearchQuery={setSearchQuery}
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
              table={table}
              rows={rows}
              visibleColumnCount={visibleColumnCount}
              virtualItems={virtualItems}
              virtualPaddingTop={virtualPaddingTop}
              virtualPaddingBottom={virtualPaddingBottom}
              selectedDatasetId={selectedDatasetId}
              allTags={allTags}
              isUpdatingTags={isUpdatingTags}
              registerRowElement={selection.registerRowElement}
              handleRowPointerDown={selection.handleRowPointerDown}
              handleRowPointerEnter={selection.handleRowPointerEnter}
              handleRowKeyDown={selection.handleRowKeyDown}
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
