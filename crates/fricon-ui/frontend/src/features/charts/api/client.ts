import {
  commands,
  type DatasetWriteStatus,
  type TableData as WireFilterTableData,
} from "@/shared/lib/bindings";
import { invoke } from "@/shared/lib/tauri";
import {
  normalizeChartSnapshot,
  normalizeFilterTableData,
  normalizeLiveChartUpdate,
  toWireChartOptions,
  toWireLiveChartOptions,
  type ChartDataOptions,
  type FilterTableData,
  type FilterTableOptions,
  type LiveChartDataOptions,
} from "./types";

export async function fetchChartData(id: number, options: ChartDataOptions) {
  return normalizeChartSnapshot(
    await invoke(commands.datasetChartData(id, toWireChartOptions(options))),
  );
}

export async function fetchLiveChartData(
  id: number,
  options: LiveChartDataOptions,
  knownRowCount: number | null,
) {
  return normalizeLiveChartUpdate(
    await invoke(
      commands.datasetLiveChartData(
        id,
        toWireLiveChartOptions({
          ...options,
          knownRowCount,
        }),
      ),
    ),
  );
}

export async function getFilterTableData(
  id: number,
  options: FilterTableOptions,
): Promise<FilterTableData> {
  const result: WireFilterTableData = await invoke(
    commands.getFilterTableData(id, options),
  );
  return normalizeFilterTableData(result);
}

export async function getDatasetWriteStatus(
  id: number,
): Promise<DatasetWriteStatus> {
  return invoke(commands.getDatasetWriteStatus(id));
}
