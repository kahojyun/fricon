import { useEffect, useMemo, useRef, useState } from "react";
import {
  flexRender,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useDatasetTableData } from "../api/useDatasetTableData";
import { useDatasetColumnVisibility } from "../model/useDatasetColumnVisibility";
import { createDatasetColumns } from "./DatasetTableColumns";
import { DatasetTableToolbar } from "./DatasetTableToolbar";
import { TooltipProvider } from "@/shared/ui/tooltip";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/shared/ui/table";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "@/shared/ui/context-menu";
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
import { Trash2 } from "lucide-react";
import { toast } from "sonner";

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
    tagFilterQuery,
    setTagFilterQuery,
    sorting,
    setSorting,
    filteredTagOptions,
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
  } = useDatasetTableData();

  const [rowSelection, setRowSelection] = useState<Record<string, boolean>>({});
  const [idsToDelete, setIdsToDelete] = useState<number[]>([]);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const [anchorIndex, setAnchorIndex] = useState<number | null>(null);
  const [dragState, setDragState] = useState<{
    initialSelection: Record<string, boolean>;
    mode: "replace" | "toggle";
    targetValue?: boolean;
  } | null>(null);

  useEffect(() => {
    const handleMouseUp = () => setDragState(null);
    window.addEventListener("pointerup", handleMouseUp);
    return () => window.removeEventListener("pointerup", handleMouseUp);
  }, []);

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

  const table = useReactTable({
    data: datasets,
    columns,
    state: {
      sorting,
      columnVisibility,
      rowSelection,
    },
    onSortingChange: setSorting,
    onRowSelectionChange: setRowSelection,
    getCoreRowModel: getCoreRowModel(),
    getRowId: (row) => row.id.toString(),
  });

  const { rows } = table.getRowModel();
  const visibleColumnCount = table.getVisibleLeafColumns().length;
  const tableContainerRef = useRef<HTMLDivElement>(null);

  const rowVirtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => tableContainerRef.current,
    estimateSize: () => 36,
    overscan: 8,
  });

  const virtualItems = rowVirtualizer.getVirtualItems();

  useEffect(() => {
    const last = virtualItems.at(-1);
    if (!last) return;
    if (hasMore && last.index >= rows.length - 10) {
      void loadNextPage();
    }
  }, [hasMore, loadNextPage, virtualItems, rows.length]);

  const virtualPaddingTop =
    virtualItems.length > 0 ? (virtualItems[0]?.start ?? 0) : 0;
  const virtualPaddingBottom =
    virtualItems.length > 0
      ? rowVirtualizer.getTotalSize() -
        (virtualItems[virtualItems.length - 1]?.end ?? 0)
      : 0;

  const handleRowPointerDown = (
    e: React.PointerEvent<HTMLTableRowElement>,
    rowIndex: number,
    rowId: string,
    datasetId: number,
  ) => {
    if (e.button !== 0) return;

    // Don't interfere with buttons or links
    if ((e.target as HTMLElement).closest('button:not([role="checkbox"]), a'))
      return;

    const isCheckbox = (e.target as HTMLElement).closest(
      'button[role="checkbox"]',
    );

    if (e.shiftKey) {
      e.preventDefault();
      const effectiveAnchor = anchorIndex ?? 0;
      const start = Math.min(effectiveAnchor, rowIndex);
      const end = Math.max(effectiveAnchor, rowIndex);
      const newSelection: Record<string, boolean> = {};
      for (let i = start; i <= end; i++) {
        const id = rows[i]?.id;
        if (id) newSelection[id] = true;
      }
      setRowSelection(newSelection);
      setDragState({
        initialSelection: {},
        mode: "replace",
      });
      onDatasetSelected(datasetId);
      return;
    }

    if (e.ctrlKey || e.metaKey || isCheckbox) {
      e.preventDefault();
      const isSelected = !!rowSelection[rowId];
      const nextValue = !isSelected;
      setRowSelection((prev) => ({ ...prev, [rowId]: nextValue }));
      setAnchorIndex(rowIndex);
      setDragState({
        initialSelection: rowSelection,
        mode: "toggle",
        targetValue: nextValue,
      });
      onDatasetSelected(datasetId);
      return;
    }

    // Normal click
    setRowSelection({ [rowId]: true });
    setAnchorIndex(rowIndex);
    setDragState({
      initialSelection: {},
      mode: "replace",
    });
    onDatasetSelected(datasetId);
  };

  const handleRowPointerEnter = (rowIndex: number) => {
    if (!dragState) return;

    if (dragState.mode === "replace") {
      const effectiveAnchor = anchorIndex ?? 0;
      const start = Math.min(effectiveAnchor, rowIndex);
      const end = Math.max(effectiveAnchor, rowIndex);
      const nextSelection: Record<string, boolean> = {};
      for (let i = start; i <= end; i++) {
        const id = rows[i]?.id;
        if (id) nextSelection[id] = true;
      }
      setRowSelection(nextSelection);
    } else if (dragState.mode === "toggle") {
      const effectiveAnchor = anchorIndex ?? 0;
      const start = Math.min(effectiveAnchor, rowIndex);
      const end = Math.max(effectiveAnchor, rowIndex);
      const nextSelection = { ...dragState.initialSelection };
      for (let i = start; i <= end; i++) {
        const id = rows[i]?.id;
        if (id) nextSelection[id] = !!dragState.targetValue;
      }
      setRowSelection(nextSelection);
    }
  };

  const handleDeleteClick = (ids: number[]) => {
    setIdsToDelete(ids);
    setIsDeleteDialogOpen(true);
  };

  const confirmDelete = async () => {
    try {
      const results = await deleteDatasets(idsToDelete);
      setRowSelection({});

      const successIds = results.filter((r) => r.success).map((r) => r.id);
      const failedResults = results.filter((r) => !r.success);

      // Invalidate selection if it was deleted
      if (selectedDatasetId && successIds.includes(selectedDatasetId)) {
        onDatasetSelected(undefined);
      }

      setIsDeleteDialogOpen(false);

      if (failedResults.length === 0) {
        setIdsToDelete([]);
        toast.success(`Successfully deleted ${successIds.length} dataset(s)`);
      } else if (successIds.length === 0) {
        // All failed
        toast.error(`Failed to delete ${failedResults.length} dataset(s)`);
      } else {
        // Partial success
        setIdsToDelete(failedResults.map((r) => r.id));
        toast.warning(
          `Successfully deleted ${successIds.length} dataset(s), but ${failedResults.length} failed.`,
          {
            description: failedResults
              .map((r) => `ID ${r.id}: ${r.error}`)
              .join("\n"),
          },
        );
      }
    } catch (error) {
      toast.error(
        error instanceof Error ? error.message : "Failed to delete dataset(s)",
      );
    }
  };

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
          tagFilterQuery={tagFilterQuery}
          filteredTagOptions={filteredTagOptions}
          setFavoriteOnly={setFavoriteOnly}
          setSearchQuery={setSearchQuery}
          setTagFilterQuery={setTagFilterQuery}
          handleTagToggle={handleTagToggle}
          handleStatusToggle={handleStatusToggle}
          clearFilters={clearFilters}
          resetColumnVisibilityToDefault={resetColumnVisibilityToDefault}
          showAllColumns={showAllColumns}
          onColumnVisibilityChange={handleColumnVisibilityChange}
        />
        <div
          className="min-h-0 flex-1 overflow-auto bg-background"
          ref={tableContainerRef}
        >
          <Table withContainer={false}>
            <TableHeader className="sticky top-0 z-10 border-b bg-background shadow-sm">
              {table.getHeaderGroups().map((headerGroup) => (
                <TableRow key={headerGroup.id}>
                  {headerGroup.headers.map((header) => {
                    return (
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
                    );
                  })}
                </TableRow>
              ))}
            </TableHeader>
            <TableBody>
              {rows.length === 0 ? (
                <TableRow>
                  <TableCell
                    colSpan={visibleColumnCount}
                    className="h-24 text-center"
                  >
                    No datasets matched the current filters.
                  </TableCell>
                </TableRow>
              ) : (
                <>
                  {virtualPaddingTop > 0 && (
                    <TableRow className="h-0 border-0 hover:bg-transparent">
                      <TableCell
                        colSpan={visibleColumnCount}
                        style={{ height: `${virtualPaddingTop}px`, padding: 0 }}
                        className="border-0 p-0"
                      />
                    </TableRow>
                  )}
                  {virtualItems.map((virtualRow) => {
                    const row = rows[virtualRow.index];
                    if (!row) return null;
                    const dataset = row.original;
                    const isSelected = dataset.id === selectedDatasetId;
                    const isRowSelected = row.getIsSelected();
                    const selectedRows =
                      table.getFilteredSelectedRowModel().rows;
                    const selectedCount = selectedRows.length;

                    return (
                      <ContextMenu key={row.id}>
                        <ContextMenuTrigger
                          render={
                            <TableRow
                              data-state={
                                (isSelected && "selected") ||
                                (isRowSelected && "selected")
                              }
                              ref={rowVirtualizer.measureElement}
                              data-index={virtualRow.index}
                              className="cursor-pointer select-none"
                              onPointerDown={(e) =>
                                handleRowPointerDown(
                                  e,
                                  virtualRow.index,
                                  row.id,
                                  dataset.id,
                                )
                              }
                              onPointerEnter={() =>
                                handleRowPointerEnter(virtualRow.index)
                              }
                              onKeyDown={(event) => {
                                if (
                                  event.key === "Enter" ||
                                  event.key === " "
                                ) {
                                  onDatasetSelected(dataset.id);
                                }
                              }}
                              tabIndex={0}
                            >
                              {row.getVisibleCells().map((cell) => (
                                <TableCell key={cell.id}>
                                  {flexRender(
                                    cell.column.columnDef.cell,
                                    cell.getContext(),
                                  )}
                                </TableCell>
                              ))}
                            </TableRow>
                          }
                        />
                        <ContextMenuContent className="w-64">
                          <ContextMenuItem
                            onClick={() => onDatasetSelected(dataset.id)}
                          >
                            View Details
                          </ContextMenuItem>
                          <ContextMenuSeparator />
                          <ContextMenuItem
                            variant="destructive"
                            onClick={() => handleDeleteClick([dataset.id])}
                          >
                            <Trash2 data-icon="inline-start" />
                            Delete
                          </ContextMenuItem>
                          {selectedCount > 1 &&
                            selectedRows.some(
                              (r) => r.original.id === dataset.id,
                            ) && (
                              <ContextMenuItem
                                variant="destructive"
                                onClick={() =>
                                  handleDeleteClick(
                                    selectedRows.map((r) => r.original.id),
                                  )
                                }
                              >
                                <Trash2 data-icon="inline-start" />
                                Delete Selected ({selectedCount})
                              </ContextMenuItem>
                            )}
                        </ContextMenuContent>
                      </ContextMenu>
                    );
                  })}
                  {virtualPaddingBottom > 0 && (
                    <TableRow className="h-0 border-0 hover:bg-transparent">
                      <TableCell
                        colSpan={visibleColumnCount}
                        style={{
                          height: `${virtualPaddingBottom}px`,
                          padding: 0,
                        }}
                        className="border-0 p-0"
                      />
                    </TableRow>
                  )}
                </>
              )}
            </TableBody>
          </Table>
        </div>
      </div>

      <AlertDialog
        open={isDeleteDialogOpen}
        onOpenChange={(open) => {
          if (!open && isDeleting) return;
          setIsDeleteDialogOpen(open);
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Dataset</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete {idsToDelete.length} dataset(s)?
              This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={isDeleting}>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={(e) => {
                e.preventDefault();
                void confirmDelete();
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
