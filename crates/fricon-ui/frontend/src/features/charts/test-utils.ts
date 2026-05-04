import type {
  ChartSemanticCapabilities,
  ChartSemanticDescriptor,
  ChartSemantics,
  ColumnInfo,
  DatasetDetail,
  FilterTableData,
} from "./api/types";

type ColumnInput = Partial<ColumnInfo> & { name: string };

export function makeSemanticDescriptor(
  overrides: Partial<ChartSemanticDescriptor> = {},
): ChartSemanticDescriptor {
  return {
    valueKind: "numeric",
    shapeKind: "scalar",
    role: "value",
    ...overrides,
  };
}

export function makeSemanticCapabilities(
  semantic: ChartSemanticDescriptor,
): ChartSemanticCapabilities {
  const isSystem = semantic.role === "system";
  const isDisplay = semantic.role === "display";
  const isScalar = semantic.shapeKind === "scalar";
  const isTrace = semantic.shapeKind === "trace";
  const isNumeric = semantic.valueKind === "numeric";
  const isComplex = semantic.valueKind === "complex";
  const isValue = semantic.role === "value";
  return {
    numericCoordinate: !isSystem && !isDisplay && isScalar && isNumeric,
    filterable: !isSystem && !isDisplay && isScalar,
    groupable: !isSystem && !isDisplay && isScalar,
    traceSource: isTrace,
    complexProjectable: isComplex,
    plottableValue: isValue && (isNumeric || isComplex || isTrace),
  };
}

type DatasetDetailInput = Partial<Omit<DatasetDetail, "columns">> & {
  columns?: ColumnInput[];
};

export function columnId(name: string) {
  return `column:${name}`;
}

export function logicalIndexId(name: string) {
  return `logicalIndex:${name}`;
}

export function makeColumn(overrides: ColumnInput): ColumnInfo {
  const { name, ...rest } = overrides;
  const semantic = rest.semantic ?? makeSemanticDescriptor();
  return {
    name,
    semantic,
    capabilities: rest.capabilities ?? makeSemanticCapabilities(semantic),
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
      semantic: makeSemanticDescriptor({
        ...column.semantic,
        role: "logical_index",
      }),
      capabilities: makeSemanticCapabilities(
        makeSemanticDescriptor({
          ...column.semantic,
          role: "logical_index",
        }),
      ),
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
        semantic: column.semantic,
        capabilities: column.capabilities,
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
