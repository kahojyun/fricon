import type { StructRowProxy, Table, Field } from "apache-arrow";

export interface ColumnValueOption {
  value: unknown;
  displayValue: string;
}

export interface FilterTableData {
  table: Table;
  rows: { row: StructRowProxy; index: number }[];
  fields: Field[];
}

/**
 * Processes the input table to filter out the X column and find unique rows.
 * WARNING: Uses JSON.stringify for row deduplication which can be expensive for large datasets.
 */
export function buildFilterTableData(
  indexTableValue: Table | undefined,
  xColumnName?: string,
): FilterTableData | undefined {
  if (!indexTableValue) return undefined;

  const columnsExceptX = indexTableValue.schema.fields
    .filter((c) => c.name !== xColumnName)
    .map((c) => c.name);

  const filteredTable = indexTableValue.select(columnsExceptX);

  const rows = filteredTable.toArray() as StructRowProxy[];
  const uniqueRowsMap = new Map<
    string,
    { row: StructRowProxy; index: number }
  >();

  rows.forEach((row, i) => {
    const key = JSON.stringify(row);
    if (!uniqueRowsMap.has(key)) {
      uniqueRowsMap.set(key, { row, index: i });
    }
  });

  const uniqueRows = Array.from(uniqueRowsMap.values());

  return {
    table: filteredTable,
    rows: uniqueRows,
    fields: filteredTable.schema.fields,
  };
}

/**
 * Computes unique values for each column in the filtered table data.
 */
export function getColumnUniqueValues(
  filterTableValue: FilterTableData | undefined,
): Record<string, ColumnValueOption[]> {
  if (!filterTableValue) return {};

  const uniqueValues: Record<string, ColumnValueOption[]> = {};

  filterTableValue.fields.forEach((field) => {
    const values = new Set<unknown>();
    filterTableValue.rows.forEach((row) => {
      const value = row.row[field.name] as unknown;
      values.add(value);
    });

    uniqueValues[field.name] = Array.from(values).map((value) => {
      let displayValue = "null";
      if (value !== null && value !== undefined) {
        displayValue =
          typeof value === "object" ? JSON.stringify(value) : value.toString();
      }
      return { value, displayValue };
    });
  });

  return uniqueValues;
}

/**
 * Finds the first row matching the individual column selections.
 */
export function findMatchingRowFromSelections(
  filterTableValue: FilterTableData | undefined,
  selections: Record<string, unknown[]>,
): { row: StructRowProxy; index: number } | null {
  if (!filterTableValue) return null;

  const fieldNames = filterTableValue.fields.map((f) => f.name);
  const selectedValues = fieldNames.map(
    (fieldName) => selections[fieldName] ?? [],
  );

  if (selectedValues.every((values) => values.length === 0)) return null;

  const matchingRows = filterTableValue.rows.filter((row) => {
    return fieldNames.every((fieldName, idx) => {
      const selectedForColumn = selectedValues[idx];
      if (selectedForColumn!.length === 0) return true;
      return selectedForColumn!.includes(row.row[fieldName]);
    });
  });

  if (matchingRows.length > 0) {
    return matchingRows[0]!;
  }

  return null;
}
