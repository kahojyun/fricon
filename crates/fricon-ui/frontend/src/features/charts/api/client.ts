import {
  type DatasetWriteStatus,
  type TableData as WireFilterTableData,
  commands,
} from "@/shared/lib/bindings";
import { invoke, invokeRawBytes } from "@/shared/lib/tauri";
import {
  normalizeFilterTableData,
  toWireChartOptions,
  toWireLiveChartOptions,
  type ChartDataOptions,
  type FilterTableData,
  type FilterTableOptions,
  type LiveChartDataOptions,
} from "./types";
import { decodeChartSnapshot, decodeLiveChartUpdate } from "./wire";

export async function fetchChartData(id: number, options: ChartDataOptions) {
  return decodeChartSnapshot(
    await invokeRawBytes("dataset_chart_data", {
      id,
      options: toWireChartOptions(options),
    }),
  );
}

export async function fetchLiveChartData(
  id: number,
  options: LiveChartDataOptions,
  knownRowCount: number | null,
) {
  return decodeLiveChartUpdate(
    await invokeRawBytes("dataset_live_chart_data", {
      id,
      options: toWireLiveChartOptions({
        ...options,
        knownRowCount,
      }),
    }),
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
