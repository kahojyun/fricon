import type {
  ChartSemantics,
  ColumnInfo,
  DatasetDetail,
  FilterTableData,
} from "./api/types";

export function columnId(name: string) {
  return `column:${name}`;
}

export function logicalIndexId(name: string) {
  return `logicalIndex:${name}`;
}

export function makeColumn(
  overrides: Partial<ColumnInfo> & { name: string },
): ColumnInfo {
  const { name, ...rest } = overrides;
  return {
    name,
    isComplex: false,
    isTrace: false,
    isInferredAxis: false,
    ...rest,
  };
}

export function makeDatasetDetail(
  input: Partial<DatasetDetail> | ColumnInfo[] = {},
): DatasetDetail {
  const overrides = Array.isArray(input) ? { columns: input } : input;
  const columns = overrides.columns ?? [];
  return {
    status: "Completed",
    payloadAvailable: true,
    columns,
    chartSemantics: makeInferredSemantics(columns),
    ...overrides,
  };
}

export function makeInferredSemantics(columns: ColumnInfo[]): ChartSemantics {
  const axes = columns
    .filter((column) => column.isInferredAxis)
    .map((column) => ({
      id: columnId(column.name),
      name: column.name,
      label: column.label ?? null,
      kind: "column" as const,
      numeric: true,
      isInferredAxis: true,
      physicalColumn: column.name,
    }));
  return {
    duplicatePolicy:
      axes.length > 0 ? "row_order_placeholder" : "latest_by_record_id",
    indexRealization: "none",
    axes,
    valueColumns: columns
      .filter((column) => !column.isInferredAxis)
      .map((column) => ({
        id: columnId(column.name),
        name: column.name,
        label: column.label ?? null,
        isComplex: column.isComplex,
        isTrace: column.isTrace,
        hiddenByDefault: column.hiddenByDefault ?? false,
      })),
    chartAxisCandidates: axes,
  };
}

export function makeFilterTableData(
  overrides: Partial<FilterTableData> = {},
): FilterTableData {
  return {
    fields: [],
    fieldLabels: {},
    rows: [],
    columnUniqueValues: {},
    ...overrides,
  };
}
