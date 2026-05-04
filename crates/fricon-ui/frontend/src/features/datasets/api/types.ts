import type {
  ColumnInfo as WireColumnInfo,
  DatasetDeleteResult,
  DatasetOperationError,
  DatasetTagBatchResult,
  DatasetDetail as WireDatasetDetail,
  DatasetInfo as WireDatasetInfo,
  DatasetInfoUpdate,
  UiDatasetSortBy as DatasetListSortBy,
  UiDatasetStatus as DatasetStatus,
  UiSortDirection as DatasetListSortDir,
  UiPreviewImportResult,
  UiImportPreview,
  UiImportConflict,
  UiFieldDiff,
  UiExportedMetadata,
} from "@/shared/lib/bindings";
import { normalizeDatasetDates } from "@/shared/lib/tauri";

export type DatasetInfo = Omit<
  WireDatasetInfo,
  "createdAt" | "trashedAt" | "deletedAt"
> & {
  createdAt: Date;
  trashedAt: Date | null;
  deletedAt: Date | null;
};

export type DatasetDetail = Omit<
  WireDatasetDetail,
  "createdAt" | "trashedAt" | "deletedAt" | "columns" | "chartSemantics"
> & {
  createdAt: Date;
  trashedAt: Date | null;
  deletedAt: Date | null;
  columns: DatasetColumnInfo[];
  chartSemantics: ChartSemantics | null;
};

export const DATASET_PAGE_SIZE = 200;

export interface DatasetColumnInfo {
  name: string;
  label: string | null;
  unit: string | null;
  semantic: ChartSemanticDescriptor;
  capabilities: ChartSemanticCapabilities;
  isInferredAxis: boolean;
  hiddenByDefault: boolean;
  isChartAxisCandidate: boolean;
}

export type ChartDuplicatePolicy =
  | "latest_by_record_id"
  | "row_order_placeholder";
export type ChartIndexRealization = "none" | "implicit" | "sidecar";
export type ChartSemanticAxisKind = "logical_index" | "column";
export type ChartSemanticValueKind =
  | "numeric"
  | "categorical"
  | "boolean"
  | "timestamp"
  | "complex"
  | "display";
export type ChartSemanticShapeKind = "scalar" | "trace";
export type ChartSemanticRole =
  | "value"
  | "logical_index"
  | "system"
  | "display";

export interface ChartSemanticDescriptor {
  valueKind: ChartSemanticValueKind;
  shapeKind: ChartSemanticShapeKind;
  role: ChartSemanticRole;
}

export interface ChartSemanticCapabilities {
  numericCoordinate: boolean;
  filterable: boolean;
  groupable: boolean;
  traceSource: boolean;
  complexProjectable: boolean;
  plottableValue: boolean;
}

export interface ChartSemanticColumn {
  id: string;
  name: string;
  label: string | null;
  semantic: ChartSemanticDescriptor;
  capabilities: ChartSemanticCapabilities;
  hiddenByDefault: boolean;
}

export interface ChartSemanticAxis {
  id: string;
  name: string;
  label: string | null;
  kind: ChartSemanticAxisKind;
  semantic: ChartSemanticDescriptor;
  capabilities: ChartSemanticCapabilities;
  isInferredAxis: boolean;
  physicalColumn: string | null;
}

export interface ChartSemantics {
  duplicatePolicy: ChartDuplicatePolicy;
  indexRealization: ChartIndexRealization;
  axes: ChartSemanticAxis[];
  valueColumns: ChartSemanticColumn[];
  chartAxisCandidates: ChartSemanticAxis[];
}

export type ColumnInfo = DatasetColumnInfo;

export type {
  DatasetDeleteResult,
  DatasetOperationError,
  DatasetInfoUpdate,
  DatasetListSortBy,
  DatasetListSortDir,
  DatasetTagBatchResult,
  DatasetStatus,
  UiPreviewImportResult,
  UiImportPreview,
  UiImportConflict,
  UiFieldDiff,
  UiExportedMetadata,
};

export type DatasetViewMode = "active" | "trash";

export interface ListDatasetsOptions {
  search?: string;
  tags?: string[];
  favoriteOnly?: boolean;
  statuses?: DatasetStatus[];
  trashed?: boolean;
  sortBy?: DatasetListSortBy;
  sortDir?: DatasetListSortDir;
  limit?: number;
  offset?: number;
}

export function normalizeDataset(value: WireDatasetInfo): DatasetInfo {
  return normalizeDatasetDates(value);
}

export function normalizeDatasetDetail(
  value: WireDatasetDetail,
): DatasetDetail {
  const normalized = normalizeDatasetDates(value);
  return {
    ...normalized,
    columns: value.columns.map(normalizeDatasetColumnInfo),
    chartSemantics: value.chartSemantics ?? null,
  };
}

function normalizeDatasetColumnInfo(value: WireColumnInfo): DatasetColumnInfo {
  return {
    name: value.name,
    label: value.label,
    unit: value.unit,
    semantic: value.semantic,
    capabilities: value.capabilities,
    isInferredAxis: value.isInferredAxis,
    hiddenByDefault: value.hiddenByDefault,
    isChartAxisCandidate: value.isChartAxisCandidate,
  };
}
