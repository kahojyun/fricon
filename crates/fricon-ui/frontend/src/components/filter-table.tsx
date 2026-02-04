import { useEffect, useMemo, useRef, useState } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import type {
  ColumnUniqueValue,
  FilterTableData,
  FilterTableRow,
} from "@/lib/backend";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Switch } from "@/components/ui/switch";
import { cn } from "@/lib/utils";

interface FilterTableProps {
  value?: FilterTableRow;
  onChange: (value: FilterTableRow | undefined) => void;
  filterTableData?: FilterTableData;
  datasetId: string;
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
  const viewportRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!scrollRootRef.current) return;
    const viewport = scrollRootRef.current.querySelector(
      '[data-slot="scroll-area-viewport"]',
    );
    if (viewport instanceof HTMLDivElement) {
      viewportRef.current = viewport;
    }
  }, []);

  const rowVirtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => viewportRef.current,
    estimateSize: () => 32,
    overscan: 8,
  });

  return (
    <div className="min-w-0 flex-1">
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
  value,
  onChange,
  filterTableData,
  datasetId,
}: FilterTableProps) {
  const [isIndividualFilterMode, setIsIndividualFilterMode] = useState(false);
  const [individualColumnSelections, setIndividualColumnSelections] = useState<
    Record<string, number | undefined>
  >({});
  const previousDatasetId = useRef<string | undefined>(undefined);
  const headerScrollRef = useRef<HTMLDivElement | null>(null);
  const bodyScrollRootRef = useRef<HTMLDivElement | null>(null);
  const bodyViewportRef = useRef<HTMLDivElement | null>(null);

  const showFilterToggle = Boolean(
    filterTableData && filterTableData.fields.length > 1,
  );

  const isFilterTableEmpty =
    !filterTableData || filterTableData.rows.length === 0;
  const gridTemplate = useMemo(() => {
    if (!filterTableData) return "none";
    return `repeat(${filterTableData.fields.length}, minmax(80px, 1fr))`;
  }, [filterTableData]);
  const minTableWidth = useMemo(() => {
    if (!filterTableData) return "0px";
    const minWidth = Math.max(filterTableData.fields.length * 80, 320);
    return `${minWidth}px`;
  }, [filterTableData]);

  const rowVirtualizer = useVirtualizer({
    count: filterTableData?.rows.length ?? 0,
    getScrollElement: () => bodyViewportRef.current,
    estimateSize: () => 32,
    overscan: 8,
  });

  useEffect(() => {
    if (isIndividualFilterMode) return;
    if (!bodyScrollRootRef.current) return;
    const viewport = bodyScrollRootRef.current.querySelector(
      '[data-slot="scroll-area-viewport"]',
    );
    if (!(viewport instanceof HTMLDivElement)) return;
    bodyViewportRef.current = viewport;
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
  }, [filterTableData?.fields.length, isIndividualFilterMode]);

  const columnUniqueValues = useMemo<
    Record<string, ColumnUniqueValue[]>
  >(() => {
    if (!isIndividualFilterMode || !filterTableData) return {};
    return filterTableData.columnUniqueValues;
  }, [filterTableData, isIndividualFilterMode]);

  const syncIndividualSelectionsFromModel = () => {
    if (!filterTableData || !value) return;
    const fieldNames = filterTableData.fields;
    const next: Record<string, number | undefined> = {};
    fieldNames.forEach((fieldName, idx) => {
      next[fieldName] = value.valueIndices[idx];
    });
    setIndividualColumnSelections(next);
  };

  const findMatchingRowFromSelections = (
    data: FilterTableData,
    selections: Record<string, number | undefined>,
  ): FilterTableRow | null => {
    const fieldNames = data.fields;
    if (Object.keys(selections).length === 0) return null;
    const matching = data.rows.filter((row) =>
      fieldNames.every((fieldName, idx) => {
        const selectionIndex = selections[fieldName];
        if (selectionIndex === undefined) return true;
        return row.valueIndices[idx] === selectionIndex;
      }),
    );
    return matching.length > 0 ? matching[0] : null;
  };

  useEffect(() => {
    if (!isIndividualFilterMode && value) {
      syncIndividualSelectionsFromModel();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [value, isIndividualFilterMode]);

  useEffect(() => {
    const datasetChanged = previousDatasetId.current !== datasetId;

    if (!filterTableData || filterTableData.rows.length === 0) {
      onChange(undefined);
      if (datasetChanged) {
        setIndividualColumnSelections({});
      }
      previousDatasetId.current = datasetId;
      return;
    }

    if (datasetChanged) {
      onChange(filterTableData.rows[0]);
      setIndividualColumnSelections({});
    } else {
      if (value) {
        const preservedRow = filterTableData.rows.find(
          (row) => row.index === value.index,
        );
        if (preservedRow) {
          onChange(preservedRow);
        } else {
          onChange(filterTableData.rows[0]);
          setIndividualColumnSelections({});
        }
      } else {
        onChange(filterTableData.rows[0]);
      }
    }

    previousDatasetId.current = datasetId;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [datasetId, filterTableData]);

  useEffect(() => {
    if (!filterTableData) return;
    if (isIndividualFilterMode) {
      const row = findMatchingRowFromSelections(
        filterTableData,
        individualColumnSelections,
      );
      if (row) {
        onChange(row);
      } else {
        onChange(filterTableData.rows[0]);
      }
    } else {
      if (
        !value ||
        !filterTableData.rows.some((row) => row.index === value.index)
      ) {
        onChange(filterTableData.rows[0]);
      }
    }
  }, [
    filterTableData,
    individualColumnSelections,
    isIndividualFilterMode,
    onChange,
    value,
  ]);

  if (!filterTableData || filterTableData.rows.length === 0) {
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
            checked={isIndividualFilterMode}
            onCheckedChange={setIsIndividualFilterMode}
          />
          <span className="text-sm">Split columns</span>
        </div>
      ) : null}

      {!isIndividualFilterMode && isFilterTableEmpty ? (
        <div className="text-muted-foreground flex h-full items-center justify-center text-sm">
          No data available
        </div>
      ) : null}

      {!isIndividualFilterMode ? (
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
              {filterTableData.fields.map((field) => (
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
                const row = filterTableData.rows[virtualRow.index];
                if (!row) return null;
                const isSelected = value?.index === row.index;
                return (
                  <div
                    key={row.index}
                    ref={rowVirtualizer.measureElement}
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
                    onClick={() => onChange(row)}
                  >
                    {filterTableData.fields.map((field, idx) => (
                      <div key={field}>{row.displayValues[idx]}</div>
                    ))}
                  </div>
                );
              })}
            </div>
          </ScrollArea>
        </>
      ) : null}

      {isIndividualFilterMode && isFilterTableEmpty ? (
        <div className="text-muted-foreground flex h-full items-center justify-center text-sm">
          No data available
        </div>
      ) : null}

      {isIndividualFilterMode && filterTableData ? (
        <div className="flex min-h-0 flex-1 overflow-hidden">
          {filterTableData.fields.map((field, index) => (
            <div key={field} className="flex min-w-0 flex-1">
              <FilterTableColumn
                field={field}
                items={columnUniqueValues[field] ?? []}
                selectedIndex={individualColumnSelections[field]}
                onSelect={(selectedIndex) =>
                  setIndividualColumnSelections((prev) => ({
                    ...prev,
                    [field]: selectedIndex,
                  }))
                }
              />
              {index < filterTableData.fields.length - 1 ? (
                <div className="bg-border/60 w-px shrink-0" />
              ) : null}
            </div>
          ))}
        </div>
      ) : null}
    </div>
  );
}
