import type {
  ChartSemantics,
  ColumnInfo,
  DatasetDetail,
  FilterTableData,
} from "./api/types";

type ColumnInput = Partial<ColumnInfo> & { name: string };

type DatasetDetailInput = Partial<Omit<DatasetDetail, "columns">> & {
  columns?: ColumnInput[];
};

export function columnId(name: string) {
  return `column:${name}`;
}

export function logicalIndexId(name: string) {
  return `logicalIndex:${name}`;
}

export function makeColumn(
  overrides: ColumnInput,
): ColumnInfo {
  const { name, ...rest } = overrides;
  const semanticKind =
    rest.semanticKind ??
    (rest.isTrace ? "trace" : rest.isComplex ? "complex" : "numeric");
  return {
    name,
    semanticKind,
    isComplex: false,
    isTrace: false,
    isInferredAxis: false,
    ...rest,
  };
}

export function makeDatasetDetail(
  input: DatasetDetailInput | ColumnInput[] = {},
): DatasetDetail {
  const overrides = Array.isArray(input) ? { columns: input } : input;
  const columns = (overrides.columns ?? []).map(makeColumn);
  return {
    status: "Completed",
    payloadAvailable: true,
    ...overrides,
    columns,
    chartSemantics:
      overrides.chartSemantics === undefined
        ? makeInferredSemantics(columns)
        : overrides.chartSemantics,
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
      semanticKind: column.semanticKind,
      numeric: column.semanticKind === "numeric",
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
        semanticKind: column.semanticKind,
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
