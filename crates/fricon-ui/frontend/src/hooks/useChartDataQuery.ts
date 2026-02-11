import { useQuery } from "@tanstack/react-query";
import { fetchChartData, type ChartDataOptions } from "@/lib/backend";

function buildChartDataKey(
  datasetId: number,
  options: ChartDataOptions | null,
) {
  if (!options) return ["chartData", datasetId, "disabled"];
  const complexViewsKey =
    options.chartType === "line" && options.complexViews
      ? [...options.complexViews].sort().join(",")
      : "";
  const indexFiltersKey = options.indexFilters
    ? options.indexFilters.join(",")
    : "";
  const excludeColumnsKey = options.excludeColumns
    ? [...options.excludeColumns].sort().join("|")
    : "";

  const baseKey = [
    "chartData",
    datasetId,
    options.chartType,
    indexFiltersKey,
    excludeColumnsKey,
  ];

  if (options.chartType === "line") {
    return [...baseKey, options.series, options.xColumn ?? "", complexViewsKey];
  }

  if (options.chartType === "heatmap") {
    return [
      ...baseKey,
      options.series,
      options.xColumn ?? "",
      options.yColumn,
      options.complexViewSingle ?? "",
    ];
  }

  return [
    ...baseKey,
    options.scatter.mode,
    options.scatter.mode === "complex" ? options.scatter.series : "",
    options.scatter.mode === "xy" ? options.scatter.xColumn : "",
    options.scatter.mode === "xy" ? options.scatter.yColumn : "",
    options.scatter.mode === "trace_xy" ? options.scatter.traceXColumn : "",
    options.scatter.mode === "trace_xy" ? options.scatter.traceYColumn : "",
    options.scatter.mode === "xy" ? (options.scatter.binColumn ?? "") : "",
  ];
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
