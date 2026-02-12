import { useEffect, useMemo, useRef, useState } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { ColumnUniqueValue, FilterTableData } from "@/lib/backend";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Switch } from "@/components/ui/switch";
import { cn } from "@/lib/utils";
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
  const scrollRootRef = useRef<HTMLDivElement | null>(null);
  const [viewportElement, setViewportElement] = useState<HTMLDivElement | null>(
    null,
  );

  useEffect(() => {
    if (!scrollRootRef.current) return;
    const viewport = scrollRootRef.current.querySelector(
      '[data-slot="scroll-area-viewport"]',
    );
    if (viewport instanceof HTMLDivElement) {
      setViewportElement(viewport);
    }
  }, []);

  const rowVirtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => viewportElement,
    estimateSize: () => 32,
    overscan: 8,
  });

  return (
    <div
      data-testid={`filter-column-${field}`}
      className="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden"
    >
      <div className="bg-muted text-muted-foreground border-b px-2 py-2 text-xs font-semibold">
        {field}
      </div>
      <ScrollArea ref={scrollRootRef} className="min-h-0 flex-1">
        <div
          className="relative"
          style={{ height: rowVirtualizer.getTotalSize() }}
        >
          {rowVirtualizer.getVirtualItems().map((virtualRow) => {
            const item = items[virtualRow.index];
            if (!item) return null;
            const isSelected = selectedIndex === item.index;
            return (
              <div
                key={item.index}
                ref={rowVirtualizer.measureElement}
                data-index={virtualRow.index}
                className={cn(
                  "cursor-pointer border-b px-2 py-2 text-xs",
                  isSelected ? "bg-primary/10" : "hover:bg-muted/40",
                )}
                style={{
                  position: "absolute",
                  top: 0,
                  left: 0,
                  width: "100%",
                  transform: `translateY(${virtualRow.start}px)`,
                }}
                onClick={() => onSelect(item.index)}
              >
                {item.displayValue}
              </div>
            );
          })}
        </div>
      </ScrollArea>
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
  const headerScrollRef = useRef<HTMLDivElement | null>(null);
  const bodyScrollRootRef = useRef<HTMLDivElement | null>(null);
  const [bodyViewportElement, setBodyViewportElement] =
    useState<HTMLDivElement | null>(null);

  const showFilterToggle = Boolean(data && data.fields.length > 1);

  const isFilterTableEmpty = !data || data.rows.length === 0;
  const gridTemplate = useMemo(() => {
    if (!data) return "none";
    return `repeat(${data.fields.length}, minmax(80px, 1fr))`;
  }, [data]);
  const minTableWidth = useMemo(() => {
    if (!data) return "0px";
    const minWidth = Math.max(data.fields.length * 80, 320);
    return `${minWidth}px`;
  }, [data]);

  const rowVirtualizer = useVirtualizer({
    count: data?.rows.length ?? 0,
    getScrollElement: () => bodyViewportElement,
    estimateSize: () => 32,
    overscan: 8,
  });

  useEffect(() => {
    if (mode === "split") return;
    if (!bodyScrollRootRef.current) return;
    const viewport = bodyScrollRootRef.current.querySelector(
      '[data-slot="scroll-area-viewport"]',
    );
    if (!(viewport instanceof HTMLDivElement)) return;
    setBodyViewportElement(viewport);
    const handleScroll = () => {
      if (headerScrollRef.current) {
        headerScrollRef.current.scrollLeft = viewport.scrollLeft;
      }
    };
    handleScroll();
    viewport.addEventListener("scroll", handleScroll, { passive: true });
    return () => {
      viewport.removeEventListener("scroll", handleScroll);
    };
  }, [data?.fields.length, mode]);

  const columnUniqueValues: Record<string, ColumnUniqueValue[]> =
    mode === "split" && data ? data.columnUniqueValues : {};

  if (!data || data.rows.length === 0) {
    return (
      <div className="text-muted-foreground flex h-full items-center justify-center text-sm">
        No data available
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col">
      {showFilterToggle ? (
        <div className="flex items-center gap-2 px-2 pb-2">
          <Switch
            checked={mode === "split"}
            onCheckedChange={(checked) =>
              onModeChange(checked ? "split" : "row")
            }
          />
          <span className="text-sm">Split columns</span>
        </div>
      ) : null}

      {mode === "row" && isFilterTableEmpty ? (
        <div className="text-muted-foreground flex h-full items-center justify-center text-sm">
          No data available
        </div>
      ) : null}

      {mode === "row" ? (
        <>
          <div
            ref={headerScrollRef}
            className="bg-muted overflow-hidden border-b"
          >
            <div
              className="text-muted-foreground grid px-2 py-2 text-xs font-semibold"
              style={{
                gridTemplateColumns: gridTemplate,
                minWidth: minTableWidth,
              }}
            >
              {data.fields.map((field) => (
                <div key={field}>{field}</div>
              ))}
            </div>
          </div>
          <ScrollArea ref={bodyScrollRootRef} className="min-h-0 flex-1">
            <div
              className="relative"
              style={{
                minWidth: minTableWidth,
                height: rowVirtualizer.getTotalSize(),
              }}
            >
              {rowVirtualizer.getVirtualItems().map((virtualRow) => {
                const row = data.rows[virtualRow.index];
                if (!row) return null;
                const isSelected = selectedRowIndex === row.index;
                return (
                  <div
                    key={row.index}
                    ref={rowVirtualizer.measureElement}
                    data-index={virtualRow.index}
                    className={cn(
                      "grid cursor-pointer border-b px-2 py-2 text-xs",
                      isSelected ? "bg-primary/10" : "hover:bg-muted/40",
                    )}
                    style={{
                      gridTemplateColumns: gridTemplate,
                      position: "absolute",
                      top: 0,
                      left: 0,
                      width: "100%",
                      transform: `translateY(${virtualRow.start}px)`,
                    }}
                    onClick={() => onSelectRow(row.index)}
                  >
                    {data.fields.map((field, idx) => (
                      <div key={field}>{row.displayValues[idx]}</div>
                    ))}
                  </div>
                );
              })}
            </div>
          </ScrollArea>
        </>
      ) : null}

      {mode === "split" && isFilterTableEmpty ? (
        <div className="text-muted-foreground flex h-full items-center justify-center text-sm">
          No data available
        </div>
      ) : null}

      {mode === "split" && data ? (
        <div className="flex min-h-0 flex-1 overflow-hidden">
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
                <div className="bg-border/60 w-px shrink-0" />
              ) : null}
            </div>
          ))}
        </div>
      ) : null}
    </div>
  );
}
