import { flexRender, type Row } from "@tanstack/react-table";
import type { DatasetDeleteResult, DatasetInfo } from "../api/types";
import { DatasetTableRowActions } from "./DatasetTableRowActions";
import { TableBody, TableCell, TableRow } from "@/shared/ui/table";

interface VirtualRowLike {
  index: number;
  start: number;
  end: number;
}

interface DatasetTableBodyProps {
  rows: Row<DatasetInfo>[];
  rowSelection: Record<string, boolean>;
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
  rows,
  rowSelection,
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
  const selectedDatasets = rows
    .filter((row) => rowSelection[row.id])
    .map((row) => row.original);

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
            const isRowSelected = !!rowSelection[row.id];

            return (
              <DatasetTableRowActions
                key={row.id}
                dataset={dataset}
                selectedDatasets={selectedDatasets}
                allTags={allTags}
                isUpdatingTags={isUpdatingTags}
                onDatasetSelected={onDatasetSelected}
                openDeleteDialog={openDeleteDialog}
                batchAddTags={batchAddTags}
                batchRemoveTags={batchRemoveTags}
              >
                <TableRow
                  data-state={
                    (isSelected && "selected") || (isRowSelected && "selected")
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
                  onPointerEnter={() => handleRowPointerEnter(virtualRow.index)}
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
              </DatasetTableRowActions>
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
