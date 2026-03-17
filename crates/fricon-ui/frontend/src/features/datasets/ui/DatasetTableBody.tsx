import { flexRender, type Row, type Table } from "@tanstack/react-table";
import type { DatasetDeleteResult, DatasetInfo } from "../api/types";
import { DatasetRowTagMenus } from "./DatasetTableTagMenu";
import {
  deriveDatasetTagMenuTarget,
  runDatasetTagMutation,
} from "../model/datasetTableTagMenuLogic";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "@/shared/ui/context-menu";
import { TableBody, TableCell, TableRow } from "@/shared/ui/table";
import { Trash2 } from "lucide-react";
import { toast } from "sonner";

interface VirtualRowLike {
  index: number;
  start: number;
  end: number;
}

interface DatasetTableBodyProps {
  table: Table<DatasetInfo>;
  rows: Row<DatasetInfo>[];
  visibleColumnCount: number;
  virtualItems: VirtualRowLike[];
  virtualPaddingTop: number;
  virtualPaddingBottom: number;
  selectedDatasetId?: number;
  allTags: string[];
  isUpdatingTags: boolean;
  registerRowElement: (
    rowId: string,
    rowElement: HTMLTableRowElement | null,
  ) => void;
  handleRowPointerDown: (
    event: React.PointerEvent<HTMLTableRowElement>,
    rowIndex: number,
    rowId: string,
    datasetId: number,
  ) => void;
  handleRowPointerEnter: (rowIndex: number) => void;
  handleRowKeyDown: (
    event: React.KeyboardEvent<HTMLTableRowElement>,
    rowIndex: number,
  ) => void;
  onDatasetSelected: (id?: number) => void;
  openDeleteDialog: (ids: number[]) => void;
  batchAddTags: (
    ids: number[],
    tags: string[],
  ) => Promise<DatasetDeleteResult[]>;
  batchRemoveTags: (
    ids: number[],
    tags: string[],
  ) => Promise<DatasetDeleteResult[]>;
}

export function DatasetTableBody({
  table,
  rows,
  visibleColumnCount,
  virtualItems,
  virtualPaddingTop,
  virtualPaddingBottom,
  selectedDatasetId,
  allTags,
  isUpdatingTags,
  registerRowElement,
  handleRowPointerDown,
  handleRowPointerEnter,
  handleRowKeyDown,
  onDatasetSelected,
  openDeleteDialog,
  batchAddTags,
  batchRemoveTags,
}: DatasetTableBodyProps) {
  const selectedRows = table.getFilteredSelectedRowModel().rows;
  const selectedDatasets = selectedRows.map((row) => row.original);
  const selectedCount = selectedRows.length;

  const handleBatchTagMutation = async (
    operation: "add" | "remove",
    targetIds: number[],
    tag: string,
  ) => {
    await runDatasetTagMutation({
      operation,
      targetIds,
      tag,
      batchAddTags,
      batchRemoveTags,
      notify: toast,
    });
  };

  return (
    <TableBody>
      {rows.length === 0 ? (
        <TableRow>
          <TableCell colSpan={visibleColumnCount} className="h-24 text-center">
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
            if (!row) {
              return null;
            }

            const dataset = row.original;
            const isSelected = dataset.id === selectedDatasetId;
            const isRowSelected = row.getIsSelected();
            const target = deriveDatasetTagMenuTarget(
              dataset,
              selectedDatasets,
            );

            return (
              <ContextMenu key={row.id}>
                <ContextMenuTrigger
                  render={
                    <TableRow
                      data-state={
                        (isSelected && "selected") ||
                        (isRowSelected && "selected")
                      }
                      ref={(element) => registerRowElement(row.id, element)}
                      data-index={virtualRow.index}
                      className="cursor-pointer select-none"
                      onPointerDown={(event) =>
                        handleRowPointerDown(
                          event,
                          virtualRow.index,
                          row.id,
                          dataset.id,
                        )
                      }
                      onPointerEnter={() =>
                        handleRowPointerEnter(virtualRow.index)
                      }
                      onKeyDown={(event) =>
                        handleRowKeyDown(event, virtualRow.index)
                      }
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
                  <DatasetRowTagMenus
                    allTags={allTags}
                    isUpdatingTags={isUpdatingTags}
                    target={target}
                    onAddTag={(tag) => {
                      void handleBatchTagMutation("add", target.targetIds, tag);
                    }}
                    onRemoveTag={(tag) => {
                      void handleBatchTagMutation(
                        "remove",
                        target.targetIds,
                        tag,
                      );
                    }}
                  />
                  <ContextMenuSeparator />
                  <ContextMenuItem
                    variant="destructive"
                    onClick={() => openDeleteDialog([dataset.id])}
                  >
                    <Trash2 data-icon="inline-start" />
                    Delete
                  </ContextMenuItem>
                  {selectedCount > 1 &&
                    selectedRows.some(
                      (selectedRow) => selectedRow.original.id === dataset.id,
                    ) && (
                      <ContextMenuItem
                        variant="destructive"
                        onClick={() =>
                          openDeleteDialog(
                            selectedRows.map(
                              (selectedRow) => selectedRow.original.id,
                            ),
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
                style={{ height: `${virtualPaddingBottom}px`, padding: 0 }}
                className="border-0 p-0"
              />
            </TableRow>
          )}
        </>
      )}
    </TableBody>
  );
}
