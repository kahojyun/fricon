import { useEffect, useRef, type KeyboardEvent } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { CascadeMode } from "../model/cascadeReducer";
import type { ColumnUniqueValue, FilterTableData } from "../api/types";
import { Switch } from "@/shared/ui/switch";
import { Separator } from "@/shared/ui/separator";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/shared/ui/table";

interface FilterTableProps {
  data?: FilterTableData;
  mode: CascadeMode;
  onModeChange: (mode: CascadeMode) => void;
  selectedRowIndex: number | null;
  onSelectRow: (rowIndex: number) => void;
  selectedValueIndices?: number[];
  onSelectFieldValue: (fieldIndex: number, valueIndex: number) => void;
}

interface FilterTableColumnProps {
  columnIndex: number;
  field: string;
  items: ColumnUniqueValue[];
  selectedIndex?: number;
  onSelect: (index: number) => void;
  onRegisterFocusItem: (
    columnIndex: number,
    focusItem: ((itemIndex: number, fallbackItemIndex?: number) => void) | null,
  ) => void;
  onFocusColumnItem: (columnIndex: number, fallbackItemIndex?: number) => void;
}

const focusRowClassName =
  "cursor-pointer outline-none [&:focus>td]:bg-accent/70 [&:focus>td]:shadow-[inset_0_0_0_2px_color-mix(in_oklab,var(--color-ring)_45%,transparent)] [&:focus>td]:relative";

function getAdjacentIndex(
  currentIndex: number,
  key: string,
  totalCount: number,
): number {
  if (key === "ArrowUp") {
    return Math.max(currentIndex - 1, 0);
  }
  if (key === "ArrowDown") {
    return Math.min(currentIndex + 1, totalCount - 1);
  }
  return currentIndex;
}

function handleSelectableRowKeyDown(
  event: KeyboardEvent<HTMLTableRowElement>,
  options: {
    currentIndex: number;
    totalCount: number;
    onSelect: () => void;
    onNavigate: (nextIndex: number) => void;
  },
) {
  if (event.key === "ArrowUp" || event.key === "ArrowDown") {
    event.preventDefault();
    const nextIndex = getAdjacentIndex(
      options.currentIndex,
      event.key,
      options.totalCount,
    );

    if (nextIndex !== options.currentIndex) {
      options.onNavigate(nextIndex);
    }
    return;
  }

  if (event.key === "Enter" || event.key === " ") {
    event.preventDefault();
    options.onSelect();
  }
  if (event.metaKey || event.ctrlKey) {
    event.stopPropagation();
  }
}

function handleColumnBoundaryKeyDown(
  event: KeyboardEvent<HTMLTableRowElement>,
  options: {
    columnIndex: number;
    selectedIndex?: number;
    onFocusColumnItem: (
      columnIndex: number,
      fallbackItemIndex?: number,
    ) => void;
  },
) {
  if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") {
    return false;
  }

  event.preventDefault();
  options.onFocusColumnItem(
    options.columnIndex + (event.key === "ArrowRight" ? 1 : -1),
    options.selectedIndex,
  );
  return true;
}

function FilterTableColumn({
  columnIndex,
  field,
  items,
  selectedIndex,
  onSelect,
  onRegisterFocusItem,
  onFocusColumnItem,
}: FilterTableColumnProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const rowElementMapRef = useRef(new Map<number, HTMLTableRowElement>());
  const pendingFocusIndexRef = useRef<number | null>(null);

  // TanStack Virtual is an intentional compiler boundary for this component.
  // eslint-disable-next-line react-hooks/incompatible-library
  const rowVirtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => 28,
    overscan: 8,
  });

  const virtualItems = rowVirtualizer.getVirtualItems();

  const virtualPaddingTop =
    virtualItems.length > 0 ? (virtualItems[0]?.start ?? 0) : 0;
  const virtualPaddingBottom =
    virtualItems.length > 0
      ? rowVirtualizer.getTotalSize() -
        (virtualItems[virtualItems.length - 1]?.end ?? 0)
      : 0;

  const focusItem = (itemIndex: number) => {
    pendingFocusIndexRef.current = itemIndex;
    const rowElement = rowElementMapRef.current.get(itemIndex);
    if (!rowElement) return;
    rowElement.focus();
    pendingFocusIndexRef.current = null;
  };

  const registerRowElement = (
    itemIndex: number,
    rowElement: HTMLTableRowElement | null,
  ) => {
    if (!rowElement) {
      rowElementMapRef.current.delete(itemIndex);
      return;
    }

    rowElementMapRef.current.set(itemIndex, rowElement);
    rowVirtualizer.measureElement(rowElement);

    if (pendingFocusIndexRef.current === itemIndex) {
      rowElement.focus();
      pendingFocusIndexRef.current = null;
    }
  };

  const selectItemAt = (
    itemPosition: number,
    options: {
      focus?: boolean;
      scroll?: boolean;
    } = {},
  ) => {
    const item = items[itemPosition];
    if (!item) return;

    if (options.scroll) {
      rowVirtualizer.scrollToIndex?.(itemPosition, { align: "auto" });
    }

    onSelect(item.index);

    if (options.focus) {
      focusItem(item.index);
    }
  };

  useEffect(() => {
    onRegisterFocusItem(columnIndex, (itemIndex, fallbackItemIndex) => {
      const itemPosition = items.findIndex((item) => item.index === itemIndex);
      if (itemPosition !== -1) {
        rowVirtualizer.scrollToIndex?.(itemPosition, { align: "auto" });
        focusItem(items[itemPosition].index);
        return;
      }

      if (fallbackItemIndex != null) {
        const fallbackPosition = items.findIndex(
          (item) => item.index === fallbackItemIndex,
        );
        if (fallbackPosition !== -1) {
          rowVirtualizer.scrollToIndex?.(fallbackPosition, { align: "auto" });
          focusItem(items[fallbackPosition].index);
          return;
        }
      }

      if (items.length > 0) {
        rowVirtualizer.scrollToIndex?.(0, { align: "auto" });
        focusItem(items[0].index);
      }
    });

    return () => onRegisterFocusItem(columnIndex, null);
  }, [columnIndex, items, onRegisterFocusItem, rowVirtualizer]);

  return (
    <div
      data-testid={`filter-column-${field}`}
      className="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden bg-background"
    >
      <div ref={scrollRef} className="min-h-0 flex-1 overflow-auto">
        <Table withContainer={false}>
          <TableHeader className="sticky top-0 z-10 border-b bg-background shadow-sm">
            <TableRow>
              <TableHead className="text-muted-foreground">{field}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {virtualPaddingTop > 0 && (
              <TableRow className="h-0 border-0 hover:bg-transparent">
                <TableCell
                  style={{ height: `${virtualPaddingTop}px`, padding: 0 }}
                  className="border-0 p-0"
                />
              </TableRow>
            )}
            {virtualItems.map((virtualRow) => {
              const item = items[virtualRow.index];
              if (!item) return null;
              const isSelected = selectedIndex === item.index;
              return (
                <TableRow
                  key={item.index}
                  data-state={isSelected && "selected"}
                  ref={(element) => registerRowElement(item.index, element)}
                  data-index={virtualRow.index}
                  className={focusRowClassName}
                  onClick={(event) => {
                    event.currentTarget.focus();
                    onSelect(item.index);
                  }}
                  onKeyDown={(event) => {
                    if (
                      handleColumnBoundaryKeyDown(event, {
                        columnIndex,
                        selectedIndex,
                        onFocusColumnItem,
                      })
                    ) {
                      return;
                    }

                    handleSelectableRowKeyDown(event, {
                      currentIndex: virtualRow.index,
                      totalCount: items.length,
                      onSelect: () => onSelect(item.index),
                      onNavigate: (nextIndex) =>
                        selectItemAt(nextIndex, {
                          focus: true,
                          scroll: true,
                        }),
                    });
                  }}
                  tabIndex={0}
                >
                  <TableCell className="truncate">
                    {item.displayValue}
                  </TableCell>
                </TableRow>
              );
            })}
            {virtualPaddingBottom > 0 && (
              <TableRow className="h-0 border-0 hover:bg-transparent">
                <TableCell
                  style={{ height: `${virtualPaddingBottom}px`, padding: 0 }}
                  className="border-0 p-0"
                />
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  );
}

export function FilterTable({
  data,
  mode,
  onModeChange,
  selectedRowIndex,
  onSelectRow,
  selectedValueIndices,
  onSelectFieldValue,
}: FilterTableProps) {
  const tableContainerRef = useRef<HTMLDivElement>(null);
  const rowElementMapRef = useRef(new Map<number, HTMLTableRowElement>());
  const pendingFocusIndexRef = useRef<number | null>(null);

  const showFilterToggle = Boolean(data && data.fields.length > 1);
  const isFilterTableEmpty = !data || data.rows.length === 0;

  const minTableWidth = data
    ? `${Math.max(data.fields.length * 80, 320)}px`
    : "0px";

  // TanStack Virtual is an intentional compiler boundary for this component.
  // eslint-disable-next-line react-hooks/incompatible-library
  const rowVirtualizer = useVirtualizer({
    count: data?.rows.length ?? 0,
    getScrollElement: () => tableContainerRef.current,
    estimateSize: () => 28,
    overscan: 8,
  });

  const virtualItems = rowVirtualizer.getVirtualItems();

  const columnUniqueValues: Record<string, ColumnUniqueValue[]> =
    mode === "split" && data ? data.columnUniqueValues : {};

  const splitColumnFocusersRef = useRef<
    Map<number, (itemIndex: number, fallbackItemIndex?: number) => void>
  >(new Map());

  if (isFilterTableEmpty) {
    return (
      <div className="flex h-full items-center justify-center bg-background text-sm text-muted-foreground">
        No data available
      </div>
    );
  }

  const virtualPaddingTop =
    virtualItems.length > 0 ? (virtualItems[0]?.start ?? 0) : 0;
  const virtualPaddingBottom =
    virtualItems.length > 0
      ? rowVirtualizer.getTotalSize() -
        (virtualItems[virtualItems.length - 1]?.end ?? 0)
      : 0;

  const focusRow = (rowIndex: number) => {
    pendingFocusIndexRef.current = rowIndex;
    const rowElement = rowElementMapRef.current.get(rowIndex);
    if (!rowElement) return;
    rowElement.focus();
    pendingFocusIndexRef.current = null;
  };

  const registerRowElement = (
    rowIndex: number,
    rowElement: HTMLTableRowElement | null,
  ) => {
    if (!rowElement) {
      rowElementMapRef.current.delete(rowIndex);
      return;
    }

    rowElementMapRef.current.set(rowIndex, rowElement);
    rowVirtualizer.measureElement(rowElement);

    if (pendingFocusIndexRef.current === rowIndex) {
      rowElement.focus();
      pendingFocusIndexRef.current = null;
    }
  };

  const selectRowAt = (
    rowPosition: number,
    options: {
      focus?: boolean;
      scroll?: boolean;
    } = {},
  ) => {
    const row = data.rows[rowPosition];
    if (!row) return;

    if (options.scroll) {
      rowVirtualizer.scrollToIndex?.(rowPosition, { align: "auto" });
    }

    onSelectRow(row.index);

    if (options.focus) {
      focusRow(row.index);
    }
  };

  const registerSplitColumnFocuser = (
    columnIndex: number,
    focusItem: ((itemIndex: number, fallbackItemIndex?: number) => void) | null,
  ) => {
    if (!focusItem) {
      splitColumnFocusersRef.current.delete(columnIndex);
      return;
    }
    splitColumnFocusersRef.current.set(columnIndex, focusItem);
  };

  const focusSplitColumnItem = (
    columnIndex: number,
    fallbackItemIndex?: number,
  ) => {
    const preferredItemIndex = selectedValueIndices?.[columnIndex];
    splitColumnFocusersRef.current.get(columnIndex)?.(
      preferredItemIndex ?? fallbackItemIndex ?? -1,
      fallbackItemIndex,
    );
  };

  return (
    <div className="flex h-full min-h-0 flex-col bg-background">
      {showFilterToggle ? (
        <div className="flex items-center gap-2 px-2 py-1.5">
          <Switch
            checked={mode === "split"}
            onCheckedChange={(checked) =>
              onModeChange(checked ? "split" : "row")
            }
          />
          <span className="text-xs">Split columns</span>
        </div>
      ) : null}

      {mode === "row" && data ? (
        <div
          className="min-h-0 flex-1 overflow-auto border-t bg-background"
          ref={tableContainerRef}
        >
          <Table
            withContainer={false}
            style={{ minWidth: minTableWidth, tableLayout: "fixed" }}
          >
            <TableHeader className="sticky top-0 z-10 border-b bg-background shadow-sm">
              <TableRow>
                {data.fields.map((field) => (
                  <TableHead key={field} className="text-muted-foreground">
                    {field}
                  </TableHead>
                ))}
              </TableRow>
            </TableHeader>
            <TableBody>
              {data.rows.length === 0 ? (
                <TableRow>
                  <TableCell
                    colSpan={data.fields.length}
                    className="h-24 text-center text-muted-foreground"
                  >
                    No data available
                  </TableCell>
                </TableRow>
              ) : (
                <>
                  {virtualPaddingTop > 0 && (
                    <TableRow className="h-0 border-0 hover:bg-transparent">
                      <TableCell
                        colSpan={data.fields.length}
                        style={{ height: `${virtualPaddingTop}px`, padding: 0 }}
                        className="border-0 p-0"
                      />
                    </TableRow>
                  )}
                  {virtualItems.map((virtualRow) => {
                    const row = data.rows[virtualRow.index];
                    if (!row) return null;
                    const isSelected = selectedRowIndex === row.index;
                    return (
                      <TableRow
                        key={row.index}
                        data-state={isSelected && "selected"}
                        ref={(element) =>
                          registerRowElement(row.index, element)
                        }
                        data-index={virtualRow.index}
                        className={focusRowClassName}
                        onClick={(event) => {
                          event.currentTarget.focus();
                          onSelectRow(row.index);
                        }}
                        onKeyDown={(event) =>
                          handleSelectableRowKeyDown(event, {
                            currentIndex: virtualRow.index,
                            totalCount: data.rows.length,
                            onSelect: () => onSelectRow(row.index),
                            onNavigate: (nextIndex) =>
                              selectRowAt(nextIndex, {
                                focus: true,
                                scroll: true,
                              }),
                          })
                        }
                        tabIndex={0}
                      >
                        {data.fields.map((field, idx) => (
                          <TableCell key={field} className="truncate">
                            {row.displayValues[idx]}
                          </TableCell>
                        ))}
                      </TableRow>
                    );
                  })}
                  {virtualPaddingBottom > 0 && (
                    <TableRow className="h-0 border-0 hover:bg-transparent">
                      <TableCell
                        colSpan={data.fields.length}
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
      ) : null}

      {mode === "split" && data ? (
        <div className="flex min-h-0 flex-1 overflow-hidden border-t">
          {data.fields.map((field, index) => (
            <div key={field} className="flex min-h-0 min-w-0 flex-1">
              <FilterTableColumn
                columnIndex={index}
                field={field}
                items={columnUniqueValues[field] ?? []}
                selectedIndex={selectedValueIndices?.[index]}
                onSelect={(selectedIndex) =>
                  onSelectFieldValue(index, selectedIndex)
                }
                onRegisterFocusItem={registerSplitColumnFocuser}
                onFocusColumnItem={focusSplitColumnItem}
              />
              {index < data.fields.length - 1 ? (
                <Separator orientation="vertical" className="bg-border/60" />
              ) : null}
            </div>
          ))}
        </div>
      ) : null}
    </div>
  );
}
