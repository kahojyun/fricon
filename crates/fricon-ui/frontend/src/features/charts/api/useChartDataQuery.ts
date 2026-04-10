import { useQuery } from "@tanstack/react-query";
import { fetchChartData } from "./client";
import { chartKeys } from "./queryKeys";
import { type ChartDataOptions } from "./types";

function buildChartDataKey(
  datasetId: number,
  options: ChartDataOptions | null,
) {
  if (!options) {
    return [...chartKeys.chartData(datasetId), { disabled: true }] as const;
  }

  const common = {
    view: options.view,
    indexFilters: options.indexFilters ?? null,
    excludeColumns: options.excludeColumns
      ? [...options.excludeColumns].sort()
      : null,
  };

  if (options.view === "heatmap") {
    return [
      ...chartKeys.chartData(datasetId),
      {
        ...common,
        quantity: options.quantity,
        xColumn: options.xColumn ?? null,
        yColumn: options.yColumn,
        complexViewSingle: options.complexViewSingle ?? null,
      },
    ] as const;
  }

  const roleOptions = {
    traceGroupIndexColumns: options.traceGroupIndexColumns
      ? [...options.traceGroupIndexColumns].sort()
      : null,
    sweepIndexColumn: options.sweepIndexColumn ?? null,
  };

  switch (options.plotMode) {
    case "quantity_vs_sweep":
      return [
        ...chartKeys.chartData(datasetId),
        {
          ...common,
          ...roleOptions,
          drawStyle: options.drawStyle,
          plotMode: "quantity_vs_sweep",
          quantity: options.quantity,
          complexViews: options.complexViews
            ? [...options.complexViews].sort()
            : null,
        },
      ] as const;
    case "xy":
      return [
        ...chartKeys.chartData(datasetId),
        {
          ...common,
          ...roleOptions,
          drawStyle: options.drawStyle,
          plotMode: "xy",
          xColumn: options.xColumn,
          yColumn: options.yColumn,
        },
      ] as const;
    case "complex_plane":
      return [
        ...chartKeys.chartData(datasetId),
        {
          ...common,
          ...roleOptions,
          drawStyle: options.drawStyle,
          plotMode: "complex_plane",
          quantity: options.quantity,
        },
      ] as const;
  }
}

export function useChartDataQuery(
  datasetId: number,
  options: ChartDataOptions | null,
) {
  return useQuery({
    queryKey: buildChartDataKey(datasetId, options),
    queryFn: () => fetchChartData(datasetId, options!),
    enabled: Boolean(options),
  });
}
