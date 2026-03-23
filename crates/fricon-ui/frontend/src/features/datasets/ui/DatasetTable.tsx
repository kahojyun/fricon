import { useEffect, useEffectEvent, useMemo, useRef, useState } from "react";
import {
  flexRender,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";
import { useVirtualizer } from "@tanstack/react-virtual";
import { toast } from "sonner";
import { onDatasetArchiveDrop } from "../api/events";
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
import { useDatasetImportFlow } from "./useDatasetImportFlow";
import { ImportDatasetDialog } from "./ImportDatasetDialog";
import { summarizeDatasetDeleteResults } from "../model/datasetTableDeleteFlowLogic";
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
  return undefined;
}

export function DatasetTable({
  selectedDatasetId,
  onDatasetSelected,
}: DatasetTableProps) {
  "use no memo";
  const {
    datasets,
    viewMode,
    setViewMode,
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
    trashDatasets,
    restoreDatasets,
    handleTagToggle,
    handleStatusToggle,
    clearFilters,
    loadNextPage,
    deleteDatasets,
    emptyTrash,
    isMutatingDatasets,
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

  useEffect(() => {
    setRowSelection({});
  }, [setRowSelection, viewMode]);

  useEffect(() => {
    if (
      selectedDatasetId !== undefined &&
      !datasets.some((dataset) => dataset.id === selectedDatasetId)
    ) {
      onDatasetSelected(undefined);
    }
  }, [datasets, onDatasetSelected, selectedDatasetId]);

  const virtualPaddingTop =
    virtualItems.length > 0 ? (virtualItems[0]?.start ?? 0) : 0;
  const virtualPaddingBottom =
    virtualItems.length > 0
      ? rowVirtualizer.getTotalSize() -
        (virtualItems[virtualItems.length - 1]?.end ?? 0)
      : 0;

  const trashFlow = useDatasetDeleteFlow({
    deleteDatasets: trashDatasets,
    isDeleting: isMutatingDatasets,
    selectedDatasetId,
    onDatasetSelected,
    setRowSelection,
    notify: toast,
    messages: {
      actionLabel: "Move to Trash",
      success: (count) => `Moved ${count} dataset(s) to trash`,
      failure: (count) => `Failed to move ${count} dataset(s) to trash`,
      partial: (successCount, failureCount) =>
        `Moved ${successCount} dataset(s) to trash, but ${failureCount} failed.`,
    },
  });

  const restoreFlow = useDatasetDeleteFlow({
    deleteDatasets: restoreDatasets,
    isDeleting: isMutatingDatasets,
    selectedDatasetId,
    onDatasetSelected,
    setRowSelection,
    notify: toast,
    messages: {
      actionLabel: "Restore",
      success: (count) => `Restored ${count} dataset(s)`,
      failure: (count) => `Failed to restore ${count} dataset(s)`,
      partial: (successCount, failureCount) =>
        `Restored ${successCount} dataset(s), but ${failureCount} failed.`,
    },
  });

  const purgeFlow = useDatasetDeleteFlow({
    deleteDatasets,
    isDeleting: isMutatingDatasets,
    selectedDatasetId,
    onDatasetSelected,
    setRowSelection,
    notify: toast,
    messages: {
      actionLabel: "Permanently Delete",
      success: (count) => `Permanently deleted ${count} dataset(s)`,
      failure: (count) => `Failed to permanently delete ${count} dataset(s)`,
      partial: (successCount, failureCount) =>
        `Permanently deleted ${successCount} dataset(s), but ${failureCount} failed.`,
    },
  });

  const importFlow = useDatasetImportFlow();
  const handleDatasetArchiveDrop = useEffectEvent((paths: string[]) => {
    importFlow.startImportFromFiles(paths);
  });

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let active = true;

    try {
      void onDatasetArchiveDrop((paths) => {
        if (!active) {
          return;
        }
        handleDatasetArchiveDrop(paths);
      })
        .then((nextUnlisten) => {
          if (!active) {
            nextUnlisten();
            return;
          }
          unlisten = nextUnlisten;
        })
        .catch(() => undefined);
    } catch {
      return;
    }

    return () => {
      active = false;
      unlisten?.();
    };
  }, []);

  const [isEmptyTrashDialogOpen, setIsEmptyTrashDialogOpen] = useState(false);

  const handleEmptyTrash = async () => {
    try {
      const results = await emptyTrash();
      const summary = summarizeDatasetDeleteResults(results);

      if (
        selectedDatasetId !== undefined &&
        summary.successIds.includes(selectedDatasetId)
      ) {
        onDatasetSelected(undefined);
      }

      if (summary.outcome === "success") {
        setRowSelection({});
        setIsEmptyTrashDialogOpen(false);
        toast.success(
          `Permanently deleted ${summary.successIds.length} dataset(s)`,
        );
        return;
      }

      setRowSelection(
        Object.fromEntries(
          summary.failedIds.map((id) => [id.toString(), true]),
        ),
      );

      if (summary.outcome === "failure") {
        toast.error(
          `Failed to permanently delete ${summary.failedResults.length} dataset(s)`,
          {
            description: summary.failedResults
              .map((result) => `ID ${result.id}: ${result.error}`)
              .join("\n"),
          },
        );
        return;
      }

      toast.warning(
        `Permanently deleted ${summary.successIds.length} dataset(s), but ${summary.failedResults.length} failed.`,
        {
          description: summary.failedResults
            .map((result) => `ID ${result.id}: ${result.error}`)
            .join("\n"),
        },
      );
    } catch (error) {
      toast.error(
        error instanceof Error ? error.message : "Failed to empty trash",
      );
    }
  };

  return (
    <TooltipProvider>
      <div className="flex h-full min-h-0 flex-col bg-background">
        <DatasetTableToolbar
          table={table}
          viewMode={viewMode}
          hasActiveFilters={hasActiveFilters}
          activeTags={activeTags}
          activeStatuses={activeStatuses}
          showFavoritesOnly={showFavoritesOnly}
          searchInput={searchInput}
          allTags={allTags}
          isUpdatingTags={isUpdatingTags}
          isMutatingDatasets={isMutatingDatasets}
          setShowFavoritesOnly={setShowFavoritesOnly}
          setViewMode={setViewMode}
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
          onEmptyTrash={() => setIsEmptyTrashDialogOpen(true)}
          onImportDataset={importFlow.startImportDialog}
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
                      aria-sort={
                        header.column.getCanSort()
                          ? sortStateToAriaSort(header.column.getIsSorted())
                          : undefined
                      }
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
              viewMode={viewMode}
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
              onTrash={(ids) => {
                void trashFlow.performDelete(ids);
              }}
              onRestore={(ids) => {
                void restoreFlow.performDelete(ids);
              }}
              onPermanentDelete={purgeFlow.openDeleteDialog}
              batchAddTags={batchAddTags}
              batchRemoveTags={batchRemoveTags}
            />
          </Table>
        </div>
      </div>

      <AlertDialog
        open={purgeFlow.isDeleteDialogOpen}
        onOpenChange={(open) => {
          if (!open) {
            purgeFlow.closeDeleteDialog();
            return;
          }

          purgeFlow.setIsDeleteDialogOpen(true);
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Permanently Delete Dataset</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to permanently delete{" "}
              {purgeFlow.idsToDelete.length} dataset(s)? This action cannot be
              undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={isMutatingDatasets}>
              Cancel
            </AlertDialogCancel>
            <AlertDialogAction
              onClick={(event) => {
                event.preventDefault();
                void purgeFlow.confirmDelete();
              }}
              disabled={isMutatingDatasets}
              className="text-destructive-foreground bg-destructive hover:bg-destructive/90"
            >
              {isMutatingDatasets ? "Deleting..." : "Delete"}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <AlertDialog
        open={isEmptyTrashDialogOpen}
        onOpenChange={(open) => {
          if (!open && isMutatingDatasets) {
            return;
          }

          setIsEmptyTrashDialogOpen(open);
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Empty Trash</AlertDialogTitle>
            <AlertDialogDescription>
              Permanently delete all datasets currently in trash? This action
              cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={isMutatingDatasets}>
              Cancel
            </AlertDialogCancel>
            <AlertDialogAction
              onClick={(event) => {
                event.preventDefault();
                void handleEmptyTrash();
              }}
              disabled={isMutatingDatasets}
              className="text-destructive-foreground bg-destructive hover:bg-destructive/90"
            >
              {isMutatingDatasets ? "Deleting..." : "Empty Trash"}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <ImportDatasetDialog
        open={importFlow.isDialogOpen}
        onOpenChange={(open) => {
          if (!open) {
            importFlow.closeDialog();
          }
        }}
        previewResults={importFlow.previewResults}
        duplicateBatchConflicts={importFlow.duplicateBatchConflicts}
        isImporting={importFlow.isImporting}
        onConfirm={() => {
          void importFlow.confirmImport();
        }}
      />
    </TooltipProvider>
  );
}
