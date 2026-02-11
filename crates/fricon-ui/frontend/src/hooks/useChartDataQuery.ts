import { useQuery } from "@tanstack/react-query";
import { fetchChartData, type ChartDataOptions } from "@/lib/backend";

function buildChartDataKey(
  datasetId: number,
  options: ChartDataOptions | null,
) {
  if (!options) {
    return ["chartData", datasetId, { disabled: true }] as const;
  }

  const common = {
    chartType: options.chartType,
    indexFilters: options.indexFilters ?? null,
    excludeColumns: options.excludeColumns
      ? [...options.excludeColumns].sort()
      : null,
  };

  if (options.chartType === "line") {
    return [
      "chartData",
      datasetId,
      {
        ...common,
        series: options.series,
        xColumn: options.xColumn ?? null,
        complexViews: options.complexViews ? [...options.complexViews].sort() : null,
      },
    ] as const;
  }

  if (options.chartType === "heatmap") {
    return [
      "chartData",
      datasetId,
      {
        ...common,
        series: options.series,
        xColumn: options.xColumn ?? null,
        yColumn: options.yColumn,
        complexViewSingle: options.complexViewSingle ?? null,
      },
    ] as const;
  }

  return [
    "chartData",
    datasetId,
    {
      ...common,
      scatter: {
        mode: options.scatter.mode,
        series: options.scatter.mode === "complex" ? options.scatter.series : null,
        xColumn: options.scatter.mode === "xy" ? options.scatter.xColumn : null,
        yColumn: options.scatter.mode === "xy" ? options.scatter.yColumn : null,
        traceXColumn:
          options.scatter.mode === "trace_xy" ? options.scatter.traceXColumn : null,
        traceYColumn:
          options.scatter.mode === "trace_xy" ? options.scatter.traceYColumn : null,
        binColumn: options.scatter.mode === "xy" ? (options.scatter.binColumn ?? null) : null,
      },
    },
  ] as const;
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
