import { useEffect, useMemo, useRef, useState } from "react";
import type {
  ColumnUniqueValue,
  FilterTableData,
  FilterTableRow,
} from "@/lib/backend";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Switch } from "@/components/ui/switch";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { cn } from "@/lib/utils";

interface FilterTableProps {
  value?: FilterTableRow;
  onChange: (value: FilterTableRow | undefined) => void;
  filterTableData?: FilterTableData;
  datasetId: string;
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

  const showFilterToggle = Boolean(
    filterTableData && filterTableData.fields.length > 1,
  );

  const isFilterTableEmpty =
    !filterTableData || filterTableData.rows.length === 0;
  const gridTemplate = useMemo(() => {
    if (!filterTableData) return "none";
    return `repeat(${filterTableData.fields.length}, minmax(120px, 1fr))`;
  }, [filterTableData]);

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
            className="bg-muted text-muted-foreground grid border-b px-2 py-2 text-xs font-semibold"
            style={{ gridTemplateColumns: gridTemplate }}
          >
            {filterTableData.fields.map((field) => (
              <div key={field}>{field}</div>
            ))}
          </div>
          <ScrollArea className="min-h-0 flex-1">
            <div className="min-w-[640px]">
              {filterTableData.rows.map((row) => {
                const isSelected = value?.index === row.index;
                return (
                  <div
                    key={row.index}
                    className={cn(
                      "grid cursor-pointer border-b px-2 py-2 text-xs",
                      isSelected ? "bg-primary/10" : "hover:bg-muted/40",
                    )}
                    style={{ gridTemplateColumns: gridTemplate }}
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
            <div key={field} className="min-w-0 flex-1">
              <Table className="text-xs">
                <TableHeader className="bg-muted text-muted-foreground">
                  <TableRow>
                    <TableHead className="px-2 py-2">{field}</TableHead>
                  </TableRow>
                </TableHeader>
              </Table>
              <ScrollArea className="min-h-0 flex-1">
                <Table className="text-xs">
                  <TableBody>
                    {(columnUniqueValues[field] ?? []).map((item) => {
                      const isSelected =
                        individualColumnSelections[field] === item.index;
                      return (
                        <TableRow
                          key={item.index}
                          className={cn(
                            "cursor-pointer border-b",
                            isSelected ? "bg-primary/10" : "hover:bg-muted/40",
                          )}
                          onClick={() => {
                            setIndividualColumnSelections((prev) => ({
                              ...prev,
                              [field]: item.index,
                            }));
                          }}
                        >
                          <TableCell className="px-2 py-2">
                            {item.displayValue}
                          </TableCell>
                        </TableRow>
                      );
                    })}
                  </TableBody>
                </Table>
              </ScrollArea>
              {index < filterTableData.fields.length - 1 ? (
                <div className="bg-border/60 h-full w-px" />
              ) : null}
            </div>
          ))}
        </div>
      ) : null}
    </div>
  );
}
