import { useMemo, useRef } from "react";
import { flexRender } from "@tanstack/react-table";
import { toast } from "sonner";
import { useDatasetTableData } from "../api/useDatasetTableData";
import { useDatasetColumnVisibility } from "../model/useDatasetColumnVisibility";
import { createDatasetColumns } from "./DatasetTableColumns";
import { DatasetTableBody } from "./DatasetTableBody";
import { DatasetTableToolbar } from "./DatasetTableToolbar";
import { useDatasetTableController } from "./useDatasetTableController";
import { useDatasetDeleteFlow } from "./useDatasetDeleteFlow";
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
  const controller = useDatasetTableController({
    datasets,
    columns: dataColumns,
    sorting,
    setSorting,
    columnVisibility,
    hasMore,
    loadNextPage,
    onDatasetSelected,
    tableContainerRef,
  });

  const deleteFlow = useDatasetDeleteFlow({
    deleteDatasets,
    isDeleting,
    selectedDatasetId,
    onDatasetSelected,
    setRowSelection: controller.setRowSelection,
    notify: toast,
  });

  return (
    <TooltipProvider>
      <div className="flex h-full min-h-0 flex-col bg-background">
        <DatasetTableToolbar
          table={controller.table}
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
              {controller.table.getHeaderGroups().map((headerGroup) => (
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
              rows={controller.rows}
              rowSelection={controller.rowSelection}
              visibleColumnCount={controller.visibleColumnCount}
              virtualItems={controller.virtualItems}
              virtualPaddingTop={controller.virtualPaddingTop}
              virtualPaddingBottom={controller.virtualPaddingBottom}
              selectedDatasetId={selectedDatasetId}
              allTags={allTags}
              isUpdatingTags={isUpdatingTags}
              registerRowElement={controller.registerRowElement}
              handleRowPointerDown={controller.handleRowPointerDown}
              handleRowPointerEnter={controller.handleRowPointerEnter}
              handleRowKeyDown={controller.handleRowKeyDown}
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
