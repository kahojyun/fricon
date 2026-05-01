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
  "createdAt" | "trashedAt" | "deletedAt" | "columns"
> & {
  createdAt: Date;
  trashedAt: Date | null;
  deletedAt: Date | null;
  columns: DatasetColumnInfo[];
};

export const DATASET_PAGE_SIZE = 200;

export interface DatasetColumnInfo {
  name: string;
  label: string | null;
  unit: string | null;
  isComplex: boolean;
  isTrace: boolean;
  isIndex: boolean;
  hiddenByDefault: boolean;
  isChartAxisCandidate: boolean;
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
  };
}

function normalizeDatasetColumnInfo(value: WireColumnInfo): DatasetColumnInfo {
  return {
    name: value.name,
    label: value.label,
    unit: value.unit,
    isComplex: value.isComplex,
    isTrace: value.isTrace,
    isIndex: value.isIndex,
    hiddenByDefault: value.hiddenByDefault,
    isChartAxisCandidate: value.isChartAxisCandidate,
  };
}
