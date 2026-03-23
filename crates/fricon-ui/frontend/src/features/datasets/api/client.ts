import {
  commands,
  type DatasetDeleteResult,
  type DatasetDetail as WireDatasetDetail,
  type DatasetFavoriteUpdate,
  type DatasetInfo as WireDatasetInfo,
  type DatasetListOptions as WireDatasetListOptions,
} from "@/shared/lib/bindings";
import { invoke } from "@/shared/lib/tauri";
import {
  normalizeDataset,
  normalizeDatasetDetail,
  type DatasetDetail,
  type DatasetInfo,
  type DatasetInfoUpdate,
  type ListDatasetsOptions,
  type UiPreviewImportResult,
} from "./types";

export async function listDatasets(
  options: ListDatasetsOptions = {},
): Promise<DatasetInfo[]> {
  const {
    search,
    tags,
    favoriteOnly,
    statuses,
    trashed,
    sortBy,
    sortDir,
    limit,
    offset,
  } = options;
  const wireOptions: WireDatasetListOptions = {
    search: search?.trim(),
    tags: tags && tags.length > 0 ? tags : undefined,
    favoriteOnly: favoriteOnly ? true : undefined,
    statuses: statuses && statuses.length > 0 ? statuses : undefined,
    trashed,
    sortBy,
    sortDir,
    limit,
    offset,
  };
  const datasets: WireDatasetInfo[] = await invoke(
    commands.listDatasets(wireOptions),
  );
  return datasets.map(normalizeDataset);
}

export async function listDatasetTags(): Promise<string[]> {
  return invoke(commands.listDatasetTags());
}

export async function updateDatasetFavorite(
  id: number,
  favorite: boolean,
): Promise<void> {
  const update: DatasetFavoriteUpdate = { favorite };
  await invoke(commands.updateDatasetFavorite(id, update));
}

export async function updateDatasetInfo(
  id: number,
  update: DatasetInfoUpdate,
): Promise<void> {
  await invoke(commands.updateDatasetInfo(id, update));
}

export async function getDatasetDetail(id: number): Promise<DatasetDetail> {
  const rawDetail: WireDatasetDetail = await invoke(commands.datasetDetail(id));
  return normalizeDatasetDetail(rawDetail);
}

export async function deleteDatasets(
  ids: number[],
): Promise<DatasetDeleteResult[]> {
  return invoke(commands.deleteDatasets(ids));
}

export async function trashDatasets(
  ids: number[],
): Promise<DatasetDeleteResult[]> {
  return invoke(commands.trashDatasets(ids));
}

export async function restoreDatasets(
  ids: number[],
): Promise<DatasetDeleteResult[]> {
  return invoke(commands.restoreDatasets(ids));
}

export async function emptyTrash(): Promise<DatasetDeleteResult[]> {
  return invoke(commands.emptyTrash());
}

export async function batchUpdateDatasetTags(
  ids: number[],
  add: string[] = [],
  remove: string[] = [],
): Promise<DatasetDeleteResult[]> {
  return invoke(commands.batchUpdateDatasetTags({ ids, add, remove }));
}

export async function deleteTag(tag: string): Promise<void> {
  await invoke(commands.deleteTag(tag));
}

export async function renameTag(
  oldName: string,
  newName: string,
): Promise<void> {
  await invoke(commands.renameTag(oldName, newName));
}

export async function mergeTag(source: string, target: string): Promise<void> {
  await invoke(commands.mergeTag(source, target));
}

export async function exportDatasetsDialog(
  ids: number[],
): Promise<string[] | null> {
  return invoke(commands.exportDatasetsDialog(ids));
}

export async function previewImportDialog(): Promise<
  UiPreviewImportResult[] | null
> {
  return invoke(commands.previewImportDialog());
}

export async function previewImportFiles(
  paths: string[],
): Promise<UiPreviewImportResult[]> {
  return invoke(commands.previewImportFiles(paths));
}

export async function importDataset(
  archivePath: string,
  force: boolean,
): Promise<void> {
  await invoke(commands.importDataset(archivePath, force));
}
