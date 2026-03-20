import type {
  ColumnInfo,
  DatasetDeleteResult,
  DatasetDetail as WireDatasetDetail,
  DatasetInfo as WireDatasetInfo,
  DatasetInfoUpdate,
  EmptyTrashResult,
  UiDatasetSortBy as DatasetListSortBy,
  UiDatasetStatus as DatasetStatus,
  UiSortDirection as DatasetListSortDir,
} from "@/shared/lib/bindings";
import { normalizeCreatedAtDate } from "@/shared/lib/tauri";

export type DatasetInfo = Omit<WireDatasetInfo, "createdAt"> & {
  createdAt: Date;
};

export type DatasetDetail = Omit<WireDatasetDetail, "createdAt"> & {
  createdAt: Date;
};

export const DATASET_PAGE_SIZE = 200;

export type {
  ColumnInfo,
  DatasetDeleteResult,
  DatasetInfoUpdate,
  DatasetListSortBy,
  DatasetListSortDir,
  DatasetStatus,
  EmptyTrashResult,
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
  return normalizeCreatedAtDate(value);
}

export function normalizeDatasetDetail(
  value: WireDatasetDetail,
): DatasetDetail {
  return normalizeCreatedAtDate(value);
}
