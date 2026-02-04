import { useEffect, useMemo, useRef, useState } from "react";
import type {
  ColumnUniqueValue,
  FilterTableData,
  FilterTableRow,
} from "@/lib/backend";
import { Switch } from "@/components/ui/switch";
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
        <div className="min-h-0 flex-1 overflow-auto">
          <table className="w-full text-xs">
            <thead className="bg-muted/40 text-muted-foreground sticky top-0">
              <tr>
                {filterTableData.fields.map((field) => (
                  <th key={field} className="px-2 py-2 text-left font-semibold">
                    {field}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {filterTableData.rows.map((row) => {
                const isSelected = value?.index === row.index;
                return (
                  <tr
                    key={row.index}
                    className={cn(
                      "cursor-pointer border-b",
                      isSelected ? "bg-primary/10" : "hover:bg-muted/40",
                    )}
                    onClick={() => onChange(row)}
                  >
                    {filterTableData.fields.map((field, idx) => (
                      <td key={field} className="px-2 py-2">
                        {row.displayValues[idx]}
                      </td>
                    ))}
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
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
              <div className="bg-muted/40 text-muted-foreground sticky top-0 px-2 py-2 text-xs font-semibold">
                {field}
              </div>
              <div className="min-h-0 flex-1 overflow-auto">
                <table className="w-full text-xs">
                  <tbody>
                    {(columnUniqueValues[field] ?? []).map((item) => {
                      const isSelected =
                        individualColumnSelections[field] === item.index;
                      return (
                        <tr
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
                          <td className="px-2 py-2">{item.displayValue}</td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>
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
