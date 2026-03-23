import type {
  ColumnInfo,
  DatasetDeleteResult,
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
  "createdAt" | "trashedAt" | "deletedAt"
> & {
  createdAt: Date;
  trashedAt: Date | null;
  deletedAt: Date | null;
};

export const DATASET_PAGE_SIZE = 200;

export type {
  ColumnInfo,
  DatasetDeleteResult,
  DatasetInfoUpdate,
  DatasetListSortBy,
  DatasetListSortDir,
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
  return normalizeDatasetDates(value);
}
