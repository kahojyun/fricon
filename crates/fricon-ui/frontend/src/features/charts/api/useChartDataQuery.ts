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
        series: options.series,
        xColumn: options.xColumn ?? null,
        yColumn: options.yColumn,
        complexViewSingle: options.complexViewSingle ?? null,
      },
    ] as const;
  }

  const roleOptions = {
    groupByIndexColumns: options.groupByIndexColumns
      ? [...options.groupByIndexColumns].sort()
      : null,
    orderByIndexColumn: options.orderByIndexColumn ?? null,
  };

  switch (options.projection) {
    case "trend":
      return [
        ...chartKeys.chartData(datasetId),
        {
          ...common,
          ...roleOptions,
          drawStyle: options.drawStyle,
          projection: "trend",
          series: options.series,
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
          projection: "xy",
          xColumn: options.xColumn,
          yColumn: options.yColumn,
        },
      ] as const;
    case "complex_xy":
      return [
        ...chartKeys.chartData(datasetId),
        {
          ...common,
          ...roleOptions,
          drawStyle: options.drawStyle,
          projection: "complex_xy",
          series: options.series,
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
