import { useMemo, useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { ColumnUniqueValue, FilterTableData } from "@/lib/backend";
import { Switch } from "@/components/ui/switch";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import type { CascadeMode } from "@/hooks/cascadeReducer";

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
  field: string;
  items: ColumnUniqueValue[];
  selectedIndex?: number;
  onSelect: (index: number) => void;
}

function FilterTableColumn({
  field,
  items,
  selectedIndex,
  onSelect,
}: FilterTableColumnProps) {
  const scrollRef = useRef<HTMLDivElement>(null);

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
                  ref={rowVirtualizer.measureElement}
                  data-index={virtualRow.index}
                  className="cursor-pointer"
                  onClick={() => onSelect(item.index)}
                >
                  <TableCell className="overflow-hidden text-ellipsis">
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

  const showFilterToggle = Boolean(data && data.fields.length > 1);
  const isFilterTableEmpty = !data || data.rows.length === 0;

  const minTableWidth = useMemo(() => {
    if (!data) return "0px";
    const minWidth = Math.max(data.fields.length * 80, 320);
    return `${minWidth}px`;
  }, [data]);

  const rowVirtualizer = useVirtualizer({
    count: data?.rows.length ?? 0,
    getScrollElement: () => tableContainerRef.current,
    estimateSize: () => 28,
    overscan: 8,
  });

  const virtualItems = rowVirtualizer.getVirtualItems();

  const columnUniqueValues: Record<string, ColumnUniqueValue[]> =
    mode === "split" && data ? data.columnUniqueValues : {};

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
                        ref={rowVirtualizer.measureElement}
                        data-index={virtualRow.index}
                        className="cursor-pointer"
                        onClick={() => onSelectRow(row.index)}
                        onKeyDown={(event) => {
                          if (event.key === "Enter" || event.key === " ") {
                            onSelectRow(row.index);
                          }
                          if (event.metaKey || event.ctrlKey) {
                            event.stopPropagation();
                          }
                        }}
                        tabIndex={0}
                      >
                        {data.fields.map((field, idx) => (
                          <TableCell
                            key={field}
                            className="overflow-hidden text-ellipsis"
                          >
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
                field={field}
                items={columnUniqueValues[field] ?? []}
                selectedIndex={selectedValueIndices?.[index]}
                onSelect={(selectedIndex) =>
                  onSelectFieldValue(index, selectedIndex)
                }
              />
              {index < data.fields.length - 1 ? (
                <div className="w-px shrink-0 bg-border/60" />
              ) : null}
            </div>
          ))}
        </div>
      ) : null}
    </div>
  );
}
